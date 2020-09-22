use crate::error::{Error, Result};
use crate::storage::{DeserializeInput, SerializeOutput, StorageObject, StorageRefSerializer};
use crate::unit::{AngleUnit, UnitConversion};
use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::TryInto;
use intel_dfp::Decimal;
use num_bigint::{BigInt, BigUint, Sign, ToBigInt, ToBigUint};
use num_integer::Integer;

// Maximum integer size before it is converted into a floating point number.
pub const MAX_INTEGER_BITS: u64 = 8192;

// Maximum integer exponent (10^x). This should match the above value in magnitude.
pub const MAX_INTEGER_EXPONENT: isize = 2466;

// Maximum denominator size. This will keep the maximum possible precision of the
// 128-bit float available in fractional form.
pub const MAX_DENOMINATOR_BITS: u64 = 128;

// Maximum numerator size is the maximum integer portion plus the range of the denominator.
pub const MAX_NUMERATOR_BITS: u64 = MAX_INTEGER_BITS + MAX_DENOMINATOR_BITS;

// Number of integer bits to attempt to render in short form (i.e. stack display)
pub const MAX_SHORT_DISPLAY_BITS: u64 = 128;

#[derive(Clone)]
pub enum Number {
	Integer(BigInt),
	Rational(BigInt, BigUint),
	Decimal(Decimal),
}

