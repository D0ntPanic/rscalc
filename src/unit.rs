use crate::error::{Error, Result};
use crate::functions::Function;
use crate::layout::Layout;
use crate::menu::{Menu, MenuItem, MenuItemFunction, MenuItemLayout};
use crate::number::{Number, ToNumber};
use crate::screen::Screen;
use crate::state::State;
use crate::storage::{DeserializeInput, SerializeOutput, StorageObject, StorageRefSerializer};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::vec::Vec;
use intel_dfp::Decimal;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum AngleUnit {
	Degrees,
	Radians,
	Gradians,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum AreaUnit {
	Hectares,
	Acres,
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
pub enum EnergyUnit {
	Joules,
	Millijoules,
	Kilojoules,
	Megajoules,
	Calories,
	Kilocalories,
	BTU,
	FootPounds,
	FootPoundals,
	WattHours,
	KilowattHours,
	Erg,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ForceUnit {
	Newton,
	Kilonewton,
	Dyne,
	KilogramForce,
	PoundForce,
	Poundal,
	Kip,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum MassUnit {
	Grams,
	Milligrams,
	Kilograms,
	MetricTons,
	Pounds,
	Ounces,
	Stones,
	Tons,
	UKTons,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum PowerUnit {
	Watts,
	Milliwatts,
	Kilowatts,
	Megawatts,
	Gigawatts,
	MechanicalHorsepower,
	MetricHorsepower,
	ElectricalHorsepower,
	TonsOfRefrigeration,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum PressureUnit {
	Pascals,
	Kilopascals,
	Bars,
	Millibars,
	Atmospheres,
	InchesOfMercury,
	MillimetersOfMercury,
	InchesOfWater,
	MillimetersOfWater,
	PoundsPerSquareInch,
	Torr,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TemperatureUnit {
	Celsius,
	Fahrenheit,
	Kelvin,
	Rankine,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TimeUnit {
	Nanoseconds,
	Microseconds,
	Milliseconds,
	Seconds,
	Minutes,
	Hours,
	Days,
	Weeks,
	Months,
	Years,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum VolumeUnit {
	Litre,
	Millilitre,
	Gallons,
	Quarts,
	Pints,
	Cups,
	FluidOunces,
	ImperialGallons,
	ImperialQuarts,
	ImperialPints,
	ImperialOunces,
	Tablespoons,
	Teaspoons,
	UKTablespoons,
	UKTeaspoons,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Unit {
	Angle(AngleUnit),
	Area(AreaUnit),
	Distance(DistanceUnit),
	Energy(EnergyUnit),
	Force(ForceUnit),
	Mass(MassUnit),
	Power(PowerUnit),
	Pressure(PressureUnit),
	Temperature(TemperatureUnit),
	Time(TimeUnit),
	Volume(VolumeUnit),
}

impl AngleUnit {
	pub fn to_str(&self) -> &'static str {
		match self {
			AngleUnit::Degrees => "°",
			AngleUnit::Radians => "rad",
			AngleUnit::Gradians => "grad",
		}
	}

	fn units() -> &'static [Unit] {
		&[
			Unit::Angle(AngleUnit::Degrees),
			Unit::Angle(AngleUnit::Radians),
			Unit::Angle(AngleUnit::Gradians),
		]
	}
}

impl AreaUnit {
	pub fn to_str(&self) -> &'static str {
		match self {
			AreaUnit::Hectares => "ha",
			AreaUnit::Acres => "acre",
		}
	}

	fn to_square_meters(&self, value: &Number) -> Number {
		value * &self.multiplier_to_standard() * 10_000.to_number()
	}

	fn from_square_meters(&self, value: &Number) -> Number {
		value / &(self.multiplier_to_standard() * 10_000.to_number())
	}

	fn to_square_meters_with_power(&self, value: &Number, power: i32) -> Number {
		if power < 0 {
			let mut result = value.clone();
			for _ in 0..-power {
				result = self.from_square_meters(&result);
			}
			result
		} else if power > 0 {
			let mut result = value.clone();
			for _ in 0..power {
				result = self.to_square_meters(&result);
			}
			result
		} else {
			value.clone()
		}
	}

	fn from_square_meters_with_power(&self, value: &Number, power: i32) -> Number {
		if power < 0 {
			let mut result = value.clone();
			for _ in 0..-power {
				result = self.to_square_meters(&result);
			}
			result
		} else if power > 0 {
			let mut result = value.clone();
			for _ in 0..power {
				result = self.from_square_meters(&result);
			}
			result
		} else {
			value.clone()
		}
	}

	fn units() -> &'static [Unit] {
		&[
			Unit::Area(AreaUnit::Acres),
			Unit::Area(AreaUnit::Hectares),
			Unit::Distance(DistanceUnit::Meters),
			Unit::Distance(DistanceUnit::Millimeters),
			Unit::Distance(DistanceUnit::Centimeters),
			Unit::Distance(DistanceUnit::Kilometers),
			Unit::Distance(DistanceUnit::Inches),
			Unit::Distance(DistanceUnit::Feet),
			Unit::Distance(DistanceUnit::Yards),
			Unit::Distance(DistanceUnit::Miles),
		]
	}
}

impl DistanceUnit {
	pub fn to_str(&self) -> &'static str {
		match self {
			DistanceUnit::Nanometers => "nm",
			DistanceUnit::Micrometers => "μm",
			DistanceUnit::Millimeters => "mm",
			DistanceUnit::Centimeters => "cm",
			DistanceUnit::Meters => "m",
			DistanceUnit::Kilometers => "km",
			DistanceUnit::Inches => "in",
			DistanceUnit::Feet => "ft",
			DistanceUnit::Yards => "yd",
			DistanceUnit::Miles => "mi",
			DistanceUnit::NauticalMiles => "nmi",
			DistanceUnit::AstronomicalUnits => "au",
		}
	}

	fn units() -> &'static [Unit] {
		&[
			Unit::Distance(DistanceUnit::Meters),
			Unit::Distance(DistanceUnit::Nanometers),
			Unit::Distance(DistanceUnit::Micrometers),
			Unit::Distance(DistanceUnit::Millimeters),
			Unit::Distance(DistanceUnit::Centimeters),
			Unit::Distance(DistanceUnit::Kilometers),
			Unit::Distance(DistanceUnit::Inches),
			Unit::Distance(DistanceUnit::Feet),
			Unit::Distance(DistanceUnit::Yards),
			Unit::Distance(DistanceUnit::Miles),
			Unit::Distance(DistanceUnit::NauticalMiles),
			Unit::Distance(DistanceUnit::AstronomicalUnits),
		]
	}
}

impl EnergyUnit {
	pub fn to_str(&self) -> &'static str {
		match self {
			EnergyUnit::Joules => "J",
			EnergyUnit::Millijoules => "mJ",
			EnergyUnit::Kilojoules => "kJ",
			EnergyUnit::Megajoules => "MJ",
			EnergyUnit::Calories => "cal",
			EnergyUnit::Kilocalories => "kcal",
			EnergyUnit::BTU => "BTU",
			EnergyUnit::FootPounds => "ftlbf",
			EnergyUnit::FootPoundals => "ftpdl",
			EnergyUnit::WattHours => "Wh",
			EnergyUnit::KilowattHours => "kWh",
			EnergyUnit::Erg => "erg",
		}
	}

	fn units() -> &'static [Unit] {
		&[
			Unit::Energy(EnergyUnit::Joules),
			Unit::Energy(EnergyUnit::Millijoules),
			Unit::Energy(EnergyUnit::Kilojoules),
			Unit::Energy(EnergyUnit::Megajoules),
			Unit::Energy(EnergyUnit::Calories),
			Unit::Energy(EnergyUnit::Kilocalories),
			Unit::Energy(EnergyUnit::BTU),
			Unit::Energy(EnergyUnit::FootPounds),
			Unit::Energy(EnergyUnit::FootPoundals),
			Unit::Energy(EnergyUnit::WattHours),
			Unit::Energy(EnergyUnit::KilowattHours),
			Unit::Energy(EnergyUnit::Erg),
		]
	}
}

impl ForceUnit {
	pub fn to_str(&self) -> &'static str {
		match self {
			ForceUnit::Newton => "N",
			ForceUnit::Kilonewton => "kN",
			ForceUnit::Dyne => "dyn",
			ForceUnit::KilogramForce => "kgf",
			ForceUnit::PoundForce => "lbf",
			ForceUnit::Poundal => "pdl",
			ForceUnit::Kip => "kip",
		}
	}

	fn units() -> &'static [Unit] {
		&[
			Unit::Force(ForceUnit::Newton),
			Unit::Force(ForceUnit::Kilonewton),
			Unit::Force(ForceUnit::Dyne),
			Unit::Force(ForceUnit::KilogramForce),
			Unit::Force(ForceUnit::PoundForce),
			Unit::Force(ForceUnit::Poundal),
			Unit::Force(ForceUnit::Kip),
		]
	}
}

