use crate::error::{Error, Result};
use crate::functions::Function;
use crate::layout::Layout;
use crate::menu::{Menu, MenuItem, MenuItemFunction};
use crate::number::{Number, ToNumber};
use crate::screen::Screen;
use crate::state::State;
use crate::storage::{DeserializeInput, SerializeOutput, StorageObject, StorageRefSerializer};
use crate::value::Value;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use intel_dfp::Decimal;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TimeUnit {
	Nanoseconds,
	Microseconds,
	Milliseconds,
	Seconds,
	Minutes,
	Hours,
	Days,
	Years,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum DistanceUnit {
	Nanometers,
	Micrometers,
	Millimeters,
	Centimeters,
	Meters,
	Kilometers,
	Inches,
	Feet,
	Yards,
	Miles,
	NauticalMiles,
	AstronomicalUnits,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum AngleUnit {
	Degrees,
	Radians,
	Gradians,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Unit {
	Time(TimeUnit),
	Distance(DistanceUnit),
	Angle(AngleUnit),
}

impl TimeUnit {
	pub fn to_str(&self) -> String {
		match self {
			TimeUnit::Nanoseconds => "ns".to_string(),
			TimeUnit::Microseconds => "μs".to_string(),
			TimeUnit::Milliseconds => "ms".to_string(),
			TimeUnit::Seconds => "sec".to_string(),
			TimeUnit::Minutes => "min".to_string(),
			TimeUnit::Hours => "hr".to_string(),
			TimeUnit::Days => "day".to_string(),
			TimeUnit::Years => "yr".to_string(),
		}
	}
}

impl DistanceUnit {
	pub fn to_str(&self) -> String {
		match self {
			DistanceUnit::Nanometers => "nm".to_string(),
			DistanceUnit::Micrometers => "μm".to_string(),
			DistanceUnit::Millimeters => "mm".to_string(),
			DistanceUnit::Centimeters => "cm".to_string(),
			DistanceUnit::Meters => "m".to_string(),
			DistanceUnit::Kilometers => "km".to_string(),
			DistanceUnit::Inches => "in".to_string(),
			DistanceUnit::Feet => "ft".to_string(),
			DistanceUnit::Yards => "yd".to_string(),
			DistanceUnit::Miles => "mi".to_string(),
			DistanceUnit::NauticalMiles => "nmi".to_string(),
			DistanceUnit::AstronomicalUnits => "au".to_string(),
		}
	}
}

impl AngleUnit {
	pub fn to_str(&self) -> String {
		match self {
			AngleUnit::Degrees => "°".to_string(),
			AngleUnit::Radians => "rad".to_string(),
			AngleUnit::Gradians => "grad".to_string(),
		}
	}
}

impl Unit {
	pub fn to_str(&self) -> String {
		match self {
			Unit::Time(unit) => unit.to_str(),
			Unit::Distance(unit) => unit.to_str(),
			Unit::Angle(unit) => unit.to_str(),
		}
	}

	pub fn to_u16(&self) -> u16 {
		match self {
			Unit::Time(TimeUnit::Nanoseconds) => 0x0000,
			Unit::Time(TimeUnit::Microseconds) => 0x0001,
			Unit::Time(TimeUnit::Milliseconds) => 0x0002,
			Unit::Time(TimeUnit::Seconds) => 0x0003,
			Unit::Time(TimeUnit::Minutes) => 0x0004,
			Unit::Time(TimeUnit::Hours) => 0x0005,
			Unit::Time(TimeUnit::Days) => 0x0006,
			Unit::Time(TimeUnit::Years) => 0x0007,
			Unit::Distance(DistanceUnit::Nanometers) => 0x0100,
			Unit::Distance(DistanceUnit::Micrometers) => 0x0101,
			Unit::Distance(DistanceUnit::Millimeters) => 0x0102,
			Unit::Distance(DistanceUnit::Centimeters) => 0x0103,
			Unit::Distance(DistanceUnit::Meters) => 0x0104,
			Unit::Distance(DistanceUnit::Kilometers) => 0x0105,
			Unit::Distance(DistanceUnit::Inches) => 0x0110,
			Unit::Distance(DistanceUnit::Feet) => 0x0111,
			Unit::Distance(DistanceUnit::Yards) => 0x0112,
			Unit::Distance(DistanceUnit::Miles) => 0x0113,
			Unit::Distance(DistanceUnit::NauticalMiles) => 0x0114,
			Unit::Distance(DistanceUnit::AstronomicalUnits) => 0x0120,
			Unit::Angle(AngleUnit::Degrees) => 0x0200,
			Unit::Angle(AngleUnit::Radians) => 0x0201,
			Unit::Angle(AngleUnit::Gradians) => 0x0202,
		}
	}

	pub fn from_u16(value: u16) -> Option<Self> {
		match value {
			0x0000 => Some(Unit::Time(TimeUnit::Nanoseconds)),
			0x0001 => Some(Unit::Time(TimeUnit::Microseconds)),
			0x0002 => Some(Unit::Time(TimeUnit::Milliseconds)),
			0x0003 => Some(Unit::Time(TimeUnit::Seconds)),
			0x0004 => Some(Unit::Time(TimeUnit::Minutes)),
			0x0005 => Some(Unit::Time(TimeUnit::Hours)),
			0x0006 => Some(Unit::Time(TimeUnit::Days)),
			0x0007 => Some(Unit::Time(TimeUnit::Years)),
			0x0100 => Some(Unit::Distance(DistanceUnit::Nanometers)),
			0x0101 => Some(Unit::Distance(DistanceUnit::Micrometers)),
			0x0102 => Some(Unit::Distance(DistanceUnit::Millimeters)),
			0x0103 => Some(Unit::Distance(DistanceUnit::Centimeters)),
			0x0104 => Some(Unit::Distance(DistanceUnit::Meters)),
			0x0105 => Some(Unit::Distance(DistanceUnit::Kilometers)),
			0x0110 => Some(Unit::Distance(DistanceUnit::Inches)),
			0x0111 => Some(Unit::Distance(DistanceUnit::Feet)),
			0x0112 => Some(Unit::Distance(DistanceUnit::Yards)),
			0x0113 => Some(Unit::Distance(DistanceUnit::Miles)),
			0x0114 => Some(Unit::Distance(DistanceUnit::NauticalMiles)),
			0x0120 => Some(Unit::Distance(DistanceUnit::AstronomicalUnits)),
			0x0200 => Some(Unit::Angle(AngleUnit::Degrees)),
			0x0201 => Some(Unit::Angle(AngleUnit::Radians)),
			0x0202 => Some(Unit::Angle(AngleUnit::Gradians)),
			_ => None,
		}
	}
}

pub trait UnitConversion: Eq {
	/// Gets the conversion factor from this unit to the standard unit of this type
	fn multiplier_to_standard(&self) -> Number;

	/// Converts a value from this unit to a target unit
	fn to_unit(&self, value: &Number, target_unit: &Self) -> Number {
		if self == target_unit {
			return value.clone();
		}
		let secs = value * &self.multiplier_to_standard();
		secs / target_unit.multiplier_to_standard()
	}

	/// Converts a value from this unit to a target unit when the unit is inverted (for
	/// example, the seconds in meters per second)
	fn to_unit_inv(&self, value: &Number, target_unit: &Self) -> Number {
		target_unit.to_unit(value, self)
	}

	/// Converts a value from this unit to a target unit with the unit raised to
	/// the given power
	fn to_unit_with_power(&self, value: &Number, target_unit: &Self, power: i32) -> Number {
		if self == target_unit {
			return value.clone();
		}
		if power < 0 {
			let mut result = value.clone();
			for _ in 0..-power {
				result = self.to_unit_inv(&result, target_unit);
			}
			result
		} else if power > 0 {
			let mut result = value.clone();
			for _ in 0..power {
				result = self.to_unit(&result, target_unit);
			}
			result
		} else {
			value.clone()
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum UnitType {
	Time,
	Distance,
	Angle,
}

impl UnitType {
	pub fn to_str(&self) -> String {
		match self {
			UnitType::Time => "Time".to_string(),
			UnitType::Distance => "Dist".to_string(),
			UnitType::Angle => "Angle".to_string(),
		}
	}
}

#[derive(Clone)]
pub struct CompositeUnit {
	pub units: BTreeMap<UnitType, (Unit, i32)>,
}

impl UnitConversion for TimeUnit {
	fn multiplier_to_standard(&self) -> Number {
		match self {
			TimeUnit::Nanoseconds => 1.to_number() / 1_000_000_000.to_number(),
			TimeUnit::Microseconds => 1.to_number() / 1_000_000.to_number(),
			TimeUnit::Milliseconds => 1.to_number() / 1000.to_number(),
			TimeUnit::Seconds => 1.to_number(),
			TimeUnit::Minutes => 60.to_number(),
			TimeUnit::Hours => 3600.to_number(),
			TimeUnit::Days => (3600 * 24).to_number(),
			TimeUnit::Years => 31556952.to_number(), // Average length of year over 400 years
		}
	}
}

impl UnitConversion for DistanceUnit {
	fn multiplier_to_standard(&self) -> Number {
		match self {
			DistanceUnit::Nanometers => 1.to_number() / 1_000_000_000.to_number(),
			DistanceUnit::Micrometers => 1.to_number() / 1_000_000.to_number(),
			DistanceUnit::Millimeters => 1.to_number() / 1000.to_number(),
			DistanceUnit::Centimeters => 1.to_number() / 100.to_number(),
			DistanceUnit::Meters => 1.to_number(),
			DistanceUnit::Kilometers => 1000.to_number(),
			DistanceUnit::Inches => 127.to_number() / 5000.to_number(),
			DistanceUnit::Feet => 381.to_number() / 1250.to_number(),
			DistanceUnit::Yards => 1143.to_number() / 1250.to_number(),
			DistanceUnit::Miles => 201168.to_number() / 125.to_number(),
			DistanceUnit::NauticalMiles => 1852.to_number(),
			DistanceUnit::AstronomicalUnits => 149_597_870_700u64.to_number(),
		}
	}
}

impl UnitConversion for AngleUnit {
	fn multiplier_to_standard(&self) -> Number {
		match self {
			AngleUnit::Degrees => 1.to_number(),
			AngleUnit::Radians => {
				Decimal::from_str("57.29577951308232087679815481410517").to_number()
			}
			AngleUnit::Gradians => 9.to_number() / 10.to_number(),
		}
	}
}

impl Unit {
	pub fn unit_type(&self) -> UnitType {
		match self {
			Unit::Time(_) => UnitType::Time,
			Unit::Distance(_) => UnitType::Distance,
			Unit::Angle(_) => UnitType::Angle,
		}
	}
}

impl From<TimeUnit> for Unit {
	fn from(unit: TimeUnit) -> Self {
		Unit::Time(unit)
	}
}

impl From<DistanceUnit> for Unit {
	fn from(unit: DistanceUnit) -> Self {
		Unit::Distance(unit)
	}
}

impl From<AngleUnit> for Unit {
	fn from(unit: AngleUnit) -> Self {
		Unit::Angle(unit)
	}
}

impl CompositeUnit {
	pub fn new() -> Self {
		CompositeUnit {
			units: BTreeMap::new(),
		}
	}

	pub fn single_unit(unit: Unit) -> Self {
		let mut units = BTreeMap::new();
		units.insert(unit.unit_type(), (unit, 1));
		CompositeUnit { units }
	}

	pub fn single_unit_inv(unit: Unit) -> Self {
		let mut units = BTreeMap::new();
		units.insert(unit.unit_type(), (unit, -1));
		CompositeUnit { units }
	}

	pub fn ratio_unit(numer: Unit, denom: Unit) -> Self {
		let mut units = BTreeMap::new();
		units.insert(numer.unit_type(), (numer, 1));
		units.insert(denom.unit_type(), (denom, -1));
		CompositeUnit { units }
	}

	pub fn unitless(&self) -> bool {
		self.units.len() == 0
	}

	fn convert_value_of_unit(
		value: &Number,
		from_unit: &Unit,
		to_unit: &Unit,
		power: i32,
	) -> Result<Number> {
		match from_unit {
			Unit::Time(from) => match to_unit {
				Unit::Time(to) => Ok(from.to_unit_with_power(value, to, power)),
				_ => Err(Error::IncompatibleUnits),
			},
			Unit::Distance(from) => match to_unit {
				Unit::Distance(to) => Ok(from.to_unit_with_power(value, to, power)),
				_ => Err(Error::IncompatibleUnits),
			},
			Unit::Angle(from) => match to_unit {
				Unit::Angle(to) => Ok(from.to_unit_with_power(value, to, power)),
				_ => Err(Error::IncompatibleUnits),
			},
		}
	}

	pub fn add_unit(&mut self, value: &Number, unit: Unit) -> Number {
		let unit_type = unit.unit_type();
		if let Some(existing_unit) = self.units.get_mut(&unit_type) {
			let value =
				Self::convert_value_of_unit(value, &existing_unit.0, &unit, existing_unit.1)
					.unwrap();
			existing_unit.0 = unit;
			existing_unit.1 += 1;
			if existing_unit.1 == 0 {
				self.units.remove(&unit_type);
			}
			value
		} else {
			self.units.insert(unit_type, (unit, 1));
			value.clone()
		}
	}

	pub fn add_unit_inv(&mut self, value: &Number, unit: Unit) -> Number {
		let unit_type = unit.unit_type();
		if let Some(existing_unit) = self.units.get_mut(&unit_type) {
			let value =
				Self::convert_value_of_unit(value, &existing_unit.0, &unit, existing_unit.1)
					.unwrap();
			existing_unit.0 = unit;
			existing_unit.1 -= 1;
			if existing_unit.1 == 0 {
				self.units.remove(&unit_type);
			}
			value
		} else {
			self.units.insert(unit_type, (unit, -1));
			value.clone()
		}
	}

	pub fn inverse(&self) -> Self {
		let mut units = BTreeMap::new();
		for (unit_type, unit) in self.units.iter() {
			units.insert(*unit_type, (unit.0, -unit.1));
		}
		CompositeUnit { units }
	}

	pub fn convert_single_unit(&mut self, value: &Number, target_unit: Unit) -> Result<Number> {
		let unit_type = target_unit.unit_type();
		if let Some(existing_unit) = self.units.get_mut(&unit_type) {
			let value = Self::convert_value_of_unit(
				value,
				&existing_unit.0,
				&target_unit,
				existing_unit.1,
			)?;
			existing_unit.0 = target_unit;
			Ok(value)
		} else {
			Err(Error::IncompatibleUnits)
		}
	}

	pub fn coerce_to_other(&self, value: &Number, target_units: &CompositeUnit) -> Result<Number> {
		// Check units to make sure they are compatible. There must be the same set of
		// unit types and each unit type must be the same power.
		for (unit_type, unit) in self.units.iter() {
			if let Some(target) = target_units.units.get(&unit_type) {
				if unit.1 != target.1 {
					return Err(Error::IncompatibleUnits);
				}
			} else {
				return Err(Error::IncompatibleUnits);
			}
		}
		for (unit_type, unit) in target_units.units.iter() {
			if let Some(target) = self.units.get(&unit_type) {
				if unit.1 != target.1 {
					return Err(Error::IncompatibleUnits);
				}
			} else {
				return Err(Error::IncompatibleUnits);
			}
		}

		// Convert units to the target unit
		let mut result = value.clone();
		let mut unit = self.clone();
		for (_, value) in target_units.units.iter() {
			result = unit.convert_single_unit(&result, value.0).unwrap();
		}
		Ok(result)
	}

	pub fn combine(&mut self, value: &Number, target_units: &CompositeUnit) -> Number {
		let mut result = value.clone();
		for (unit_type, unit) in target_units.units.iter() {
			if let Some(target) = self.units.get_mut(&unit_type) {
				result =
					Self::convert_value_of_unit(&result, &target.0, &unit.0, target.1).unwrap();
				target.0 = unit.0;
				target.1 += unit.1;
				if target.1 == 0 {
					self.units.remove(&unit_type);
				}
			} else {
				self.units.insert(*unit_type, unit.clone());
			}
		}
		result
	}
}

impl StorageObject for CompositeUnit {
	fn serialize<Ref: StorageRefSerializer, Out: SerializeOutput>(
		&self,
		output: &mut Out,
		_: &Ref,
	) -> Result<()> {
		output.write_u32(self.units.len() as u32)?;
		for (_, unit) in &self.units {
			output.write_u16(unit.0.to_u16())?;
			output.write_i32(unit.1)?;
		}
		Ok(())
	}

	unsafe fn deserialize<T: StorageRefSerializer>(
		input: &mut DeserializeInput,
		_: &T,
	) -> Result<Self> {
		let count = input.read_u32()?;
		let mut result = CompositeUnit::new();
		for _ in 0..count {
			let unit = match Unit::from_u16(input.read_u16()?) {
				Some(unit) => unit,
				None => return Err(Error::CorruptData),
			};
			let power = input.read_i32()?;
			let unit_type = unit.unit_type();
			result.units.insert(unit_type, (unit, power));
		}
		Ok(result)
	}
}

fn value_layout<ScreenT: Screen>(state: &State, screen: &ScreenT, value: &Value) -> Layout {
	let value_layout = value.render(&state.format, &None, screen.width());
	let mut layout_items = Vec::new();
	layout_items.push(Layout::HorizontalRule);
	layout_items.push(value_layout);
	Layout::Vertical(layout_items)
}

pub fn unit_menu<ScreenT: Screen>(state: &State, screen: &ScreenT, value: &Value) -> Menu {
	let mut items = Vec::new();
	items.push(MenuItem {
		layout: MenuItem::string_layout("Angle".to_string()),
		function: MenuItemFunction::Action(Function::UnitMenu(UnitType::Angle)),
	});
	items.push(MenuItem {
		layout: MenuItem::string_layout("Distance".to_string()),
		function: MenuItemFunction::Action(Function::UnitMenu(UnitType::Distance)),
	});
	items.push(MenuItem {
		layout: MenuItem::string_layout("Time".to_string()),
		function: MenuItemFunction::Action(Function::UnitMenu(UnitType::Time)),
	});

	Menu::new_with_bottom(
		"Units".to_string(),
		items,
		value_layout(state, screen, value),
	)
}

pub fn unit_menu_of_type<ScreenT: Screen>(
	state: &State,
	screen: &ScreenT,
	value: &Value,
	unit_type: UnitType,
) -> Menu {
	match unit_type {
		UnitType::Angle => angle_unit_menu(state, screen, value),
		UnitType::Distance => distance_unit_menu(state, screen, value),
		UnitType::Time => time_unit_menu(state, screen, value),
	}
}

fn angle_unit_menu<ScreenT: Screen>(state: &State, screen: &ScreenT, value: &Value) -> Menu {
	let mut items = Vec::new();
	for unit in &[AngleUnit::Degrees, AngleUnit::Radians, AngleUnit::Gradians] {
		items.push(MenuItem {
			layout: MenuItem::string_layout(unit.to_str()),
			function: MenuItemFunction::ConversionAction(
				Function::AddUnit(Unit::Angle(*unit)),
				Function::AddInvUnit(Unit::Angle(*unit)),
				Function::ConvertToUnit(Unit::Angle(*unit)),
			),
		});
	}

	Menu::new_with_bottom(
		"Angle (×,÷ Assign; x≷y Convert)".to_string(),
		items,
		value_layout(state, screen, value),
	)
}

fn distance_unit_menu<ScreenT: Screen>(state: &State, screen: &ScreenT, value: &Value) -> Menu {
	let mut items = Vec::new();
	for unit in &[
		DistanceUnit::Meters,
		DistanceUnit::Nanometers,
		DistanceUnit::Micrometers,
		DistanceUnit::Millimeters,
		DistanceUnit::Centimeters,
		DistanceUnit::Kilometers,
		DistanceUnit::Inches,
		DistanceUnit::Feet,
		DistanceUnit::Yards,
		DistanceUnit::Miles,
		DistanceUnit::NauticalMiles,
		DistanceUnit::AstronomicalUnits,
	] {
		items.push(MenuItem {
			layout: MenuItem::string_layout(unit.to_str()),
			function: MenuItemFunction::ConversionAction(
				Function::AddUnit(Unit::Distance(*unit)),
				Function::AddInvUnit(Unit::Distance(*unit)),
				Function::ConvertToUnit(Unit::Distance(*unit)),
			),
		});
	}

	let mut menu = Menu::new_with_bottom(
		"Distance (×,÷ Assign; x≷y Convert)".to_string(),
		items,
		value_layout(state, screen, value),
	);
	menu.set_columns(3);
	menu
}

fn time_unit_menu<ScreenT: Screen>(state: &State, screen: &ScreenT, value: &Value) -> Menu {
	let mut items = Vec::new();
	for unit in &[
		TimeUnit::Seconds,
		TimeUnit::Nanoseconds,
		TimeUnit::Microseconds,
		TimeUnit::Milliseconds,
		TimeUnit::Minutes,
		TimeUnit::Hours,
		TimeUnit::Days,
		TimeUnit::Years,
	] {
		items.push(MenuItem {
			layout: MenuItem::string_layout(unit.to_str()),
			function: MenuItemFunction::ConversionAction(
				Function::AddUnit(Unit::Time(*unit)),
				Function::AddInvUnit(Unit::Time(*unit)),
				Function::ConvertToUnit(Unit::Time(*unit)),
			),
		});
	}

	let mut menu = Menu::new_with_bottom(
		"Time (×,÷ Assign; x≷y Convert)".to_string(),
		items,
		value_layout(state, screen, value),
	);
	menu.set_columns(2);
	menu
}