pub trait ToNumber {
	fn to_number(self) -> Number;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NumberFormatMode {
	Normal,
	Rational,
	Scientific,
	Engineering,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NumberDecimalPointMode {
	Period,
	Comma,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IntegerMode {
	Float,
	BigInteger,
	SizedInteger(usize, bool),
}

#[derive(Clone)]
pub struct NumberFormat {
	pub mode: NumberFormatMode,
	pub integer_mode: IntegerMode,
	pub decimal_point: NumberDecimalPointMode,
	pub thousands: bool,
	pub precision: usize,
	pub trailing_zeros: bool,
	pub integer_radix: u8,
	pub show_alt_hex: bool,
	pub show_alt_float: bool,
	pub limit_size: bool,
}

impl Number {
	pub fn new() -> Self {
		Number::Integer(0.into())
	}

	pub fn bigint_to_decimal(int: &BigInt) -> Decimal {
		let mut result: Decimal = 0.into();
		let mut digit_factor: Decimal = 1.into();

		// Convert big integer into its u32 "digits"
		let (sign, digits) = int.to_u32_digits();

		// Add in the digits of the number from lowest to highest
		for digit in digits {
			let digit_decimal: Decimal = digit.into();
			result += &digit_decimal * &digit_factor;
			digit_factor *= (1u64 << 32).into();
		}

		// Match the sign
		if sign == Sign::Minus {
			result = -result;
		}

		result
	}

	pub fn to_decimal<'a>(&'a self) -> Cow<'a, Decimal> {
		match self {
			Number::Integer(int) => Cow::Owned(Self::bigint_to_decimal(&int)),
			Number::Rational(num, denom) => Cow::Owned(
				Self::bigint_to_decimal(&num)
					/ Self::bigint_to_decimal(&denom.to_bigint().unwrap()),
			),
			Number::Decimal(value) => Cow::Borrowed(value),
		}
	}

	pub fn to_int<'a>(&'a self) -> Result<Cow<'a, BigInt>> {
		match self {
			Number::Integer(int) => Ok(Cow::Borrowed(int)),
			Number::Rational(num, denom) => Ok(Cow::Owned(num / denom.to_bigint().unwrap())),
			Number::Decimal(num) => {
				let num = num.trunc();

				let raw_str = num.to_string();

				// Split string on the 'E' to decode parts of number. For non inf/NaN there
				// will always be an exponent.
				let parts: Vec<&str> = raw_str.split('E').collect();
				if parts.len() == 1 {
					// Not a normal number, cannot be converted to integer
					return Err(Error::InvalidInteger);
				}
				// There is always a sign at the start of the string
				let sign = &raw_str[0..1] == "-";

				// Get digits and parse exponent
				let digit_str = &parts[0][1..];
				let exponent: isize = parts[1].parse().unwrap();

				// Compute the number of digits in the integer portion of the number.
				let integer_part_digits = digit_str.len() as isize + exponent;
				if integer_part_digits <= 0 {
					// Number is less than one, so the integer is zero.
					return Ok(Cow::Owned(0.into()));
				} else if integer_part_digits > MAX_INTEGER_EXPONENT {
					return Err(Error::ValueOutOfRange);
				}

				let mut result = 0.to_bigint().unwrap();
				for ch in digit_str.chars() {
					result *= 10.to_bigint().unwrap();
					result += (ch as u32 as u8 - '0' as u32 as u8).to_bigint().unwrap();
				}

				if integer_part_digits > digit_str.len() as isize {
					result *= 10
						.to_bigint()
						.unwrap()
						.pow(integer_part_digits as u32 - digit_str.len() as u32);
				}

				if sign {
					result = -result;
				}
				Ok(Cow::Owned(result))
			}
		}
	}

	pub fn to_string(&self) -> String {
		NumberFormat::new().format_number(self)
	}

	pub fn is_zero(&self) -> bool {
		match self {
			Number::Integer(value) => value == &0.to_bigint().unwrap(),
			Number::Rational(numerator, _) => numerator == &0.to_bigint().unwrap(),
			Number::Decimal(value) => value == &Decimal::zero(),
		}
	}

	pub fn is_negative(&self) -> bool {
		match self {
			Number::Integer(value) => value.sign() == Sign::Minus,
			Number::Rational(numerator, _) => numerator.sign() == Sign::Minus,
			Number::Decimal(value) => value < &Decimal::zero(),
		}
	}

	pub fn is_rational(&self) -> bool {
		match self {
			Number::Rational(_, _) => true,
			_ => false,
		}
	}

	pub fn is_infinite(&self) -> bool {
		match self {
			Number::Decimal(value) => value.is_infinite(),
			_ => false,
		}
	}

	pub fn is_nan(&self) -> bool {
		match self {
			Number::Decimal(value) => value.is_nan(),
			_ => false,
		}
	}

	pub fn sqrt(&self) -> Number {
		match &self {
			Number::Integer(value) => {
				if value < &0.to_bigint().unwrap() {
					// Imaginary
					return Number::Decimal(self.to_decimal().sqrt());
				}
				let result = value.sqrt();
				if &result * &result == *value {
					// Integer root
					Number::Integer(result)
				} else {
					// Irrational root
					Number::Decimal(self.to_decimal().sqrt())
				}
			}
			Number::Rational(_, _) => Number::Decimal(self.to_decimal().sqrt()),
			Number::Decimal(value) => Number::Decimal(value.sqrt()),
		}
	}

	pub fn pow(&self, power: &Number) -> Number {
		match &self {
			Number::Integer(left) => match power {
				Number::Integer(right) => {
					if right < &0.to_bigint().unwrap() {
						// Fractional power, use float
						return Number::Decimal(self.to_decimal().pow(&power.to_decimal()));
					}
					if let Ok(int_power) = right.try_into() {
						let left_bits = left.bits();
						if left_bits > 0 && ((left_bits - 1) * int_power as u64) > MAX_INTEGER_BITS
						{
							Number::Decimal(self.to_decimal().pow(&power.to_decimal()))
						} else {
							Self::check_int_bounds(Number::Integer(left.pow(int_power)))
						}
					} else {
						Number::Decimal(self.to_decimal().pow(&power.to_decimal()))
					}
				}
				Number::Rational(_, _) => {
					Number::Decimal(self.to_decimal().pow(&power.to_decimal()))
				}
				Number::Decimal(right) => Number::Decimal(self.to_decimal().pow(right)),
			},
			Number::Rational(_, _) => Number::Decimal(self.to_decimal().pow(&power.to_decimal())),
			Number::Decimal(left) => Number::Decimal(left.pow(&power.to_decimal())),
		}
	}

	pub fn sin(&self) -> Number {
		Number::Decimal(self.to_decimal().sin())
	}

	pub fn cos(&self) -> Number {
		Number::Decimal(self.to_decimal().cos())
	}

	pub fn tan(&self) -> Number {
		Number::Decimal(self.to_decimal().tan())
	}

	pub fn asin(&self) -> Number {
		Number::Decimal(self.to_decimal().asin())
	}

	pub fn acos(&self) -> Number {
		Number::Decimal(self.to_decimal().acos())
	}

	pub fn atan(&self) -> Number {
		Number::Decimal(self.to_decimal().atan())
	}

	pub fn sinh(&self) -> Number {
		Number::Decimal(self.to_decimal().sinh())
	}

	pub fn cosh(&self) -> Number {
		Number::Decimal(self.to_decimal().cosh())
	}

	pub fn tanh(&self) -> Number {
		Number::Decimal(self.to_decimal().tanh())
	}

	pub fn asinh(&self) -> Number {
		Number::Decimal(self.to_decimal().asinh())
	}

	pub fn acosh(&self) -> Number {
		Number::Decimal(self.to_decimal().acosh())
	}

	pub fn atanh(&self) -> Number {
		Number::Decimal(self.to_decimal().atanh())
	}

	pub fn angle_to_radians<'a>(&'a self, angle_mode: AngleUnit) -> Cow<'a, Number> {
		match angle_mode {
			AngleUnit::Radians => Cow::Borrowed(self),
			_ => Cow::Owned(angle_mode.to_unit(self, &AngleUnit::Radians)),
		}
	}

	pub fn angle_from_radians<'a>(&'a self, angle_mode: AngleUnit) -> Cow<'a, Number> {
		match angle_mode {
			AngleUnit::Radians => Cow::Borrowed(self),
			_ => Cow::Owned(AngleUnit::Radians.to_unit(self, &angle_mode)),
		}
	}

	pub fn log(&self) -> Number {
		Number::Decimal(self.to_decimal().log10())
	}

	pub fn ln(&self) -> Number {
		Number::Decimal(self.to_decimal().ln())
	}

	pub fn exp10(&self) -> Number {
		Number::Decimal(self.to_decimal().exp10())
	}

	pub fn exp(&self) -> Number {
		Number::Decimal(self.to_decimal().exp())
	}

	fn simplify(self) -> Self {
		match self {
			Number::Rational(num, denom) => {
				let num_abs = if num.sign() == Sign::Minus {
					(-&num).to_biguint().unwrap()
				} else {
					(&num).to_biguint().unwrap()
				};
				let gcd = num_abs.gcd(&denom);
				let num = num / gcd.to_bigint().unwrap();
				let denom = denom / gcd;
				if denom == 1.to_biguint().unwrap() {
					Self::check_int_bounds(Number::Integer(num))
				} else {
					Self::check_int_bounds(Number::Rational(num, denom))
				}
			}
			_ => self,
		}
	}

	pub fn check_int_bounds(value: Self) -> Self {
		match &value {
			Number::Integer(int) => {
				if int.bits() > MAX_INTEGER_BITS {
					Number::Decimal(value.to_decimal().into_owned())
				} else {
					value
				}
			}
			Number::Rational(numer, denom) => {
				if numer.bits() > MAX_NUMERATOR_BITS || denom.bits() > MAX_DENOMINATOR_BITS {
					Number::Decimal(value.to_decimal().into_owned())
				} else {
					value
				}
			}
			_ => value,
		}
	}

	pub fn check_int_bounds_with_bit_count(value: Self, int_bits: u64, denom_bits: u64) -> Self {
		match &value {
			Number::Integer(int) => {
				if int.bits() > int_bits {
					Number::Decimal(value.to_decimal().into_owned())
				} else {
					value
				}
			}
			Number::Rational(numer, denom) => {
				if numer.bits() > int_bits + denom_bits || denom.bits() > denom_bits {
					Number::Decimal(value.to_decimal().into_owned())
				} else {
					value
				}
			}
			_ => value,
		}
	}

	fn num_add(&self, rhs: &Number) -> Number {
		match &self {
			Number::Integer(left) => match rhs {
				Number::Integer(right) => Self::check_int_bounds(Number::Integer(left + right)),
				Number::Rational(right_num, right_denom) => {
					let num = left * right_denom.to_bigint().unwrap() + right_num;
					Number::Rational(num, right_denom.clone()).simplify()
				}
				Number::Decimal(right) => Number::Decimal(&*self.to_decimal() + right),
			},
			Number::Rational(left_num, left_denom) => match rhs {
				Number::Integer(right) => {
					let num = left_num + right * left_denom.to_bigint().unwrap();
					Number::Rational(num, left_denom.clone()).simplify()
				}
				Number::Rational(right_num, right_denom) => {
					let num = left_num * right_denom.to_bigint().unwrap()
						+ right_num * left_denom.to_bigint().unwrap();
					let denom = left_denom * right_denom;
					Number::Rational(num, denom).simplify()
				}
				Number::Decimal(right) => Number::Decimal(&*self.to_decimal() + right),
			},
			Number::Decimal(left) => Number::Decimal(left + &rhs.to_decimal()),
		}
	}

	fn num_sub(&self, rhs: &Number) -> Number {
		match &self {
			Number::Integer(left) => match rhs {
				Number::Integer(right) => Self::check_int_bounds(Number::Integer(left - right)),
				Number::Rational(right_num, right_denom) => {
					let num = left * right_denom.to_bigint().unwrap() - right_num;
					Number::Rational(num, right_denom.clone()).simplify()
				}
				Number::Decimal(right) => Number::Decimal(&*self.to_decimal() - right),
			},
			Number::Rational(left_num, left_denom) => match rhs {
				Number::Integer(right) => {
					let num = left_num - right * left_denom.to_bigint().unwrap();
					Number::Rational(num, left_denom.clone()).simplify()
				}
				Number::Rational(right_num, right_denom) => {
					let num = left_num * right_denom.to_bigint().unwrap()
						- right_num * left_denom.to_bigint().unwrap();
					let denom = left_denom * right_denom;
					Number::Rational(num, denom).simplify()
				}
				Number::Decimal(right) => Number::Decimal(&*self.to_decimal() - right),
			},
			Number::Decimal(left) => Number::Decimal(left - &rhs.to_decimal()),
		}
	}

	fn num_mul(&self, rhs: &Number) -> Number {
		match &self {
			Number::Integer(left) => match rhs {
				Number::Integer(right) => Self::check_int_bounds(Number::Integer(left * right)),
				Number::Rational(right_num, right_denom) => {
					Number::Rational(left * right_num, right_denom.clone()).simplify()
				}
				Number::Decimal(right) => Number::Decimal(&*self.to_decimal() * right),
			},
			Number::Rational(left_num, left_denom) => match rhs {
				Number::Integer(right) => {
					Number::Rational(left_num * right, left_denom.clone()).simplify()
				}
				Number::Rational(right_num, right_denom) => {
					Number::Rational(left_num * right_num, left_denom * right_denom).simplify()
				}
				Number::Decimal(right) => Number::Decimal(&*self.to_decimal() * right),
			},
			Number::Decimal(left) => Number::Decimal(left * &rhs.to_decimal()),
		}
	}

	fn num_div(&self, rhs: &Number) -> Number {
		match &self {
			Number::Integer(left) => match rhs {
				Number::Integer(right) => {
					if right == &0.to_bigint().unwrap() {
						// Divide by zero, use float to get the right inf/NaN
						return Number::Decimal(&*self.to_decimal() / &*rhs.to_decimal());
					}
					if right.sign() == Sign::Minus {
						Number::Rational(-left.to_bigint().unwrap(), (-right).to_biguint().unwrap())
							.simplify()
					} else {
						Number::Rational(left.to_bigint().unwrap(), right.to_biguint().unwrap())
							.simplify()
					}
				}
				Number::Rational(right_num, right_denom) => {
					if right_num.sign() == Sign::Minus {
						Number::Rational(
							left * -right_denom.to_bigint().unwrap(),
							(-right_num).to_biguint().unwrap(),
						)
						.simplify()
					} else {
						Number::Rational(
							left * right_denom.to_bigint().unwrap(),
							right_num.to_biguint().unwrap(),
						)
						.simplify()
					}
				}
				Number::Decimal(right) => Number::Decimal(&*self.to_decimal() / right),
			},
			Number::Rational(left_num, left_denom) => match rhs {
				Number::Integer(right) => {
					if right.sign() == Sign::Minus {
						Number::Rational(-left_num, left_denom * right.to_biguint().unwrap())
							.simplify()
					} else {
						Number::Rational(left_num.clone(), left_denom * right.to_biguint().unwrap())
							.simplify()
					}
				}
				Number::Rational(right_num, right_denom) => {
					if left_num.sign() == Sign::Minus {
						if right_num.sign() == Sign::Minus {
							Number::Rational(
								-left_num * right_denom.to_bigint().unwrap(),
								left_denom * (-right_num).to_biguint().unwrap(),
							)
							.simplify()
						} else {
							Number::Rational(
								left_num * right_denom.to_bigint().unwrap(),
								left_denom * right_num.to_biguint().unwrap(),
							)
							.simplify()
						}
					} else {
						if right_num.sign() == Sign::Minus {
							Number::Rational(
								-left_num * right_denom.to_bigint().unwrap(),
								left_denom * (-right_num).to_biguint().unwrap(),
							)
							.simplify()
						} else {
							Number::Rational(
								left_num * right_denom.to_bigint().unwrap(),
								left_denom * right_num.to_biguint().unwrap(),
							)
							.simplify()
						}
					}
				}
				Number::Decimal(right) => Number::Decimal(&*self.to_decimal() / right),
			},
			Number::Decimal(left) => Number::Decimal(left / &rhs.to_decimal()),
		}
	}
}