impl MassUnit {
	pub fn to_str(&self) -> &'static str {
		match self {
			MassUnit::Grams => "g",
			MassUnit::Milligrams => "mg",
			MassUnit::Kilograms => "kg",
			MassUnit::MetricTons => "t",
			MassUnit::Pounds => "lb",
			MassUnit::Ounces => "oz",
			MassUnit::Stones => "st",
			MassUnit::Tons => "ton",
			MassUnit::UKTons => "UK ton",
		}
	}

	fn units() -> &'static [Unit] {
		&[
			Unit::Mass(MassUnit::Kilograms),
			Unit::Mass(MassUnit::Grams),
			Unit::Mass(MassUnit::Milligrams),
			Unit::Mass(MassUnit::MetricTons),
			Unit::Mass(MassUnit::Pounds),
			Unit::Mass(MassUnit::Ounces),
			Unit::Mass(MassUnit::Stones),
			Unit::Mass(MassUnit::Tons),
			Unit::Mass(MassUnit::UKTons),
		]
	}
}

impl PowerUnit {
	pub fn to_str(&self) -> &'static str {
		match self {
			PowerUnit::Watts => "W",
			PowerUnit::Milliwatts => "mW",
			PowerUnit::Kilowatts => "kW",
			PowerUnit::Megawatts => "MW",
			PowerUnit::Gigawatts => "GW",
			PowerUnit::MechanicalHorsepower => "hp",
			PowerUnit::MetricHorsepower => "hpM",
			PowerUnit::ElectricalHorsepower => "hpE",
			PowerUnit::TonsOfRefrigeration => "RT",
		}
	}

	fn units() -> &'static [Unit] {
		&[
			Unit::Power(PowerUnit::Watts),
			Unit::Power(PowerUnit::Milliwatts),
			Unit::Power(PowerUnit::Kilowatts),
			Unit::Power(PowerUnit::Megawatts),
			Unit::Power(PowerUnit::Gigawatts),
			Unit::Power(PowerUnit::MechanicalHorsepower),
			Unit::Power(PowerUnit::MetricHorsepower),
			Unit::Power(PowerUnit::ElectricalHorsepower),
			Unit::Power(PowerUnit::TonsOfRefrigeration),
		]
	}
}

impl PressureUnit {
	pub fn to_str(&self) -> &'static str {
		match self {
			PressureUnit::Pascals => "Pa",
			PressureUnit::Kilopascals => "kPa",
			PressureUnit::Bars => "bar",
			PressureUnit::Millibars => "mbar",
			PressureUnit::Atmospheres => "atm",
			PressureUnit::InchesOfMercury => "inHg",
			PressureUnit::MillimetersOfMercury => "mmHg",
			PressureUnit::InchesOfWater => "inH₂O",
			PressureUnit::MillimetersOfWater => "mmH₂O",
			PressureUnit::PoundsPerSquareInch => "psi",
			PressureUnit::Torr => "Torr",
		}
	}

	fn units() -> &'static [Unit] {
		&[
			Unit::Pressure(PressureUnit::Pascals),
			Unit::Pressure(PressureUnit::Kilopascals),
			Unit::Pressure(PressureUnit::Bars),
			Unit::Pressure(PressureUnit::Millibars),
			Unit::Pressure(PressureUnit::Atmospheres),
			Unit::Pressure(PressureUnit::InchesOfMercury),
			Unit::Pressure(PressureUnit::MillimetersOfMercury),
			Unit::Pressure(PressureUnit::InchesOfWater),
			Unit::Pressure(PressureUnit::MillimetersOfWater),
			Unit::Pressure(PressureUnit::PoundsPerSquareInch),
			Unit::Pressure(PressureUnit::Torr),
		]
	}
}

impl TemperatureUnit {
	pub fn to_str(&self) -> &'static str {
		match self {
			TemperatureUnit::Celsius => "°C",
			TemperatureUnit::Fahrenheit => "°F",
			TemperatureUnit::Kelvin => "K",
			TemperatureUnit::Rankine => "°R",
		}
	}

	fn to_celsius<'a>(&self, value: &'a Number) -> Cow<'a, Number> {
		match self {
			TemperatureUnit::Celsius => Cow::Borrowed(value),
			TemperatureUnit::Fahrenheit => {
				Cow::Owned((value - &32.to_number()) * 5.to_number() / 9.to_number())
			}
			TemperatureUnit::Kelvin => Cow::Owned(value - &(5463.to_number() / 20.to_number())),
			TemperatureUnit::Rankine => Cow::Owned(
				(value - &(49_167.to_number() / 100.to_number())) * 5.to_number() / 9.to_number(),
			),
		}
	}

	fn from_celsius<'a>(&self, value: &'a Number) -> Cow<'a, Number> {
		match self {
			TemperatureUnit::Celsius => Cow::Borrowed(value),
			TemperatureUnit::Fahrenheit => {
				Cow::Owned((value * &9.to_number() / 5.to_number()) + 32.to_number())
			}
			TemperatureUnit::Kelvin => Cow::Owned(value + &(5463.to_number() / 20.to_number())),
			TemperatureUnit::Rankine => Cow::Owned(
				(value * &9.to_number() / 5.to_number()) + (49_167.to_number() / 100.to_number()),
			),
		}
	}

	fn units() -> &'static [Unit] {
		&[
			Unit::Temperature(TemperatureUnit::Celsius),
			Unit::Temperature(TemperatureUnit::Fahrenheit),
			Unit::Temperature(TemperatureUnit::Kelvin),
			Unit::Temperature(TemperatureUnit::Rankine),
		]
	}
}

