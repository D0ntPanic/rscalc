use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::TryInto;
use intel_dfp::Decimal;
use num_bigint::{BigInt, BigUint, Sign, ToBigInt, ToBigUint};

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

	pub fn to_decimal(&self) -> Decimal {
		match self {
			Number::Integer(int) => Self::bigint_to_decimal(&int),
			Number::Rational(num, denom) => {
				Self::bigint_to_decimal(&num) / Self::bigint_to_decimal(&denom.to_bigint().unwrap())
			}
			Number::Decimal(value) => value.clone(),
		}
	}

	pub fn to_int(&self) -> Option<BigInt> {
		match self {
			Number::Integer(int) => Some(int.clone()),
			Number::Rational(num, denom) => Some(num / denom.to_bigint().unwrap()),
			Number::Decimal(num) => {
				let num = num.trunc();

				let raw_str = num.to_str();

				// Split string on the 'E' to decode parts of number. For non inf/NaN there
				// will always be an exponent.
				let parts: Vec<&str> = raw_str.split('E').collect();
				if parts.len() == 1 {
					// Not a normal number, cannot be converted to integer
					return None;
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
					return Some(0.into());
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
				Some(result)
			}
		}
	}

	pub fn to_str(&self) -> String {
		NumberFormat::new().format_number(self)
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
					if let Ok(power) = right.try_into() {
						Number::Integer(left.pow(power))
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

	fn gcd(x: &BigUint, y: &BigUint) -> BigUint {
		let mut x = x.clone();
		let mut y = y.clone();
		while y != 0.to_biguint().unwrap() {
			let t = y.clone();
			y = x % y;
			x = t;
		}
		x
	}

	fn simplify(&self) -> Self {
		match self {
			Number::Rational(num, denom) => {
				let num_abs = if num.sign() == Sign::Minus {
					(-num).to_biguint().unwrap()
				} else {
					num.to_biguint().unwrap()
				};
				let gcd = Self::gcd(&num_abs, &denom);
				let num = num / gcd.to_bigint().unwrap();
				let denom = denom / gcd;
				if denom == 1.to_biguint().unwrap() {
					Number::Integer(num)
				} else {
					Number::Rational(num, denom)
				}
			}
			_ => self.clone(),
		}
	}

	fn num_add(&self, rhs: &Number) -> Number {
		match &self {
			Number::Integer(left) => match rhs {
				Number::Integer(right) => Number::Integer(left + right),
				Number::Rational(right_num, right_denom) => {
					let num = left * right_denom.to_bigint().unwrap() + right_num;
					Number::Rational(num, right_denom.clone()).simplify()
				}
				Number::Decimal(right) => Number::Decimal(&self.to_decimal() + right),
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
				Number::Decimal(right) => Number::Decimal(&self.to_decimal() + right),
			},
			Number::Decimal(left) => Number::Decimal(left + &rhs.to_decimal()),
		}
	}

	fn num_sub(&self, rhs: &Number) -> Number {
		match &self {
			Number::Integer(left) => match rhs {
				Number::Integer(right) => Number::Integer(left - right),
				Number::Rational(right_num, right_denom) => {
					let num = left * right_denom.to_bigint().unwrap() - right_num;
					Number::Rational(num, right_denom.clone()).simplify()
				}
				Number::Decimal(right) => Number::Decimal(&self.to_decimal() - right),
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
				Number::Decimal(right) => Number::Decimal(&self.to_decimal() - right),
			},
			Number::Decimal(left) => Number::Decimal(left - &rhs.to_decimal()),
		}
	}

	fn num_mul(&self, rhs: &Number) -> Number {
		match &self {
			Number::Integer(left) => match rhs {
				Number::Integer(right) => Number::Integer(left * right),
				Number::Rational(right_num, right_denom) => {
					Number::Rational(left * right_num, right_denom.clone()).simplify()
				}
				Number::Decimal(right) => Number::Decimal(&self.to_decimal() * right),
			},
			Number::Rational(left_num, left_denom) => match rhs {
				Number::Integer(right) => {
					Number::Rational(left_num * right, left_denom.clone()).simplify()
				}
				Number::Rational(right_num, right_denom) => {
					Number::Rational(left_num * right_num, left_denom * right_denom).simplify()
				}
				Number::Decimal(right) => Number::Decimal(&self.to_decimal() * right),
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
						return Number::Decimal(self.to_decimal() / rhs.to_decimal());
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
				Number::Decimal(right) => Number::Decimal(&self.to_decimal() / right),
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
				Number::Decimal(right) => Number::Decimal(&self.to_decimal() / right),
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
		}
	}

	pub fn decimal_format(&self) -> Self {
		NumberFormat {
			mode: NumberFormatMode::Normal,
			integer_mode: self.integer_mode,
			decimal_point: self.decimal_point,
			thousands: self.thousands,
			precision: self.precision,
			trailing_zeros: self.trailing_zeros,
			integer_radix: 10,
			show_alt_hex: self.show_alt_hex,
			show_alt_float: self.show_alt_float,
		}
	}

	pub fn format_number(&self, num: &Number) -> String {
		match num {
			Number::Integer(int) => match self.mode {
				NumberFormatMode::Normal | NumberFormatMode::Rational => self.format_bigint(int),
				NumberFormatMode::Scientific | NumberFormatMode::Engineering => {
					if self.integer_radix == 10 {
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
		let raw_str = num.to_str();

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
				let offset = display % 3;
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
		let raw_str = num.to_str();

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
			let mut rounded = ((&num.abs() / &factor) + adjust.clone()).trunc() * factor.clone();

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
