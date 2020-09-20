use crate::error::{Error, Result};
use crate::font::SANS_13;
use crate::input::InputEvent;
use crate::menu::settings_menu;
use crate::number::{
	IntegerMode, Number, NumberDecimalPointMode, NumberFormat, NumberFormatMode, ToNumber,
	MAX_INTEGER_BITS,
};
use crate::screen::{Color, Rect, Screen};
use crate::state::{State, StatusBarLeftDisplayType};
use crate::time::Now;
use crate::unit::{
	unit_menu, unit_menu_of_type, AngleUnit, CompositeUnit, DistanceUnit, TimeUnit, Unit, UnitType,
};
use crate::value::Value;
use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use core::cell::RefCell;
use core::convert::TryFrom;
use num_bigint::ToBigInt;

#[cfg(feature = "dm42")]
use crate::dm42::{set_time_24_hour, time_24_hour};
#[cfg(not(feature = "dm42"))]
use crate::time::{set_time_24_hour, time_24_hour};

#[derive(PartialEq, Eq, Clone, Copy)]
#[allow(dead_code)]
pub enum Function {
	Input(InputEvent),
	NormalFormat,
	RationalFormat,
	ScientificFormat,
	EngineeringFormat,
	AlternateHex,
	AlternateFloat,
	ThousandsSeparatorOff,
	ThousandsSeparatorOn,
	DecimalPointPeriod,
	DecimalPointComma,
	Float,
	SignedInteger,
	UnsignedInteger,
	BigInteger,
	Signed8Bit,
	Signed16Bit,
	Signed32Bit,
	Signed64Bit,
	Signed128Bit,
	Unsigned8Bit,
	Unsigned16Bit,
	Unsigned32Bit,
	Unsigned64Bit,
	Unsigned128Bit,
	And,
	Or,
	Xor,
	Not,
	ShiftLeft,
	ShiftRight,
	RotateLeft,
	RotateRight,
	Hex,
	Octal,
	Decimal,
	ConstCatalog,
	SpeedOfLight,
	TimeCatalog,
	Now,
	Date,
	Time,
	Degrees,
	Radians,
	Gradians,
	UnitMenu(UnitType),
	AddUnit(Unit, UnitType),
	AddUnitSquared(Unit, UnitType),
	AddUnitCubed(Unit, UnitType),
	AddInvUnit(Unit, UnitType),
	AddInvUnitSquared(Unit, UnitType),
	AddInvUnitCubed(Unit, UnitType),
	ConvertToUnit(Unit, UnitType),
	SettingsMenu,
	SystemMenu,
	Time24HourToggle,
	StatusBarLeftDisplayToggle,
}

