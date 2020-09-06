use crate::font::SANS_13;
use crate::input::InputEvent;
use crate::number::{NumberFormat, NumberFormatMode};
use crate::screen::{Color, Rect, Screen};
use crate::state::State;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Function {
	Input(InputEvent),
	NormalFormat,
	RationalFormat,
	ScientificFormat,
	EngineeringFormat,
	Hex,
	Octal,
	Decimal,
}

impl Function {
	pub fn to_str(&self, state: &State) -> String {
		match self {
			Function::Input(input) => input.to_str(),
			Function::NormalFormat => {
				if state.format.mode == NumberFormatMode::Normal {
					"▪Norm".to_string()
				} else {
					"Norm".to_string()
				}
			}
			Function::RationalFormat => {
				if state.format.mode == NumberFormatMode::Rational {
					"▪Frac".to_string()
				} else {
					"Frac".to_string()
				}
			}
			Function::ScientificFormat => {
				if state.format.mode == NumberFormatMode::Scientific {
					"▪Sci".to_string()
				} else {
					"Sci".to_string()
				}
			}
			Function::EngineeringFormat => {
				if state.format.mode == NumberFormatMode::Engineering {
					"▪Eng".to_string()
				} else {
					"Eng".to_string()
				}
			}
			Function::Hex => {
				if state.format.integer_radix == 16 {
					"▪Hex".to_string()
				} else {
					"Hex".to_string()
				}
			}
			Function::Octal => {
				if state.format.integer_radix == 8 {
					"▪Oct".to_string()
				} else {
					"Oct".to_string()
				}
			}
			Function::Decimal => {
				if state.format.integer_radix == 10 {
					"▪Dec".to_string()
				} else {
					"Dec".to_string()
				}
			}
		}
	}

	pub fn execute(&self, state: &mut State) {
		match self {
			Function::Input(input) => {
				state.handle_input(*input);
			}
			Function::NormalFormat => {
				state.format.mode = NumberFormatMode::Normal;
				state.stack.end_edit();
			}
			Function::RationalFormat => {
				state.format.mode = NumberFormatMode::Rational;
				state.stack.end_edit();
			}
			Function::ScientificFormat => {
				state.format.mode = NumberFormatMode::Scientific;
				state.stack.end_edit();
			}
			Function::EngineeringFormat => {
				state.format.mode = NumberFormatMode::Engineering;
				state.stack.end_edit();
			}
			Function::Hex => {
				state.format.integer_radix = 16;
				state.stack.end_edit();
			}
			Function::Octal => {
				state.format.integer_radix = 8;
				state.stack.end_edit();
			}
			Function::Decimal => {
				state.format.integer_radix = 10;
				state.stack.end_edit();
			}
		}
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FunctionMenu {
	Disp,
	Base,
}

impl FunctionMenu {
	pub fn functions(&self) -> Vec<Option<Function>> {
		match self {
			FunctionMenu::Disp => [
				Some(Function::NormalFormat),
				Some(Function::RationalFormat),
				Some(Function::ScientificFormat),
				Some(Function::EngineeringFormat),
			]
			.to_vec(),
			FunctionMenu::Base => [
				Some(Function::Decimal),
				Some(Function::Octal),
				Some(Function::Hex),
			]
			.to_vec(),
		}
	}
}

pub struct FunctionKeyState {
	menu: Option<FunctionMenu>,
	functions: Vec<Option<Function>>,
	page: usize,
	menu_stack: Vec<(Option<FunctionMenu>, usize)>,
	quick_functions: Vec<Option<Function>>,
}

impl FunctionKeyState {
	pub fn new() -> Self {
		FunctionKeyState {
			menu: None,
			functions: Vec::new(),
			page: 0,
			menu_stack: Vec::new(),
			quick_functions: Vec::new(),
		}
	}

	pub fn function(&self, idx: u8) -> Option<Function> {
		if let Some(func) = self.functions.get(self.page * 6 + (idx as usize - 1)) {
			func.clone()
		} else {
			None
		}
	}

	fn quick_functions(&self, format: &NumberFormat) -> Vec<Option<Function>> {
		let mut result = Vec::new();
		if format.integer_radix == 16 {
			result.push(Some(Function::Input(InputEvent::Character('A'))));
			result.push(Some(Function::Input(InputEvent::Character('B'))));
			result.push(Some(Function::Input(InputEvent::Character('C'))));
			result.push(Some(Function::Input(InputEvent::Character('D'))));
			result.push(Some(Function::Input(InputEvent::Character('E'))));
			result.push(Some(Function::Input(InputEvent::Character('F'))));
		}
		result.append(&mut self.quick_functions.clone());
		result
	}

	pub fn update(&mut self, format: &NumberFormat) {
		// Update function list from current menu
		if let Some(menu) = self.menu {
			self.functions = menu.functions();
		} else {
			self.functions = self.quick_functions(format);
		}

		// Ensure current page is within bounds
		if self.functions.len() == 0 {
			self.page = 0;
		} else {
			let max_page = (self.functions.len() + 5) / 6;
			if self.page >= max_page {
				self.page = max_page - 1;
			}
		}
	}

	pub fn exit_menu(&mut self, format: &NumberFormat) {
		// Set menu state from previous stack entry and update the function list
		if let Some((menu, page)) = self.menu_stack.pop() {
			self.menu = menu;
			self.page = page;
			self.update(format);
		}
	}

	pub fn exit_all_menus(&mut self, format: &NumberFormat) {
		// Pop off the menu stack until it is empty
		while self.menu_stack.len() > 0 {
			self.exit_menu(format);
		}
	}

	pub fn show_menu(&mut self, menu: FunctionMenu) {
		self.menu_stack.push((self.menu, self.page));
		self.menu = Some(menu);
		self.functions = menu.functions();
		self.page = 0;
	}

	pub fn show_toplevel_menu(&mut self, menu: FunctionMenu) {
		self.menu_stack.clear();
		self.menu_stack.push((None, 0));
		self.menu = Some(menu);
		self.functions = menu.functions();
		self.page = 0;
	}

	pub fn render<ScreenT: Screen>(&self, screen: &mut ScreenT, state: &State) {
		let top = screen.height() - SANS_13.height;

		// Clear menu area
		screen.fill(
			Rect {
				x: 0,
				y: top - 1,
				w: screen.width(),
				h: SANS_13.height + 1,
			},
			Color::ContentBackground,
		);

		// Render each function key display
		for i in 0..6 {
			let min_x = (screen.width() - 1) * i / 6;
			let max_x = (screen.width() - 1) * (i + 1) / 6;

			// Render key background
			screen.fill(
				Rect {
					x: min_x + 1,
					y: top,
					w: max_x - min_x - 1,
					h: SANS_13.height,
				},
				Color::MenuBackground,
			);
			screen.set_pixel(min_x + 1, top, Color::ContentBackground);
			screen.set_pixel(max_x - 1, top, Color::ContentBackground);

			// Render key text if there is one
			if let Some(function) = self.function((i + 1) as u8) {
				let mut string = function.to_str(state);

				// Trim string until it fits
				let mut width = SANS_13.width(&string);
				while string.len() > 1 {
					if width > max_x - min_x {
						string.pop();
						width = SANS_13.width(&string);
					} else {
						break;
					}
				}

				// Draw key text centered in button
				SANS_13.draw(
					screen,
					(min_x + max_x) / 2 - (width / 2),
					top,
					&string,
					Color::MenuText,
				);
			}
		}
	}

	pub fn height(&self) -> i32 {
		SANS_13.height + 1
	}
}
