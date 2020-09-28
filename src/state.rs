use crate::catalog::{assign_menu, catalog_menu};
use crate::complex::ComplexNumber;
use crate::error::{Error, Result};
use crate::font::{SANS_13, SANS_16, SANS_24};
use crate::functions::{Function, FunctionKeyState, FunctionMenu};
use crate::input::{AlphaMode, InputEvent, InputMode, InputQueue};
use crate::layout::Layout;
use crate::menu::{setup_menu, Menu, MenuItemFunction};
use crate::number::{IntegerMode, Number, NumberFormat, ToNumber};
use crate::screen::{Color, Font, Rect, Screen};
use crate::stack::{Stack, MAX_STACK_INDEX_DIGITS};
use crate::storage::{available_bytes, store};
use crate::time::{Now, SimpleDateTimeFormat, SimpleDateTimeToString};
use crate::undo::{clear_undo_buffer, pop_undo_action};
use crate::unit::{unit_menu, AngleUnit};
use crate::value::{Value, ValueRef};
use crate::vector::Vector;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use chrono::NaiveDateTime;
use intel_dfp::Decimal;

#[cfg(feature = "dm42")]
use crate::dm42::{read_power_voltage, show_system_setup_menu, usb_powered};

const MAX_MEMORY_INDEX_DIGITS: usize = 2;

/// Cached state for rendering the status bar. This is used to optimize the rendering
/// of the status bar such that it is only drawn when it is updated.
struct CachedStatusBarState {
	alpha: AlphaMode,
	shift: bool,
	integer_radix: u8,
	integer_mode: IntegerMode,
	angle_mode: AngleUnit,
	multiple_pages: bool,
	left_string: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Location {
	Integer(usize),
	StackOffset(usize),
	Variable(char),
}

#[derive(Clone)]
struct LocationEntryState {
	name: &'static str,
	stack: bool,
	value: Vec<u8>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum InputState {
	Normal,
	Recall,
	Store,
	Menu,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum StatusBarLeftDisplayType {
	CurrentTime,
	FreeMemory,
}

pub struct State {
	pub stack: Stack,
	input_mode: InputMode,
	format: NumberFormat,
	function_keys: FunctionKeyState,
	default_integer_format: IntegerMode,
	prev_decimal_integer_mode: IntegerMode,
	angle_mode: AngleUnit,
	status_bar_left_display: StatusBarLeftDisplayType,
	memory: BTreeMap<Location, ValueRef>,
	input_state: InputState,
	location_entry: LocationEntryState,
	error: Option<Error>,
	menus: Vec<Menu>,
	cached_status_bar_state: CachedStatusBarState,
	force_refresh: bool,
}

pub enum InputResult {
	Normal,
	Suspend,
}

pub enum LocationInputResult {
	Intermediate(InputResult),
	Finished(Location),
	Invalid,
	Exit,
}

impl LocationEntryState {
	fn new(name: &'static str) -> Self {
		LocationEntryState {
			name,
			stack: false,
			value: Vec::new(),
		}
	}

	fn int_value(&self) -> usize {
		let mut result = 0;
		for digit in &self.value {
			result *= 10;
			result += *digit as usize;
		}
		result
	}
}

impl State {
	pub fn new() -> Self {
		let input_mode = InputMode {
			alpha: AlphaMode::Normal,
			shift: false,
		};
		let format = NumberFormat::new();

		let cached_status_bar_state = CachedStatusBarState {
			alpha: input_mode.alpha,
			shift: input_mode.shift,
			integer_radix: format.integer_radix,
			integer_mode: format.integer_mode,
			angle_mode: AngleUnit::Degrees,
			multiple_pages: false,
			left_string: State::time_string(),
		};

		State {
			stack: Stack::new(),
			input_mode,
			format,
			function_keys: FunctionKeyState::new(),
			default_integer_format: IntegerMode::BigInteger,
			prev_decimal_integer_mode: IntegerMode::Float,
			angle_mode: AngleUnit::Degrees,
			status_bar_left_display: StatusBarLeftDisplayType::CurrentTime,
			memory: BTreeMap::new(),
			input_state: InputState::Normal,
			location_entry: LocationEntryState::new(""),
			error: None,
			menus: Vec::new(),
			cached_status_bar_state,
			force_refresh: true,
		}
	}

	pub fn format(&self) -> &NumberFormat {
		&self.format
	}

	pub fn format_mut(&mut self) -> &mut NumberFormat {
		self.stack.invalidate_rendering();
		&mut self.format
	}

	pub fn function_keys(&mut self) -> &mut FunctionKeyState {
		&mut self.function_keys
	}

	pub fn default_integer_format(&self) -> &IntegerMode {
		&self.default_integer_format
	}

	pub fn set_default_integer_format(&mut self, mode: IntegerMode) {
		self.default_integer_format = mode;
	}

	pub fn prev_decimal_integer_mode(&self) -> &IntegerMode {
		&self.prev_decimal_integer_mode
	}

	pub fn set_prev_decimal_integer_mode(&mut self, mode: IntegerMode) {
		self.prev_decimal_integer_mode = mode;
	}

	pub fn angle_mode(&self) -> &AngleUnit {
		&self.angle_mode
	}

	pub fn set_angle_mode(&mut self, unit: AngleUnit) {
		self.angle_mode = unit;
	}

	pub fn status_bar_left_display(&self) -> &StatusBarLeftDisplayType {
		&self.status_bar_left_display
	}

	pub fn set_status_bar_left_display(&mut self, display_type: StatusBarLeftDisplayType) {
		self.status_bar_left_display = display_type;
	}

	pub fn custom_function(&self, idx: usize) -> Option<Function> {
		self.function_keys.custom_function(idx)
	}

	pub fn set_custom_function(&mut self, idx: usize, func: Option<Function>) {
		self.function_keys.set_custom_function(idx, func);
	}

	pub fn show_error(&mut self, error: Error) {
		self.error = Some(error);
		self.input_state = InputState::Normal;
		self.input_mode.alpha = AlphaMode::Normal;
		self.stack.end_edit();
	}

	pub fn hide_error(&mut self) {
		self.error = None;
	}

	fn time_string() -> String {
		NaiveDateTime::now().simple_format(&SimpleDateTimeFormat::status_bar())
	}

	pub fn top<'a>(&'a self) -> Value {
		Stack::value_for_integer_mode(&self.format.integer_mode, self.stack.top())
	}

	pub fn entry<'a>(&'a self, idx: usize) -> Result<Value> {
		Ok(Stack::value_for_integer_mode(
			&self.format.integer_mode,
			self.stack.entry(idx)?,
		))
	}