impl Function {
	pub fn to_string(&self, state: &State) -> String {
		match self {
			Function::Input(input) => input.to_string(),
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
			Function::AlternateHex => {
				if state.format.show_alt_hex {
					"▪↓Hex".to_string()
				} else {
					"↓Hex".to_string()
				}
			}
			Function::AlternateFloat => {
				if state.format.show_alt_float {
					"▪↓Flt".to_string()
				} else {
					"↓Flt".to_string()
				}
			}
			Function::ThousandsSeparatorOff => {
				if state.format.thousands {
					"1000".to_string()
				} else {
					"▪1000".to_string()
				}
			}
			Function::ThousandsSeparatorOn => {
				if state.format.thousands {
					"▪1,000".to_string()
				} else {
					"1,000".to_string()
				}
			}
			Function::DecimalPointPeriod => {
				if state.format.decimal_point == NumberDecimalPointMode::Period {
					"▪0.5".to_string()
				} else {
					"0.5".to_string()
				}
			}
			Function::DecimalPointComma => {
				if state.format.decimal_point == NumberDecimalPointMode::Comma {
					"▪0,5".to_string()
				} else {
					"0,5".to_string()
				}
			}
			Function::Float => {
				if state.format.integer_mode == IntegerMode::Float {
					"▪float".to_string()
				} else {
					"float".to_string()
				}
			}
			Function::SignedInteger => match state.format.integer_mode {
				IntegerMode::BigInteger | IntegerMode::SizedInteger(_, true) => "▪int".to_string(),
				_ => "int".to_string(),
			},
			Function::UnsignedInteger => match state.format.integer_mode {
				IntegerMode::SizedInteger(_, false) => "▪uint".to_string(),
				_ => "uint".to_string(),
			},
			Function::BigInteger => {
				if state.format.integer_mode == IntegerMode::BigInteger {
					"▪int∞".to_string()
				} else {
					"int∞".to_string()
				}
			}
			Function::Signed8Bit => {
				if state.format.integer_mode == IntegerMode::SizedInteger(8, true) {
					"▪i8".to_string()
				} else {
					"i8".to_string()
				}
			}
			Function::Signed16Bit => {
				if state.format.integer_mode == IntegerMode::SizedInteger(16, true) {
					"▪i16".to_string()
				} else {
					"i16".to_string()
				}
			}
			Function::Signed32Bit => {
				if state.format.integer_mode == IntegerMode::SizedInteger(32, true) {
					"▪i32".to_string()
				} else {
					"i32".to_string()
				}
			}
			Function::Signed64Bit => {
				if state.format.integer_mode == IntegerMode::SizedInteger(64, true) {
					"▪i64".to_string()
				} else {
					"i64".to_string()
				}
			}
			Function::Signed128Bit => {
				if state.format.integer_mode == IntegerMode::SizedInteger(128, true) {
					"▪i128".to_string()
				} else {
					"i128".to_string()
				}
			}
			Function::Unsigned8Bit => {
				if state.format.integer_mode == IntegerMode::SizedInteger(8, false) {
					"▪u8".to_string()
				} else {
					"u8".to_string()
				}
			}
			Function::Unsigned16Bit => {
				if state.format.integer_mode == IntegerMode::SizedInteger(16, false) {
					"▪u16".to_string()
				} else {
					"u16".to_string()
				}
			}
			Function::Unsigned32Bit => {
				if state.format.integer_mode == IntegerMode::SizedInteger(32, false) {
					"▪u32".to_string()
				} else {
					"u32".to_string()
				}
			}
			Function::Unsigned64Bit => {
				if state.format.integer_mode == IntegerMode::SizedInteger(64, false) {
					"▪u64".to_string()
				} else {
					"u64".to_string()
				}
			}
			Function::Unsigned128Bit => {
				if state.format.integer_mode == IntegerMode::SizedInteger(128, false) {
					"▪u128".to_string()
				} else {
					"u128".to_string()
				}
			}
			Function::And => "and".to_string(),
			Function::Or => "or".to_string(),
			Function::Xor => "xor".to_string(),
			Function::Not => "not".to_string(),
			Function::ShiftLeft => "<<".to_string(),
			Function::ShiftRight => ">>".to_string(),
			Function::RotateLeft => "rol".to_string(),
			Function::RotateRight => "ror".to_string(),
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
			Function::ConstCatalog => "Const".to_string(),
			Function::SpeedOfLight => "c".to_string(),
			Function::TimeCatalog => "Time".to_string(),
			Function::Now => "Now".to_string(),
			Function::Date => "Date".to_string(),
			Function::Time => "Time".to_string(),
			Function::Degrees => {
				if state.angle_mode == AngleUnit::Degrees {
					"▪Deg".to_string()
				} else {
					"Deg".to_string()
				}
			}
			Function::Radians => {
				if state.angle_mode == AngleUnit::Radians {
					"▪Rad".to_string()
				} else {
					"Rad".to_string()
				}
			}
			Function::Gradians => {
				if state.angle_mode == AngleUnit::Gradians {
					"▪Grad".to_string()
				} else {
					"Grad".to_string()
				}
			}
			Function::UnitMenu(unit_type) => unit_type.to_str().to_string(),
			Function::AddUnit(unit, _) => unit.to_str().to_string(),
			Function::AddUnitSquared(unit, _) => unit.to_str().to_string() + "²",
			Function::AddUnitCubed(unit, _) => unit.to_str().to_string() + "³",
			Function::AddInvUnit(unit, _) => "/".to_string() + &unit.to_str(),
			Function::AddInvUnitSquared(unit, _) => "/".to_string() + &unit.to_str() + "²",
			Function::AddInvUnitCubed(unit, _) => "/".to_string() + &unit.to_str() + "³",
			Function::ConvertToUnit(unit, _) => "▸".to_string() + &unit.to_str(),
			Function::SettingsMenu => "Settings".to_string(),
			Function::SystemMenu => "Sys".to_string(),
			Function::Time24HourToggle => "24Hr".to_string(),
			Function::StatusBarLeftDisplayToggle => "StatusBar".to_string(),
		}
	}

