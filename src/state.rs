use crate::font::{SANS_13, SANS_16};
use crate::functions::{FunctionKeyState, FunctionMenu};
use crate::input::{AlphaMode, InputEvent, InputMode};
use crate::number::{IntegerMode, Number, NumberFormat};
use crate::screen::{Color, Font, Rect, Screen};
use crate::stack::Stack;
use crate::time::{Now, SimpleDateTimeFormat, SimpleDateTimeToString};
use crate::value::Value;
use alloc::string::{String, ToString};
use chrono::NaiveDateTime;
use intel_dfp::Decimal;

#[cfg(feature = "dm42")]
use crate::dm42::{read_power_voltage, show_system_setup_menu, usb_powered};

/// Cached state for rendering the status bar. This is used to optimize the rendering
/// of the status bar such that it is only drawn when it is updated.
struct CachedStatusBarState {
	alpha: AlphaMode,
	shift: bool,
	integer_radix: u8,
	integer_mode: IntegerMode,
	time_string: String,
}

pub struct State {
	pub stack: Stack,
	pub input_mode: InputMode,
	pub format: NumberFormat,
	pub function_keys: FunctionKeyState,
	pub default_integer_format: IntegerMode,
	pub prev_decimal_integer_mode: IntegerMode,
	cached_status_bar_state: CachedStatusBarState,
	force_refresh: bool,
}

