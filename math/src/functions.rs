use crate::constant::Constant;
use crate::context::Context;
use crate::error::Result;
use crate::format::{DecimalPointMode, FormatMode, IntegerMode};
use crate::unit::AngleUnit;
use crate::unit::Unit;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

#[derive(PartialEq, Eq, Clone)]
pub enum StackFunction {
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
	BaseToggle,
	Constant(Constant),
	Now,
	Date,
	Time,
	Degrees,
	Radians,
	Gradians,
	ClearUnits,
	AddUnit(Unit),
	AddUnitSquared(Unit),
	AddUnitCubed(Unit),
	AddInvUnit(Unit),
	AddInvUnitSquared(Unit),
	AddInvUnitCubed(Unit),
	ConvertToUnit(Unit),
	Log,
	Exp10,
	Ln,
	Exp,
	Sin,
	Cos,
	Tan,
	Asin,
	Acos,
	Atan,
	Sinh,
	Cosh,
	Tanh,
	Asinh,
	Acosh,
	Atanh,
	Sum,
	Mean,
	DotProduct,
	CrossProduct,
	Magnitude,
	Normalize,
	ToMatrix,
	RowsToMatrix,
	ColsToMatrix,
	IdentityMatrix,
	Transpose,
}

impl StackFunction {
	pub fn to_string(&self, context: &Context) -> String {
		match self {
			StackFunction::NormalFormat => {
				if context.format().mode == FormatMode::Normal {
					"▪Norm".to_string()
				} else {
					"Norm".to_string()
				}
			}
			StackFunction::RationalFormat => {
				if context.format().mode == FormatMode::Rational {
					"▪Frac".to_string()
				} else {
					"Frac".to_string()
				}
			}
			StackFunction::ScientificFormat => {
				if context.format().mode == FormatMode::Scientific {
					"▪Sci".to_string()
				} else {
					"Sci".to_string()
				}
			}
			StackFunction::EngineeringFormat => {
				if context.format().mode == FormatMode::Engineering {
					"▪Eng".to_string()
				} else {
					"Eng".to_string()
				}
			}
			StackFunction::AlternateHex => {
				if context.format().show_alt_hex {
					"▪↓Hex".to_string()
				} else {
					"↓Hex".to_string()
				}
			}
			StackFunction::AlternateFloat => {
				if context.format().show_alt_float {
					"▪↓Flt".to_string()
				} else {
					"↓Flt".to_string()
				}
			}
			StackFunction::ThousandsSeparatorOff => {
				if context.format().thousands {
					"1000".to_string()
				} else {
					"▪1000".to_string()
				}
			}
			StackFunction::ThousandsSeparatorOn => {
				if context.format().thousands {
					"▪1,000".to_string()
				} else {
					"1,000".to_string()
				}
			}
			StackFunction::DecimalPointPeriod => {
				if context.format().decimal_point == DecimalPointMode::Period {
					"▪0.5".to_string()
				} else {
					"0.5".to_string()
				}
			}
			StackFunction::DecimalPointComma => {
				if context.format().decimal_point == DecimalPointMode::Comma {
					"▪0,5".to_string()
				} else {
					"0,5".to_string()
				}
			}
			StackFunction::Float => {
				if context.format().integer_mode == IntegerMode::Float {
					"▪float".to_string()
				} else {
					"float".to_string()
				}
			}
			StackFunction::BigInteger => {
				if context.format().integer_mode == IntegerMode::BigInteger {
					"▪int∞".to_string()
				} else {
					"int∞".to_string()
				}
			}
			StackFunction::Signed8Bit => {
				if context.format().integer_mode == IntegerMode::SizedInteger(8, true) {
					"▪i8".to_string()
				} else {
					"i8".to_string()
				}
			}
			StackFunction::Signed16Bit => {
				if context.format().integer_mode == IntegerMode::SizedInteger(16, true) {
					"▪i16".to_string()
				} else {
					"i16".to_string()
				}
			}
			StackFunction::Signed32Bit => {
				if context.format().integer_mode == IntegerMode::SizedInteger(32, true) {
					"▪i32".to_string()
				} else {
					"i32".to_string()
				}
			}
			StackFunction::Signed64Bit => {
				if context.format().integer_mode == IntegerMode::SizedInteger(64, true) {
					"▪i64".to_string()
				} else {
					"i64".to_string()
				}
			}
			StackFunction::Signed128Bit => {
				if context.format().integer_mode == IntegerMode::SizedInteger(128, true) {
					"▪i128".to_string()
				} else {
					"i128".to_string()
				}
			}
			StackFunction::Unsigned8Bit => {
				if context.format().integer_mode == IntegerMode::SizedInteger(8, false) {
					"▪u8".to_string()
				} else {
					"u8".to_string()
				}
			}
			StackFunction::Unsigned16Bit => {
				if context.format().integer_mode == IntegerMode::SizedInteger(16, false) {
					"▪u16".to_string()
				} else {
					"u16".to_string()
				}
			}
			StackFunction::Unsigned32Bit => {
				if context.format().integer_mode == IntegerMode::SizedInteger(32, false) {
					"▪u32".to_string()
				} else {
					"u32".to_string()
				}
			}
			StackFunction::Unsigned64Bit => {
				if context.format().integer_mode == IntegerMode::SizedInteger(64, false) {
					"▪u64".to_string()
				} else {
					"u64".to_string()
				}
			}
			StackFunction::Unsigned128Bit => {
				if context.format().integer_mode == IntegerMode::SizedInteger(128, false) {
					"▪u128".to_string()
				} else {
					"u128".to_string()
				}
			}
			StackFunction::And => "and".to_string(),
			StackFunction::Or => "or".to_string(),
			StackFunction::Xor => "xor".to_string(),
			StackFunction::Not => "not".to_string(),
			StackFunction::ShiftLeft => "<<".to_string(),
			StackFunction::ShiftRight => ">>".to_string(),
			StackFunction::RotateLeft => "rol".to_string(),
			StackFunction::RotateRight => "ror".to_string(),
			StackFunction::Hex => {
				if context.format().integer_radix == 16 {
					"▪Hex".to_string()
				} else {
					"Hex".to_string()
				}
			}
			StackFunction::Octal => {
				if context.format().integer_radix == 8 {
					"▪Oct".to_string()
				} else {
					"Oct".to_string()
				}
			}
			StackFunction::Decimal => {
				if context.format().integer_radix == 10 {
					"▪Dec".to_string()
				} else {
					"Dec".to_string()
				}
			}
			StackFunction::BaseToggle => "Hex≷Dec".to_string(),
			StackFunction::Constant(constant) => constant.to_str().to_string(),
			StackFunction::Now => "Now".to_string(),
			StackFunction::Date => "Date".to_string(),
			StackFunction::Time => "Time".to_string(),
			StackFunction::Degrees => {
				if context.angle_mode() == &AngleUnit::Degrees {
					"▪Deg".to_string()
				} else {
					"Deg".to_string()
				}
			}
			StackFunction::Radians => {
				if context.angle_mode() == &AngleUnit::Radians {
					"▪Rad".to_string()
				} else {
					"Rad".to_string()
				}
			}
			StackFunction::Gradians => {
				if context.angle_mode() == &AngleUnit::Gradians {
					"▪Grad".to_string()
				} else {
					"Grad".to_string()
				}
			}
			StackFunction::ClearUnits => "←Unit".to_string(),
			StackFunction::AddUnit(unit) => unit.to_str().to_string(),
			StackFunction::AddUnitSquared(unit) => unit.to_str().to_string() + "²",
			StackFunction::AddUnitCubed(unit) => unit.to_str().to_string() + "³",
			StackFunction::AddInvUnit(unit) => "/".to_string() + &unit.to_str(),
			StackFunction::AddInvUnitSquared(unit) => "/".to_string() + &unit.to_str() + "²",
			StackFunction::AddInvUnitCubed(unit) => "/".to_string() + &unit.to_str() + "³",
			StackFunction::ConvertToUnit(unit) => "▸".to_string() + &unit.to_str(),
			StackFunction::Log => "log".to_string(),
			StackFunction::Exp10 => "10ˣ".to_string(),
			StackFunction::Ln => "ln".to_string(),
			StackFunction::Exp => "eˣ".to_string(),
			StackFunction::Sin => "sin".to_string(),
			StackFunction::Cos => "cos".to_string(),
			StackFunction::Tan => "tan".to_string(),
			StackFunction::Asin => "asin".to_string(),
			StackFunction::Acos => "acos".to_string(),
			StackFunction::Atan => "atan".to_string(),
			StackFunction::Sinh => "sinh".to_string(),
			StackFunction::Cosh => "cosh".to_string(),
			StackFunction::Tanh => "tanh".to_string(),
			StackFunction::Asinh => "asinh".to_string(),
			StackFunction::Acosh => "acosh".to_string(),
			StackFunction::Atanh => "atanh".to_string(),
			StackFunction::Sum => "sum".to_string(),
			StackFunction::Mean => "mean".to_string(),
			StackFunction::DotProduct => "dot".to_string(),
			StackFunction::CrossProduct => "cross".to_string(),
			StackFunction::Magnitude => "mag".to_string(),
			StackFunction::Normalize => "norm".to_string(),
			StackFunction::ToMatrix => "▸Mat".to_string(),
			StackFunction::RowsToMatrix => "R▸Mat".to_string(),
			StackFunction::ColsToMatrix => "C▸Mat".to_string(),
			StackFunction::IdentityMatrix => "ident".to_string(),
			StackFunction::Transpose => "transp".to_string(),
		}
	}

