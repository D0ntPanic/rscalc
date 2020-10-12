use crate::dm42::catalog::{assign_menu, catalog_menu, CatalogPage};
use crate::dm42::input::InputEvent;
use crate::dm42::menu::settings_menu;
use crate::dm42::screen::{RenderMode, Screen};
use crate::dm42::state::{State, StatusBarLeftDisplayType};
use crate::dm42::unit::{unit_catalog_menu, unit_catalog_menu_of_type, unit_menu_of_type};
use rscalc_layout::font::Font;
use rscalc_layout::layout::{LayoutRenderer, Rect, TokenType};
use rscalc_math::error::Result;
use rscalc_math::format::{AlternateFormatMode, Format, IntegerMode};
use rscalc_math::functions::StackFunction;
use rscalc_math::unit::UnitType;

#[cfg(not(feature = "dm42"))]
use std::cell::RefCell;

#[cfg(feature = "dm42")]
use alloc::boxed::Box;
#[cfg(feature = "dm42")]
use alloc::string::{String, ToString};
#[cfg(feature = "dm42")]
use alloc::vec::Vec;
#[cfg(feature = "dm42")]
use core::cell::RefCell;

#[cfg(feature = "dm42")]
use crate::dm42::device::{set_time_24_hour, time_24_hour};

#[derive(PartialEq, Eq, Clone)]
#[allow(dead_code)]
pub enum Function {
	Stack(StackFunction),
	Input(InputEvent),
	SignedInteger,
	UnsignedInteger,
	CatalogPage(CatalogPage),
	AddUnitCatalogMenu,
	AddUnitCatalogPage(UnitType),
	AddInvUnitCatalogMenu,
	AddInvUnitCatalogPage(UnitType),
	ConvertUnitCatalogMenu,
	ConvertUnitCatalogPage(UnitType),
	AssignCatalogMenu(usize),
	AssignCatalogPage(usize, CatalogPage),
	AssignAddUnitCatalogMenu(usize),
	AssignAddUnitCatalogPage(usize, UnitType),
	AssignAddInvUnitCatalogMenu(usize),
	AssignAddInvUnitCatalogPage(usize, UnitType),
	AssignConvertUnitCatalogMenu(usize),
	AssignConvertUnitCatalogPage(usize, UnitType),
	AssignCatalogFunction(usize, Box<Function>),
	RemoveCustomAssign(usize),
	UnitMenu(UnitType),
	SettingsMenu,
	SystemMenu,
	Time24HourToggle,
	StatusBarLeftDisplayToggle,
	StackLabelXYZToggle,
	ShowEmptySoftKeyToggle,
	StatusBarToggle,
	FontSizeToggle,
	AlternateFormatModeToggle,
	NewMatrix,
}