pub enum InputResult {
	Normal,
	Suspend,
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
			time_string: State::time_string(),
		};

		State {
			stack: Stack::new(),
			input_mode,
			format,
			function_keys: FunctionKeyState::new(),
			default_integer_format: IntegerMode::BigInteger,
			prev_decimal_integer_mode: IntegerMode::Float,
			cached_status_bar_state,
			force_refresh: true,
		}
	}

	fn time_string() -> String {
		NaiveDateTime::now().to_str(&SimpleDateTimeFormat::new())
	}

	pub fn top(&self) -> Value {
		Stack::value_for_integer_mode(&self.format.integer_mode, self.stack.top())
	}

	pub fn entry(&self, idx: usize) -> Value {
		Stack::value_for_integer_mode(&self.format.integer_mode, self.stack.entry(idx))
	}

	pub fn replace_entries(&mut self, count: usize, value: Value) {
		let value = Stack::value_for_integer_mode(&self.format.integer_mode, &value);
		self.stack.replace_entries(count, value);
	}

	pub fn set_top(&mut self, value: Value) {
		let value = Stack::value_for_integer_mode(&self.format.integer_mode, &value);
		self.stack.set_top(value);
	}

	pub fn handle_input(&mut self, input: InputEvent) -> InputResult {
		match input {
			InputEvent::Character(ch) => match ch {
				'0'..='9' | 'A'..='Z' | 'a'..='z' | '.' => {
					if ch != '.' || self.format.integer_mode == IntegerMode::Float {
						self.stack.push_char(ch, &self.format);
					}
				}
				_ => (),
			},
			InputEvent::E => {
				if self.format.integer_mode == IntegerMode::Float {
					self.stack.exponent();
				}
			}
			InputEvent::Enter => {
				self.stack.enter();
			}
			InputEvent::Backspace => {
				self.stack.backspace();
			}
			InputEvent::Neg => {
				if self.stack.editing() {
					self.stack.neg();
				} else {
					if let Some(value) = -self.top() {
						self.set_top(value);
					}
				}
			}
			InputEvent::Add => {
				if self.stack.len() >= 2 {
					if let Some(value) = self.entry(1) + self.entry(0) {
						self.replace_entries(2, value);
					}
				}
			}
			InputEvent::Sub => {
				if self.stack.len() >= 2 {
					if let Some(value) = self.entry(1) - self.entry(0) {
						self.replace_entries(2, value);
					}
				}
			}
			InputEvent::Mul => {
				if self.stack.len() >= 2 {
					if let Some(value) = self.entry(1) * self.entry(0) {
						self.replace_entries(2, value);
					}
				}
			}
			InputEvent::Div => {
				if self.stack.len() >= 2 {
					if let Some(value) = self.entry(1) / self.entry(0) {
						self.replace_entries(2, value);
					}
				}
			}
			InputEvent::Recip => {
				let one: Value = 1.into();
				if let Some(value) = one / self.top() {
					self.set_top(value);
				}
			}
			InputEvent::Pow => {
				if self.stack.len() >= 2 {
					if let Some(value) = self.entry(1).pow(&self.entry(0)) {
						self.replace_entries(2, value);
					}
				}
			}
			InputEvent::Sqrt => {
				if let Some(value) = self.top().sqrt() {
					self.set_top(value);
				}
			}
			InputEvent::Square => {
				if let Some(value) = self.top() * self.top() {
					self.set_top(value);
				}
			}
			InputEvent::Log => {
				if let Some(value) = self.top().log() {
					self.set_top(value);
				}
			}
			InputEvent::TenX => {
				if let Some(value) = self.top().exp10() {
					self.set_top(value);
				}
			}
			InputEvent::Ln => {
				if let Some(value) = self.top().log() {
					self.set_top(value);
				}
			}
			InputEvent::EX => {
				if let Some(value) = self.top().exp() {
					self.set_top(value);
				}
			}
			InputEvent::Percent => {
				if self.stack.len() >= 2 {
					let one_hundred: Value = 100.into();
					if let Some(factor) = self.entry(0) / one_hundred {
						if let Some(value) = self.entry(1) * factor {
							self.set_top(value);
						}
					}
				}
			}
			InputEvent::Pi => self
				.stack
				.input_value(Value::Number(Number::Decimal(Decimal::pi()))),
			InputEvent::Sin => {
				if let Some(value) = self.top().sin() {
					self.set_top(value);
				}
			}
			InputEvent::Cos => {
				if let Some(value) = self.top().cos() {
					self.set_top(value);
				}
			}
			InputEvent::Tan => {
				if let Some(value) = self.top().tan() {
					self.set_top(value);
				}
			}
			InputEvent::Asin => {
				if let Some(value) = self.top().asin() {
					self.set_top(value);
				}
			}
			InputEvent::Acos => {
				if let Some(value) = self.top().acos() {
					self.set_top(value);
				}
			}
			InputEvent::Atan => {
				if let Some(value) = self.top().atan() {
					self.set_top(value);
				}
			}
			InputEvent::RotateDown => {
				if self.stack.len() >= 2 {
					self.stack.rotate_down();
				}
			}
			InputEvent::Swap => {
				if self.stack.len() >= 2 {
					self.stack.swap(0, 1);
				}
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
			InputEvent::FunctionKey(func, _) => {
				if let Some(func) = self.function_keys.function(func) {
					func.execute(self);
				}
			}
			InputEvent::Up => {
				self.function_keys.prev_page();
			}
			InputEvent::Down => {
				self.function_keys.next_page();
			}
			InputEvent::Setup => {
				#[cfg(feature = "dm42")]
				show_system_setup_menu();
				self.force_refresh = true;
			}
			InputEvent::Exit => {
				if self.stack.editing() {
					self.stack.end_edit();
				} else {
					self.function_keys.exit_menu(&self.format);
				}
			}
			InputEvent::Off => {
				return InputResult::Suspend;
			}
			_ => (),
		}
		InputResult::Normal
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

		// Check for time updates
		if NaiveDateTime::clock_minute_updated() {
			let time_string = State::time_string();
			self.cached_status_bar_state.time_string = time_string.clone();
			changed = true;
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

		// Render current time
		let time_string = &self.cached_status_bar_state.time_string;
		let time_width = SANS_13.width(time_string) + 8;
		if 4 + time_width < x {
			SANS_13.draw(screen, 4, 0, time_string, Color::StatusBarText);
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

		// Render the stack
		let stack_area = Rect {
			x: 0,
			y: self.status_bar_size(),
			w: screen.width(),
			h: screen.height() - self.status_bar_size() - self.function_keys.height(),
		};
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
