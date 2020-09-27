use crate::functions::Function;
use crate::menu::{Menu, MenuItem, MenuItemFunction, MenuItemLayout};
use alloc::boxed::Box;
use alloc::vec::Vec;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CatalogPage {
	Constants,
	Time,
	Transcendental,
	Units,
}

impl CatalogPage {
	pub fn to_str(&self) -> &'static str {
		match self {
			CatalogPage::Constants => "Constants",
			CatalogPage::Time => "Time",
			CatalogPage::Transcendental => "Transcendental",
			CatalogPage::Units => "Units",
		}
	}

	pub fn menu(
		&self,
		_page: &dyn Fn(CatalogPage) -> Function,
		func: &dyn Fn(Function) -> Function,
	) -> Menu {
		match self {
			CatalogPage::Constants => constant_catalog_menu(func),
			CatalogPage::Time => time_catalog_menu(func),
			CatalogPage::Transcendental => transcendental_catalog_menu(func),
			CatalogPage::Units => main_unit_catalog_menu(func),
		}
	}
}

fn create_parent_items(items: &[(&'static str, Function)]) -> Vec<MenuItem> {
	let mut result = Vec::new();
	for item in items {
		result.push(MenuItem {
			layout: MenuItemLayout::Static(MenuItem::static_string_layout(item.0)),
			function: MenuItemFunction::InMenuAction(item.1.clone()),
		});
	}
	result
}

fn create_action_items(items: &[(&'static str, Function)]) -> Vec<MenuItem> {
	let mut result = Vec::new();
	for item in items {
		result.push(MenuItem {
			layout: MenuItemLayout::Static(MenuItem::static_string_layout(item.0)),
			function: MenuItemFunction::Action(item.1.clone()),
		});
	}
	result
}

pub fn catalog_menu(func: &dyn Fn(CatalogPage) -> Function) -> Menu {
	Menu::new(
		"Catalog",
		create_parent_items(&[
			("Constants", func(CatalogPage::Constants)),
			("Time", func(CatalogPage::Time)),
			("Transcendental", func(CatalogPage::Transcendental)),
			("Units", func(CatalogPage::Units)),
		]),
	)
}

fn constant_catalog_menu(func: &dyn Fn(Function) -> Function) -> Menu {
	Menu::new(
		"Constants",
		create_action_items(&[("c - Speed of Light", func(Function::SpeedOfLight))]),
	)
}

fn time_catalog_menu(func: &dyn Fn(Function) -> Function) -> Menu {
	Menu::new(
		"Time",
		create_action_items(&[
			("Now", func(Function::Now)),
			("Date", func(Function::Date)),
			("Time", func(Function::Time)),
		]),
	)
}

fn transcendental_catalog_menu(func: &dyn Fn(Function) -> Function) -> Menu {
	let mut menu = Menu::new(
		"Transcendental",
		create_action_items(&[
			("log", func(Function::Log)),
			("10ˣ", func(Function::Exp10)),
			("ln", func(Function::Ln)),
			("eˣ", func(Function::Exp)),
			("sin", func(Function::Sin)),
			("cos", func(Function::Cos)),
			("tan", func(Function::Tan)),
			("sinh", func(Function::Sinh)),
			("cosh", func(Function::Cosh)),
			("tanh", func(Function::Tanh)),
			("asin", func(Function::Asin)),
			("acos", func(Function::Acos)),
			("atan", func(Function::Atan)),
			("asinh", func(Function::Asinh)),
			("acosh", func(Function::Acosh)),
			("atanh", func(Function::Atanh)),
		]),
	);
	menu.set_columns(2);
	menu
}

fn main_unit_catalog_menu(func: &dyn Fn(Function) -> Function) -> Menu {
	Menu::new(
		"Units",
		create_parent_items(&[
			("Assign Unit", func(Function::AddUnitCatalogMenu)),
			("Assign Inverse Unit", func(Function::AddInvUnitCatalogMenu)),
			("Convert Unit", func(Function::ConvertUnitCatalogMenu)),
		]),
	)
}

pub fn assign_menu() -> Menu {
	let mut items = Vec::new();
	for i in 0..18 {
		items.push(MenuItem {
			layout: MenuItemLayout::Dynamic(Box::new(move |state, _screen| {
				if let Some(func) = state.custom_function(i) {
					MenuItem::string_layout(func.to_string(state))
				} else {
					MenuItem::static_string_layout("(None)")
				}
			})),
			function: MenuItemFunction::InMenuActionWithDelete(
				Function::AssignCatalogMenu(i),
				Function::RemoveCustomAssign(i),
			),
		});
	}
	let mut menu = Menu::new("Assign Custom Functions", items);
	menu.set_columns(3);
	menu
}