	pub fn execute(&self, context: &mut Context) -> Result<()> {
		match self {
			StackFunction::NormalFormat => {
				context.set_format_mode(FormatMode::Normal);
				Ok(())
			}
			StackFunction::RationalFormat => {
				context.set_format_mode(FormatMode::Rational);
				Ok(())
			}
			StackFunction::ScientificFormat => {
				context.set_format_mode(FormatMode::Scientific);
				Ok(())
			}
			StackFunction::EngineeringFormat => {
				context.set_format_mode(FormatMode::Engineering);
				Ok(())
			}
			StackFunction::AlternateHex => {
				context.toggle_alt_hex();
				Ok(())
			}
			StackFunction::AlternateFloat => {
				context.toggle_alt_float();
				Ok(())
			}
			StackFunction::ThousandsSeparatorOff => {
				context.set_thousands_separator(false);
				Ok(())
			}
			StackFunction::ThousandsSeparatorOn => {
				context.set_thousands_separator(true);
				Ok(())
			}
			StackFunction::DecimalPointPeriod => {
				context.set_decimal_point_mode(DecimalPointMode::Period);
				Ok(())
			}
			StackFunction::DecimalPointComma => {
				context.set_decimal_point_mode(DecimalPointMode::Comma);
				Ok(())
			}
			StackFunction::Float => context.set_float_mode(),
			StackFunction::BigInteger => {
				context.set_integer_mode(IntegerMode::BigInteger);
				Ok(())
			}
			StackFunction::Signed8Bit => {
				context.set_integer_mode(IntegerMode::SizedInteger(8, true));
				Ok(())
			}
			StackFunction::Signed16Bit => {
				context.set_integer_mode(IntegerMode::SizedInteger(16, true));
				Ok(())
			}
			StackFunction::Signed32Bit => {
				context.set_integer_mode(IntegerMode::SizedInteger(32, true));
				Ok(())
			}
			StackFunction::Signed64Bit => {
				context.set_integer_mode(IntegerMode::SizedInteger(64, true));
				Ok(())
			}
			StackFunction::Signed128Bit => {
				context.set_integer_mode(IntegerMode::SizedInteger(128, true));
				Ok(())
			}
			StackFunction::Unsigned8Bit => {
				context.set_integer_mode(IntegerMode::SizedInteger(8, false));
				Ok(())
			}
			StackFunction::Unsigned16Bit => {
				context.set_integer_mode(IntegerMode::SizedInteger(16, false));
				Ok(())
			}
			StackFunction::Unsigned32Bit => {
				context.set_integer_mode(IntegerMode::SizedInteger(32, false));
				Ok(())
			}
			StackFunction::Unsigned64Bit => {
				context.set_integer_mode(IntegerMode::SizedInteger(64, false));
				Ok(())
			}
			StackFunction::Unsigned128Bit => {
				context.set_integer_mode(IntegerMode::SizedInteger(128, false));
				Ok(())
			}
			StackFunction::And => context.and(),
			StackFunction::Or => context.or(),
			StackFunction::Xor => context.xor(),
			StackFunction::Not => context.not(),
			StackFunction::ShiftLeft => context.shl(),
			StackFunction::ShiftRight => context.shr(),
			StackFunction::RotateLeft => context.rotate_left(),
			StackFunction::RotateRight => context.rotate_right(),
			StackFunction::Hex => {
				context.set_integer_radix(16);
				Ok(())
			}
			StackFunction::Octal => {
				context.set_integer_radix(8);
				Ok(())
			}
			StackFunction::Decimal => {
				context.set_integer_radix(10);
				Ok(())
			}
			StackFunction::BaseToggle => {
				context.toggle_integer_radix();
				Ok(())
			}
			StackFunction::Constant(constant) => context.push_constant(*constant),
			StackFunction::Now => context.now(),
			StackFunction::Date => context.date(),
			StackFunction::Time => context.time(),
			StackFunction::Degrees => {
				context.set_angle_mode(AngleUnit::Degrees);
				Ok(())
			}
			StackFunction::Radians => {
				context.set_angle_mode(AngleUnit::Radians);
				Ok(())
			}
			StackFunction::Gradians => {
				context.set_angle_mode(AngleUnit::Gradians);
				Ok(())
			}
			StackFunction::ClearUnits => context.clear_units(),
			StackFunction::AddUnit(unit) => context.add_unit(*unit),
			StackFunction::AddUnitSquared(unit) => context.add_unit_squared(*unit),
			StackFunction::AddUnitCubed(unit) => context.add_unit_cubed(*unit),
			StackFunction::AddInvUnit(unit) => context.add_inv_unit(*unit),
			StackFunction::AddInvUnitSquared(unit) => context.add_inv_unit_squared(*unit),
			StackFunction::AddInvUnitCubed(unit) => context.add_inv_unit_cubed(*unit),
			StackFunction::ConvertToUnit(unit) => context.convert_to_unit(*unit),
			StackFunction::Log => context.log(),
			StackFunction::Exp10 => context.exp10(),
			StackFunction::Ln => context.ln(),
			StackFunction::Exp => context.exp(),
			StackFunction::Sin => context.sin(),
			StackFunction::Cos => context.cos(),
			StackFunction::Tan => context.tan(),
			StackFunction::Asin => context.asin(),
			StackFunction::Acos => context.acos(),
			StackFunction::Atan => context.atan(),
			StackFunction::Sinh => context.sinh(),
			StackFunction::Cosh => context.cosh(),
			StackFunction::Tanh => context.tanh(),
			StackFunction::Asinh => context.asinh(),
			StackFunction::Acosh => context.acosh(),
			StackFunction::Atanh => context.atanh(),
			StackFunction::Sum => context.sum(),
			StackFunction::Mean => context.mean(),
			StackFunction::DotProduct => context.dot_product(),
			StackFunction::CrossProduct => context.cross_product(),
			StackFunction::Magnitude => context.magnitude(),
			StackFunction::Normalize => context.normalize(),
			StackFunction::ToMatrix => context.to_matrix(),
			StackFunction::RowsToMatrix => context.rows_to_matrix(),
			StackFunction::ColsToMatrix => context.cols_to_matrix(),
			StackFunction::IdentityMatrix => context.identity_matrix(),
			StackFunction::Transpose => context.transpose(),
		}
	}
}