impl NumberFormat {
	pub fn new() -> Self {
		NumberFormat {
			mode: NumberFormatMode::Rational,
			integer_mode: IntegerMode::Float,
			decimal_point: NumberDecimalPointMode::Period,
			thousands: true,
			precision: 12,
			trailing_zeros: false,
			integer_radix: 10,
			show_alt_hex: true,
			show_alt_float: true,
			limit_size: true,
		}
	}

	pub fn exponent_format(&self) -> Self {
		NumberFormat {
			mode: NumberFormatMode::Normal,
			integer_mode: IntegerMode::BigInteger,
			decimal_point: self.decimal_point,
			thousands: false,
			precision: 4,
			trailing_zeros: true,
			integer_radix: 10,
			show_alt_hex: false,
			show_alt_float: false,
			limit_size: true,
		}
	}

	pub fn hex_format(&self) -> Self {
		NumberFormat {
			mode: NumberFormatMode::Normal,
			integer_mode: match &self.integer_mode {
				IntegerMode::Float => IntegerMode::BigInteger,
				integer_mode => *integer_mode,
			},
			decimal_point: self.decimal_point,
			thousands: self.thousands,
			precision: self.precision,
			trailing_zeros: self.trailing_zeros,
			integer_radix: 16,
			show_alt_hex: self.show_alt_hex,
			show_alt_float: self.show_alt_float,
			limit_size: self.limit_size,
		}
	}

