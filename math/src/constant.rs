use crate::number::{Number, ToNumber};
use crate::unit::{CompositeUnit, DistanceUnit, TimeUnit};
use crate::value::Value;
use intel_dfp::Decimal;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Constant {
	Pi,
	SpeedOfLight,
}

impl Constant {
	pub fn to_str(&self) -> &'static str {
		match self {
			Constant::Pi => "π",
			Constant::SpeedOfLight => "c",
		}
	}

	pub fn value(&self) -> Value {
		match self {
			Constant::Pi => Value::Number(Number::Decimal(Decimal::pi())),
			Constant::SpeedOfLight => Value::NumberWithUnit(
				299_792_458.to_number(),
				CompositeUnit::ratio_unit(DistanceUnit::Meters.into(), TimeUnit::Seconds.into()),
			),
		}
	}
}