	pub fn execute<ScreenT: Screen>(&self, state: &mut State, screen: &ScreenT) -> Result<()> {
		match self {
			Function::Input(input) => {
				state.handle_input(*input, screen)?;
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
			Function::AlternateHex => {
				state.format.show_alt_hex = !state.format.show_alt_hex;
			}
			Function::AlternateFloat => {
				state.format.show_alt_float = !state.format.show_alt_float;
			}
			Function::ThousandsSeparatorOff => {
				state.format.thousands = false;
			}
			Function::ThousandsSeparatorOn => {
				state.format.thousands = true;
			}
			Function::DecimalPointPeriod => {
				state.format.decimal_point = NumberDecimalPointMode::Period;
			}
			Function::DecimalPointComma => {
				state.format.decimal_point = NumberDecimalPointMode::Comma;
			}
			Function::Float => {
				if state.format.integer_radix == 10 {
					state.format.integer_mode = IntegerMode::Float;
					state.stack.end_edit();
				} else {
					return Err(Error::FloatRequiresDecimalMode);
				}
			}
			Function::SignedInteger => {
				state.function_keys.show_menu(FunctionMenu::SignedInteger);
			}
			Function::UnsignedInteger => {
				state.function_keys.show_menu(FunctionMenu::UnsignedInteger);
			}
			Function::BigInteger => {
				state.format.integer_mode = IntegerMode::BigInteger;
				state.default_integer_format = state.format.integer_mode;
				state.stack.end_edit();
			}
			Function::Signed8Bit => {
				state.format.integer_mode = IntegerMode::SizedInteger(8, true);
				state.default_integer_format = state.format.integer_mode;
				state.stack.end_edit();
			}
			Function::Signed16Bit => {
				state.format.integer_mode = IntegerMode::SizedInteger(16, true);
				state.default_integer_format = state.format.integer_mode;
				state.stack.end_edit();
			}
			Function::Signed32Bit => {
				state.format.integer_mode = IntegerMode::SizedInteger(32, true);
				state.default_integer_format = state.format.integer_mode;
				state.stack.end_edit();
			}
			Function::Signed64Bit => {
				state.format.integer_mode = IntegerMode::SizedInteger(64, true);
				state.default_integer_format = state.format.integer_mode;
				state.stack.end_edit();
			}
			Function::Signed128Bit => {
				state.format.integer_mode = IntegerMode::SizedInteger(128, true);
				state.default_integer_format = state.format.integer_mode;
				state.stack.end_edit();
			}
			Function::Unsigned8Bit => {
				state.format.integer_mode = IntegerMode::SizedInteger(8, false);
				state.default_integer_format = state.format.integer_mode;
				state.stack.end_edit();
			}
			Function::Unsigned16Bit => {
				state.format.integer_mode = IntegerMode::SizedInteger(16, false);
				state.default_integer_format = state.format.integer_mode;
				state.stack.end_edit();
			}
			Function::Unsigned32Bit => {
				state.format.integer_mode = IntegerMode::SizedInteger(32, false);
				state.default_integer_format = state.format.integer_mode;
				state.stack.end_edit();
			}
			Function::Unsigned64Bit => {
				state.format.integer_mode = IntegerMode::SizedInteger(64, false);
				state.default_integer_format = state.format.integer_mode;
				state.stack.end_edit();
			}
			Function::Unsigned128Bit => {
				state.format.integer_mode = IntegerMode::SizedInteger(128, false);
				state.default_integer_format = state.format.integer_mode;
				state.stack.end_edit();
			}
			Function::And => {
				let value = Value::Number(Number::Integer(
					&*state.stack.entry(1)?.to_int()? & &*state.stack.entry(0)?.to_int()?,
				));
				state.replace_entries(2, value)?;
			}
			Function::Or => {
				let value = Value::Number(Number::Integer(
					&*state.stack.entry(1)?.to_int()? | &*state.stack.entry(0)?.to_int()?,
				));
				state.replace_entries(2, value)?;
			}
			Function::Xor => {
				let value = Value::Number(Number::Integer(
					&*state.stack.entry(1)?.to_int()? ^ &*state.stack.entry(0)?.to_int()?,
				));
				state.replace_entries(2, value)?;
			}
			Function::Not => {
				let value = Number::Integer(!&*state.stack.top().to_int()?);
				state.set_top(Value::Number(value))?;
			}
			Function::ShiftLeft => {
				let x = state.stack.entry(0)?;
				let mut x = x.to_int()?;
				if let IntegerMode::SizedInteger(size, _) = state.format.integer_mode {
					if size.is_power_of_two() {
						x = Cow::Owned(&*x & &(size - 1).to_bigint().unwrap());
					}
				}
				let x = u32::try_from(&*x)?;
				let y = state.stack.entry(1)?;
				let y = y.to_int()?;
				if (y.bits() + x as u64) > MAX_INTEGER_BITS {
					return Err(Error::ValueOutOfRange);
				}
				let value = Value::Number(Number::Integer(&*y << x));
				state.replace_entries(2, value)?;
			}
			Function::ShiftRight => {
				let x = state.stack.entry(0)?;
				let mut x = x.to_int()?;
				if let IntegerMode::SizedInteger(size, _) = state.format.integer_mode {
					if size.is_power_of_two() {
						x = Cow::Owned(&*x & (size - 1).to_bigint().unwrap());
					}
				}
				let x = u32::try_from(&*x)?;
				let y = state.stack.entry(1)?;
				let y = y.to_int()?;
				let value = Value::Number(Number::Integer(&*y >> x));
				state.replace_entries(2, value)?;
			}
			Function::RotateLeft => {
				if let IntegerMode::SizedInteger(size, _) = state.format.integer_mode {
					let x = state.stack.entry(0)?;
					let mut x = x.to_int()?;
					if size.is_power_of_two() {
						x = Cow::Owned(&*x & (size - 1).to_bigint().unwrap());
					}
					if let Ok(x) = u32::try_from(&*x) {
						let y = state.stack.entry(1)?;
						let y = y.to_int()?;
						let value = (&*y << x) | (&*y >> ((size as u32) - x));
						state.replace_entries(2, Value::Number(Number::Integer(value)))?;
					}
				} else {
					return Err(Error::RequiresSizedIntegerMode);
				}
			}
			Function::RotateRight => {
				if let IntegerMode::SizedInteger(size, _) = state.format.integer_mode {
					let x = state.stack.entry(0)?;
					let mut x = x.to_int()?;
					if size.is_power_of_two() {
						x = Cow::Owned(&*x & (size - 1).to_bigint().unwrap());
					}
					if let Ok(x) = u32::try_from(&*x) {
						let y = state.stack.entry(1)?;
						let y = y.to_int()?;
						let value = (&*y >> x) | (&*y << ((size as u32) - x));
						state.replace_entries(2, Value::Number(Number::Integer(value)))?;
					}
				} else {
					return Err(Error::RequiresSizedIntegerMode);
				}
			}
			Function::Hex => {
				if state.format.integer_radix == 10 {
					state.prev_decimal_integer_mode = state.format.integer_mode;
					state.format.integer_mode = state.default_integer_format;
				}
				state.format.integer_radix = 16;
				state.stack.end_edit();
			}
			Function::Octal => {
				if state.format.integer_radix == 10 {
					state.prev_decimal_integer_mode = state.format.integer_mode;
					state.format.integer_mode = state.default_integer_format;
				}
				state.format.integer_radix = 8;
				state.stack.end_edit();
			}
			Function::Decimal => {
				if state.format.integer_radix != 10 {
					state.format.integer_mode = state.prev_decimal_integer_mode;
				}
				state.format.integer_radix = 10;
				state.stack.end_edit();
			}
			Function::ConstCatalog => state.function_keys.show_menu(FunctionMenu::ConstCatalog),
			Function::SpeedOfLight => {
				state.stack.input_value(Value::NumberWithUnit(
					299_792_458.to_number(),
					CompositeUnit::ratio_unit(
						DistanceUnit::Meters.into(),
						TimeUnit::Seconds.into(),
					),
				))?;
			}
			Function::TimeCatalog => state.function_keys.show_menu(FunctionMenu::TimeCatalog),
			Function::Now => {
				state
					.stack
					.input_value(Value::DateTime(NaiveDateTime::now()))?;
			}
			Function::Date => {
				if let Value::DateTime(dt) = state.stack.top() {
					let date = dt.date();
					state.stack.set_top(Value::Date(date))?;
				} else {
					let year = i32::try_from(&*state.stack.entry(2)?.to_int()?)?;
					let month = u8::try_from(&*state.stack.entry(1)?.to_int()?)?;
					let day = u8::try_from(&*state.stack.entry(0)?.to_int()?)?;
					let date = NaiveDate::from_ymd_opt(year, month as u32, day as u32)
						.ok_or(Error::InvalidDate)?;
					{
						state.stack.replace_entries(3, Value::Date(date))?;
					}
				}
			}
			Function::Time => {
				if let Value::DateTime(dt) = state.stack.top() {
					let time = dt.time();
					state.stack.set_top(Value::Time(time))?;
				} else {
					let nano = (state.stack.entry(0)?
						* Value::Number(Number::Integer(1_000_000_000.to_bigint().unwrap())))?;
					let hr = u8::try_from(&*state.stack.entry(2)?.to_int()?)?;
					let min = u8::try_from(&*state.stack.entry(1)?.to_int()?)?;
					let sec = u64::try_from(&*nano.to_int()?)?;
					let nsec = (sec % 1_000_000_000) as u32;
					let sec = (sec / 1_000_000_000) as u32;
					let time = NaiveTime::from_hms_nano_opt(hr as u32, min as u32, sec, nsec)
						.ok_or(Error::InvalidTime)?;
					state.stack.replace_entries(3, Value::Time(time))?;
				}
			}
			Function::Degrees => {
				state.angle_mode = AngleUnit::Degrees;
			}
			Function::Radians => {
				state.angle_mode = AngleUnit::Radians;
			}
			Function::Gradians => {
				state.angle_mode = AngleUnit::Gradians;
			}
			Function::UnitMenu(unit_type) => {
				let menu = unit_menu_of_type(state, screen, &state.top(), *unit_type);
				state.show_menu(menu);
			}
			Function::AddUnit(unit, unit_type) => {
				let value = state.stack.top().add_unit(*unit)?;
				state.set_top(value)?;
				let menu = unit_menu_of_type(state, screen, &state.top(), *unit_type);
				let parent = unit_menu(state, screen, &state.top());
				let mut menus = Vec::new();
				menus.push(parent);
				menus.push(menu);
				state.refresh_menu_stack(menus);
			}
			Function::AddUnitSquared(unit, unit_type) => {
				let value = state.stack.top().add_unit(*unit)?;
				let value = value.add_unit(*unit)?;
				state.set_top(value)?;
				let menu = unit_menu_of_type(state, screen, &state.top(), *unit_type);
				let parent = unit_menu(state, screen, &state.top());
				let mut menus = Vec::new();
				menus.push(parent);
				menus.push(menu);
				state.refresh_menu_stack(menus);
			}
			Function::AddUnitCubed(unit, unit_type) => {
				let value = state.stack.top().add_unit(*unit)?;
				let value = value.add_unit(*unit)?;
				let value = value.add_unit(*unit)?;
				state.set_top(value)?;
				let menu = unit_menu_of_type(state, screen, &state.top(), *unit_type);
				let parent = unit_menu(state, screen, &state.top());
				let mut menus = Vec::new();
				menus.push(parent);
				menus.push(menu);
				state.refresh_menu_stack(menus);
			}
			Function::AddInvUnit(unit, unit_type) => {
				let value = state.stack.top().add_unit_inv(*unit)?;
				state.set_top(value)?;
				let menu = unit_menu_of_type(state, screen, &state.top(), *unit_type);
				let parent = unit_menu(state, screen, &state.top());
				let mut menus = Vec::new();
				menus.push(parent);
				menus.push(menu);
				state.refresh_menu_stack(menus);
			}
			Function::AddInvUnitSquared(unit, unit_type) => {
				let value = state.stack.top().add_unit_inv(*unit)?;
				let value = value.add_unit_inv(*unit)?;
				state.set_top(value)?;
				let menu = unit_menu_of_type(state, screen, &state.top(), *unit_type);
				let parent = unit_menu(state, screen, &state.top());
				let mut menus = Vec::new();
				menus.push(parent);
				menus.push(menu);
				state.refresh_menu_stack(menus);
			}
			Function::AddInvUnitCubed(unit, unit_type) => {
				let value = state.stack.top().add_unit_inv(*unit)?;
				let value = value.add_unit_inv(*unit)?;
				let value = value.add_unit_inv(*unit)?;
				state.set_top(value)?;
				let menu = unit_menu_of_type(state, screen, &state.top(), *unit_type);
				let parent = unit_menu(state, screen, &state.top());
				let mut menus = Vec::new();
				menus.push(parent);
				menus.push(menu);
				state.refresh_menu_stack(menus);
			}
			Function::ConvertToUnit(unit, unit_type) => {
				let value = state.stack.top().convert_single_unit(*unit)?;
				state.set_top(value)?;
				let menu = unit_menu_of_type(state, screen, &state.top(), *unit_type);
				let parent = unit_menu(state, screen, &state.top());
				let mut menus = Vec::new();
				menus.push(parent);
				menus.push(menu);
				state.refresh_menu_stack(menus);
			}
			Function::SettingsMenu => {
				let menu = settings_menu(state);
				state.show_menu(menu);
			}
			Function::SystemMenu => {
				state.show_system_setup_menu();
			}
			Function::Time24HourToggle => {
				set_time_24_hour(!time_24_hour());
				let menu = settings_menu(state);
				state.refresh_menu(menu);
			}
			Function::StatusBarLeftDisplayToggle => {
				state.status_bar_left_display = match state.status_bar_left_display {
					StatusBarLeftDisplayType::CurrentTime => StatusBarLeftDisplayType::FreeMemory,
					StatusBarLeftDisplayType::FreeMemory => StatusBarLeftDisplayType::CurrentTime,
				};
				let menu = settings_menu(state);
				state.refresh_menu(menu);
			}
		}
		Ok(())
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FunctionMenu {
	Disp,
	Mode,
	Base,
	SignedInteger,
	UnsignedInteger,
	Logic,
	Catalog,
	ConstCatalog,
	TimeCatalog,
}

impl FunctionMenu {
	pub fn functions(&self) -> Vec<Option<Function>> {
		match self {
			FunctionMenu::Disp => [
				Some(Function::NormalFormat),
				Some(Function::RationalFormat),
				Some(Function::ScientificFormat),
				Some(Function::EngineeringFormat),
				Some(Function::AlternateHex),
				Some(Function::AlternateFloat),
				Some(Function::ThousandsSeparatorOff),
				Some(Function::ThousandsSeparatorOn),
				Some(Function::DecimalPointPeriod),
				Some(Function::DecimalPointComma),
			]
			.to_vec(),
			FunctionMenu::Mode => [
				Some(Function::Degrees),
				Some(Function::Radians),
				Some(Function::Gradians),
			]
			.to_vec(),
			FunctionMenu::Base => [
				Some(Function::Decimal),
				Some(Function::Octal),
				Some(Function::Hex),
				Some(Function::Float),
				Some(Function::SignedInteger),
				Some(Function::UnsignedInteger),
			]
			.to_vec(),
			FunctionMenu::SignedInteger => [
				Some(Function::BigInteger),
				Some(Function::Signed8Bit),
				Some(Function::Signed16Bit),
				Some(Function::Signed32Bit),
				Some(Function::Signed64Bit),
				Some(Function::Signed128Bit),
			]
			.to_vec(),
			FunctionMenu::UnsignedInteger => [
				Some(Function::BigInteger),
				Some(Function::Unsigned8Bit),
				Some(Function::Unsigned16Bit),
				Some(Function::Unsigned32Bit),
				Some(Function::Unsigned64Bit),
				Some(Function::Unsigned128Bit),
			]
			.to_vec(),
			FunctionMenu::Logic => [
				Some(Function::And),
				Some(Function::Or),
				Some(Function::Xor),
				Some(Function::Not),
				Some(Function::ShiftLeft),
				Some(Function::ShiftRight),
				Some(Function::RotateLeft),
				Some(Function::RotateRight),
			]
			.to_vec(),
			FunctionMenu::Catalog => {
				[Some(Function::ConstCatalog), Some(Function::TimeCatalog)].to_vec()
			}
			FunctionMenu::ConstCatalog => [Some(Function::SpeedOfLight)].to_vec(),
			FunctionMenu::TimeCatalog => [
				Some(Function::Now),
				Some(Function::Date),
				Some(Function::Time),
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
	menu_strings: RefCell<Vec<String>>,
}

impl FunctionKeyState {
	pub fn new() -> Self {
		FunctionKeyState {
			menu: None,
			functions: Vec::new(),
			page: 0,
			menu_stack: Vec::new(),
			quick_functions: Vec::new(),
			menu_strings: RefCell::new(Vec::new()),
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

	pub fn exit_menu(&mut self, format: &NumberFormat) {
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

	pub fn render<ScreenT: Screen>(&self, screen: &mut ScreenT) {
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
			if let Some(string) = self.menu_strings.borrow().get(i as usize) {
				let mut string = string.clone();

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