	pub fn decimal_format(&self) -> Self {
		NumberFormat {
			mode: self.mode,
			integer_mode: self.integer_mode,
			decimal_point: self.decimal_point,
			thousands: self.thousands,
			precision: self.precision,
			trailing_zeros: self.trailing_zeros,
			integer_radix: 10,
			show_alt_hex: self.show_alt_hex,
			show_alt_float: self.show_alt_float,
			limit_size: self.limit_size,
		}
	}

	pub fn with_max_precision(&self, max_precision: usize) -> Self {
		NumberFormat {
			mode: self.mode,
			integer_mode: self.integer_mode,
			decimal_point: self.decimal_point,
			thousands: self.thousands,
			precision: core::cmp::min(self.precision, max_precision),
			trailing_zeros: self.trailing_zeros,
			integer_radix: self.integer_radix,
			show_alt_hex: self.show_alt_hex,
			show_alt_float: self.show_alt_float,
			limit_size: self.limit_size,
		}
	}

	pub fn format_number(&self, num: &Number) -> String {
		match num {
			Number::Integer(int) => match self.mode {
				NumberFormatMode::Normal | NumberFormatMode::Rational => {
					if self.limit_size && int.bits() > MAX_SHORT_DISPLAY_BITS {
						self.format_decimal(&num.to_decimal())
					} else {
						self.format_bigint(int)
					}
				}
				NumberFormatMode::Scientific | NumberFormatMode::Engineering => {
					if self.integer_radix == 10
						|| (self.limit_size && int.bits() > MAX_SHORT_DISPLAY_BITS)
					{
						self.format_decimal(&num.to_decimal())
					} else {
						self.format_bigint(int)
					}
				}
			},
			Number::Rational(_, _) => self.format_decimal(&num.to_decimal()),
			Number::Decimal(value) => self.format_decimal(value),
		}
	}