impl Function {
	pub fn to_string(&self, state: &State) -> String {
		match self {
			Function::Stack(func) => func.to_string(state.context()),
			Function::Input(input) => input.to_string(),
			Function::SignedInteger => match state.context().format().integer_mode {
				IntegerMode::BigInteger | IntegerMode::SizedInteger(_, true) => "▪int".to_string(),
				_ => "int".to_string(),
			},
			Function::UnsignedInteger => match state.context().format().integer_mode {
				IntegerMode::SizedInteger(_, false) => "▪uint".to_string(),
				_ => "uint".to_string(),
			},
			Function::CatalogPage(page) => page.to_str().to_string(),
			Function::AddUnitCatalogMenu => "Unit".to_string(),
			Function::AddUnitCatalogPage(unit_type) => unit_type.to_str().to_string(),
			Function::AddInvUnitCatalogMenu => "/Unit".to_string(),
			Function::AddInvUnitCatalogPage(unit_type) => "/".to_string() + unit_type.to_str(),
			Function::ConvertUnitCatalogMenu => "▸Unit".to_string(),
			Function::ConvertUnitCatalogPage(unit_type) => "▸".to_string() + unit_type.to_str(),
			Function::AssignCatalogMenu(_) => "Catalog".to_string(),
			Function::AssignCatalogPage(_, page) => page.to_str().to_string(),
			Function::AssignAddUnitCatalogMenu(_) => "Unit".to_string(),
			Function::AssignAddUnitCatalogPage(_, unit_type) => unit_type.to_str().to_string(),
			Function::AssignAddInvUnitCatalogMenu(_) => "/Unit".to_string(),
			Function::AssignAddInvUnitCatalogPage(_, unit_type) => {
				"/".to_string() + unit_type.to_str()
			}
			Function::AssignConvertUnitCatalogMenu(_) => "▸Unit".to_string(),
			Function::AssignConvertUnitCatalogPage(_, unit_type) => {
				"▸".to_string() + unit_type.to_str()
			}
			Function::AssignCatalogFunction(_, func) => func.to_string(state),
			Function::RemoveCustomAssign(_) => "(None)".to_string(),
			Function::UnitMenu(unit_type) => unit_type.to_str().to_string(),
			Function::SettingsMenu => "Settings".to_string(),
			Function::SystemMenu => "Sys".to_string(),
			Function::Time24HourToggle => "24Hr".to_string(),
			Function::StatusBarLeftDisplayToggle => "StatusDisp".to_string(),
			Function::StackLabelXYZToggle => "xyz".to_string(),
			Function::ShowEmptySoftKeyToggle => "Empty".to_string(),
			Function::StatusBarToggle => "StatusBar".to_string(),
			Function::FontSizeToggle => "Font".to_string(),
			Function::AlternateFormatModeToggle => "Alt".to_string(),
			Function::NewMatrix => "New".to_string(),
		}
	}