impl TimeUnit {
	pub fn to_str(&self) -> &'static str {
		match self {
			TimeUnit::Nanoseconds => "ns",
			TimeUnit::Microseconds => "μs",
			TimeUnit::Milliseconds => "ms",
			TimeUnit::Seconds => "sec",
			TimeUnit::Minutes => "min",
			TimeUnit::Hours => "hr",
			TimeUnit::Days => "day",
			TimeUnit::Weeks => "wk",
			TimeUnit::Months => "month",
			TimeUnit::Years => "yr",
		}
	}

	fn units() -> &'static [Unit] {
		&[
			Unit::Time(TimeUnit::Seconds),
			Unit::Time(TimeUnit::Nanoseconds),
			Unit::Time(TimeUnit::Microseconds),
			Unit::Time(TimeUnit::Milliseconds),
			Unit::Time(TimeUnit::Minutes),
			Unit::Time(TimeUnit::Hours),
			Unit::Time(TimeUnit::Days),
			Unit::Time(TimeUnit::Weeks),
			Unit::Time(TimeUnit::Months),
			Unit::Time(TimeUnit::Years),
		]
	}
}

impl VolumeUnit {
	pub fn to_str(&self) -> &'static str {
		match self {
			VolumeUnit::Litre => "L",
			VolumeUnit::Millilitre => "mL",
			VolumeUnit::Gallons => "gal",
			VolumeUnit::Quarts => "qt",
			VolumeUnit::Pints => "pt",
			VolumeUnit::Cups => "cup",
			VolumeUnit::FluidOunces => "fl oz",
			VolumeUnit::ImperialGallons => "UK gal",
			VolumeUnit::ImperialQuarts => "UK qt",
			VolumeUnit::ImperialPints => "UK pt",
			VolumeUnit::ImperialOunces => "UK oz",
			VolumeUnit::Tablespoons => "tbsp",
			VolumeUnit::Teaspoons => "tsp",
			VolumeUnit::UKTablespoons => "UK tbsp",
			VolumeUnit::UKTeaspoons => "UK tsp",
		}
	}

	fn to_cubic_meters(&self, value: &Number) -> Number {
		value * &self.multiplier_to_standard() / 1000.to_number()
	}

	fn from_cubic_meters(&self, value: &Number) -> Number {
		value / &(self.multiplier_to_standard() / 1000.to_number())
	}

	fn to_cubic_meters_with_power(&self, value: &Number, power: i32) -> Number {
		if power < 0 {
			let mut result = value.clone();
			for _ in 0..-power {
				result = self.from_cubic_meters(&result);
			}
			result
		} else if power > 0 {
			let mut result = value.clone();
			for _ in 0..power {
				result = self.to_cubic_meters(&result);
			}
			result
		} else {
			value.clone()
		}
	}

	fn from_cubic_meters_with_power(&self, value: &Number, power: i32) -> Number {
		if power < 0 {
			let mut result = value.clone();
			for _ in 0..-power {
				result = self.to_cubic_meters(&result);
			}
			result
		} else if power > 0 {
			let mut result = value.clone();
			for _ in 0..power {
				result = self.from_cubic_meters(&result);
			}
			result
		} else {
			value.clone()
		}
	}

	fn units() -> &'static [Unit] {
		&[
			Unit::Volume(VolumeUnit::Litre),
			Unit::Volume(VolumeUnit::Millilitre),
			Unit::Volume(VolumeUnit::Gallons),
			Unit::Volume(VolumeUnit::Quarts),
			Unit::Volume(VolumeUnit::Pints),
			Unit::Volume(VolumeUnit::Cups),
			Unit::Volume(VolumeUnit::FluidOunces),
			Unit::Volume(VolumeUnit::ImperialGallons),
			Unit::Volume(VolumeUnit::ImperialQuarts),
			Unit::Volume(VolumeUnit::ImperialPints),
			Unit::Volume(VolumeUnit::ImperialOunces),
			Unit::Volume(VolumeUnit::Tablespoons),
			Unit::Volume(VolumeUnit::Teaspoons),
			Unit::Volume(VolumeUnit::UKTablespoons),
			Unit::Volume(VolumeUnit::UKTeaspoons),
			Unit::Distance(DistanceUnit::Meters),
			Unit::Distance(DistanceUnit::Centimeters),
			Unit::Distance(DistanceUnit::Inches),
			Unit::Distance(DistanceUnit::Feet),
		]
	}
}

