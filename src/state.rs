use crate::font::{SANS_13, SANS_16};
use crate::functions::{FunctionKeyState, FunctionMenu};
use crate::input::{AlphaMode, InputEvent, InputMode};
use crate::number::{IntegerMode, Number, NumberFormat};
use crate::screen::{Color, Font, Rect, Screen};
use crate::stack::Stack;
use alloc::string::ToString;
use intel_dfp::Decimal;

#[cfg(feature = "dm42")]
use crate::dm42::{read_power_voltage, show_system_setup_menu, usb_powered};

pub struct State {
	pub stack: Stack,
	pub input_mode: InputMode,
	pub format: NumberFormat,
	pub function_keys: FunctionKeyState,
	pub default_integer_format: IntegerMode,
	pub prev_decimal_integer_mode: IntegerMode,
}

pub enum InputResult {
	Normal,
	Suspend,
}

impl State {
	pub fn new() -> Self {
		State {
			stack: Stack::new(),
			input_mode: InputMode {
				alpha: AlphaMode::Normal,
				shift: false,
			},
			format: NumberFormat::new(),
			function_keys: FunctionKeyState::new(),
			default_integer_format: IntegerMode::BigInteger,
			prev_decimal_integer_mode: IntegerMode::Float,
		}
	}

	pub fn top(&self) -> Number {
		Stack::value_for_integer_mode(&self.format.integer_mode, self.stack.top())
	}

	pub fn entry(&self, idx: usize) -> Number {
		Stack::value_for_integer_mode(&self.format.integer_mode, self.stack.entry(idx))
	}

	pub fn replace_entries(&mut self, count: usize, value: Number) {
		let value = Stack::value_for_integer_mode(&self.format.integer_mode, &value);
		self.stack.replace_entries(count, value);
	}