	pub fn format_bigint(&self, int: &BigInt) -> String {
		assert!(self.integer_radix > 1 && self.integer_radix <= 36);

		// String will be constructed in reverse to simplify implementation
		let mut result = Vec::new();

		// Format the magnitude of the number ignoring sign, the sign will be
		// added later.
		let mut val = int.magnitude().clone();

		// Get big integers for the needed constants
		let radix: BigUint = self.integer_radix.into();

		let mut digits = 0;
		let mut non_decimal = false;
		while val != 0.to_biguint().unwrap() {
			// Check for thousands separator
			if digits % 3 == 0 && digits > 0 && self.integer_radix == 10 && self.thousands {
				match self.decimal_point {
					NumberDecimalPointMode::Period => result.push(','),
					NumberDecimalPointMode::Comma => result.push('.'),
				}
			} else if digits % 4 == 0 && digits > 0 && self.integer_radix == 16 && self.thousands {
				result.push('\'');
			}

			// Get the lowest digit for the current radix and push it
			// onto the result.
			let digit: u8 = (&val % &radix).try_into().unwrap();
			if digit >= 10 {
				result.push(char::from_u32('A' as u32 + digit as u32 - 10).unwrap());
				non_decimal = true;
			} else {
				result.push(char::from_u32('0' as u32 + digit as u32).unwrap());
			}

			// Update value to exclude this digit
			val /= &radix;
			digits += 1;
		}

		// If value was zero, ensure the string isn't blank
		if result.len() == 0 {
			result.push('0');
		}

		// Add prefixes for hex and oct modes
		if self.integer_radix == 16 && (result.len() > 1 || non_decimal) {
			result.push('x');
			result.push('0');
		}
		if self.integer_radix == 8 && result.len() > 1 {
			result.push('0');
		}

		// Add in sign
		if int.sign() == Sign::Minus {
			result.push('-');
		}

		// Create string
		result.reverse();
		result.iter().collect()
	}