impl Unit {
	pub fn to_str(&self) -> &'static str {
		match self {
			Unit::Angle(unit) => unit.to_str(),
			Unit::Area(unit) => unit.to_str(),
			Unit::Distance(unit) => unit.to_str(),
			Unit::Energy(unit) => unit.to_str(),
			Unit::Force(unit) => unit.to_str(),
			Unit::Mass(unit) => unit.to_str(),
			Unit::Power(unit) => unit.to_str(),
			Unit::Pressure(unit) => unit.to_str(),
			Unit::Temperature(unit) => unit.to_str(),
			Unit::Time(unit) => unit.to_str(),
			Unit::Volume(unit) => unit.to_str(),
		}
	}

	pub fn to_u16(&self) -> u16 {
		match self {
			Unit::Angle(AngleUnit::Degrees) => 0x0000,
			Unit::Angle(AngleUnit::Radians) => 0x0001,
			Unit::Angle(AngleUnit::Gradians) => 0x0002,
			Unit::Area(AreaUnit::Hectares) => 0x0100,
			Unit::Area(AreaUnit::Acres) => 0x0101,
			Unit::Distance(DistanceUnit::Nanometers) => 0x0200,
			Unit::Distance(DistanceUnit::Micrometers) => 0x0201,
			Unit::Distance(DistanceUnit::Millimeters) => 0x0202,
			Unit::Distance(DistanceUnit::Centimeters) => 0x0203,
			Unit::Distance(DistanceUnit::Meters) => 0x0204,
			Unit::Distance(DistanceUnit::Kilometers) => 0x0205,
			Unit::Distance(DistanceUnit::Inches) => 0x0210,
			Unit::Distance(DistanceUnit::Feet) => 0x0211,
			Unit::Distance(DistanceUnit::Yards) => 0x0212,
			Unit::Distance(DistanceUnit::Miles) => 0x0213,
			Unit::Distance(DistanceUnit::NauticalMiles) => 0x0214,
			Unit::Distance(DistanceUnit::AstronomicalUnits) => 0x0220,
			Unit::Energy(EnergyUnit::Joules) => 0x0300,
			Unit::Energy(EnergyUnit::Millijoules) => 0x0301,
			Unit::Energy(EnergyUnit::Kilojoules) => 0x0302,
			Unit::Energy(EnergyUnit::Megajoules) => 0x0303,
			Unit::Energy(EnergyUnit::Calories) => 0x0304,
			Unit::Energy(EnergyUnit::Kilocalories) => 0x0305,
			Unit::Energy(EnergyUnit::BTU) => 0x0306,
			Unit::Energy(EnergyUnit::FootPounds) => 0x0307,
			Unit::Energy(EnergyUnit::FootPoundals) => 0x0308,
			Unit::Energy(EnergyUnit::WattHours) => 0x0309,
			Unit::Energy(EnergyUnit::KilowattHours) => 0x030a,
			Unit::Energy(EnergyUnit::Erg) => 0x030b,
			Unit::Force(ForceUnit::Newton) => 0x0400,
			Unit::Force(ForceUnit::Kilonewton) => 0x0401,
			Unit::Force(ForceUnit::Dyne) => 0x0402,
			Unit::Force(ForceUnit::KilogramForce) => 0x0403,
			Unit::Force(ForceUnit::PoundForce) => 0x0404,
			Unit::Force(ForceUnit::Poundal) => 0x0405,
			Unit::Force(ForceUnit::Kip) => 0x0406,
			Unit::Mass(MassUnit::Grams) => 0x0500,
			Unit::Mass(MassUnit::Milligrams) => 0x0501,
			Unit::Mass(MassUnit::Kilograms) => 0x0502,
			Unit::Mass(MassUnit::MetricTons) => 0x0503,
			Unit::Mass(MassUnit::Pounds) => 0x0504,
			Unit::Mass(MassUnit::Ounces) => 0x0505,
			Unit::Mass(MassUnit::Stones) => 0x0506,
			Unit::Mass(MassUnit::Tons) => 0x0507,
			Unit::Mass(MassUnit::UKTons) => 0x0508,
			Unit::Power(PowerUnit::Watts) => 0x0600,
			Unit::Power(PowerUnit::Milliwatts) => 0x0601,
			Unit::Power(PowerUnit::Kilowatts) => 0x0602,
			Unit::Power(PowerUnit::Megawatts) => 0x0603,
			Unit::Power(PowerUnit::Gigawatts) => 0x0604,
			Unit::Power(PowerUnit::MechanicalHorsepower) => 0x0605,
			Unit::Power(PowerUnit::MetricHorsepower) => 0x0606,
			Unit::Power(PowerUnit::ElectricalHorsepower) => 0x0607,
			Unit::Power(PowerUnit::TonsOfRefrigeration) => 0x0608,
			Unit::Pressure(PressureUnit::Pascals) => 0x0700,
			Unit::Pressure(PressureUnit::Kilopascals) => 0x0701,
			Unit::Pressure(PressureUnit::Bars) => 0x0702,
			Unit::Pressure(PressureUnit::Millibars) => 0x0703,
			Unit::Pressure(PressureUnit::Atmospheres) => 0x0704,
			Unit::Pressure(PressureUnit::InchesOfMercury) => 0x0705,
			Unit::Pressure(PressureUnit::MillimetersOfMercury) => 0x0706,
			Unit::Pressure(PressureUnit::InchesOfWater) => 0x0707,
			Unit::Pressure(PressureUnit::MillimetersOfWater) => 0x0708,
			Unit::Pressure(PressureUnit::PoundsPerSquareInch) => 0x0709,
			Unit::Pressure(PressureUnit::Torr) => 0x070a,
			Unit::Temperature(TemperatureUnit::Celsius) => 0x0800,
			Unit::Temperature(TemperatureUnit::Fahrenheit) => 0x0801,
			Unit::Temperature(TemperatureUnit::Kelvin) => 0x0802,
			Unit::Temperature(TemperatureUnit::Rankine) => 0x0803,
			Unit::Time(TimeUnit::Nanoseconds) => 0x0900,
			Unit::Time(TimeUnit::Microseconds) => 0x0901,
			Unit::Time(TimeUnit::Milliseconds) => 0x0902,
			Unit::Time(TimeUnit::Seconds) => 0x0903,
			Unit::Time(TimeUnit::Minutes) => 0x0904,
			Unit::Time(TimeUnit::Hours) => 0x0905,
			Unit::Time(TimeUnit::Days) => 0x0906,
			Unit::Time(TimeUnit::Weeks) => 0x0907,
			Unit::Time(TimeUnit::Months) => 0x0908,
			Unit::Time(TimeUnit::Years) => 0x0909,
			Unit::Volume(VolumeUnit::Litre) => 0x0a00,
			Unit::Volume(VolumeUnit::Millilitre) => 0x0a01,
			Unit::Volume(VolumeUnit::Gallons) => 0x0a02,
			Unit::Volume(VolumeUnit::Quarts) => 0x0a03,
			Unit::Volume(VolumeUnit::Pints) => 0x0a04,
			Unit::Volume(VolumeUnit::Cups) => 0x0a05,
			Unit::Volume(VolumeUnit::FluidOunces) => 0x0a06,
			Unit::Volume(VolumeUnit::ImperialGallons) => 0x0a07,
			Unit::Volume(VolumeUnit::ImperialQuarts) => 0x0a08,
			Unit::Volume(VolumeUnit::ImperialPints) => 0x0a09,
			Unit::Volume(VolumeUnit::ImperialOunces) => 0x0a0a,
			Unit::Volume(VolumeUnit::Tablespoons) => 0x0a0b,
			Unit::Volume(VolumeUnit::Teaspoons) => 0x0a0c,
			Unit::Volume(VolumeUnit::UKTablespoons) => 0x0a0d,
			Unit::Volume(VolumeUnit::UKTeaspoons) => 0x0a0e,
		}
	}

	pub fn from_u16(value: u16) -> Option<Self> {
		match value {
			0x0000 => Some(Unit::Angle(AngleUnit::Degrees)),
			0x0001 => Some(Unit::Angle(AngleUnit::Radians)),
			0x0002 => Some(Unit::Angle(AngleUnit::Gradians)),
			0x0100 => Some(Unit::Area(AreaUnit::Hectares)),
			0x0101 => Some(Unit::Area(AreaUnit::Acres)),
			0x0200 => Some(Unit::Distance(DistanceUnit::Nanometers)),
			0x0201 => Some(Unit::Distance(DistanceUnit::Micrometers)),
			0x0202 => Some(Unit::Distance(DistanceUnit::Millimeters)),
			0x0203 => Some(Unit::Distance(DistanceUnit::Centimeters)),
			0x0204 => Some(Unit::Distance(DistanceUnit::Meters)),
			0x0205 => Some(Unit::Distance(DistanceUnit::Kilometers)),
			0x0210 => Some(Unit::Distance(DistanceUnit::Inches)),
			0x0211 => Some(Unit::Distance(DistanceUnit::Feet)),
			0x0212 => Some(Unit::Distance(DistanceUnit::Yards)),
			0x0213 => Some(Unit::Distance(DistanceUnit::Miles)),
			0x0214 => Some(Unit::Distance(DistanceUnit::NauticalMiles)),
			0x0220 => Some(Unit::Distance(DistanceUnit::AstronomicalUnits)),
			0x0300 => Some(Unit::Energy(EnergyUnit::Joules)),
			0x0301 => Some(Unit::Energy(EnergyUnit::Millijoules)),
			0x0302 => Some(Unit::Energy(EnergyUnit::Kilojoules)),
			0x0303 => Some(Unit::Energy(EnergyUnit::Megajoules)),
			0x0304 => Some(Unit::Energy(EnergyUnit::Calories)),
			0x0305 => Some(Unit::Energy(EnergyUnit::Kilocalories)),
			0x0306 => Some(Unit::Energy(EnergyUnit::BTU)),
			0x0307 => Some(Unit::Energy(EnergyUnit::FootPounds)),
			0x0308 => Some(Unit::Energy(EnergyUnit::FootPoundals)),
			0x0309 => Some(Unit::Energy(EnergyUnit::WattHours)),
			0x030a => Some(Unit::Energy(EnergyUnit::KilowattHours)),
			0x030b => Some(Unit::Energy(EnergyUnit::Erg)),
			0x0400 => Some(Unit::Force(ForceUnit::Newton)),
			0x0401 => Some(Unit::Force(ForceUnit::Kilonewton)),
			0x0402 => Some(Unit::Force(ForceUnit::Dyne)),
			0x0403 => Some(Unit::Force(ForceUnit::KilogramForce)),
			0x0404 => Some(Unit::Force(ForceUnit::PoundForce)),
			0x0405 => Some(Unit::Force(ForceUnit::Poundal)),
			0x0406 => Some(Unit::Force(ForceUnit::Kip)),
			0x0500 => Some(Unit::Mass(MassUnit::Grams)),
			0x0501 => Some(Unit::Mass(MassUnit::Milligrams)),
			0x0502 => Some(Unit::Mass(MassUnit::Kilograms)),
			0x0503 => Some(Unit::Mass(MassUnit::MetricTons)),
			0x0504 => Some(Unit::Mass(MassUnit::Pounds)),
			0x0505 => Some(Unit::Mass(MassUnit::Ounces)),
			0x0506 => Some(Unit::Mass(MassUnit::Stones)),
			0x0507 => Some(Unit::Mass(MassUnit::Tons)),
			0x0508 => Some(Unit::Mass(MassUnit::UKTons)),
			0x0600 => Some(Unit::Power(PowerUnit::Watts)),
			0x0601 => Some(Unit::Power(PowerUnit::Milliwatts)),
			0x0602 => Some(Unit::Power(PowerUnit::Kilowatts)),
			0x0603 => Some(Unit::Power(PowerUnit::Megawatts)),
			0x0604 => Some(Unit::Power(PowerUnit::Gigawatts)),
			0x0605 => Some(Unit::Power(PowerUnit::MechanicalHorsepower)),
			0x0606 => Some(Unit::Power(PowerUnit::MetricHorsepower)),
			0x0607 => Some(Unit::Power(PowerUnit::ElectricalHorsepower)),
			0x0608 => Some(Unit::Power(PowerUnit::TonsOfRefrigeration)),
			0x0700 => Some(Unit::Pressure(PressureUnit::Pascals)),
			0x0701 => Some(Unit::Pressure(PressureUnit::Kilopascals)),
			0x0702 => Some(Unit::Pressure(PressureUnit::Bars)),
			0x0703 => Some(Unit::Pressure(PressureUnit::Millibars)),
			0x0704 => Some(Unit::Pressure(PressureUnit::Atmospheres)),
			0x0705 => Some(Unit::Pressure(PressureUnit::InchesOfMercury)),
			0x0706 => Some(Unit::Pressure(PressureUnit::MillimetersOfMercury)),
			0x0707 => Some(Unit::Pressure(PressureUnit::InchesOfWater)),
			0x0708 => Some(Unit::Pressure(PressureUnit::MillimetersOfWater)),
			0x0709 => Some(Unit::Pressure(PressureUnit::PoundsPerSquareInch)),
			0x070a => Some(Unit::Pressure(PressureUnit::Torr)),
			0x0800 => Some(Unit::Temperature(TemperatureUnit::Celsius)),
			0x0801 => Some(Unit::Temperature(TemperatureUnit::Fahrenheit)),
			0x0802 => Some(Unit::Temperature(TemperatureUnit::Kelvin)),
			0x0803 => Some(Unit::Temperature(TemperatureUnit::Rankine)),
			0x0900 => Some(Unit::Time(TimeUnit::Nanoseconds)),
			0x0901 => Some(Unit::Time(TimeUnit::Microseconds)),
			0x0902 => Some(Unit::Time(TimeUnit::Milliseconds)),
			0x0903 => Some(Unit::Time(TimeUnit::Seconds)),
			0x0904 => Some(Unit::Time(TimeUnit::Minutes)),
			0x0905 => Some(Unit::Time(TimeUnit::Hours)),
			0x0906 => Some(Unit::Time(TimeUnit::Days)),
			0x0907 => Some(Unit::Time(TimeUnit::Weeks)),
			0x0908 => Some(Unit::Time(TimeUnit::Months)),
			0x0909 => Some(Unit::Time(TimeUnit::Years)),
			0x0a00 => Some(Unit::Volume(VolumeUnit::Litre)),
			0x0a01 => Some(Unit::Volume(VolumeUnit::Millilitre)),
			0x0a02 => Some(Unit::Volume(VolumeUnit::Gallons)),
			0x0a03 => Some(Unit::Volume(VolumeUnit::Quarts)),
			0x0a04 => Some(Unit::Volume(VolumeUnit::Pints)),
			0x0a05 => Some(Unit::Volume(VolumeUnit::Cups)),
			0x0a06 => Some(Unit::Volume(VolumeUnit::FluidOunces)),
			0x0a07 => Some(Unit::Volume(VolumeUnit::ImperialGallons)),
			0x0a08 => Some(Unit::Volume(VolumeUnit::ImperialQuarts)),
			0x0a09 => Some(Unit::Volume(VolumeUnit::ImperialPints)),
			0x0a0a => Some(Unit::Volume(VolumeUnit::ImperialOunces)),
			0x0a0b => Some(Unit::Volume(VolumeUnit::Tablespoons)),
			0x0a0c => Some(Unit::Volume(VolumeUnit::Teaspoons)),
			0x0a0d => Some(Unit::Volume(VolumeUnit::UKTablespoons)),
			0x0a0e => Some(Unit::Volume(VolumeUnit::UKTeaspoons)),
			_ => None,
		}
	}
}

