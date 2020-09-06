use crate::functions::{FunctionKeyState, FunctionMenu};
use crate::input::{AlphaMode, InputEvent, InputMode};
use crate::number::{Number, NumberFormat};
use crate::stack::Stack;
use intel_dfp::Decimal;

#[cfg(feature = "dm42")]
use crate::dm42::show_system_setup_menu;

pub struct State {
	pub stack: Stack,
	pub input_mode: InputMode,
	pub format: NumberFormat,
	pub function_keys: FunctionKeyState,
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
		}
	}

	pub fn handle_input(&mut self, input: InputEvent) -> InputResult {
		match input {
			InputEvent::Character(ch) => match ch {
				'0'..='9' | 'A'..='Z' | 'a'..='z' | '.' => {
					self.stack.push_char(ch, &self.format);
				}
				_ => (),
			},
			InputEvent::E => {
				self.stack.exponent();
			}
			InputEvent::Enter => {
				self.stack.enter();
			}
			InputEvent::Backspace => {
				self.stack.backspace();
			}
			InputEvent::Neg => {
				self.stack.neg();
			}
			InputEvent::Add => {
				if self.stack.len() >= 2 {
					let value = self.stack.entry(1) + self.stack.entry(0);
					self.stack.replace_entries(2, value);
				}
			}
			InputEvent::Sub => {
				if self.stack.len() >= 2 {
					let value = self.stack.entry(1) - self.stack.entry(0);
					self.stack.replace_entries(2, value);
				}
			}
			InputEvent::Mul => {
				if self.stack.len() >= 2 {
					let value = self.stack.entry(1) * self.stack.entry(0);
					self.stack.replace_entries(2, value);
				}
			}
			InputEvent::Div => {
				if self.stack.len() >= 2 {
					let value = self.stack.entry(1) / self.stack.entry(0);
					self.stack.replace_entries(2, value);
				}
			}
			InputEvent::Recip => {
				let one: Number = 1.into();
				let value = &one / self.stack.top();
				self.stack.set_top(value);
			}
			InputEvent::Pow => {
				if self.stack.len() >= 2 {
					let value = self.stack.entry(1).pow(self.stack.entry(0));
					self.stack.replace_entries(2, value);
				}
			}
			InputEvent::Sqrt => {
				let value = self.stack.top().sqrt();
				self.stack.set_top(value);
			}
			InputEvent::Square => {
				let value = self.stack.top() * self.stack.top();
				self.stack.set_top(value);
			}
			InputEvent::Log => {
				let value = self.stack.top().log();
				self.stack.set_top(value);
			}
			InputEvent::TenX => {
				let value = self.stack.top().exp10();
				self.stack.set_top(value);
			}
			InputEvent::Ln => {
				let value = self.stack.top().log();
				self.stack.set_top(value);
			}
			InputEvent::EX => {
				let value = self.stack.top().exp();
				self.stack.set_top(value);
			}
			InputEvent::Percent => {
				if self.stack.len() >= 2 {
					let one_hundred: Number = 100.into();
					let value = self.stack.entry(1) * &(self.stack.entry(0) / &one_hundred);
					self.stack.set_top(value);
				}
			}
			InputEvent::Pi => self.stack.input_num(Number::Decimal(Decimal::pi())),
			InputEvent::Sin => {
				let value = self.stack.top().sin();
				self.stack.set_top(value);
			}
			InputEvent::Cos => {
				let value = self.stack.top().cos();
				self.stack.set_top(value);
			}
			InputEvent::Tan => {
				let value = self.stack.top().tan();
				self.stack.set_top(value);
			}
			InputEvent::Asin => {
				let value = self.stack.top().asin();
				self.stack.set_top(value);
			}
			InputEvent::Acos => {
				let value = self.stack.top().acos();
				self.stack.set_top(value);
			}
			InputEvent::Atan => {
				let value = self.stack.top().atan();
				self.stack.set_top(value);
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
}
