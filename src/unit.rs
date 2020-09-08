use crate::number::{Number, ToNumber};
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};

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
pub enum Unit {
	Time(TimeUnit),
	Distance(DistanceUnit),
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

impl Unit {
	pub fn to_str(&self) -> String {
		match self {
			Unit::Time(unit) => unit.to_str(),
			Unit::Distance(unit) => unit.to_str(),
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

impl Unit {
	pub fn unit_type(&self) -> UnitType {
		match self {
			Unit::Time(_) => UnitType::Time,
			Unit::Distance(_) => UnitType::Distance,
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
	) -> Option<Number> {
		match from_unit {
			Unit::Time(from) => match to_unit {
				Unit::Time(to) => Some(from.to_unit_with_power(value, to, power)),
				_ => None,
			},
			Unit::Distance(from) => match to_unit {
				Unit::Distance(to) => Some(from.to_unit_with_power(value, to, power)),
				_ => None,
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

	pub fn convert_single_unit(&mut self, value: &Number, target_unit: Unit) -> Option<Number> {
		let unit_type = target_unit.unit_type();
		if let Some(existing_unit) = self.units.get_mut(&unit_type) {
			let value =
				Self::convert_value_of_unit(value, &existing_unit.0, &target_unit, existing_unit.1)
					.unwrap();
			existing_unit.0 = target_unit;
			Some(value)
		} else {
			None
		}
	}

	pub fn coerce_to_other(&self, value: &Number, target_units: &CompositeUnit) -> Option<Number> {
		// Check units to make sure they are compatible. There must be the same set of
		// unit types and each unit type must be the same power.
		for (unit_type, unit) in self.units.iter() {
			if let Some(target) = target_units.units.get(&unit_type) {
				if unit.1 != target.1 {
					return None;
				}
			} else {
				return None;
			}
		}
		for (unit_type, unit) in target_units.units.iter() {
			if let Some(target) = self.units.get(&unit_type) {
				if unit.1 != target.1 {
					return None;
				}
			} else {
				return None;
			}
		}

		// Convert units to the target unit
		let mut result = value.clone();
		let mut unit = self.clone();
		for (_, value) in target_units.units.iter() {
			result = unit.convert_single_unit(&result, value.0).unwrap();
		}
		Some(result)
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