pub trait UnitConversion: Eq {
	/// Converts a value from this unit to a target unit
	fn to_unit(&self, value: &Number, target_unit: &Self) -> Number;

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

pub trait MultiplierUnitConversion: UnitConversion {
	/// Gets the conversion factor from this unit to the standard unit of this type
	fn multiplier_to_standard(&self) -> Number;
}

impl<T: MultiplierUnitConversion> UnitConversion for T {
	/// Converts a value from this unit to a target unit
	fn to_unit(&self, value: &Number, target_unit: &Self) -> Number {
		if self == target_unit {
			return value.clone();
		}
		let value = value * &self.multiplier_to_standard();
		value / target_unit.multiplier_to_standard()
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum UnitType {
	Angle,
	Area,
	Distance,
	Energy,
	Force,
	Mass,
	Power,
	Pressure,
	Temperature,
	Time,
	Volume,
}

impl UnitType {
	pub fn to_str(&self) -> &str {
		match self {
			UnitType::Angle => "Angle",
			UnitType::Area => "Area",
			UnitType::Distance => "Distance",
			UnitType::Energy => "Energy",
			UnitType::Force => "Force",
			UnitType::Mass => "Mass",
			UnitType::Power => "Power",
			UnitType::Pressure => "Pressure",
			UnitType::Temperature => "Temp",
			UnitType::Time => "Time",
			UnitType::Volume => "Volume",
		}
	}

	pub fn units(&self) -> &'static [Unit] {
		match self {
			UnitType::Angle => AngleUnit::units(),
			UnitType::Area => AreaUnit::units(),
			UnitType::Distance => DistanceUnit::units(),
			UnitType::Energy => EnergyUnit::units(),
			UnitType::Force => ForceUnit::units(),
			UnitType::Mass => MassUnit::units(),
			UnitType::Power => PowerUnit::units(),
			UnitType::Pressure => PressureUnit::units(),
			UnitType::Temperature => TemperatureUnit::units(),
			UnitType::Time => TimeUnit::units(),
			UnitType::Volume => VolumeUnit::units(),
		}
	}
}

#[derive(Clone)]
pub struct CompositeUnit {
	pub units: BTreeMap<UnitType, (Unit, i32)>,
}

impl MultiplierUnitConversion for AngleUnit {
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

impl MultiplierUnitConversion for AreaUnit {
	fn multiplier_to_standard(&self) -> Number {
		match self {
			AreaUnit::Hectares => 1.to_number(),
			AreaUnit::Acres => 158_080_329.to_number() / 390_625_000.to_number(),
		}
	}
}

impl MultiplierUnitConversion for DistanceUnit {
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

impl MultiplierUnitConversion for EnergyUnit {
	fn multiplier_to_standard(&self) -> Number {
		match self {
			EnergyUnit::Joules => 1.to_number(),
			EnergyUnit::Millijoules => 1.to_number() / 1000.to_number(),
			EnergyUnit::Kilojoules => 1000.to_number(),
			EnergyUnit::Megajoules => 1_000_000.to_number(),
			EnergyUnit::Calories => 523.to_number() / 125.to_number(),
			EnergyUnit::Kilocalories => 4184.to_number(),
			EnergyUnit::BTU => 23_722_880_951i64.to_number() / 22_500_000i64.to_number(),
			EnergyUnit::FootPounds => {
				3_389_544_870_828_501i64.to_number() / 2_500_000_000_000_000i64.to_number()
			}
			EnergyUnit::FootPoundals => {
				6_584_392_202_157i64.to_number() / 156_250_000_000_000i64.to_number()
			}
			EnergyUnit::WattHours => 3600.to_number(),
			EnergyUnit::KilowattHours => 3_600_000.to_number(),
			EnergyUnit::Erg => 1.to_number() / 10_000_000.to_number(),
		}
	}
}

impl MultiplierUnitConversion for ForceUnit {
	fn multiplier_to_standard(&self) -> Number {
		match self {
			ForceUnit::Newton => 1.to_number(),
			ForceUnit::Kilonewton => 1000.to_number(),
			ForceUnit::Dyne => 1.to_number() / 100_000.to_number(),
			ForceUnit::KilogramForce => 196_133.to_number() / 20_000.to_number(),
			ForceUnit::PoundForce => {
				8_896_443_230_521i64.to_number() / 2_000_000_000_000i64.to_number()
			}
			ForceUnit::Poundal => 17_281_869_297i64.to_number() / 125_000_000_000i64.to_number(),
			ForceUnit::Kip => 8_896_443_230_521i64.to_number() / 2_000_000_000i64.to_number(),
		}
	}
}

impl MultiplierUnitConversion for MassUnit {
	fn multiplier_to_standard(&self) -> Number {
		match self {
			MassUnit::Grams => 1.to_number(),
			MassUnit::Milligrams => 1.to_number() / 1000.to_number(),
			MassUnit::Kilograms => 1000.to_number(),
			MassUnit::MetricTons => 1_000_000.to_number(),
			MassUnit::Pounds => 45_359_237.to_number() / 100_000.to_number(),
			MassUnit::Ounces => 45_359_237.to_number() / 1_600_000.to_number(),
			MassUnit::Stones => 317_514_659.to_number() / 50_000.to_number(),
			MassUnit::Tons => 45_359_237.to_number() / 50.to_number(),
			MassUnit::UKTons => 635_029_318.to_number() / 625.to_number(),
		}
	}
}

impl MultiplierUnitConversion for PowerUnit {
	fn multiplier_to_standard(&self) -> Number {
		match self {
			PowerUnit::Watts => 1.to_number(),
			PowerUnit::Milliwatts => 1.to_number() / 1000.to_number(),
			PowerUnit::Kilowatts => 1000.to_number(),
			PowerUnit::Megawatts => 1_000_000.to_number(),
			PowerUnit::Gigawatts => 1_000_000_000.to_number(),
			PowerUnit::MechanicalHorsepower => {
				37_284_993_579_113_511i64.to_number() / 50_000_000_000_000i64.to_number()
			}
			PowerUnit::MetricHorsepower => 588_399.to_number() / 800.to_number(),
			PowerUnit::ElectricalHorsepower => 746.to_number(),
			PowerUnit::TonsOfRefrigeration => {
				52_752_792_631i64.to_number() / 15_000_000i64.to_number()
			}
		}
	}
}

impl MultiplierUnitConversion for PressureUnit {
	fn multiplier_to_standard(&self) -> Number {
		match self {
			PressureUnit::Pascals => 1.to_number(),
			PressureUnit::Kilopascals => 1000.to_number(),
			PressureUnit::Bars => 100_000.to_number(),
			PressureUnit::Millibars => 100.to_number(),
			PressureUnit::Atmospheres => 101_325.to_number(),
			PressureUnit::InchesOfMercury => {
				3_386_388_640_341i64.to_number() / 1_000_000_000i64.to_number()
			}
			PressureUnit::MillimetersOfMercury => {
				26_664_477_483i64.to_number() / 200_000_000i64.to_number()
			}
			PressureUnit::InchesOfWater => 24_908_891.to_number() / 100_000.to_number(),
			PressureUnit::MillimetersOfWater => 196_133.to_number() / 20_000.to_number(),
			PressureUnit::PoundsPerSquareInch => {
				8_896_443_230_521i64.to_number() / 1_290_320_000i64.to_number()
			}
			PressureUnit::Torr => 20_265.to_number() / 152.to_number(),
		}
	}
}

impl UnitConversion for TemperatureUnit {
	fn to_unit(&self, value: &Number, target_unit: &Self) -> Number {
		if self == target_unit {
			return value.clone();
		}
		target_unit
			.from_celsius(&*self.to_celsius(value))
			.into_owned()
	}
}

impl MultiplierUnitConversion for TimeUnit {
	fn multiplier_to_standard(&self) -> Number {
		match self {
			TimeUnit::Nanoseconds => 1.to_number() / 1_000_000_000.to_number(),
			TimeUnit::Microseconds => 1.to_number() / 1_000_000.to_number(),
			TimeUnit::Milliseconds => 1.to_number() / 1000.to_number(),
			TimeUnit::Seconds => 1.to_number(),
			TimeUnit::Minutes => 60.to_number(),
			TimeUnit::Hours => 3600.to_number(),
			TimeUnit::Days => (3600 * 24).to_number(),
			TimeUnit::Weeks => (3600 * 24 * 7).to_number(),
			TimeUnit::Months => 2_629_746.to_number(), // Average length of 1/12 year over 400 years
			TimeUnit::Years => 31_556_952.to_number(), // Average length of year over 400 years
		}
	}
}

impl MultiplierUnitConversion for VolumeUnit {
	fn multiplier_to_standard(&self) -> Number {
		match self {
			VolumeUnit::Litre => 1.to_number(),
			VolumeUnit::Millilitre => 1.to_number() / 1000.to_number(),
			VolumeUnit::Gallons => 473_176_473.to_number() / 125_000_000.to_number(),
			VolumeUnit::Quarts => 473_176_473.to_number() / 500_000_000.to_number(),
			VolumeUnit::Pints => 473_176_473.to_number() / 1_000_000_000.to_number(),
			VolumeUnit::Cups => 473_176_473.to_number() / 2_000_000_000.to_number(),
			VolumeUnit::FluidOunces => 473_176_473i64.to_number() / 16_000_000_000i64.to_number(),
			VolumeUnit::ImperialGallons => 454_609.to_number() / 100_000.to_number(),
			VolumeUnit::ImperialQuarts => 454_609.to_number() / 400_000.to_number(),
			VolumeUnit::ImperialPints => 454_609.to_number() / 800_000.to_number(),
			VolumeUnit::ImperialOunces => 454_609.to_number() / 16_000_000.to_number(),
			VolumeUnit::Tablespoons => 473_176_473i64.to_number() / 32_000_000_000i64.to_number(),
			VolumeUnit::Teaspoons => 473_176_473i64.to_number() / 96_000_000_000i64.to_number(),
			VolumeUnit::UKTablespoons => 3.to_number() / 200.to_number(),
			VolumeUnit::UKTeaspoons => 1.to_number() / 200.to_number(),
		}
	}
}

impl Unit {
	pub fn unit_type(&self) -> UnitType {
		match self {
			Unit::Angle(_) => UnitType::Angle,
			Unit::Area(_) => UnitType::Area,
			Unit::Distance(_) => UnitType::Distance,
			Unit::Energy(_) => UnitType::Energy,
			Unit::Force(_) => UnitType::Force,
			Unit::Mass(_) => UnitType::Mass,
			Unit::Power(_) => UnitType::Power,
			Unit::Pressure(_) => UnitType::Pressure,
			Unit::Temperature(_) => UnitType::Temperature,
			Unit::Time(_) => UnitType::Time,
			Unit::Volume(_) => UnitType::Volume,
		}
	}
}

impl From<AngleUnit> for Unit {
	fn from(unit: AngleUnit) -> Self {
		Unit::Angle(unit)
	}
}

impl From<AreaUnit> for Unit {
	fn from(unit: AreaUnit) -> Self {
		Unit::Area(unit)
	}
}

impl From<DistanceUnit> for Unit {
	fn from(unit: DistanceUnit) -> Self {
		Unit::Distance(unit)
	}
}

impl From<EnergyUnit> for Unit {
	fn from(unit: EnergyUnit) -> Self {
		Unit::Energy(unit)
	}
}

impl From<ForceUnit> for Unit {
	fn from(unit: ForceUnit) -> Self {
		Unit::Force(unit)
	}
}

impl From<MassUnit> for Unit {
	fn from(unit: MassUnit) -> Self {
		Unit::Mass(unit)
	}
}

impl From<PowerUnit> for Unit {
	fn from(unit: PowerUnit) -> Self {
		Unit::Power(unit)
	}
}

impl From<PressureUnit> for Unit {
	fn from(unit: PressureUnit) -> Self {
		Unit::Pressure(unit)
	}
}

impl From<TemperatureUnit> for Unit {
	fn from(unit: TemperatureUnit) -> Self {
		Unit::Temperature(unit)
	}
}

impl From<TimeUnit> for Unit {
	fn from(unit: TimeUnit) -> Self {
		Unit::Time(unit)
	}
}

impl From<VolumeUnit> for Unit {
	fn from(unit: VolumeUnit) -> Self {
		Unit::Volume(unit)
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
			Unit::Angle(from) => match to_unit {
				Unit::Angle(to) => Ok(from.to_unit_with_power(value, to, power)),
				_ => Err(Error::IncompatibleUnits),
			},
			Unit::Area(from) => match to_unit {
				Unit::Area(to) => Ok(from.to_unit_with_power(value, to, power)),
				Unit::Distance(to) => Ok(DistanceUnit::Meters.to_unit_with_power(
					&from.to_square_meters_with_power(value, power),
					to,
					power * 2,
				)),
				_ => Err(Error::IncompatibleUnits),
			},
			Unit::Distance(from) => match to_unit {
				Unit::Area(to) => {
					if power % 2 == 0 {
						Ok(to.from_square_meters_with_power(
							&from.to_unit_with_power(value, &DistanceUnit::Meters, power),
							power / 2,
						))
					} else {
						Err(Error::IncompatibleUnits)
					}
				}
				Unit::Volume(to) => {
					if power % 3 == 0 {
						Ok(to.from_cubic_meters_with_power(
							&from.to_unit_with_power(value, &DistanceUnit::Meters, power),
							power / 3,
						))
					} else {
						Err(Error::IncompatibleUnits)
					}
				}
				Unit::Distance(to) => Ok(from.to_unit_with_power(value, to, power)),
				_ => Err(Error::IncompatibleUnits),
			},
			Unit::Energy(from) => match to_unit {
				Unit::Energy(to) => Ok(from.to_unit_with_power(value, to, power)),
				_ => Err(Error::IncompatibleUnits),
			},
			Unit::Force(from) => match to_unit {
				Unit::Force(to) => Ok(from.to_unit_with_power(value, to, power)),
				_ => Err(Error::IncompatibleUnits),
			},
			Unit::Mass(from) => match to_unit {
				Unit::Mass(to) => Ok(from.to_unit_with_power(value, to, power)),
				_ => Err(Error::IncompatibleUnits),
			},
			Unit::Power(from) => match to_unit {
				Unit::Power(to) => Ok(from.to_unit_with_power(value, to, power)),
				_ => Err(Error::IncompatibleUnits),
			},
			Unit::Pressure(from) => match to_unit {
				Unit::Pressure(to) => Ok(from.to_unit_with_power(value, to, power)),
				_ => Err(Error::IncompatibleUnits),
			},
			Unit::Temperature(from) => match to_unit {
				Unit::Temperature(to) => Ok(from.to_unit_with_power(value, to, power)),
				_ => Err(Error::IncompatibleUnits),
			},
			Unit::Time(from) => match to_unit {
				Unit::Time(to) => Ok(from.to_unit_with_power(value, to, power)),
				_ => Err(Error::IncompatibleUnits),
			},
			Unit::Volume(from) => match to_unit {
				Unit::Volume(to) => Ok(from.to_unit_with_power(value, to, power)),
				Unit::Distance(to) => Ok(DistanceUnit::Meters.to_unit_with_power(
					&from.to_cubic_meters_with_power(value, power),
					to,
					power * 3,
				)),
				_ => Err(Error::IncompatibleUnits),
			},
		}
	}