	pub fn execute(&self, state: &mut State, screen: &dyn Screen) -> Result<()> {
		match self {
			Function::Stack(func) => {
				state.end_edit()?;
				func.execute(state.context_mut())?;
			}
			Function::Input(input) => {
				state.handle_input(*input, screen)?;
			}
			Function::SignedInteger => {
				state
					.function_keys_mut()
					.show_menu(FunctionMenu::SignedInteger);
			}
			Function::UnsignedInteger => {
				state
					.function_keys_mut()
					.show_menu(FunctionMenu::UnsignedInteger);
			}
			Function::CatalogPage(page) => {
				state.show_menu(page.menu(&|page| Function::CatalogPage(page), &|func| func))?;
			}
			Function::AddUnitCatalogMenu => {
				state.show_menu(unit_catalog_menu("Assign Unit", &|unit_type| {
					Function::AddUnitCatalogPage(unit_type)
				}))?;
			}
			Function::AddUnitCatalogPage(unit_type) => {
				state.show_menu(unit_catalog_menu_of_type(
					*unit_type,
					"",
					&|unit| Function::Stack(StackFunction::AddUnit(unit)),
					&|unit| Function::Stack(StackFunction::AddUnitSquared(unit)),
					&|unit| Function::Stack(StackFunction::AddUnitCubed(unit)),
				))?;
			}
			Function::AddInvUnitCatalogMenu => {
				state.show_menu(unit_catalog_menu("Assign Inverse Unit", &|unit_type| {
					Function::AddInvUnitCatalogPage(unit_type)
				}))?;
			}
			Function::AddInvUnitCatalogPage(unit_type) => {
				state.show_menu(unit_catalog_menu_of_type(
					*unit_type,
					"/",
					&|unit| Function::Stack(StackFunction::AddInvUnit(unit)),
					&|unit| Function::Stack(StackFunction::AddInvUnitSquared(unit)),
					&|unit| Function::Stack(StackFunction::AddInvUnitCubed(unit)),
				))?;
			}
			Function::ConvertUnitCatalogMenu => {
				state.show_menu(unit_catalog_menu("Convert Unit", &|unit_type| {
					Function::ConvertUnitCatalogPage(unit_type)
				}))?;
			}
			Function::ConvertUnitCatalogPage(unit_type) => {
				state.show_menu(unit_catalog_menu_of_type(
					*unit_type,
					"▸",
					&|unit| Function::Stack(StackFunction::ConvertToUnit(unit)),
					&|unit| Function::Stack(StackFunction::ConvertToUnit(unit)),
					&|unit| Function::Stack(StackFunction::ConvertToUnit(unit)),
				))?;
			}
			Function::AssignCatalogMenu(idx) => {
				state.show_menu(catalog_menu(&|page| {
					Function::AssignCatalogPage(*idx, page)
				}))?;
			}
			Function::AssignCatalogPage(idx, page) => {
				state.show_menu(page.menu(
					&|page| Function::AssignCatalogPage(*idx, page),
					&|func| match func {
						Function::AddUnitCatalogMenu => Function::AssignAddUnitCatalogMenu(*idx),
						Function::AddInvUnitCatalogMenu => {
							Function::AssignAddInvUnitCatalogMenu(*idx)
						}
						Function::ConvertUnitCatalogMenu => {
							Function::AssignConvertUnitCatalogMenu(*idx)
						}
						_ => Function::AssignCatalogFunction(*idx, Box::new(func)),
					},
				))?;
			}
			Function::AssignAddUnitCatalogMenu(idx) => {
				state.show_menu(unit_catalog_menu("Assign Unit", &|unit_type| {
					Function::AssignAddUnitCatalogPage(*idx, unit_type)
				}))?;
			}
			Function::AssignAddUnitCatalogPage(idx, unit_type) => {
				state.show_menu(unit_catalog_menu_of_type(
					*unit_type,
					"",
					&|unit| {
						Function::AssignCatalogFunction(
							*idx,
							Box::new(Function::Stack(StackFunction::AddUnit(unit))),
						)
					},
					&|unit| {
						Function::AssignCatalogFunction(
							*idx,
							Box::new(Function::Stack(StackFunction::AddUnitSquared(unit))),
						)
					},
					&|unit| {
						Function::AssignCatalogFunction(
							*idx,
							Box::new(Function::Stack(StackFunction::AddUnitCubed(unit))),
						)
					},
				))?;
			}
			Function::AssignAddInvUnitCatalogMenu(idx) => {
				state.show_menu(unit_catalog_menu("Assign Inverse Unit", &|unit_type| {
					Function::AssignAddInvUnitCatalogPage(*idx, unit_type)
				}))?;
			}
			Function::AssignAddInvUnitCatalogPage(idx, unit_type) => {
				state.show_menu(unit_catalog_menu_of_type(
					*unit_type,
					"/",
					&|unit| {
						Function::AssignCatalogFunction(
							*idx,
							Box::new(Function::Stack(StackFunction::AddInvUnit(unit))),
						)
					},
					&|unit| {
						Function::AssignCatalogFunction(
							*idx,
							Box::new(Function::Stack(StackFunction::AddInvUnitSquared(unit))),
						)
					},
					&|unit| {
						Function::AssignCatalogFunction(
							*idx,
							Box::new(Function::Stack(StackFunction::AddInvUnitCubed(unit))),
						)
					},
				))?;
			}
			Function::AssignConvertUnitCatalogMenu(idx) => {
				state.show_menu(unit_catalog_menu("Convert Unit", &|unit_type| {
					Function::AssignConvertUnitCatalogPage(*idx, unit_type)
				}))?;
			}
			Function::AssignConvertUnitCatalogPage(idx, unit_type) => {
				state.show_menu(unit_catalog_menu_of_type(
					*unit_type,
					"▸",
					&|unit| {
						Function::AssignCatalogFunction(
							*idx,
							Box::new(Function::Stack(StackFunction::ConvertToUnit(unit))),
						)
					},
					&|unit| {
						Function::AssignCatalogFunction(
							*idx,
							Box::new(Function::Stack(StackFunction::ConvertToUnit(unit))),
						)
					},
					&|unit| {
						Function::AssignCatalogFunction(
							*idx,
							Box::new(Function::Stack(StackFunction::ConvertToUnit(unit))),
						)
					},
				))?;
			}
			Function::AssignCatalogFunction(idx, func) => {
				state.set_custom_function(*idx, Some(func.as_ref().clone()));
				let mut menu = assign_menu();
				menu.set_selection(*idx);
				state.show_menu(menu)?;
			}
			Function::RemoveCustomAssign(idx) => {
				state.set_custom_function(*idx, None);
			}
			Function::UnitMenu(unit_type) => {
				let menu = unit_menu_of_type(*unit_type);
				state.show_menu(menu)?;
			}
			Function::SettingsMenu => {
				let menu = settings_menu();
				state.show_menu(menu)?;
			}
			Function::SystemMenu => {
				state.show_system_setup_menu();
			}
			Function::Time24HourToggle => {
				#[cfg(feature = "dm42")]
				{
					set_time_24_hour(!time_24_hour());
					state.context_mut().format_mut().time_24_hour = time_24_hour();
				}

				#[cfg(not(feature = "dm42"))]
				{
					let value = !state.context().format().time_24_hour;
					state.context_mut().format_mut().time_24_hour = value;
				}
			}
			Function::StatusBarLeftDisplayToggle => {
				state.set_status_bar_left_display(match state.status_bar_left_display() {
					StatusBarLeftDisplayType::CurrentTime => StatusBarLeftDisplayType::FreeMemory,
					StatusBarLeftDisplayType::FreeMemory => StatusBarLeftDisplayType::CurrentTime,
				});
			}
			Function::StackLabelXYZToggle => {
				let value = !state.context().format().stack_xyz;
				state.context_mut().format_mut().stack_xyz = value;
			}
			Function::ShowEmptySoftKeyToggle => {
				let value = !state.function_keys().show_empty();
				state.function_keys_mut().set_show_empty(value);
			}
			Function::StatusBarToggle => {
				let value = !state.status_bar_enabled();
				state.set_status_bar_enabled(value);
			}
			Function::FontSizeToggle => {
				let value = match state.base_font() {
					Font::Smallest => Font::Small,
					Font::Small => Font::Medium,
					Font::Medium => Font::Large,
					Font::Large => Font::Small,
				};
				state.set_base_font(value);
			}
			Function::AlternateFormatModeToggle => {
				let value = match state.context().format().alt_mode {
					AlternateFormatMode::Smart => AlternateFormatMode::Bottom,
					AlternateFormatMode::Bottom => AlternateFormatMode::Left,
					AlternateFormatMode::Left => AlternateFormatMode::Smart,
				};
				state.context_mut().format_mut().alt_mode = value;
			}
			Function::NewMatrix => state.function_keys_mut().show_menu(FunctionMenu::NewMatrix),
		}
		Ok(())
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FunctionMenu {
	Custom,
	Disp,
	Mode,
	Base,
	SignedInteger,
	UnsignedInteger,
	Logic,
	Stats,
	Matrix,
	NewMatrix,
}

impl FunctionMenu {
	pub fn functions(&self, state: &FunctionKeyState) -> Vec<Option<Function>> {
		match self {
			FunctionMenu::Custom => state.custom_functions.clone(),
			FunctionMenu::Disp => [
				Some(Function::Stack(StackFunction::NormalFormat)),
				Some(Function::Stack(StackFunction::RationalFormat)),
				Some(Function::Stack(StackFunction::ScientificFormat)),
				Some(Function::Stack(StackFunction::EngineeringFormat)),
				Some(Function::Stack(StackFunction::AlternateHex)),
				Some(Function::Stack(StackFunction::AlternateFloat)),
				Some(Function::Stack(StackFunction::ThousandsSeparatorOff)),
				Some(Function::Stack(StackFunction::ThousandsSeparatorOn)),
				Some(Function::Stack(StackFunction::DecimalPointPeriod)),
				Some(Function::Stack(StackFunction::DecimalPointComma)),
			]
			.to_vec(),
			FunctionMenu::Mode => [
				Some(Function::Stack(StackFunction::Degrees)),
				Some(Function::Stack(StackFunction::Radians)),
				Some(Function::Stack(StackFunction::Gradians)),
			]
			.to_vec(),
			FunctionMenu::Base => [
				Some(Function::Stack(StackFunction::Decimal)),
				Some(Function::Stack(StackFunction::Octal)),
				Some(Function::Stack(StackFunction::Hex)),
				Some(Function::Stack(StackFunction::Float)),
				Some(Function::SignedInteger),
				Some(Function::UnsignedInteger),
			]
			.to_vec(),
			FunctionMenu::SignedInteger => [
				Some(Function::Stack(StackFunction::BigInteger)),
				Some(Function::Stack(StackFunction::Signed8Bit)),
				Some(Function::Stack(StackFunction::Signed16Bit)),
				Some(Function::Stack(StackFunction::Signed32Bit)),
				Some(Function::Stack(StackFunction::Signed64Bit)),
				Some(Function::Stack(StackFunction::Signed128Bit)),
			]
			.to_vec(),
			FunctionMenu::UnsignedInteger => [
				Some(Function::Stack(StackFunction::BigInteger)),
				Some(Function::Stack(StackFunction::Unsigned8Bit)),
				Some(Function::Stack(StackFunction::Unsigned16Bit)),
				Some(Function::Stack(StackFunction::Unsigned32Bit)),
				Some(Function::Stack(StackFunction::Unsigned64Bit)),
				Some(Function::Stack(StackFunction::Unsigned128Bit)),
			]
			.to_vec(),
			FunctionMenu::Logic => [
				Some(Function::Stack(StackFunction::And)),
				Some(Function::Stack(StackFunction::Or)),
				Some(Function::Stack(StackFunction::Xor)),
				Some(Function::Stack(StackFunction::Not)),
				Some(Function::Stack(StackFunction::ShiftLeft)),
				Some(Function::Stack(StackFunction::ShiftRight)),
				Some(Function::Stack(StackFunction::RotateLeft)),
				Some(Function::Stack(StackFunction::RotateRight)),
			]
			.to_vec(),
			FunctionMenu::Stats => [
				Some(Function::Stack(StackFunction::Sum)),
				Some(Function::Stack(StackFunction::Mean)),
			]
			.to_vec(),
			FunctionMenu::Matrix => [
				Some(Function::NewMatrix),
				Some(Function::Stack(StackFunction::Transpose)),
				Some(Function::Stack(StackFunction::DotProduct)),
				Some(Function::Stack(StackFunction::CrossProduct)),
				Some(Function::Stack(StackFunction::Magnitude)),
				Some(Function::Stack(StackFunction::Normalize)),
			]
			.to_vec(),
			FunctionMenu::NewMatrix => [
				Some(Function::Stack(StackFunction::ToMatrix)),
				Some(Function::Stack(StackFunction::RowsToMatrix)),
				Some(Function::Stack(StackFunction::ColsToMatrix)),
				Some(Function::Stack(StackFunction::IdentityMatrix)),
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
	custom_functions: Vec<Option<Function>>,
	menu_strings: RefCell<Vec<String>>,
	show_empty: bool,
}

impl FunctionKeyState {
	pub fn new() -> Self {
		FunctionKeyState {
			menu: None,
			functions: Vec::new(),
			page: 0,
			menu_stack: Vec::new(),
			quick_functions: Vec::new(),
			custom_functions: Vec::new(),
			menu_strings: RefCell::new(Vec::new()),
			show_empty: false,
		}
	}

	pub fn function(&self, idx: u8) -> Option<Function> {
		if let Some(func) = self.functions.get(self.page * 6 + (idx as usize - 1)) {
			func.clone()
		} else {
			None
		}
	}

	fn quick_functions(&self, format: &Format) -> Vec<Option<Function>> {
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

	pub fn update(&mut self, format: &Format) {
		// Update function list from current menu
		if let Some(menu) = self.menu {
			self.functions = menu.functions(self);
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

	pub fn update_menu_strings(&self, state: &State) -> bool {
		let mut strings = Vec::new();
		for i in 0..6 {
			if let Some(function) = self.function((i + 1) as u8) {
				strings.push(function.to_string(state));
			} else {
				strings.push("".to_string());
			}
		}
		if strings != *self.menu_strings.borrow() {
			*self.menu_strings.borrow_mut() = strings;
			true
		} else {
			false
		}
	}

	pub fn exit_menu(&mut self, format: &Format) {
		// Set menu state from previous stack entry and update the function list
		if let Some((menu, page)) = self.menu_stack.pop() {
			self.menu = menu;
			self.page = page;
			self.update(format);
		}
	}

	pub fn show_menu(&mut self, menu: FunctionMenu) {
		self.menu_stack.push((self.menu, self.page));
		self.menu = Some(menu);
		self.functions = menu.functions(self);
		self.page = 0;
	}

	pub fn show_toplevel_menu(&mut self, menu: FunctionMenu) {
		self.menu_stack.clear();
		self.menu_stack.push((None, 0));
		self.menu = Some(menu);
		self.functions = menu.functions(self);
		self.page = 0;
	}

	pub fn prev_page(&mut self) {
		if self.page == 0 {
			let page_count = (self.functions.len() + 5) / 6;
			if page_count > 1 {
				self.page = page_count - 1;
			}
		} else {
			self.page -= 1;
		}
	}

	pub fn next_page(&mut self) {
		let page_count = (self.functions.len() + 5) / 6;
		if (self.page + 1) < page_count {
			self.page += 1;
		} else {
			self.page = 0;
		}
	}

	pub fn multiple_pages(&self) -> bool {
		self.functions.len() > 6
	}

	pub fn custom_function(&self, idx: usize) -> Option<Function> {
		if let Some(func) = self.custom_functions.get(idx) {
			func.clone()
		} else {
			None
		}
	}

	pub fn set_custom_function(&mut self, idx: usize, func: Option<Function>) {
		if let Some(dest) = self.custom_functions.get_mut(idx) {
			*dest = func;
			while let Some(None) = self.custom_functions.last() {
				self.custom_functions.pop();
			}
		} else if func.is_some() {
			while self.custom_functions.len() < idx {
				self.custom_functions.push(None);
			}
			self.custom_functions.push(func);
		}
	}

	pub fn render(&self, screen: &mut dyn Screen) {
		let top = screen.height() - screen.metrics().height(Font::Smallest);

		// Clear menu area
		let screen_width = screen.width();
		let mut renderer = screen.renderer(RenderMode::Normal);
		renderer.erase(&Rect {
			x: 0,
			y: top - 1,
			w: screen_width,
			h: renderer.metrics().height(Font::Smallest) + 1,
		});

		// Render each function key display
		let mut renderer = screen.renderer(RenderMode::FunctionKeys);
		for i in 0..6 {
			let min_x = (screen_width - 1) * i / 6;
			let max_x = (screen_width - 1) * (i + 1) / 6;

			// Render key background
			renderer.erase(&Rect {
				x: min_x + 2,
				y: top,
				w: max_x - min_x - 3,
				h: 1,
			});
			renderer.erase(&Rect {
				x: min_x + 1,
				y: top + 1,
				w: max_x - min_x - 1,
				h: renderer.metrics().height(Font::Smallest) - 1,
			});

			// Render key text if there is one
			if let Some(string) = self.menu_strings.borrow().get(i as usize) {
				let mut string = string.clone();

				// Trim string until it fits
				let mut width = renderer.metrics().width(Font::Smallest, &string);
				while string.len() > 1 {
					if width > max_x - min_x {
						string.pop();
						width = renderer.metrics().width(Font::Smallest, &string);
					} else {
						break;
					}
				}

				// Draw key text centered in button
				renderer.draw_text(
					(min_x + max_x) / 2 - (width / 2),
					top,
					&string,
					Font::Smallest,
					TokenType::Text,
					&Rect {
						x: min_x + 1,
						y: top,
						w: max_x - min_x - 1,
						h: renderer.metrics().height(Font::Smallest),
					},
				);
			}
		}
	}

	pub fn height(&self, screen: &dyn Screen) -> i32 {
		let empty = if self.menu.is_none() {
			let mut empty = true;
			for func in &self.functions {
				if func.is_some() {
					empty = false;
					break;
				}
			}
			empty
		} else {
			false
		};

		if self.show_empty || !empty {
			screen.metrics().height(Font::Smallest) + 1
		} else {
			0
		}
	}

	pub fn show_empty(&self) -> bool {
		self.show_empty
	}

	pub fn set_show_empty(&mut self, value: bool) {
		self.show_empty = value;

		// Force refresh rendering
		*self.menu_strings.borrow_mut() = Vec::new();
	}
}