	fn format_decimal_post_round(&self, num: &Decimal, mode: NumberFormatMode) -> String {
		let raw_str = num.to_string();

		// Split string on the 'E' to decode parts of number. For non inf/NaN there
		// will always be an exponent.
		let parts: Vec<&str> = raw_str.split('E').collect();
		if parts.len() == 1 {
			// Not a normal number, detect infinity vs. NaN
			if &parts[0][1..] == "Inf" {
				return raw_str[0..1].to_string() + "∞";
			} else {
				return "NaN".to_string();
			}
		}

		// There is always a sign at the start of the string
		let sign = &raw_str[0..1] == "-";

		// Get digits and parse exponent
		let digit_str = &parts[0][1..];
		let mut exponent: isize = parts[1].parse().unwrap();

		let mut display_exponent = match mode {
			NumberFormatMode::Scientific => {
				let new_exponent = 1 - digit_str.len() as isize;
				let display = exponent - new_exponent;
				exponent = new_exponent;
				display
			}
			NumberFormatMode::Engineering => {
				let mut new_exponent = 1 - digit_str.len() as isize;
				let mut display = exponent - new_exponent;
				let offset = if (display < 0) && (display % 3 != 0) {
					display % 3 + 3
				} else {
					display % 3
				};
				new_exponent += offset;
				display -= offset;
				exponent = new_exponent;
				display
			}
			_ => 0,
		};

		// Compute the number of digits in the integer portion of the number. This may
		// be negative if there are leading zeros in the fraction.
		let integer_part_digits = digit_str.len() as isize + exponent;

		// Get fraction digits
		let fraction_digits = if integer_part_digits < 0 {
			digit_str
		} else if integer_part_digits > digit_str.len() as isize {
			&""
		} else {
			&digit_str[integer_part_digits as usize..]
		};

		// Count the number of trailing zeros in the fraction part of the number. This
		// will be used to avoid displaying unnecessary parts of the fraction component
		// (unless trailing zeros are enabled).
		let mut trailing_zeros = 0;
		let digit_bytes = fraction_digits.as_bytes();
		for i in 0..fraction_digits.len() {
			if digit_bytes[(fraction_digits.len() - 1) - i] != '0' as u32 as u8 {
				break;
			}
			trailing_zeros += 1;
		}

		// Get the nonzero fraction digits from the string
		let nonzero_fraction_digits = fraction_digits.len() - trailing_zeros;
		let fraction_digits = &fraction_digits[0..nonzero_fraction_digits];

		let integer_str = if integer_part_digits > 0 {
			// Construct the string containing the integer digits. This will be constructed in
			// reverse to more easily handle the thousands separators.
			let mut integer_digits = Vec::new();
			let mut digits = 0;
			let digit_bytes = digit_str.as_bytes();
			for i in 0..integer_part_digits {
				if digits > 0 && digits % 3 == 0 && self.thousands {
					match self.decimal_point {
						NumberDecimalPointMode::Period => integer_digits.push(',' as u32 as u8),
						NumberDecimalPointMode::Comma => integer_digits.push('.' as u32 as u8),
					}
				}
				if ((integer_part_digits as usize - 1) - i as usize) < digit_bytes.len() {
					integer_digits
						.push(digit_bytes[(integer_part_digits as usize - 1) - i as usize]);
				} else {
					integer_digits.push('0' as u32 as u8);
				}
				digits += 1;
			}

			// Construct the final string
			integer_digits.reverse();
			String::from_utf8(integer_digits).unwrap()
		} else {
			// There is no integer portion, so it is zero
			"0".to_string()
		};

		// Construct fraction part of string
		let fraction_str = if integer_part_digits < 0 && fraction_digits.len() > 0 {
			// There are leading zeros in the fraction, prepend them
			let mut digits = Vec::new();
			digits.resize((-integer_part_digits) as usize, '0' as u32 as u8);
			String::from_utf8(digits).unwrap() + fraction_digits
		} else {
			// No leading zeros, use fraction digits from earlier
			fraction_digits.to_string()
		};

		if integer_str == "0" && fraction_str.len() == 0 {
			// If the value to be displayed is zero, use a zero exponent as well
			display_exponent = 0;
		}

		// Construct final string
		let sign_str = if sign { "-" } else { "" };

		let exponent_str = if display_exponent != 0 {
			"ᴇ".to_string()
				+ &self
					.exponent_format()
					.format_bigint(&display_exponent.into())
		} else if self.mode == NumberFormatMode::Scientific
			|| self.mode == NumberFormatMode::Engineering
		{
			"ᴇ0".to_string()
		} else {
			"".to_string()
		};

		if fraction_digits.len() > 0 {
			let decimal = match self.decimal_point {
				NumberDecimalPointMode::Period => ".",
				NumberDecimalPointMode::Comma => ",",
			};
			sign_str.to_string() + &integer_str + decimal + &fraction_str + &exponent_str
		} else {
			sign_str.to_string() + &integer_str + &exponent_str
		}
	}

	pub fn format_decimal(&self, num: &Decimal) -> String {
		let raw_str = num.to_string();

		// Split string on the 'E' to decode parts of number. For non inf/NaN there
		// will always be an exponent.
		let parts: Vec<&str> = raw_str.split('E').collect();
		if parts.len() == 1 {
			// Not a normal number, detect infinity vs. NaN
			if &parts[0][1..] == "Inf" {
				return raw_str[0..1].to_string() + "∞";
			} else {
				return "NaN".to_string();
			}
		}

		// Get digits and parse exponent
		let digit_str = &parts[0][1..];
		let exponent: isize = parts[1].parse().unwrap();

		// Compute the number of digits in the integer portion of the number. This may
		// be negative if there are leading zeros in the fraction.
		let integer_part_digits = digit_str.len() as isize + exponent;

		// Check to see if the number is too large or too small to display as a normal
		// decimal number (or if the mode is not decimal), and determine the display
		// mode according to this and the formatter settings.
		let mut mode = if self.mode == NumberFormatMode::Scientific
			|| self.mode == NumberFormatMode::Engineering
		{
			self.mode
		} else if integer_part_digits > self.precision as isize
			|| integer_part_digits < -4
			|| integer_part_digits < -(self.precision as isize / 2)
		{
			NumberFormatMode::Scientific
		} else {
			NumberFormatMode::Normal
		};

		// Check for rounding
		if digit_str.len() > self.precision {
			// More digits than desired precision, round at desired precision.
			let mut round_exponent =
				(exponent + digit_str.len() as isize) - self.precision as isize;
			if round_exponent > 0 && mode == NumberFormatMode::Normal {
				// If rounding was in the middle of the integer portion, always display using
				// scientific notation, as we must not display digits after the rounding point.
				mode = NumberFormatMode::Scientific;
			}

			// If there are leading zeros to display, account for this in the rounding
			if mode == NumberFormatMode::Normal && integer_part_digits < 0 {
				round_exponent -= integer_part_digits;
			}

			// Perform rounding at the desired digit
			let round_exponent_dec: Decimal = (round_exponent as i32).into();
			let factor = round_exponent_dec.exp10();
			let one: Decimal = 1.into();
			let two: Decimal = 2.into();
			let adjust = one / two;
			let mut rounded = ((&num.abs() / &factor) + adjust).trunc() * factor;

			if num.is_sign_negative() {
				rounded = -rounded;
			}

			self.format_decimal_post_round(&rounded, mode)
		} else {
			// Number of digits is under the desired precision, convert to string directly
			self.format_decimal_post_round(num, mode)
		}
	}
}