	fn collapse_composite_unit_types(&mut self, value: Number) -> Number {
		let mut value = value;

		// Check for area unit alongside distance unit
		if self.units.contains_key(&UnitType::Distance) && self.units.contains_key(&UnitType::Area)
		{
			// Collapse area unit into distance unit
			let area_unit = self.units.get(&UnitType::Area).unwrap().clone();
			self.units.remove(&UnitType::Area);
			let distance_unit = self.units.get_mut(&UnitType::Distance).unwrap();
			value =
				Self::convert_value_of_unit(&value, &area_unit.0, &distance_unit.0, area_unit.1)
					.unwrap();
			distance_unit.1 += area_unit.1 * 2;
			if distance_unit.1 == 0 {
				self.units.remove(&UnitType::Distance);
			}
		}

		// Check for volume unit alongside distance unit
		if self.units.contains_key(&UnitType::Distance)
			&& self.units.contains_key(&UnitType::Volume)
		{
			// Collapse volume unit into distance unit
			let volume_unit = self.units.get(&UnitType::Volume).unwrap().clone();
			self.units.remove(&UnitType::Volume);
			let distance_unit = self.units.get_mut(&UnitType::Distance).unwrap();
			value = Self::convert_value_of_unit(
				&value,
				&volume_unit.0,
				&distance_unit.0,
				volume_unit.1,
			)
			.unwrap();
			distance_unit.1 += volume_unit.1 * 3;
			if distance_unit.1 == 0 {
				self.units.remove(&UnitType::Distance);
			}
		}

		value
	}

