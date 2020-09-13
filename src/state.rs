use crate::error::{Error, Result};
use crate::font::{SANS_13, SANS_16, SANS_24};
use crate::functions::{FunctionKeyState, FunctionMenu};
use crate::input::{AlphaMode, InputEvent, InputMode};
use crate::layout::Layout;
use crate::number::{IntegerMode, Number, NumberFormat, ToNumber};
use crate::screen::{Color, Font, Rect, Screen};
use crate::stack::Stack;
use crate::time::{Now, SimpleDateTimeFormat, SimpleDateTimeToString};
use crate::value::Value;
use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use chrono::NaiveDateTime;
use core::convert::TryFrom;
use intel_dfp::Decimal;

#[cfg(feature = "dm42")]
use crate::dm42::{read_power_voltage, show_system_setup_menu, sys_free_mem, usb_powered};

/// Cached state for rendering the status bar. This is used to optimize the rendering
/// of the status bar such that it is only drawn when it is updated.
struct CachedStatusBarState {
	alpha: AlphaMode,
	shift: bool,
	integer_radix: u8,
	integer_mode: IntegerMode,
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
	name: String,
	stack: bool,
	value: Option<usize>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum InputState {
	Normal,
	Recall,
	Store,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum StatusBarLeftDisplayType {
	CurrentTime,
	FreeMemory,
}

pub struct State {
	pub stack: Stack,
	pub input_mode: InputMode,
	pub format: NumberFormat,
	pub function_keys: FunctionKeyState,
	pub default_integer_format: IntegerMode,
	pub prev_decimal_integer_mode: IntegerMode,
	pub status_bar_left_display: StatusBarLeftDisplayType,
	memory: BTreeMap<Location, Value>,
	input_state: InputState,
	location_entry: LocationEntryState,
	error: Option<Error>,
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
	fn new(name: String) -> Self {
		LocationEntryState {
			name,
			stack: false,
			value: None,
		}
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
			status_bar_left_display: StatusBarLeftDisplayType::CurrentTime,
			memory: BTreeMap::new(),
			input_state: InputState::Normal,
			location_entry: LocationEntryState::new("".to_string()),
			error: None,
			cached_status_bar_state,
			force_refresh: true,
		}
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
		NaiveDateTime::now().to_str(&SimpleDateTimeFormat::status_bar())
	}

	pub fn top<'a>(&'a self) -> Cow<'a, Value> {
		Stack::value_for_integer_mode(&self.format.integer_mode, self.stack.top())
	}

	pub fn entry<'a>(&'a self, idx: usize) -> Result<Cow<'a, Value>> {
		Ok(Stack::value_for_integer_mode(
			&self.format.integer_mode,
			self.stack.entry(idx)?,
		))
	}

	pub fn replace_entries(&mut self, count: usize, value: Value) -> Result<()> {
		let value = Stack::value_for_integer_mode(&self.format.integer_mode, &value);
		self.stack.replace_entries(count, value.into_owned())?;
		Ok(())
	}

	pub fn set_top(&mut self, value: Value) {
		let value = Stack::value_for_integer_mode(&self.format.integer_mode, &value);
		self.stack.set_top(value.into_owned());
	}

	pub fn set_entry(&mut self, offset: usize, value: Value) -> Result<()> {
		let value = Stack::value_for_integer_mode(&self.format.integer_mode, &value);
		*self.stack.entry_mut(offset)? = value.into_owned();
		Ok(())
	}

	pub fn read<'a>(&'a self, location: &Location) -> Result<Cow<'a, Value>> {
		match location {
			Location::StackOffset(offset) => self.entry(*offset),
			location => {
				if let Some(value) = self.memory.get(location) {
					Ok(Cow::Borrowed(value))
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
				self.memory.insert(location, value);
			}
		}
		Ok(())
	}

	pub fn handle_input(&mut self, input: InputEvent) -> Result<InputResult> {
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
							self.stack.exponent();
						}
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Enter => {
						self.stack.enter();
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Backspace => {
						self.stack.backspace();
					}
					InputEvent::Neg => {
						if self.stack.editing() {
							self.stack.neg()?;
						} else {
							self.set_top((-&*self.top())?);
						}
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Add => {
						self.replace_entries(2, (&*self.entry(1)? + &*self.entry(0)?)?)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Sub => {
						self.replace_entries(2, (&*self.entry(1)? - &*self.entry(0)?)?)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Mul => {
						self.replace_entries(2, (&*self.entry(1)? * &*self.entry(0)?)?)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Div => {
						self.replace_entries(2, (&*self.entry(1)? / &*self.entry(0)?)?)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Recip => {
						self.set_top((&Value::Number(1.into()) / &*self.top())?);
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Pow => {
						self.replace_entries(2, (&*self.entry(1)?).pow(&*self.entry(0)?)?)?;
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Sqrt => {
						self.set_top(self.top().sqrt()?);
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Square => {
						let top = &*self.top();
						let square = (top * top)?;
						self.set_top(square);
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Log => {
						self.set_top(self.top().log()?);
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::TenX => {
						self.set_top(self.top().exp10()?);
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Ln => {
						self.set_top(self.top().ln()?);
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::EX => {
						self.set_top(self.top().exp()?);
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Percent => {
						let factor = (&*self.entry(0)? / &Value::Number(100.into()))?;
						self.set_top((&*self.entry(1)? * &factor)?);
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Pi => {
						self.stack
							.input_value(Value::Number(Number::Decimal(Decimal::pi())));
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Sin => {
						self.set_top(self.top().sin()?);
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Cos => {
						self.set_top(self.top().cos()?);
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Tan => {
						self.set_top(self.top().tan()?);
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Asin => {
						self.set_top(self.top().asin()?);
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Acos => {
						self.set_top(self.top().acos()?);
						self.input_mode.alpha = AlphaMode::Normal;
					}
					InputEvent::Atan => {
						self.set_top(self.top().atan()?);
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
						self.location_entry = LocationEntryState::new("Rcl".to_string());
						self.stack.end_edit();
					}
					InputEvent::Sto => {
						self.input_state = InputState::Store;
						self.location_entry = LocationEntryState::new("Sto".to_string());
						self.stack.end_edit();
					}
					InputEvent::Disp => {
						self.function_keys.show_toplevel_menu(FunctionMenu::Disp);
					}
					InputEvent::Base => {
						self.function_keys.show_toplevel_menu(FunctionMenu::Base);
					}
					InputEvent::Logic => {
						self.function_keys.show_toplevel_menu(FunctionMenu::Logic);
					}
					InputEvent::Convert => {
						self.function_keys.show_toplevel_menu(FunctionMenu::Units);
					}
					InputEvent::Catalog => {
						self.function_keys.show_toplevel_menu(FunctionMenu::Catalog);
					}
					InputEvent::FunctionKey(func, _) => {
						if let Some(func) = self.function_keys.function(func) {
							func.execute(self)?;
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
						#[cfg(feature = "dm42")]
						show_system_setup_menu();
						self.force_refresh = true;
					}
					InputEvent::Assign => {
						// For testing memory constraints
						let size = usize::try_from(&*self.top().to_int()?)?;
						let mut vec: Vec<u8> = Vec::new();
						vec.resize(size, 0);
						core::mem::forget(vec);
					}
					InputEvent::Custom => {
						// Until there is a menu option, add a temporary toggle for memory usage display
						self.status_bar_left_display = match self.status_bar_left_display {
							StatusBarLeftDisplayType::CurrentTime => {
								StatusBarLeftDisplayType::FreeMemory
							}
							StatusBarLeftDisplayType::FreeMemory => {
								StatusBarLeftDisplayType::CurrentTime
							}
						};
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
					self.stack.input_value(self.read(&location)?.into_owned());
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
					self.write(location, self.top().into_owned())?;
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
		}
	}

	fn handle_location_input(&mut self, input: InputEvent) -> LocationInputResult {
		match input {
			InputEvent::Character(ch) => match ch {
				'0'..='9' => {
					self.location_entry.value = if let Some(value) = self.location_entry.value {
						Some(value * 10 + (ch as u32 - '0' as u32) as usize)
					} else {
						Some((ch as u32 - '0' as u32) as usize)
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
					} else if self.location_entry.value.is_some() {
						LocationInputResult::Invalid
					} else {
						LocationInputResult::Finished(Location::Variable(ch))
					}
				}
				_ => LocationInputResult::Invalid,
			},
			InputEvent::Enter => {
				if let Some(value) = self.location_entry.value {
					if self.location_entry.stack {
						LocationInputResult::Finished(Location::StackOffset(value))
					} else {
						LocationInputResult::Finished(Location::Integer(value))
					}
				} else {
					LocationInputResult::Invalid
				}
			}
			InputEvent::Backspace => {
				if let Some(value) = self.location_entry.value {
					let new_value = value / 10;
					self.location_entry.value = if new_value == 0 {
						None
					} else {
						Some(new_value)
					};
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

	fn draw_status_bar_indicator<ScreenT: Screen>(
		&self,
		screen: &mut ScreenT,
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

		if multiple_pages != self.cached_status_bar_state.multiple_pages {
			self.cached_status_bar_state.multiple_pages = multiple_pages;
			changed = true;
		}

		match self.status_bar_left_display {
			StatusBarLeftDisplayType::CurrentTime => {
				// Check for time updates
				if NaiveDateTime::clock_minute_updated() {
					let time_string = State::time_string();
					self.cached_status_bar_state.left_string = time_string;
					changed = true;
				}
			}
			StatusBarLeftDisplayType::FreeMemory => {
				#[cfg(feature = "dm42")]
				let free_memory = sys_free_mem().to_number().to_str() + " bytes free";
				#[cfg(not(feature = "dm42"))]
				let free_memory = "".to_string();
				if free_memory != self.cached_status_bar_state.left_string {
					self.cached_status_bar_state.left_string = free_memory;
					changed = true;
				}
			}
		}

		changed
	}

	#[cfg(feature = "dm42")]
	fn draw_battery_indicator<ScreenT: Screen>(&self, screen: &mut ScreenT, x: &mut i32) {
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

	fn draw_status_bar<ScreenT: Screen>(&self, screen: &mut ScreenT) {
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

	pub fn render<ScreenT: Screen>(&mut self, screen: &mut ScreenT) {
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
			items.push(Layout::Text(
				error.to_str().to_string(),
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
			layout.render(screen, rect, &clip_rect);

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
				self.location_entry.name.clone() + " ",
				&SANS_24,
				Color::ContentText,
			));

			// If this is a stack access, display "Stack"
			if self.location_entry.stack {
				items.push(Layout::Text(
					"Stack ".to_string(),
					&SANS_24,
					Color::ContentText,
				));
			}

			// Show currently edited number
			items.push(Layout::EditText(
				if let Some(value) = self.location_entry.value {
					value.to_number().to_str()
				} else {
					"".to_string()
				},
				&SANS_24,
				Color::ContentText,
			));

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
			layout.render(screen, rect, &clip_rect);

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
		self.stack.render(screen, &self.format, stack_area);

		// Refresh the LCD contents
		screen.refresh();
		self.force_refresh = false;
	}

	pub fn update_header<ScreenT: Screen>(&mut self, screen: &mut ScreenT) {
		// When specifically updating the header, always render the header
		self.update_status_bar_state();
		self.draw_status_bar(screen);
		screen.refresh();
	}
}
