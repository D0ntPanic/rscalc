use crate::functions::Function;
use crate::menu::{Menu, MenuItem, MenuItemFunction, MenuItemLayout};
use crate::screen::Screen;
use crate::state::State;
use rscalc_layout::font::Font;
use rscalc_layout::layout::Layout;
use rscalc_layout::value::ValueLayout;
use rscalc_math::functions::StackFunction;
use rscalc_math::unit::{Unit, UnitType};

#[cfg(feature = "dm42")]
use alloc::boxed::Box;
#[cfg(feature = "dm42")]
use alloc::string::ToString;
#[cfg(feature = "dm42")]
use alloc::vec::Vec;

fn value_layout() -> Box<dyn Fn(&State, &dyn Screen) -> Layout> {
	Box::new(|state, screen| {
		if let Ok(value) = state.context().top() {
			let value_layout = value.layout(
				&state.context().format(),
				Font::Large,
				screen.metrics(),
				screen.width(),
			);
			let mut layout_items = Vec::new();
			layout_items.push(Layout::HorizontalRule);
			layout_items.push(value_layout);
			Layout::Vertical(layout_items)
		} else {
			Layout::HorizontalSpace(0)
		}
	})
}

pub fn unit_menu() -> Menu {
	let mut items = Vec::new();
	for item in &[
		("Angle", UnitType::Angle),
		("Area", UnitType::Area),
		("Distance", UnitType::Distance),
		("Energy", UnitType::Energy),
		("Force", UnitType::Force),
		("Mass", UnitType::Mass),
		("Power", UnitType::Power),
		("Pressure", UnitType::Pressure),
		("Temp", UnitType::Temperature),
		("Time", UnitType::Time),
		("Volume", UnitType::Volume),
	] {
		items.push(MenuItem {
			layout: MenuItemLayout::Static(MenuItem::static_string_layout(item.0)),
			function: MenuItemFunction::InMenuActionWithDelete(
				Function::UnitMenu(item.1),
				Function::Stack(StackFunction::ClearUnits),
			),
		});
	}
	let mut menu = Menu::new_with_bottom("Units", items, value_layout());
	menu.set_columns(3);
	menu
}

pub fn unit_menu_of_type(unit_type: UnitType) -> Menu {
	let mut items = Vec::new();
	for unit in unit_type.units() {
		if unit.unit_type() == unit_type {
			let function = MenuItemFunction::ConversionAction(
				Function::Stack(StackFunction::AddUnit(*unit)),
				Function::Stack(StackFunction::AddInvUnit(*unit)),
				Function::Stack(StackFunction::ConvertToUnit(*unit)),
			);
			match unit_type {
				UnitType::Volume => items.push(MenuItem {
					layout: MenuItemLayout::Static(MenuItem::static_string_layout_small(
						unit.to_str(),
					)),
					function,
				}),
				_ => items.push(MenuItem {
					layout: MenuItemLayout::Static(MenuItem::static_string_layout(unit.to_str())),
					function,
				}),
			}
		} else {
			match unit_type {
				UnitType::Area => {
					items.push(MenuItem {
						layout: MenuItemLayout::Static(MenuItem::string_layout(
							unit.to_str().to_string() + "²",
						)),
						function: MenuItemFunction::ConversionAction(
							Function::Stack(StackFunction::AddUnitSquared(*unit)),
							Function::Stack(StackFunction::AddInvUnitSquared(*unit)),
							Function::Stack(StackFunction::ConvertToUnit(*unit)),
						),
					});
				}
				UnitType::Volume => {
					items.push(MenuItem {
						layout: MenuItemLayout::Static(MenuItem::string_layout_small(
							unit.to_str().to_string() + "³",
						)),
						function: MenuItemFunction::ConversionAction(
							Function::Stack(StackFunction::AddUnitCubed(*unit)),
							Function::Stack(StackFunction::AddInvUnitCubed(*unit)),
							Function::Stack(StackFunction::ConvertToUnit(*unit)),
						),
					});
				}
				_ => unreachable!(),
			}
		}
	}

	let columns = core::cmp::min((items.len() + 3) / 4, 4);
	let mut menu = Menu::new_with_bottom(
		&(unit_type.to_str().to_string() + " (×,÷ Assign; x≷y Convert)"),
		items,
		value_layout(),
	);
	menu.set_columns(columns);
	menu
}

pub fn unit_catalog_menu(title: &str, func: &dyn Fn(UnitType) -> Function) -> Menu {
	let mut items = Vec::new();
	for item in &[
		("Angle", UnitType::Angle),
		("Area", UnitType::Area),
		("Distance", UnitType::Distance),
		("Energy", UnitType::Energy),
		("Force", UnitType::Force),
		("Mass", UnitType::Mass),
		("Power", UnitType::Power),
		("Pressure", UnitType::Pressure),
		("Temperature", UnitType::Temperature),
		("Time", UnitType::Time),
		("Volume", UnitType::Volume),
	] {
		items.push(MenuItem {
			layout: MenuItemLayout::Static(MenuItem::static_string_layout(item.0)),
			function: MenuItemFunction::InMenuAction(func(item.1)),
		});
	}
	let mut menu = Menu::new(title, items);
	menu.set_columns(2);
	menu
}

pub fn unit_catalog_menu_of_type(
	unit_type: UnitType,
	prefix: &str,
	raw_func: &dyn Fn(Unit) -> Function,
	squared_func: &dyn Fn(Unit) -> Function,
	cubed_func: &dyn Fn(Unit) -> Function,
) -> Menu {
	let mut items = Vec::new();
	for unit in unit_type.units() {
		if unit.unit_type() == unit_type {
			let function = MenuItemFunction::Action(raw_func(*unit));
			match unit_type {
				UnitType::Volume => items.push(MenuItem {
					layout: MenuItemLayout::Static(MenuItem::string_layout_small(
						prefix.to_string() + unit.to_str(),
					)),
					function,
				}),
				_ => items.push(MenuItem {
					layout: MenuItemLayout::Static(MenuItem::string_layout(
						prefix.to_string() + unit.to_str(),
					)),
					function,
				}),
			}
		} else {
			match unit_type {
				UnitType::Area => {
					items.push(MenuItem {
						layout: MenuItemLayout::Static(MenuItem::string_layout(
							prefix.to_string() + unit.to_str() + "²",
						)),
						function: MenuItemFunction::Action(squared_func(*unit)),
					});
				}
				UnitType::Volume => {
					items.push(MenuItem {
						layout: MenuItemLayout::Static(MenuItem::string_layout_small(
							prefix.to_string() + unit.to_str() + "³",
						)),
						function: MenuItemFunction::Action(cubed_func(*unit)),
					});
				}
				_ => unreachable!(),
			}
		}
	}

	let columns = (items.len() + 7) / 8;
	let mut menu = Menu::new(unit_type.to_str(), items);
	menu.set_columns(columns);
	menu
}