	pub fn add_unit(&mut self, value: &Number, unit: Unit) -> Number {
		let unit_type = unit.unit_type();
		let new_value = if let Some(existing_unit) = self.units.get_mut(&unit_type) {
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
		};
		self.collapse_composite_unit_types(new_value)
	}

	pub fn add_unit_inv(&mut self, value: &Number, unit: Unit) -> Number {
		let unit_type = unit.unit_type();
		let new_value = if let Some(existing_unit) = self.units.get_mut(&unit_type) {
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
		};
		self.collapse_composite_unit_types(new_value)
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
		} else if unit_type == UnitType::Area {
			if let Some(existing_unit) = self.units.get_mut(&UnitType::Distance) {
				let value = Self::convert_value_of_unit(
					value,
					&existing_unit.0,
					&target_unit,
					existing_unit.1,
				)?;
				let distance_power = existing_unit.1;
				self.units.remove(&UnitType::Distance);
				self.units
					.insert(UnitType::Area, (target_unit, distance_power / 2));
				Ok(value)
			} else {
				Err(Error::IncompatibleUnits)
			}
		} else if unit_type == UnitType::Volume {
			if let Some(existing_unit) = self.units.get_mut(&UnitType::Distance) {
				let value = Self::convert_value_of_unit(
					value,
					&existing_unit.0,
					&target_unit,
					existing_unit.1,
				)?;
				let distance_power = existing_unit.1;
				self.units.remove(&UnitType::Distance);
				self.units
					.insert(UnitType::Volume, (target_unit, distance_power / 3));
				Ok(value)
			} else {
				Err(Error::IncompatibleUnits)
			}
		} else if unit_type == UnitType::Distance {
			if let Some(existing_unit) = self.units.get_mut(&UnitType::Area) {
				let value = Self::convert_value_of_unit(
					value,
					&existing_unit.0,
					&target_unit,
					existing_unit.1,
				)?;
				let area_power = existing_unit.1;
				self.units.remove(&UnitType::Area);
				self.units
					.insert(UnitType::Distance, (target_unit, area_power * 2));
				Ok(value)
			} else if let Some(existing_unit) = self.units.get_mut(&UnitType::Volume) {
				let value = Self::convert_value_of_unit(
					value,
					&existing_unit.0,
					&target_unit,
					existing_unit.1,
				)?;
				let volume_power = existing_unit.1;
				self.units.remove(&UnitType::Volume);
				self.units
					.insert(UnitType::Distance, (target_unit, volume_power * 3));
				Ok(value)
			} else {
				Err(Error::IncompatibleUnits)
			}
		} else {
			Err(Error::IncompatibleUnits)
		}
	}