impl From<u8> for Number {
	fn from(val: u8) -> Self {
		Number::Integer(val.into())
	}
}

impl From<i8> for Number {
	fn from(val: i8) -> Self {
		Number::Integer(val.into())
	}
}

impl From<u16> for Number {
	fn from(val: u16) -> Self {
		Number::Integer(val.into())
	}
}

impl From<i16> for Number {
	fn from(val: i16) -> Self {
		Number::Integer(val.into())
	}
}

impl From<u32> for Number {
	fn from(val: u32) -> Self {
		Number::Integer(val.into())
	}
}

impl From<i32> for Number {
	fn from(val: i32) -> Self {
		Number::Integer(val.into())
	}
}

impl From<u64> for Number {
	fn from(val: u64) -> Self {
		Number::Integer(val.into())
	}
}

impl From<i64> for Number {
	fn from(val: i64) -> Self {
		Number::Integer(val.into())
	}
}

impl From<u128> for Number {
	fn from(val: u128) -> Self {
		Number::Integer(val.into())
	}
}

impl From<i128> for Number {
	fn from(val: i128) -> Self {
		Number::Integer(val.into())
	}
}

impl From<usize> for Number {
	fn from(val: usize) -> Self {
		Number::Integer(val.into())
	}
}

impl From<isize> for Number {
	fn from(val: isize) -> Self {
		Number::Integer(val.into())
	}
}

impl From<f32> for Number {
	fn from(val: f32) -> Self {
		Number::Decimal(val.into())
	}
}

impl From<f64> for Number {
	fn from(val: f64) -> Self {
		Number::Decimal(val.into())
	}
}

impl From<Decimal> for Number {
	fn from(val: Decimal) -> Self {
		Number::Decimal(val)
	}
}

impl From<BigInt> for Number {
	fn from(val: BigInt) -> Self {
		Number::Integer(val)
	}
}

impl From<BigUint> for Number {
	fn from(val: BigUint) -> Self {
		Number::Integer(val.to_bigint().unwrap())
	}
}

impl ToNumber for u8 {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for i8 {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for u16 {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for i16 {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for u32 {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for i32 {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for u64 {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for i64 {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for u128 {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for i128 {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for usize {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for isize {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for f32 {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for f64 {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for Decimal {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for BigInt {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl ToNumber for BigUint {
	fn to_number(self) -> Number {
		self.into()
	}
}

impl core::ops::Add for Number {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		self.num_add(&rhs)
	}
}

impl core::ops::Add for &Number {
	type Output = Number;

	fn add(self, rhs: Self) -> Self::Output {
		self.num_add(rhs)
	}
}

impl core::ops::AddAssign for Number {
	fn add_assign(&mut self, rhs: Self) {
		*self = self.num_add(&rhs);
	}
}

impl core::ops::Sub for Number {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		self.num_sub(&rhs)
	}
}

impl core::ops::Sub for &Number {
	type Output = Number;

	fn sub(self, rhs: Self) -> Self::Output {
		self.num_sub(rhs)
	}
}

impl core::ops::SubAssign for Number {
	fn sub_assign(&mut self, rhs: Self) {
		*self = self.num_sub(&rhs);
	}
}

impl core::ops::Mul for Number {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		self.num_mul(&rhs)
	}
}

impl core::ops::Mul for &Number {
	type Output = Number;

	fn mul(self, rhs: Self) -> Self::Output {
		self.num_mul(rhs)
	}
}

impl core::ops::MulAssign for Number {
	fn mul_assign(&mut self, rhs: Self) {
		*self = self.num_mul(&rhs);
	}
}

impl core::ops::Div for Number {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		self.num_div(&rhs)
	}
}

impl core::ops::Div for &Number {
	type Output = Number;

	fn div(self, rhs: Self) -> Self::Output {
		self.num_div(rhs)
	}
}

impl core::ops::DivAssign for Number {
	fn div_assign(&mut self, rhs: Self) {
		*self = self.num_div(&rhs);
	}
}

impl core::ops::Neg for Number {
	type Output = Self;

	fn neg(self) -> Self::Output {
		0.to_number().num_sub(&self)
	}
}

impl core::ops::Neg for &Number {
	type Output = Number;