	pub fn replace_entries(&mut self, count: usize, value: Value) -> Result<()> {
		let value = Stack::value_for_integer_mode(&self.format.integer_mode, value);
		self.stack.replace_entries(count, value)?;
		Ok(())
	}

	pub fn set_top(&mut self, value: Value) -> Result<()> {
		let value = Stack::value_for_integer_mode(&self.format.integer_mode, value);
		self.stack.set_top(value)
	}

	pub fn set_entry(&mut self, offset: usize, value: Value) -> Result<()> {
		let value = Stack::value_for_integer_mode(&self.format.integer_mode, value);
		self.stack.set_entry(offset, value)?;
		Ok(())
	}

	pub fn read<'a>(&'a self, location: &Location) -> Result<Value> {
		match location {
			Location::StackOffset(offset) => self.entry(*offset),
			location => {
				if let Some(value) = self.memory.get(location) {
					Ok(value.get()?)
				} else {
					Err(Error::ValueNotDefined)
				}
			}
		}
	}

	pub fn write(&mut self, location: Location, value: Value) -> Result<()> {
		match location {
			Location::StackOffset(offset) => self.set_entry(offset, value)?,
			location => {
				self.memory.insert(location, store(value)?);
			}
		}
		Ok(())
	}

	pub fn undo(&mut self) -> Result<()> {
		self.stack.undo(pop_undo_action()?)
	}