	pub fn coerce_to_other(&self, value: &Number, target_units: &CompositeUnit) -> Result<Number> {
		// First convert composite unit types (like area) into the base unit types
		let mut result = value.clone();
		let mut collapsed_units = self.clone();
		if collapsed_units.units.contains_key(&UnitType::Area) {
			result = collapsed_units
				.convert_single_unit(&result, Unit::Distance(DistanceUnit::Meters))?;
		}
		if collapsed_units.units.contains_key(&UnitType::Volume) {
			result = collapsed_units
				.convert_single_unit(&result, Unit::Distance(DistanceUnit::Meters))?;
		}

		let mut collapsed_target_units = target_units.clone();
		if collapsed_target_units.units.contains_key(&UnitType::Area) {
			// Collapse area unit into distance unit
			let area_unit = collapsed_target_units
				.units
				.get(&UnitType::Area)
				.unwrap()
				.clone();
			collapsed_target_units.units.remove(&UnitType::Area);
			if let Some(distance_unit) = collapsed_target_units.units.get_mut(&UnitType::Distance) {
				distance_unit.1 += area_unit.1 * 2;
				if distance_unit.1 == 0 {
					collapsed_target_units.units.remove(&UnitType::Distance);
				}
			} else {
				collapsed_target_units.units.insert(
					UnitType::Distance,
					(Unit::Distance(DistanceUnit::Meters), area_unit.1 * 2),
				);
			}
		}
		if collapsed_target_units.units.contains_key(&UnitType::Volume) {
			// Collapse volume unit into distance unit
			let volume_unit = collapsed_target_units
				.units
				.get(&UnitType::Volume)
				.unwrap()
				.clone();
			collapsed_target_units.units.remove(&UnitType::Volume);
			if let Some(distance_unit) = collapsed_target_units.units.get_mut(&UnitType::Distance) {
				distance_unit.1 += volume_unit.1 * 3;
				if distance_unit.1 == 0 {
					collapsed_target_units.units.remove(&UnitType::Distance);
				}
			} else {
				collapsed_target_units.units.insert(
					UnitType::Distance,
					(Unit::Distance(DistanceUnit::Meters), volume_unit.1 * 3),
				);
			}
		}

		// Check units to make sure they are compatible. There must be the same set of
		// unit types and each unit type must be the same power.
		for (unit_type, unit) in collapsed_units.units.iter() {
			if let Some(target) = collapsed_target_units.units.get(&unit_type) {
				if unit.1 != target.1 {
					return Err(Error::IncompatibleUnits);
				}
			} else {
				return Err(Error::IncompatibleUnits);
			}
		}
		for (unit_type, unit) in collapsed_target_units.units.iter() {
			if let Some(target) = collapsed_units.units.get(&unit_type) {
				if unit.1 != target.1 {
					return Err(Error::IncompatibleUnits);
				}
			} else {
				return Err(Error::IncompatibleUnits);
			}
		}

		// Convert units to the target unit
		for (_, value) in collapsed_target_units.units.iter() {
			result = collapsed_units.convert_single_unit(&result, value.0)?;
		}

		// Convert any composite unit types collapsed earlier back to the target unit
		if let Some(area_unit) = target_units.units.get(&UnitType::Area) {
			result = collapsed_units.convert_single_unit(&result, area_unit.0)?;
		}
		if let Some(volume_unit) = target_units.units.get(&UnitType::Volume) {
			result = collapsed_units.convert_single_unit(&result, volume_unit.0)?;
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
		self.collapse_composite_unit_types(result)
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

fn value_layout() -> Box<dyn Fn(&State, &dyn Screen) -> Layout> {
	Box::new(|state, screen| {
		let value = state.top();
		let value_layout = value.render(&state.format(), &None, screen.width());
		let mut layout_items = Vec::new();
		layout_items.push(Layout::HorizontalRule);
		layout_items.push(value_layout);
		Layout::Vertical(layout_items)
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
				Function::ClearUnits,
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
				Function::AddUnit(*unit),
				Function::AddInvUnit(*unit),
				Function::ConvertToUnit(*unit),
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
							Function::AddUnitSquared(*unit),
							Function::AddInvUnitSquared(*unit),
							Function::ConvertToUnit(*unit),
						),
					});
				}
				UnitType::Volume => {
					items.push(MenuItem {
						layout: MenuItemLayout::Static(MenuItem::string_layout_small(
							unit.to_str().to_string() + "³",
						)),
						function: MenuItemFunction::ConversionAction(
							Function::AddUnitCubed(*unit),
							Function::AddInvUnitCubed(*unit),
							Function::ConvertToUnit(*unit),
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