	pub fn set_top(&mut self, value: Number) {
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
					let value = -self.top();
					self.set_top(value);
				}
			}
			InputEvent::Add => {
				if self.stack.len() >= 2 {
					let value = self.entry(1) + self.entry(0);
					self.replace_entries(2, value);
				}
			}
			InputEvent::Sub => {
				if self.stack.len() >= 2 {
					let value = self.entry(1) - self.entry(0);
					self.replace_entries(2, value);
				}
			}
			InputEvent::Mul => {
				if self.stack.len() >= 2 {
					let value = self.entry(1) * self.entry(0);
					self.replace_entries(2, value);
				}
			}
			InputEvent::Div => {
				if self.stack.len() >= 2 {
					let value = self.entry(1) / self.entry(0);
					self.replace_entries(2, value);
				}
			}
			InputEvent::Recip => {
				let one: Number = 1.into();
				let value = one / self.top();
				self.set_top(value);
			}
			InputEvent::Pow => {
				if self.stack.len() >= 2 {
					let value = self.entry(1).pow(&self.entry(0));
					self.replace_entries(2, value);
				}
			}
			InputEvent::Sqrt => {
				let value = self.top().sqrt();
				self.set_top(value);
			}
			InputEvent::Square => {
				let value = self.top() * self.top();
				self.set_top(value);
			}
			InputEvent::Log => {
				let value = self.top().log();
				self.set_top(value);
			}
			InputEvent::TenX => {
				let value = self.top().exp10();
				self.set_top(value);
			}
			InputEvent::Ln => {
				let value = self.top().log();
				self.set_top(value);
			}
			InputEvent::EX => {
				let value = self.top().exp();
				self.set_top(value);
			}
			InputEvent::Percent => {
				if self.stack.len() >= 2 {
					let one_hundred: Number = 100.into();
					let value = self.entry(1) * (self.entry(0) / one_hundred);
					self.set_top(value);
				}
			}
			InputEvent::Pi => self.stack.input_num(Number::Decimal(Decimal::pi())),
			InputEvent::Sin => {
				let value = self.top().sin();
				self.set_top(value);
			}
			InputEvent::Cos => {
				let value = self.top().cos();
				self.set_top(value);
			}
			InputEvent::Tan => {
				let value = self.top().tan();
				self.set_top(value);
			}
			InputEvent::Asin => {
				let value = self.top().asin();
				self.set_top(value);
			}
			InputEvent::Acos => {
				let value = self.top().acos();
				self.set_top(value);
			}
			InputEvent::Atan => {
				let value = self.top().atan();
				self.set_top(value);
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
			}
			InputEvent::Exit => {
				self.function_keys.exit_menu(&self.format);
			}
			InputEvent::Off => {
				return InputResult::Suspend;
			}
			_ => (),
		}
		InputResult::Normal
	}

	fn draw_header_indicator<ScreenT: Screen>(
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

	fn draw_header<ScreenT: Screen>(&self, screen: &mut ScreenT, mode: &InputMode) {
		// Render top bar
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
		{
			// Render battery indicator
			let usb = usb_powered();
			let voltage = read_power_voltage();
			let mut fill = 5 - ((2940 - voltage as i32) / 150);
			if fill < 0 {
				fill = 0;
			} else if fill > 5 {
				fill = 5;
			}

			x -= 22;
			screen.fill(
				Rect {
					x,
					y: 3,
					w: 20,
					h: SANS_13.height - 6,
				},
				Color::StatusBarText,
			);
			screen.fill(
				Rect {
					x: x + 2,
					y: 5,
					w: 16,
					h: SANS_13.height - 10,
				},
				Color::StatusBarBackground,
			);
			screen.set_pixel(x, 3, Color::StatusBarBackground);
			screen.set_pixel(x + 19, 3, Color::StatusBarBackground);
			screen.set_pixel(x, SANS_13.height - 4, Color::StatusBarBackground);
			screen.set_pixel(x + 19, SANS_13.height - 4, Color::StatusBarBackground);
			screen.fill(
				Rect {
					x: x + 20,
					y: 7,
					w: 2,
					h: SANS_13.height - 14,
				},
				Color::StatusBarText,
			);

			if usb {
				for i in 6..SANS_13.height - 6 {
					if i & 1 == 0 {
						screen.draw_bits(x + 3, i, 0x1555, 14, Color::StatusBarText);
					} else {
						screen.draw_bits(x + 3, i, 0x2aaa, 14, Color::StatusBarText);
					}
				}
			} else {
				for i in 0..fill {
					screen.fill(
						Rect {
							x: x + i * 3 + 3,
							y: 6,
							w: 2,
							h: SANS_13.height - 12,
						},
						Color::StatusBarText,
					);
				}
			}

			x -= 8;
		}

		// Render alpha mode indicator
		match mode.alpha {
			AlphaMode::UpperAlpha => self.draw_header_indicator(screen, &mut x, "[A]", &SANS_13),
			AlphaMode::LowerAlpha => self.draw_header_indicator(screen, &mut x, "[a]", &SANS_13),
			_ => (),
		}

		// Render shift indicator
		if mode.shift {
			self.draw_header_indicator(screen, &mut x, "⬏", &SANS_16);
		}

		// Render integer radix indicator
		match self.format.integer_radix {
			8 => self.draw_header_indicator(screen, &mut x, "Oct", &SANS_13),
			16 => self.draw_header_indicator(screen, &mut x, "Hex", &SANS_13),
			_ => (),
		}

		// Render integer format indicator
		match self.format.integer_mode {
			IntegerMode::Float => (),
			IntegerMode::BigInteger => self.draw_header_indicator(screen, &mut x, "int", &SANS_13),
			IntegerMode::SizedInteger(size, signed) => {
				let string = if signed {
					"i".to_string()
				} else {
					"u".to_string()
				};
				let string = string + NumberFormat::new().format_bigint(&size.into()).as_str();
				self.draw_header_indicator(screen, &mut x, &string, &SANS_13);
			}
		}
	}

	fn header_size(&self) -> i32 {
		SANS_13.height + 1
	}

	pub fn render<ScreenT: Screen>(&mut self, screen: &mut ScreenT) {
		screen.clear();
		self.draw_header(screen, &self.input_mode);

		self.function_keys.update(&self.format);
		self.function_keys.render(screen, &self);

		let stack_area = Rect {
			x: 0,
			y: self.header_size(),
			w: screen.width(),
			h: screen.height() - self.header_size() - self.function_keys.height(),
		};

		self.stack.render(screen, &self.format, stack_area);
		screen.refresh();
	}
}