	pub fn handle_input(&mut self, input: InputEvent, screen: &dyn Screen) -> Result<InputResult> {
		if self.error.is_some() {
			self.error = None;
			return match input {
				InputEvent::Off => Ok(InputResult::Suspend),
				_ => Ok(InputResult::Normal),
			};
		}

		match self.input_state {
			InputState::Normal => {
				match input {
					InputEvent::Character(ch) => match ch {
						'0'..='9' | 'A'..='Z' | 'a'..='z' | '.' => {
							if ch != '.' || self.format.integer_mode == IntegerMode::Float {
								self.stack.push_char(ch, &self.format)?;
							}
						}
						_ => (),
					},
					InputEvent::E => {
						if self.format.integer_mode == IntegerMode::Float {
							self.stack.exponent()?;
						}
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Enter => {
						self.stack.enter()?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Backspace => {
						self.stack.backspace()?;
					}
					InputEvent::Neg => {
						if self.stack.editing() {
							self.stack.neg()?;
						} else {
							self.set_top((-self.top())?)?;
						}
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Add => {
						self.replace_entries(2, (self.entry(1)? + self.entry(0)?)?)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Sub => {
						self.replace_entries(2, (self.entry(1)? - self.entry(0)?)?)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Mul => {
						self.replace_entries(2, (self.entry(1)? * self.entry(0)?)?)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Div => {
						self.replace_entries(2, (self.entry(1)? / self.entry(0)?)?)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Recip => {
						self.set_top((Value::Number(1.into()) / self.top())?)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Pow => {
						self.replace_entries(2, (self.entry(1)?).pow(&self.entry(0)?)?)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Sqrt => {
						self.set_top(self.top().sqrt()?)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Square => {
						let top = self.top();
						let square = (&top * &top)?;
						self.set_top(square)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Log => {
						Function::Log.execute(self, screen)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::TenX => {
						Function::Exp10.execute(self, screen)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Ln => {
						Function::Ln.execute(self, screen)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::EX => {
						Function::Exp.execute(self, screen)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Percent => {
						let factor = (self.entry(0)? / Value::Number(100.into()))?;
						self.set_top((self.entry(1)? * factor)?)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Pi => {
						self.stack
							.input_value(Value::Number(Number::Decimal(Decimal::pi())))?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Sin => {
						Function::Sin.execute(self, screen)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Cos => {
						Function::Cos.execute(self, screen)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Tan => {
						Function::Tan.execute(self, screen)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Asin => {
						Function::Asin.execute(self, screen)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Acos => {
						Function::Acos.execute(self, screen)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Atan => {
						Function::Atan.execute(self, screen)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::RotateDown => {
						self.stack.rotate_down();
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Swap => {
						self.stack.swap(0, 1)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Rcl => {
						self.input_state = InputState::Recall;
						self.location_entry = LocationEntryState::new("Rcl");
						self.stack.end_edit();
					}
					InputEvent::Sto => {
						self.input_state = InputState::Store;
						self.location_entry = LocationEntryState::new("Sto");
						self.stack.end_edit();
					}
					InputEvent::Complex => {
						let top = self.entry(0)?;
						if let Value::Complex(value) = top {
							// If a complex number is on the top of the stack, break it into
							// real and imaginary parts.
							let mut items = Vec::new();
							items.push(store(Value::Number(value.real_part().clone()))?);
							items.push(store(Value::Number(value.imaginary_part().clone()))?);
							self.stack.replace_top_with_multiple(items)?;
						} else {
							// Take the real and imaginary components on the top two entries
							// on the stack and create a complex number.
							let real = self.entry(1)?;
							let imaginary = top;
							self.replace_entries(
								2,
								Value::check_complex(ComplexNumber::from_parts(
									real.real_number()?.clone(),
									imaginary.real_number()?.clone(),
								))?
								.into(),
							)?;
						}
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::SigmaPlus => {
						let top = self.entry(0)?;
						if let Value::Vector(existing_vector) = top {
							// Top entry is a vector. Check entry above it.
							let prev_value = self.entry(1)?;
							if let Value::Vector(prev_vector) = prev_value {
								// Top two entries are vectors. Merge the vectors.
								let mut new_vector = prev_vector.clone();
								new_vector.extend_with(&existing_vector)?;
								self.stack.replace_entries(2, Value::Vector(new_vector))?;
							} else {
								// Fold the second entry into the vector.
								let mut new_vector = existing_vector.clone();
								new_vector.insert(0, prev_value)?;
								self.stack.replace_entries(2, Value::Vector(new_vector))?;
							}
						} else {
							// Create a vector containing the value on the top of the stack.
							let mut vector = Vector::new()?;
							vector.push(top)?;
							self.stack.set_top(Value::Vector(vector))?;
						}
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::SigmaMinus => {
						let top = self.entry(0)?;
						if let Value::Vector(vector) = top {
							// Top entry is a vector. Break apart the vector.
							let mut values: Vec<ValueRef> = Vec::new();
							for i in 0..vector.len() {
								values.push(vector.get_ref(i)?);
							}
							self.stack.replace_top_with_multiple(values)?;
						} else {
							// Batch create a vector from the entries on the stack. If there
							// is a vector or matrix on the stack, stop there.
							let mut vector = Vector::new()?;
							for i in 0..self.stack.len() {
								let value = self.entry(i)?;
								if value.is_vector_or_matrix() {
									break;
								}
								vector.insert(0, value)?;
							}
							self.stack
								.replace_entries(vector.len(), Value::Vector(vector))?;
						}
					}
					InputEvent::Print => clear_undo_buffer(),
					InputEvent::Clear => {
						self.stack.clear();
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Disp => {
						self.function_keys.show_toplevel_menu(FunctionMenu::Disp);
					}
					InputEvent::Modes => {
						self.function_keys.show_toplevel_menu(FunctionMenu::Mode);
					}
					InputEvent::Base => {
						self.function_keys.show_toplevel_menu(FunctionMenu::Base);
					}
					InputEvent::Logic => {
						self.function_keys.show_toplevel_menu(FunctionMenu::Logic);
					}
					InputEvent::Stat => {
						self.function_keys.show_toplevel_menu(FunctionMenu::Stats);
					}
					InputEvent::Matrix => {
						self.function_keys.show_toplevel_menu(FunctionMenu::Matrix);
					}
					InputEvent::Convert => {
						self.show_menu(unit_menu());
					}
					InputEvent::Assign => {
						self.show_menu(assign_menu());
					}
					InputEvent::Custom => {
						self.function_keys.show_toplevel_menu(FunctionMenu::Custom);
					}
					InputEvent::Catalog => {
						self.show_menu(catalog_menu(&|page| Function::CatalogPage(page)));
					}
					InputEvent::FunctionKey(func, _) => {
						if let Some(func) = self.function_keys.function(func) {
							func.execute(self, screen)?;
							self.input_mode.alpha = AlphaMode::Normal;
						}
					}
					InputEvent::Up => {
						self.function_keys.prev_page();
					}
					InputEvent::Down => {
						self.function_keys.next_page();
					}
					InputEvent::Setup => {
						self.input_mode.alpha = AlphaMode::Normal;
						self.input_state = InputState::Menu;
						self.menus.push(setup_menu());
					}
					InputEvent::Undo => {
						self.undo()?;
					}
					InputEvent::Exit => {
						if self.stack.editing() {
							self.stack.end_edit();
							self.input_mode.alpha = AlphaMode::Normal;
						} else {
							self.function_keys.exit_menu(&self.format);
						}
					}
					InputEvent::Off => {
						self.input_mode.alpha = AlphaMode::Normal;
						return Ok(InputResult::Suspend);
					}
					_ => (),
				}
				Ok(InputResult::Normal)
			}
			InputState::Recall => match self.handle_location_input(input) {
				LocationInputResult::Intermediate(result) => Ok(result),
				LocationInputResult::Finished(location) => {
					self.input_state = InputState::Normal;
					self.input_mode.alpha = AlphaMode::Normal;
					self.stack.input_value(self.read(&location)?)?;
					Ok(InputResult::Normal)
				}
				LocationInputResult::Exit => {
					self.input_state = InputState::Normal;
					Ok(InputResult::Normal)
				}
				LocationInputResult::Invalid => {
					self.input_state = InputState::Normal;
					Err(Error::InvalidEntry)
				}
			},
			InputState::Store => match self.handle_location_input(input) {
				LocationInputResult::Intermediate(result) => Ok(result),
				LocationInputResult::Finished(location) => {
					self.input_state = InputState::Normal;
					self.input_mode.alpha = AlphaMode::Normal;
					self.write(location, self.top())?;
					Ok(InputResult::Normal)
				}
				LocationInputResult::Exit => {
					self.input_state = InputState::Normal;
					self.input_mode.alpha = AlphaMode::Normal;
					Ok(InputResult::Normal)
				}
				LocationInputResult::Invalid => {
					self.input_state = InputState::Normal;
					self.input_mode.alpha = AlphaMode::Normal;
					Err(Error::InvalidEntry)
				}
			},
			InputState::Menu => {
				let menu = self.menus.last_mut().unwrap();
				match input {
					InputEvent::Up => menu.up(),
					InputEvent::Down => menu.down(),
					InputEvent::Enter | InputEvent::Add | InputEvent::Mul => {
						menu.force_refresh();
						let function = menu.selected_function();
						self.force_refresh = true;
						match function {
							MenuItemFunction::Action(action) => {
								self.input_state = InputState::Normal;
								self.menus.clear();
								action.execute(self, screen)?;
							}
							MenuItemFunction::InMenuAction(action) => {
								action.execute(self, screen)?;
							}
							MenuItemFunction::InMenuActionWithDelete(action, _) => {
								action.execute(self, screen)?;
							}
							MenuItemFunction::ConversionAction(action, _, _) => {
								action.execute(self, screen)?;
							}
						}
					}
					InputEvent::Sub | InputEvent::Div => {
						menu.force_refresh();
						let function = menu.selected_function();
						match function {
							MenuItemFunction::Action(_)
							| MenuItemFunction::InMenuAction(_)
							| MenuItemFunction::InMenuActionWithDelete(_, _) => (),
							MenuItemFunction::ConversionAction(_, action, _) => {
								self.force_refresh = true;
								action.execute(self, screen)?;
							}
						}
					}
					InputEvent::Swap => {
						menu.force_refresh();
						let function = menu.selected_function();
						match function {
							MenuItemFunction::Action(_)
							| MenuItemFunction::InMenuAction(_)
							| MenuItemFunction::InMenuActionWithDelete(_, _) => (),
							MenuItemFunction::ConversionAction(_, _, action) => {
								self.force_refresh = true;
								action.execute(self, screen)?;
							}
						}
					}
					InputEvent::Backspace => {
						menu.force_refresh();
						let function = menu.selected_function();
						match function {
							MenuItemFunction::Action(_)
							| MenuItemFunction::InMenuAction(_)
							| MenuItemFunction::ConversionAction(_, _, _) => (),
							MenuItemFunction::InMenuActionWithDelete(_, action) => {
								action.execute(self, screen)?;
							}
						}
					}
					InputEvent::Character(ch) => match ch {
						'1'..='9' => {
							self.direct_select_menu_item(
								(ch as u32 - '1' as u32) as usize,
								screen,
							)?;
						}
						'0' => {
							self.direct_select_menu_item(9, screen)?;
						}
						'A'..='L' => {
							self.direct_select_menu_item(
								10 + (ch as u32 - 'A' as u32) as usize,
								screen,
							)?;
						}
						'a'..='l' => {
							self.direct_select_menu_item(
								10 + (ch as u32 - 'a' as u32) as usize,
								screen,
							)?;
						}
						_ => (),
					},
					InputEvent::SigmaPlus => self.direct_select_menu_item(10, screen)?,
					InputEvent::Recip => self.direct_select_menu_item(11, screen)?,
					InputEvent::Sqrt => self.direct_select_menu_item(12, screen)?,
					InputEvent::Log => self.direct_select_menu_item(13, screen)?,
					InputEvent::Ln => self.direct_select_menu_item(14, screen)?,
					InputEvent::Xeq => self.direct_select_menu_item(15, screen)?,
					InputEvent::Sto => self.direct_select_menu_item(16, screen)?,
					InputEvent::Rcl => self.direct_select_menu_item(17, screen)?,
					InputEvent::RotateDown => self.direct_select_menu_item(18, screen)?,
					InputEvent::Sin => self.direct_select_menu_item(19, screen)?,
					InputEvent::Cos => self.direct_select_menu_item(20, screen)?,
					InputEvent::Tan => self.direct_select_menu_item(21, screen)?,
					InputEvent::Exit => {
						self.menus.pop();
						if let Some(menu) = self.menus.last_mut() {
							menu.force_refresh();
						} else {
							self.input_state = InputState::Normal;
							self.cached_status_bar_state.left_string = String::new();
							self.force_refresh = true;
						}
					}
					InputEvent::Off => return Ok(InputResult::Suspend),
					_ => (),
				}
				Ok(InputResult::Normal)
			}
		}
	}

	fn handle_location_input(&mut self, input: InputEvent) -> LocationInputResult {
		match input {
			InputEvent::Character(ch) => match ch {
				'0'..='9' => {
					self.location_entry
						.value
						.push(ch as u32 as u8 - '0' as u32 as u8);
					if self.location_entry.stack {
						if self.location_entry.value.len() >= MAX_STACK_INDEX_DIGITS {
							return LocationInputResult::Finished(Location::StackOffset(
								self.location_entry.int_value(),
							));
						}
					} else {
						if self.location_entry.value.len() >= MAX_MEMORY_INDEX_DIGITS {
							return LocationInputResult::Finished(Location::Integer(
								self.location_entry.int_value(),
							));
						}
					};
					LocationInputResult::Intermediate(InputResult::Normal)
				}
				'.' => {
					self.location_entry.stack = true;
					LocationInputResult::Intermediate(InputResult::Normal)
				}
				'A'..='Z' | 'a'..='z' | 'α'..='ω' => {
					if self.location_entry.stack {
						match ch {
							'x' | 'X' => LocationInputResult::Finished(Location::StackOffset(0)),
							'y' | 'Y' => LocationInputResult::Finished(Location::StackOffset(1)),
							'z' | 'Z' => LocationInputResult::Finished(Location::StackOffset(2)),
							_ => LocationInputResult::Invalid,
						}
					} else if self.location_entry.value.len() > 0 {
						LocationInputResult::Invalid
					} else {
						LocationInputResult::Finished(Location::Variable(ch))
					}
				}
				_ => LocationInputResult::Invalid,
			},
			InputEvent::Enter => {
				if self.location_entry.value.len() > 0 {
					if self.location_entry.stack {
						LocationInputResult::Finished(Location::StackOffset(
							self.location_entry.int_value(),
						))
					} else {
						LocationInputResult::Finished(Location::Integer(
							self.location_entry.int_value(),
						))
					}
				} else {
					LocationInputResult::Invalid
				}
			}
			InputEvent::Backspace => {
				if self.location_entry.value.len() > 0 {
					self.location_entry.value.pop();
					LocationInputResult::Intermediate(InputResult::Normal)
				} else if self.location_entry.stack {
					self.location_entry.stack = false;
					LocationInputResult::Intermediate(InputResult::Normal)
				} else {
					LocationInputResult::Exit
				}
			}
			InputEvent::Exit => LocationInputResult::Exit,
			InputEvent::Off => {
				self.input_mode.alpha = AlphaMode::Normal;
				LocationInputResult::Intermediate(InputResult::Suspend)
			}
			_ => LocationInputResult::Invalid,
		}
	}

	fn draw_status_bar_indicator(
		&self,
		screen: &mut dyn Screen,
		x: &mut i32,
		text: &str,
		font: &Font,
	) {
		*x -= font.width(text);
		font.draw(screen, *x, 0, text, Color::StatusBarText);
		*x -= 8;
	}

	fn update_status_bar_state(&mut self) -> bool {
		let mut changed = false;

		let alpha = self.input_mode.alpha;
		let shift = self.input_mode.shift;
		let integer_radix = self.format.integer_radix;
		let integer_mode = self.format.integer_mode;
		let angle_mode = self.angle_mode;
		let multiple_pages = self.function_keys.multiple_pages();

		// Check for alpha mode updates
		if alpha != self.cached_status_bar_state.alpha {
			self.cached_status_bar_state.alpha = alpha;
			changed = true;
		}

		// Check for shift state updates
		if shift != self.cached_status_bar_state.shift {
			self.cached_status_bar_state.shift = shift;
			changed = true;
		}

		// Check for integer radix updates
		if integer_radix != self.cached_status_bar_state.integer_radix {
			self.cached_status_bar_state.integer_radix = integer_radix;
			changed = true;
		}

		// Check for integer mode updates
		if integer_mode != self.cached_status_bar_state.integer_mode {
			self.cached_status_bar_state.integer_mode = integer_mode;
			changed = true;
		}

		// Check for angle mode updates
		if angle_mode != self.cached_status_bar_state.angle_mode {
			self.cached_status_bar_state.angle_mode = angle_mode;
			changed = true;
		}

		if multiple_pages != self.cached_status_bar_state.multiple_pages {
			self.cached_status_bar_state.multiple_pages = multiple_pages;
			changed = true;
		}

		match self.status_bar_left_display {
			StatusBarLeftDisplayType::CurrentTime => {
				// Check for time updates
				if NaiveDateTime::clock_minute_updated()
					|| self.cached_status_bar_state.left_string.len() == 0
				{
					let time_string = State::time_string();
					self.cached_status_bar_state.left_string = time_string;
					changed = true;
				}
			}
			StatusBarLeftDisplayType::FreeMemory => {
				let free_memory = available_bytes().to_number().to_string() + " bytes free";
				if free_memory != self.cached_status_bar_state.left_string {
					self.cached_status_bar_state.left_string = free_memory;
					changed = true;
				}
			}
		}

		changed
	}

	#[cfg(feature = "dm42")]
	fn draw_battery_indicator(&self, screen: &mut dyn Screen, x: &mut i32) {
		// Determine how many bars are present inside the battery indicator
		let usb = usb_powered();
		let voltage = read_power_voltage();
		let mut fill = 5 - ((2940 - voltage as i32) / 150);
		if fill < 0 {
			fill = 0;
		} else if fill > 5 {
			fill = 5;
		}

		// Render battery shape
		*x -= 22;
		screen.fill(
			Rect {
				x: *x,
				y: 3,
				w: 20,
				h: SANS_13.height - 6,
			},
			Color::StatusBarText,
		);
		screen.fill(
			Rect {
				x: *x + 2,
				y: 5,
				w: 16,
				h: SANS_13.height - 10,
			},
			Color::StatusBarBackground,
		);
		screen.set_pixel(*x, 3, Color::StatusBarBackground);
		screen.set_pixel(*x + 19, 3, Color::StatusBarBackground);
		screen.set_pixel(*x, SANS_13.height - 4, Color::StatusBarBackground);
		screen.set_pixel(*x + 19, SANS_13.height - 4, Color::StatusBarBackground);
		screen.fill(
			Rect {
				x: *x + 20,
				y: 7,
				w: 2,
				h: SANS_13.height - 14,
			},
			Color::StatusBarText,
		);

		// Render inside of battery indicator
		if usb {
			for i in 6..SANS_13.height - 6 {
				if i & 1 == 0 {
					screen.draw_bits(*x + 3, i, 0x1555, 14, Color::StatusBarText);
				} else {
					screen.draw_bits(*x + 3, i, 0x2aaa, 14, Color::StatusBarText);
				}
			}
		} else {
			for i in 0..fill {
				screen.fill(
					Rect {
						x: *x + i * 3 + 3,
						y: 6,
						w: 2,
						h: SANS_13.height - 12,
					},
					Color::StatusBarText,
				);
			}
		}

		*x -= 8;
	}

	fn draw_status_bar(&self, screen: &mut dyn Screen) {
		// Render status bar background
		screen.fill(
			Rect {
				x: 0,
				y: 0,
				w: screen.width(),
				h: SANS_13.height,
			},
			Color::StatusBarBackground,
		);
		screen.fill(
			Rect {
				x: 0,
				y: SANS_13.height,
				w: screen.width(),
				h: 1,
			},
			Color::ContentBackground,
		);

		let mut x = screen.width() - 4;

		#[cfg(feature = "dm42")]
		self.draw_battery_indicator(screen, &mut x);

		// Render alpha mode indicator
		match self.cached_status_bar_state.alpha {
			AlphaMode::UpperAlpha => {
				self.draw_status_bar_indicator(screen, &mut x, "[A]", &SANS_13)
			}
			AlphaMode::LowerAlpha => {
				self.draw_status_bar_indicator(screen, &mut x, "[a]", &SANS_13)
			}
			_ => (),
		}

		// Render shift indicator
		if self.cached_status_bar_state.shift {
			self.draw_status_bar_indicator(screen, &mut x, "⬏", &SANS_16);
		}

		// Render integer radix indicator
		match self.cached_status_bar_state.integer_radix {
			8 => self.draw_status_bar_indicator(screen, &mut x, "Oct", &SANS_13),
			16 => self.draw_status_bar_indicator(screen, &mut x, "Hex", &SANS_13),
			_ => (),
		}

		// Render integer format indicator
		match self.cached_status_bar_state.integer_mode {
			IntegerMode::Float => (),
			IntegerMode::BigInteger => {
				self.draw_status_bar_indicator(screen, &mut x, "int", &SANS_13)
			}
			IntegerMode::SizedInteger(size, signed) => {
				let string = if signed {
					"i".to_string()
				} else {
					"u".to_string()
				};
				let string = string + NumberFormat::new().format_bigint(&size.into()).as_str();
				self.draw_status_bar_indicator(screen, &mut x, &string, &SANS_13);
			}
		}

		// Render angle mode indicator
		match self.angle_mode {
			AngleUnit::Degrees => (),
			AngleUnit::Radians => self.draw_status_bar_indicator(screen, &mut x, "Rad", &SANS_13),
			AngleUnit::Gradians => self.draw_status_bar_indicator(screen, &mut x, "Grad", &SANS_13),
		}

		// Render menu page indicator
		if self.cached_status_bar_state.multiple_pages {
			self.draw_status_bar_indicator(screen, &mut x, "▴▾", &SANS_13);
		}

		// Render current time or alternate status text
		let left_string = &self.cached_status_bar_state.left_string;
		let left_width = SANS_13.width(left_string) + 8;
		if 4 + left_width < x {
			SANS_13.draw(screen, 4, 0, left_string, Color::StatusBarText);
		}
	}

	fn status_bar_size(&self) -> i32 {
		SANS_13.height + 1
	}

	pub fn render(&mut self, screen: &mut dyn Screen) {
		if self.input_state == InputState::Menu {
			if let Some(menu) = self.menus.last() {
				menu.render(self, screen);
				return;
			}
		}

		// Check for updates to status bar and render if changed
		if self.update_status_bar_state() || self.force_refresh {
			self.draw_status_bar(screen);
		}

		// Check for updates to function key indicators and render if changed
		self.function_keys.update(&self.format);
		if self.function_keys.update_menu_strings(&self) || self.force_refresh {
			self.function_keys.render(screen);
		}

		// Initialize stack area rectangle. It may be modified depending on extra
		// state display.
		let mut stack_area = Rect {
			x: 0,
			y: self.status_bar_size(),
			w: screen.width(),
			h: screen.height() - self.status_bar_size() - self.function_keys.height(),
		};

		// If there is an error, display the message
		if let Some(error) = self.error {
			let mut items = Vec::new();
			items.push(Layout::StaticText(
				error.to_str(),
				&SANS_24,
				Color::ContentText,
			));
			items.push(Layout::HorizontalSpace(4));
			let layout = Layout::Horizontal(items);

			let height = layout.height();
			stack_area.h -= height;
			let rect = Rect {
				x: 0,
				y: stack_area.y + stack_area.h,
				w: screen.width(),
				h: height,
			};
			let clip_rect = rect.clone();
			screen.fill(rect.clone(), Color::ContentBackground);
			layout.render(screen, rect, &clip_rect, None);

			// Render a line to separate the error from the stack area
			screen.fill(
				Rect {
					x: 0,
					y: stack_area.y + stack_area.h,
					w: screen.width(),
					h: 1,
				},
				Color::ContentText,
			);
		}

		// If there is an active location editor present, render it
		if self.input_state == InputState::Recall || self.input_state == InputState::Store {
			let mut items = Vec::new();
			// Show use of location
			items.push(Layout::Text(
				self.location_entry.name.to_string() + " ",
				&SANS_24,
				Color::ContentText,
			));

			// If this is a stack access, display "Stack"
			if self.location_entry.stack {
				items.push(Layout::StaticText("Stack ", &SANS_24, Color::ContentText));
			}

			// Show currently edited number
			let mut value_str = String::new();
			for digit in &self.location_entry.value {
				value_str.push(char::from_u32('0' as u32 + *digit as u32).unwrap());
			}
			items.push(Layout::EditText(value_str, &SANS_24, Color::ContentText));

			items.push(Layout::HorizontalSpace(4));

			// Render the layout and adjust the stack area to not include it
			let layout = Layout::Horizontal(items);
			let height = layout.height();
			stack_area.h -= height;
			let rect = Rect {
				x: 0,
				y: stack_area.y + stack_area.h,
				w: screen.width(),
				h: height,
			};
			let clip_rect = rect.clone();
			screen.fill(rect.clone(), Color::ContentBackground);
			layout.render(screen, rect, &clip_rect, None);

			// Render a line to separate the stack area from the location editor
			screen.fill(
				Rect {
					x: 0,
					y: stack_area.y + stack_area.h,
					w: screen.width(),
					h: 1,
				},
				Color::ContentText,
			);
		}

		// Render the stack
		if self.force_refresh {
			self.stack.force_refresh();
		}
		self.stack.render(screen, &self.format, stack_area);

		// Refresh the LCD contents
		screen.refresh();
		self.force_refresh = false;
	}

	pub fn update_header(&mut self, screen: &mut dyn Screen) {
		if self.input_state != InputState::Menu {
			// When specifically updating the header, always render the header
			self.update_status_bar_state();
			self.draw_status_bar(screen);
			screen.refresh();
		}
	}

	fn direct_select_menu_item(&mut self, idx: usize, screen: &dyn Screen) -> Result<()> {
		let menu = self.menus.last_mut().unwrap();
		menu.force_refresh();
		if let Some(function) = menu.specific_function(idx) {
			match function {
				MenuItemFunction::Action(action) => {
					self.input_state = InputState::Normal;
					self.menus.clear();
					self.force_refresh = true;
					action.execute(self, screen)?;
				}
				MenuItemFunction::InMenuAction(action)
				| MenuItemFunction::InMenuActionWithDelete(action, _) => {
					self.force_refresh = true;
					action.execute(self, screen)?;
				}
				_ => menu.set_selection(idx),
			}
		}
		Ok(())
	}

	pub fn show_menu(&mut self, menu: Menu) {
		self.menus.push(menu);
		self.input_state = InputState::Menu;
	}

	pub fn show_system_setup_menu(&mut self) {
		#[cfg(feature = "dm42")]
		show_system_setup_menu();
	}

	pub fn wait_for_input<InputT: InputQueue>(&mut self, input: &mut InputT) -> Option<InputEvent> {
		input.wait(&mut self.input_mode)
	}
}
