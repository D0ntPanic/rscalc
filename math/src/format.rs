use crate::number::Number;
use intel_dfp::Decimal;
use num_bigint::{BigInt, BigUint, Sign, ToBigUint};

#[cfg(feature = "std")]
use std::convert::TryInto;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use core::convert::TryInto;

// Number of integer bits to attempt to render in short form (i.e. stack display)
pub const MAX_SHORT_DISPLAY_BITS: u64 = 128;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FormatMode {
	Normal,
	Rational,
	Scientific,
	Engineering,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DecimalPointMode {
	Period,
	Comma,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IntegerMode {
	Float,
	BigInteger,
	SizedInteger(usize, bool),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AlternateFormatMode {
	Smart,
	Bottom,
	Left,
}

impl AlternateFormatMode {
	pub fn left_enabled(&self) -> bool {
		self != &AlternateFormatMode::Bottom
	}

	pub fn bottom_enabled(&self) -> bool {
		self != &AlternateFormatMode::Left
	}
}

#[derive(Clone)]
pub struct Format {
	pub mode: FormatMode,
	pub integer_mode: IntegerMode,
	pub decimal_point: DecimalPointMode,
	pub thousands: bool,
	pub precision: usize,
	pub trailing_zeros: bool,
	pub integer_radix: u8,
	pub show_alt_hex: bool,
	pub show_alt_float: bool,
	pub alt_mode: AlternateFormatMode,
	pub limit_size: bool,
	pub time_24_hour: bool,
	pub stack_xyz: bool,
}

pub enum FormatResult {
	Integer(String),
	Float(String),
	Complex(String),
	Object(String),
}

impl Format {
	pub fn new() -> Self {
		Format {
			mode: FormatMode::Rational,
			integer_mode: IntegerMode::Float,
			decimal_point: DecimalPointMode::Period,
			thousands: true,
			precision: 12,
			trailing_zeros: false,
			integer_radix: 10,
			show_alt_hex: true,
			show_alt_float: true,
			alt_mode: AlternateFormatMode::Smart,
			limit_size: true,
			time_24_hour: false,
			stack_xyz: false,
		}
	}

	pub fn exponent_format(&self) -> Self {
		Format {
			mode: FormatMode::Normal,
			integer_mode: IntegerMode::BigInteger,
			decimal_point: self.decimal_point,
			thousands: false,
			precision: 4,
			trailing_zeros: true,
			integer_radix: 10,
			show_alt_hex: false,
			show_alt_float: false,
			alt_mode: AlternateFormatMode::Smart,
			limit_size: true,
			time_24_hour: false,
			stack_xyz: false,
		}
	}

	pub fn hex_format(&self) -> Self {
		Format {
			mode: FormatMode::Normal,
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
			alt_mode: self.alt_mode,
			limit_size: self.limit_size,
			time_24_hour: self.time_24_hour,
			stack_xyz: self.stack_xyz,
		}
	}

	pub fn decimal_format(&self) -> Self {
		Format {
			mode: self.mode,
			integer_mode: self.integer_mode,
			decimal_point: self.decimal_point,
			thousands: self.thousands,
			precision: self.precision,
			trailing_zeros: self.trailing_zeros,
			integer_radix: 10,
			show_alt_hex: self.show_alt_hex,
			show_alt_float: self.show_alt_float,
			alt_mode: self.alt_mode,
			limit_size: self.limit_size,
			time_24_hour: self.time_24_hour,
			stack_xyz: self.stack_xyz,
		}
	}

	pub fn with_max_precision(&self, max_precision: usize) -> Self {
		Format {
			mode: self.mode,
			integer_mode: self.integer_mode,
			decimal_point: self.decimal_point,
			thousands: self.thousands,
			precision: core::cmp::min(self.precision, max_precision),
			trailing_zeros: self.trailing_zeros,
			integer_radix: self.integer_radix,
			show_alt_hex: self.show_alt_hex,
			show_alt_float: self.show_alt_float,
			alt_mode: self.alt_mode,
			limit_size: self.limit_size,
			time_24_hour: self.time_24_hour,
			stack_xyz: self.stack_xyz,
		}
	}

	pub fn format_number(&self, num: &Number) -> FormatResult {
		match num {
			Number::Integer(int) => match self.mode {
				FormatMode::Normal | FormatMode::Rational => {
					if self.limit_size && int.bits() > MAX_SHORT_DISPLAY_BITS {
						FormatResult::Float(self.format_decimal(&num.to_decimal()))
					} else {
						FormatResult::Integer(self.format_bigint(int))
					}
				}
				FormatMode::Scientific | FormatMode::Engineering => {
					if self.integer_radix == 10
						|| (self.limit_size && int.bits() > MAX_SHORT_DISPLAY_BITS)
					{
						FormatResult::Float(self.format_decimal(&num.to_decimal()))
					} else {
						FormatResult::Integer(self.format_bigint(int))
					}
				}
			},
			Number::Rational(_, _) => FormatResult::Float(self.format_decimal(&num.to_decimal())),
			Number::Decimal(value) => FormatResult::Float(self.format_decimal(value)),
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
					DecimalPointMode::Period => result.push(','),
					DecimalPointMode::Comma => result.push('.'),
				}
			} else if digits % 4 == 0 && digits > 0 && self.integer_radix == 16 && self.thousands {
				result.push('\'');
			}

			// Get the lowest digit for the current radix and push it
			// onto the result.
			let digit: u8 = (&val % &radix).try_into().unwrap();
			if digit >= 10 {
				result.push(core::char::from_u32('A' as u32 + digit as u32 - 10).unwrap());
				non_decimal = true;
			} else {
				result.push(core::char::from_u32('0' as u32 + digit as u32).unwrap());
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

	fn format_decimal_post_round(&self, num: &Decimal, mode: FormatMode) -> String {
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
			FormatMode::Scientific => {
				let new_exponent = 1 - digit_str.len() as isize;
				let display = exponent - new_exponent;
				exponent = new_exponent;
				display
			}
			FormatMode::Engineering => {
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
						DecimalPointMode::Period => integer_digits.push(',' as u32 as u8),
						DecimalPointMode::Comma => integer_digits.push('.' as u32 as u8),
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
		} else if self.mode == FormatMode::Scientific || self.mode == FormatMode::Engineering {
			"ᴇ0".to_string()
		} else {
			"".to_string()
		};

		if fraction_digits.len() > 0 {
			let decimal = match self.decimal_point {
				DecimalPointMode::Period => ".",
				DecimalPointMode::Comma => ",",
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
		let mut mode =
			if self.mode == FormatMode::Scientific || self.mode == FormatMode::Engineering {
				self.mode
			} else if integer_part_digits > self.precision as isize
				|| integer_part_digits < -4
				|| integer_part_digits < -(self.precision as isize / 2)
			{
				FormatMode::Scientific
			} else {
				FormatMode::Normal
			};

		// Check for rounding
		if digit_str.len() > self.precision {
			// More digits than desired precision, round at desired precision.
			let mut round_exponent =
				(exponent + digit_str.len() as isize) - self.precision as isize;
			if round_exponent > 0 && mode == FormatMode::Normal {
				// If rounding was in the middle of the integer portion, always display using
				// scientific notation, as we must not display digits after the rounding point.
				mode = FormatMode::Scientific;
			}

			// If there are leading zeros to display, account for this in the rounding
			if mode == FormatMode::Normal && integer_part_digits < 0 {
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

impl FormatResult {
	pub fn to_string(self) -> String {
		match self {
			FormatResult::Integer(string)
			| FormatResult::Float(string)
			| FormatResult::Complex(string)
			| FormatResult::Object(string) => string,
		}
	}

	pub fn to_str(&self) -> &str {
		match self {
			FormatResult::Integer(string)
			| FormatResult::Float(string)
			| FormatResult::Complex(string)
			| FormatResult::Object(string) => &string,
		}
	}
}