	fn neg(self) -> Self::Output {
		0.to_number().num_sub(self)
	}
}

const NUM_SERIALIZE_TYPE_INTEGER: u8 = 0;
const NUM_SERIALIZE_TYPE_RATIONAL: u8 = 1;
const NUM_SERIALIZE_TYPE_DECIMAL: u8 = 2;
const NUM_SERIALIZE_SIGN_NONE: u8 = 0;
const NUM_SERIALIZE_SIGN_POSITIVE: u8 = 1;
const NUM_SERIALIZE_SIGN_NEGATIVE: u8 = 2;

impl StorageObject for Number {
	fn serialize<Ref: StorageRefSerializer, Out: SerializeOutput>(
		&self,
		output: &mut Out,
		_: &Ref,
	) -> Result<()> {
		match self {
			Number::Integer(int) => {
				output.write_u8(NUM_SERIALIZE_TYPE_INTEGER)?; // Type marker

				let (sign, digits) = int.to_u32_digits();
				// Output sign
				output.write_u8(match sign {
					Sign::NoSign => NUM_SERIALIZE_SIGN_NONE,
					Sign::Plus => NUM_SERIALIZE_SIGN_POSITIVE,
					Sign::Minus => NUM_SERIALIZE_SIGN_NEGATIVE,
				})?;

				// Output size
				output.write_u32(digits.len() as u32)?;

				// Output digits
				for digit in digits {
					output.write_u32(digit)?;
				}
			}
			Number::Rational(num, denom) => {
				output.write_u8(NUM_SERIALIZE_TYPE_RATIONAL)?; // Type marker

				// Output sign
				let (sign, digits) = num.to_u32_digits();
				output.write_u8(match sign {
					Sign::NoSign => NUM_SERIALIZE_SIGN_NONE,
					Sign::Plus => NUM_SERIALIZE_SIGN_POSITIVE,
					Sign::Minus => NUM_SERIALIZE_SIGN_NEGATIVE,
				})?;

				// Output numerator size
				output.write_u32(digits.len() as u32)?;

				// Output numerator digits
				for digit in digits {
					output.write_u32(digit)?;
				}

				// Output denominator size
				let digits = denom.to_u32_digits();
				output.write_u32(digits.len() as u32)?;

				// Output denominator digits
				for digit in digits {
					output.write_u32(digit)?;
				}
			}
			Number::Decimal(value) => {
				output.write_u8(NUM_SERIALIZE_TYPE_DECIMAL)?; // Type marker

				// Decimal numbers are two u64 parts, encoding is defined by the floating
				// point library (treat as a black box).
				let parts = value.to_raw();
				output.write_u64(parts[0])?;
				output.write_u64(parts[1])?;
			}
		}
		Ok(())
	}

	unsafe fn deserialize<T: StorageRefSerializer>(
		input: &mut DeserializeInput,
		_: &T,
	) -> Result<Self> {
		match input.read_u8()? {
			NUM_SERIALIZE_TYPE_INTEGER => {
				// Decode sign
				let sign = match input.read_u8()? {
					NUM_SERIALIZE_SIGN_NONE => Sign::NoSign,
					NUM_SERIALIZE_SIGN_POSITIVE => Sign::Plus,
					NUM_SERIALIZE_SIGN_NEGATIVE => Sign::Minus,
					_ => return Err(Error::CorruptData),
				};

				// Decode size
				let size = input.read_u32()? as usize;

				// Decode digits
				let mut digits = Vec::new();
				digits.reserve(size);
				for _ in 0..size {
					digits.push(input.read_u32()?);
				}

				// Create integer from parts
				Ok(Number::Integer(BigInt::from_slice(sign, &digits)))
			}
			NUM_SERIALIZE_TYPE_RATIONAL => {
				// Decode sign
				let sign = match input.read_u8()? {
					NUM_SERIALIZE_SIGN_NONE => Sign::NoSign,
					NUM_SERIALIZE_SIGN_POSITIVE => Sign::Plus,
					NUM_SERIALIZE_SIGN_NEGATIVE => Sign::Minus,
					_ => return Err(Error::CorruptData),
				};

				// Decode numerator size
				let size = input.read_u32()? as usize;

				// Decode numerator digits
				let mut digits = Vec::new();
				digits.reserve(size);
				for _ in 0..size {
					digits.push(input.read_u32()?);
				}

				// Build numerator from parts
				let numerator = BigInt::from_slice(sign, &digits);

				// Decode denominator size
				let size = input.read_u32()? as usize;

				// Decode denominator digits
				digits.clear();
				digits.reserve(size);
				for _ in 0..size {
					digits.push(input.read_u32()?);
				}

				// Build denominator from parts
				let denominator = BigUint::from_slice(&digits);

				// Return rational from numerator and denominator
				Ok(Number::Rational(numerator, denominator))
			}
			NUM_SERIALIZE_TYPE_DECIMAL => {
				// Decode parts of decimal and pass to floating point library
				let first = input.read_u64()?;
				let second = input.read_u64()?;
				Ok(Number::Decimal(Decimal::from_raw([first, second])))
			}
			_ => Err(Error::CorruptData),
		}
	}
}
