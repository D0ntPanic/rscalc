use crate::dm42::catalog::{assign_menu, catalog_menu};
use crate::dm42::edit::NumberEditor;
use crate::dm42::functions::{Function, FunctionKeyState, FunctionMenu};
use crate::dm42::input::{AlphaMode, InputEvent, InputMode, InputQueue};
use crate::dm42::menu::{setup_menu, Menu, MenuItemFunction};
use crate::dm42::screen::{RenderMode, Screen};
use crate::dm42::unit::unit_menu;
use chrono::NaiveDateTime;
use rscalc_layout::decimal::DecimalLayout;
use rscalc_layout::font::Font;
use rscalc_layout::layout::{Layout, LayoutRenderer, Rect, TokenType};
use rscalc_layout::stack::StackRenderer;
use rscalc_layout::string::StringLayout;
use rscalc_layout::value::{AlternateLayoutType, ValueLayout};
use rscalc_math::constant::Constant;
use rscalc_math::context::{Context, Location};
use rscalc_math::error::{Error, Result};
use rscalc_math::format::{Format, IntegerMode};
use rscalc_math::number::ToNumber;
use rscalc_math::storage::available_bytes;
use rscalc_math::time::{Now, SimpleDateTimeFormat, SimpleDateTimeToString};
use rscalc_math::unit::AngleUnit;
use rscalc_math::value::Value;

#[cfg(not(feature = "dm42"))]
use std::cell::RefCell;
#[cfg(not(feature = "dm42"))]
use std::rc::Rc;

#[cfg(feature = "dm42")]
use crate::dm42::device::{read_power_voltage, show_system_setup_menu, usb_powered};
#[cfg(feature = "dm42")]
use alloc::boxed::Box;
#[cfg(feature = "dm42")]
use alloc::rc::Rc;
#[cfg(feature = "dm42")]
use alloc::string::{String, ToString};
#[cfg(feature = "dm42")]
use alloc::vec::Vec;
#[cfg(feature = "dm42")]
use core::cell::RefCell;

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

#[derive(Clone)]
struct LocationEntryState {
	name: &'static str,
	stack: bool,
	value: Vec<u8>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum InputState {
	Normal,
	NumberInput,
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
	context: Context,
	input_mode: InputMode,
	function_keys: FunctionKeyState,
	status_bar_left_display: StatusBarLeftDisplayType,
	input_state: InputState,
	location_entry: LocationEntryState,
	error: Option<Error>,
	menus: Vec<Menu>,
	editor: Option<NumberEditor>,
	status_bar_enabled: bool,
	base_font: Font,
	stack_renderer: Rc<RefCell<StackRenderer>>,
	cached_status_bar_state: CachedStatusBarState,
	force_refresh: bool,
	force_render_on_status_update: bool,
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

#[cfg(not(feature = "dm42"))]
fn clock_minute_updated() -> bool {
	true
}

#[cfg(feature = "dm42")]
fn clock_minute_updated() -> bool {
	crate::dm42::device::rtc_updated()
}

impl State {
	pub fn new() -> Self {
		let mut context = Context::new_with_undo();
		let stack_renderer = StackRenderer::new(context.stack_mut());

		let input_mode = InputMode {
			alpha: AlphaMode::Normal,
			shift: false,
		};

		let cached_status_bar_state = CachedStatusBarState {
			alpha: input_mode.alpha,
			shift: input_mode.shift,
			integer_radix: context.format().integer_radix,
			integer_mode: context.format().integer_mode,
			angle_mode: *context.angle_mode(),
			multiple_pages: false,
			left_string: State::time_string(context.format().time_24_hour),
		};

		State {
			context,
			input_mode,
			function_keys: FunctionKeyState::new(),
			status_bar_left_display: StatusBarLeftDisplayType::CurrentTime,
			input_state: InputState::Normal,
			location_entry: LocationEntryState::new(""),
			error: None,
			menus: Vec::new(),
			editor: None,
			status_bar_enabled: true,
			base_font: Font::Large,
			stack_renderer,
			cached_status_bar_state,
			force_refresh: true,
			force_render_on_status_update: false,
		}
	}

	pub fn context(&self) -> &Context {
		&self.context
	}

	pub fn context_mut(&mut self) -> &mut Context {
		&mut self.context
	}

	pub fn function_keys(&self) -> &FunctionKeyState {
		&self.function_keys
	}

	pub fn function_keys_mut(&mut self) -> &mut FunctionKeyState {
		&mut self.function_keys
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

	pub fn status_bar_enabled(&self) -> bool {
		self.status_bar_enabled
	}

	pub fn set_status_bar_enabled(&mut self, value: bool) {
		self.status_bar_enabled = value;
		self.force_refresh = true;
	}

	pub fn base_font(&self) -> Font {
		self.base_font
	}

	pub fn set_base_font(&mut self, font: Font) {
		self.base_font = font;
		self.force_refresh = true;
		self.stack_renderer.borrow_mut().invalidate_rendering();
	}

	pub fn show_error(&mut self, error: Error) {
		self.error = Some(error);
		self.input_state = InputState::Normal;
		self.input_mode.alpha = AlphaMode::Normal;
	}

	pub fn hide_error(&mut self) {
		self.error = None;
	}

	fn time_string(time_24_hour: bool) -> String {
		match NaiveDateTime::now() {
			Ok(now) => now.simple_format(&SimpleDateTimeFormat::status_bar(time_24_hour)),
			Err(_) => "Unknown time".to_string(),
		}
	}

	pub fn undo(&mut self) -> Result<()> {
		self.context.undo()
	}

	pub fn end_edit(&mut self) -> Result<()> {
		if let Some(editor) = &self.editor {
			let value = editor.number();
			self.editor = None;
			self.input_state = InputState::Normal;
			self.context.push(Value::Number(value))?;
		}
		self.input_mode.alpha = AlphaMode::Normal;
		Ok(())
	}

	fn handle_common_input(
		&mut self,
		input: InputEvent,
		screen: &dyn Screen,
	) -> Result<InputResult> {
		match input {
			InputEvent::Add => {
				self.end_edit()?;
				self.context.add()?;
			}
			InputEvent::Sub => {
				self.end_edit()?;
				self.context.sub()?;
			}
			InputEvent::Mul => {
				self.end_edit()?;
				self.context.mul()?;
			}
			InputEvent::Div => {
				self.end_edit()?;
				self.context.div()?;
			}
			InputEvent::Recip => {
				self.end_edit()?;
				self.context.recip()?;
			}
			InputEvent::Pow => {
				self.end_edit()?;
				self.context.pow()?;
			}
			InputEvent::Sqrt => {
				self.end_edit()?;
				self.context.sqrt()?;
			}
			InputEvent::Square => {
				self.end_edit()?;
				self.context.square()?;
			}
			InputEvent::Log => {
				self.end_edit()?;
				self.context.log()?;
			}
			InputEvent::TenX => {
				self.end_edit()?;
				self.context.exp10()?;
			}
			InputEvent::Ln => {
				self.end_edit()?;
				self.context.ln()?;
			}
			InputEvent::EX => {
				self.end_edit()?;
				self.context.exp()?;
			}
			InputEvent::Percent => {
				self.end_edit()?;
				self.context.percent()?;
			}
			InputEvent::Pi => {
				self.end_edit()?;
				self.context.push_constant(Constant::Pi)?;
			}
			InputEvent::Sin => {
				self.end_edit()?;
				self.context.sin()?;
			}
			InputEvent::Cos => {
				self.end_edit()?;
				self.context.cos()?;
			}
			InputEvent::Tan => {
				self.end_edit()?;
				self.context.tan()?;
			}
			InputEvent::Asin => {
				self.end_edit()?;
				self.context.asin()?;
			}
			InputEvent::Acos => {
				self.end_edit()?;
				self.context.acos()?;
			}
			InputEvent::Atan => {
				self.end_edit()?;
				self.context.atan()?;
			}
			InputEvent::RotateDown => {
				self.end_edit()?;
				self.context.rotate_down();
			}
			InputEvent::Swap => {
				self.end_edit()?;
				self.context.swap(0, 1)?;
			}
			InputEvent::Rcl => {
				self.end_edit()?;
				self.input_state = InputState::Recall;
				self.location_entry = LocationEntryState::new("Rcl");
			}
			InputEvent::Sto => {
				self.end_edit()?;
				self.input_state = InputState::Store;
				self.location_entry = LocationEntryState::new("Sto");
			}
			InputEvent::Complex => {
				self.end_edit()?;
				self.context.complex()?;
			}
			InputEvent::SigmaPlus => {
				self.end_edit()?;
				self.context.add_to_vector()?;
			}
			InputEvent::SigmaMinus => {
				self.end_edit()?;
				self.context.decompose()?;
			}
			InputEvent::Print => self.context.clear_undo_buffer(),
			InputEvent::Clear => {
				self.end_edit()?;
				self.context.clear_stack();
			}
			InputEvent::Run => {
				self.end_edit()?;
				self.context.toggle_integer_radix();
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
				self.show_menu(unit_menu())?;
			}
			InputEvent::Assign => {
				self.show_menu(assign_menu())?;
			}
			InputEvent::Custom => {
				self.function_keys.show_toplevel_menu(FunctionMenu::Custom);
			}
			InputEvent::Catalog => {
				self.show_menu(catalog_menu(&|page| Function::CatalogPage(page)))?;
			}
			InputEvent::FunctionKey(func, _) => {
				if let Some(func) = self.function_keys.function(func) {
					func.execute(self, screen)?;
				}
			}
			InputEvent::Up => {
				self.function_keys.prev_page();
			}
			InputEvent::Down => {
				self.function_keys.next_page();
			}
			InputEvent::Setup => {
				self.end_edit()?;
				self.input_state = InputState::Menu;
				self.menus.push(setup_menu());
			}
			InputEvent::Undo => {
				self.end_edit()?;
				self.undo()?;
			}
			InputEvent::Off => {
				self.input_mode.alpha = AlphaMode::Normal;
				return Ok(InputResult::Suspend);
			}
			_ => (),
		}
		Ok(InputResult::Normal)
	}

	fn handle_normal_input(
		&mut self,
		input: InputEvent,
		screen: &dyn Screen,
	) -> Result<InputResult> {
		match input {
			InputEvent::Character(_) | InputEvent::E => {
				self.editor = Some(NumberEditor::new(&self.context.format()));
				self.input_state = InputState::NumberInput;
				return self.handle_number_input(input, screen);
			}
			InputEvent::Enter => {
				self.end_edit()?;
				self.context.push(self.context.top()?)?;
			}
			InputEvent::Backspace => {
				self.end_edit()?;
				let _ = self.context.pop();
			}
			InputEvent::Neg => {
				self.end_edit()?;
				self.context.set_top((-self.context.top()?)?)?;
			}
			InputEvent::Exit => {
				self.function_keys.exit_menu(self.context.format());
			}
			_ => return self.handle_common_input(input, screen),
		}
		Ok(InputResult::Normal)
	}

	fn handle_number_input(
		&mut self,
		input: InputEvent,
		screen: &dyn Screen,
	) -> Result<InputResult> {
		let editor = match self.editor.as_mut() {
			Some(editor) => editor,
			None => {
				self.input_state = InputState::Normal;
				return Err(Error::InvalidEntry);
			}
		};

		match input {
			InputEvent::Character(ch) => match ch {
				'0'..='9' | 'A'..='Z' | 'a'..='z' | '.' => {
					if ch != '.' || self.context.format().integer_mode == IntegerMode::Float {
						editor.push_char(ch)?;
					}
				}
				_ => (),
			},
			InputEvent::E => {
				if self.context.format().integer_mode == IntegerMode::Float {
					editor.exponent();
				}
				self.input_mode.alpha = AlphaMode::Normal;
			}
			InputEvent::Enter => {
				self.end_edit()?;
			}
			InputEvent::Backspace => {
				if !editor.backspace() {
					self.editor = None;
					self.input_state = InputState::Normal;
				}
			}
			InputEvent::Neg => {
				editor.neg();
			}
			InputEvent::Exit => {
				self.editor = None;
				self.input_state = InputState::Normal;
			}
			_ => return self.handle_common_input(input, screen),
		}
		Ok(InputResult::Normal)
	}

	fn handle_recall_input(&mut self, input: InputEvent) -> Result<InputResult> {
		match self.handle_location_input(input) {
			LocationInputResult::Intermediate(result) => Ok(result),
			LocationInputResult::Finished(location) => {
				self.input_state = InputState::Normal;
				self.input_mode.alpha = AlphaMode::Normal;
				self.context.push(self.context.read(&location)?)?;
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
		}
	}

	fn handle_store_input(&mut self, input: InputEvent) -> Result<InputResult> {
		match self.handle_location_input(input) {
			LocationInputResult::Intermediate(result) => Ok(result),
			LocationInputResult::Finished(location) => {
				self.input_state = InputState::Normal;
				self.input_mode.alpha = AlphaMode::Normal;
				self.context.write(location, self.context.top()?)?;
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
		}
	}

	fn handle_menu_input(&mut self, input: InputEvent, screen: &dyn Screen) -> Result<InputResult> {
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
					self.direct_select_menu_item((ch as u32 - '1' as u32) as usize, screen)?;
				}
				'0' => {
					self.direct_select_menu_item(9, screen)?;
				}
				'A'..='L' => {
					self.direct_select_menu_item(10 + (ch as u32 - 'A' as u32) as usize, screen)?;
				}
				'a'..='l' => {
					self.direct_select_menu_item(10 + (ch as u32 - 'a' as u32) as usize, screen)?;
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

	pub fn handle_input(&mut self, input: InputEvent, screen: &dyn Screen) -> Result<InputResult> {
		if self.error.is_some() {
			self.error = None;
			return match input {
				InputEvent::Off => Ok(InputResult::Suspend),
				_ => Ok(InputResult::Normal),
			};
		}

		match self.input_state {
			InputState::Normal => self.handle_normal_input(input, screen),
			InputState::NumberInput => self.handle_number_input(input, screen),
			InputState::Recall => self.handle_recall_input(input),
			InputState::Store => self.handle_store_input(input),
			InputState::Menu => self.handle_menu_input(input, screen),
		}
	}

	fn handle_location_input(&mut self, input: InputEvent) -> LocationInputResult {
		match input {
			InputEvent::Character(ch) => match ch {
				'0'..='9' => {
					self.location_entry
						.value
						.push(ch as u32 as u8 - '0' as u32 as u8);
					if !self.location_entry.stack {
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
						if self.location_entry.int_value() == 0 {
							LocationInputResult::Invalid
						} else {
							LocationInputResult::Finished(Location::StackOffset(
								self.location_entry.int_value() - 1,
							))
						}
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
		renderer: &mut dyn LayoutRenderer,
		x: &mut i32,
		text: &str,
		font: Font,
		clip_rect: &Rect,
	) {
		*x -= renderer.metrics().width(font, text);
		renderer.draw_text(*x, 0, text, font, TokenType::Text, clip_rect);
		*x -= 8;
	}

	fn update_status_bar_state(&mut self) -> bool {
		let mut changed = false;

		let alpha = self.input_mode.alpha;
		let shift = self.input_mode.shift;
		let integer_radix = self.context.format().integer_radix;
		let integer_mode = self.context.format().integer_mode;
		let angle_mode = *self.context.angle_mode();
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
				if clock_minute_updated() || self.cached_status_bar_state.left_string.len() == 0 {
					let time_string = State::time_string(self.context.format().time_24_hour);
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
	fn draw_battery_indicator(&self, renderer: &mut dyn LayoutRenderer, x: &mut i32) {
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
		renderer.fill(
			&Rect {
				x: *x,
				y: 3,
				w: 20,
				h: renderer.metrics().height(Font::Smallest) - 6,
			},
			TokenType::Text,
		);
		renderer.erase(&Rect {
			x: *x + 2,
			y: 5,
			w: 16,
			h: renderer.metrics().height(Font::Smallest) - 10,
		});
		renderer.erase(&Rect {
			x: *x,
			y: 3,
			w: 1,
			h: 1,
		});
		renderer.erase(&Rect {
			x: *x + 19,
			y: 3,
			w: 1,
			h: 1,
		});
		renderer.erase(&Rect {
			x: *x,
			y: renderer.metrics().height(Font::Smallest) - 4,
			w: 1,
			h: 1,
		});
		renderer.erase(&Rect {
			x: *x + 19,
			y: renderer.metrics().height(Font::Smallest) - 4,
			w: 1,
			h: 1,
		});
		renderer.fill(
			&Rect {
				x: *x + 20,
				y: 7,
				w: 2,
				h: renderer.metrics().height(Font::Smallest) - 14,
			},
			TokenType::Text,
		);

		// Render inside of battery indicator
		if usb {
			for i in 6..renderer.metrics().height(Font::Smallest) - 6 {
				if i & 1 == 0 {
					renderer.horizontal_pattern(*x + 3, 14, i, 0x1555, 14, TokenType::Text);
				} else {
					renderer.horizontal_pattern(*x + 3, 14, i, 0x2aaa, 14, TokenType::Text);
				}
			}
		} else {
			for i in 0..fill {
				renderer.fill(
					&Rect {
						x: *x + i * 3 + 3,
						y: 6,
						w: 2,
						h: renderer.metrics().height(Font::Smallest) - 12,
					},
					TokenType::Text,
				);
			}
		}

		*x -= 8;
	}

	fn draw_status_bar(&self, screen: &mut dyn Screen) {
		if !self.status_bar_enabled && !self.input_mode.shift {
			return;
		}

		let screen_width = screen.width();

		// Render status bar background
		let mut renderer = screen.renderer(RenderMode::Normal);
		renderer.erase(&Rect {
			x: 0,
			y: renderer.metrics().height(Font::Smallest),
			w: screen_width,
			h: 1,
		});

		let mut renderer = screen.renderer(RenderMode::StatusBar);
		let status_bar_rect = Rect {
			x: 0,
			y: 0,
			w: screen_width,
			h: renderer.metrics().height(Font::Smallest),
		};
		renderer.erase(&status_bar_rect);

		let mut x = screen_width - 4;

		#[cfg(feature = "dm42")]
		self.draw_battery_indicator(&mut renderer, &mut x);

		// Render alpha mode indicator
		match self.cached_status_bar_state.alpha {
			AlphaMode::UpperAlpha => self.draw_status_bar_indicator(
				&mut renderer,
				&mut x,
				"[A]",
				Font::Smallest,
				&status_bar_rect,
			),
			AlphaMode::LowerAlpha => self.draw_status_bar_indicator(
				&mut renderer,
				&mut x,
				"[a]",
				Font::Smallest,
				&status_bar_rect,
			),
			_ => (),
		}

		// Render shift indicator
		if self.cached_status_bar_state.shift {
			self.draw_status_bar_indicator(
				&mut renderer,
				&mut x,
				"⬏",
				Font::Small,
				&status_bar_rect,
			);
		}

		// Render integer radix indicator
		match self.cached_status_bar_state.integer_radix {
			8 => self.draw_status_bar_indicator(
				&mut renderer,
				&mut x,
				"Oct",
				Font::Smallest,
				&status_bar_rect,
			),
			16 => self.draw_status_bar_indicator(
				&mut renderer,
				&mut x,
				"Hex",
				Font::Smallest,
				&status_bar_rect,
			),
			_ => (),
		}

		// Render integer format indicator
		match self.cached_status_bar_state.integer_mode {
			IntegerMode::Float => (),
			IntegerMode::BigInteger => self.draw_status_bar_indicator(
				&mut renderer,
				&mut x,
				"int",
				Font::Smallest,
				&status_bar_rect,
			),
			IntegerMode::SizedInteger(size, signed) => {
				let string = if signed {
					"i".to_string()
				} else {
					"u".to_string()
				};
				let string = string + Format::new().format_bigint(&size.into()).as_str();
				self.draw_status_bar_indicator(
					&mut renderer,
					&mut x,
					&string,
					Font::Smallest,
					&status_bar_rect,
				);
			}
		}

		// Render angle mode indicator
		match self.context.angle_mode() {
			AngleUnit::Degrees => (),
			AngleUnit::Radians => self.draw_status_bar_indicator(
				&mut renderer,
				&mut x,
				"Rad",
				Font::Smallest,
				&status_bar_rect,
			),
			AngleUnit::Gradians => self.draw_status_bar_indicator(
				&mut renderer,
				&mut x,
				"Grad",
				Font::Smallest,
				&status_bar_rect,
			),
		}

		// Render menu page indicator
		if self.cached_status_bar_state.multiple_pages {
			self.draw_status_bar_indicator(
				&mut renderer,
				&mut x,
				"▴▾",
				Font::Smallest,
				&status_bar_rect,
			);
		}

		// Render current time or alternate status text
		let left_string = &self.cached_status_bar_state.left_string;
		let left_width = renderer.metrics().width(Font::Smallest, left_string) + 8;
		if 4 + left_width < x {
			renderer.draw_text(
				4,
				0,
				left_string,
				Font::Smallest,
				TokenType::Text,
				&status_bar_rect,
			);
		}
	}

	fn status_bar_size(&self, screen: &mut dyn Screen) -> i32 {
		if self.status_bar_enabled || self.input_mode.shift {
			screen.metrics().height(Font::Smallest) + 1
		} else {
			0
		}
	}

	fn render_stack_bottom_layout(
		&self,
		layout: Layout,
		screen: &mut dyn Screen,
		stack_area: &mut Rect,
	) {
		let height = layout.height(screen.metrics());
		stack_area.h -= height;
		let rect = Rect {
			x: 4,
			y: stack_area.y + stack_area.h,
			w: screen.width() - 8,
			h: height,
		};
		let clip_rect = Rect {
			x: 0,
			y: rect.y,
			w: screen.width(),
			h: rect.h,
		};

		let screen_width = screen.width();
		let mut renderer = screen.renderer(RenderMode::Normal);
		renderer.erase(&clip_rect);
		layout.render(&mut renderer, rect, &clip_rect);

		// Render a line to separate the error from the stack area
		renderer.fill(
			&Rect {
				x: 0,
				y: stack_area.y + stack_area.h,
				w: screen_width,
				h: 1,
			},
			TokenType::Error,
		);
	}

	fn render_error(&self, error: &Error, screen: &mut dyn Screen, stack_area: &mut Rect) {
		let mut items = Vec::new();
		items.push(Layout::StaticText(
			error.to_str(),
			Font::Large,
			TokenType::Error,
		));
		items.push(Layout::HorizontalSpace(4));
		let layout = Layout::Horizontal(items);
		self.render_stack_bottom_layout(layout, screen, stack_area);
	}

	fn render_number_editor(
		&self,
		editor: &NumberEditor,
		screen: &mut dyn Screen,
		stack_area: &mut Rect,
	) {
		// Show an editor prompt to the left
		let prompt_layout = Layout::StaticText("⋙ ", Font::Small, TokenType::Label);
		let prompt_width = prompt_layout.width(screen.metrics());

		// Currently editing number, format editor text
		let edit_str = editor.to_string(self.context.format());
		let layout = if let Some(layout) = edit_str.double_line_layout(
			self.base_font,
			self.base_font.smaller(),
			editor.token_type(),
			screen.metrics(),
			screen.width() - prompt_width - 8,
			Some(edit_str.len()),
		) {
			// Full editor representation is OK, display it
			layout
		} else {
			// Editor text cannot fit in the layout constaints, display floating
			// point representation instead.
			let mut items = Vec::new();
			items.push(editor.number().to_decimal().single_line_layout(
				self.context.format(),
				"",
				"",
				self.base_font,
				screen.metrics(),
				screen.width() - prompt_width - 8,
			));
			items.push(Layout::EditCursor(Font::Large));
			Layout::Horizontal(items)
		};

		// If the hex representation is enabled and valid, show it below
		let (layout, alt_layout) = Value::Number(editor.number()).add_alternate_layout(
			layout,
			self.context.format(),
			self.base_font.smaller().smaller(),
			screen.metrics(),
			screen.width() - prompt_width - 8,
			true,
			false,
		);

		let mut items = Vec::new();
		items.push(match alt_layout {
			AlternateLayoutType::Left => prompt_layout,
			_ => Layout::LeftAlign(Box::new(prompt_layout)),
		});
		items.push(layout);
		self.render_stack_bottom_layout(Layout::Horizontal(items), screen, stack_area);
	}

	fn render_location_edit(&self, screen: &mut dyn Screen, stack_area: &mut Rect) {
		let mut items = Vec::new();
		// Show use of location
		items.push(Layout::Text(
			self.location_entry.name.to_string() + " ",
			Font::Large,
			TokenType::Keyword,
		));

		// If this is a stack access, display "Stack"
		if self.location_entry.stack {
			items.push(Layout::StaticText(
				"Stack ",
				Font::Large,
				TokenType::Keyword,
			));
		}

		// Show currently edited number
		let mut value_str = String::new();
		for digit in &self.location_entry.value {
			value_str.push(char::from_u32('0' as u32 + *digit as u32).unwrap());
		}
		items.push(Layout::Text(value_str, Font::Large, TokenType::Text));
		items.push(Layout::EditCursor(Font::Large));

		items.push(Layout::HorizontalSpace(4));

		let layout = Layout::Horizontal(items);
		self.render_stack_bottom_layout(layout, screen, stack_area);
	}

	pub fn render(&mut self, screen: &mut dyn Screen) {
		if self.input_state == InputState::Menu {
			if let Some(menu) = self.menus.last() {
				menu.render(self, screen);
				return;
			}
		}

		// Check for updates to status bar and render if changed
		if self.update_status_bar_state()
			|| self.force_refresh
			|| self.force_render_on_status_update
		{
			self.draw_status_bar(screen);
		}

		// Check for updates to function key indicators and render if changed
		self.function_keys.update(self.context.format());
		if self.function_keys.update_menu_strings(&self) || self.force_refresh {
			self.function_keys.render(screen);
		}

		// Initialize stack area rectangle. It may be modified depending on extra
		// state display.
		let mut stack_area = Rect {
			x: 0,
			y: self.status_bar_size(screen),
			w: screen.width(),
			h: screen.height() - self.status_bar_size(screen) - self.function_keys.height(screen),
		};

		// If there is an error, display the message
		if let Some(error) = &self.error {
			self.render_error(error, screen, &mut stack_area);
		}

		// If there is an active editor present, render it
		let mut stack_label_offset = 0;
		match self.input_state {
			InputState::NumberInput => {
				if let Some(editor) = &self.editor {
					self.render_number_editor(editor, screen, &mut stack_area);
					stack_label_offset = 1;
				}
			}
			InputState::Recall | InputState::Store => {
				self.render_location_edit(screen, &mut stack_area)
			}
			_ => (),
		}

		// Render the stack
		if self.force_refresh {
			self.stack_renderer.borrow_mut().force_refresh();
		}
		self.stack_renderer.borrow_mut().render(
			self.context.stack(),
			&mut screen.renderer(RenderMode::Normal),
			self.context.format(),
			self.base_font,
			stack_area,
			stack_label_offset,
		);

		// Refresh the LCD contents
		screen.refresh();
		self.force_refresh = false;
		self.force_render_on_status_update = false;
	}

	pub fn update_header(&mut self, screen: &mut dyn Screen) {
		if self.force_render_on_status_update {
			self.render(screen);
		} else if self.input_state != InputState::Menu {
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

	pub fn show_menu(&mut self, menu: Menu) -> Result<()> {
		self.end_edit()?;
		self.menus.push(menu);
		self.input_state = InputState::Menu;
		Ok(())
	}

	pub fn show_system_setup_menu(&mut self) {
		#[cfg(feature = "dm42")]
		show_system_setup_menu();
	}

	pub fn wait_for_input<InputT: InputQueue>(&mut self, input: &mut InputT) -> Option<InputEvent> {
		let prev_shift = self.input_mode.shift;
		let result = input.wait(&mut self.input_mode);
		if !self.status_bar_enabled && prev_shift != self.input_mode.shift {
			self.force_render_on_status_update = true;
		}
		result
	}
}
