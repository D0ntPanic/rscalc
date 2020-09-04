use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryInto;
use intel_dfp::Decimal;
use num_bigint::{BigInt, BigUint, Sign};

pub enum Number {
	Integer(BigInt),
	Decimal(Decimal),
}

pub enum NumberFormatMode {
	Normal,
	Rational,
	Scientific,
	Engineering,
}

pub enum NumberSeparatorMode {
	None,
	Comma,
	Period,
}

pub struct NumberFormat {
	pub mode: NumberFormatMode,
	pub separator: NumberSeparatorMode,
	pub precision: usize,
	pub trailing_zeros: bool,
	pub integer_radix: u8,
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
			Number::Decimal(value) => value.clone(),
		}
	}

	pub fn to_str(&self) -> String {
		NumberFormat::new().format_number(self)
	}

	pub fn sqrt(&self) -> Number {
		match &self {
			Number::Integer(value) => {
				let zero: BigInt = 0.into();
				if value < &zero {
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
			Number::Decimal(value) => Number::Decimal(value.sqrt()),
		}
	}

	pub fn pow(&self, power: &Number) -> Number {
		match &self {
			Number::Integer(left) => match power {
				Number::Integer(right) => {
					let zero: BigInt = 0.into();
					if right < &zero {
						// Fractional power, use float
						return Number::Decimal(self.to_decimal().pow(&power.to_decimal()));
					}
					if let Ok(power) = right.try_into() {
						Number::Integer(left.pow(power))
					} else {
						Number::Decimal(self.to_decimal().pow(&power.to_decimal()))
					}
				}
				Number::Decimal(right) => Number::Decimal(self.to_decimal().pow(right)),
			},
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

	fn num_add(&self, rhs: &Number) -> Number {
		match &self {
			Number::Integer(left) => match rhs {
				Number::Integer(right) => Number::Integer(left + right),
				Number::Decimal(right) => Number::Decimal(&self.to_decimal() + right),
			},
			Number::Decimal(left) => Number::Decimal(left + &rhs.to_decimal()),
		}
	}

	fn num_sub(&self, rhs: &Number) -> Number {
		match &self {
			Number::Integer(left) => match rhs {
				Number::Integer(right) => Number::Integer(left - right),
				Number::Decimal(right) => Number::Decimal(&self.to_decimal() - right),
			},
			Number::Decimal(left) => Number::Decimal(left - &rhs.to_decimal()),
		}
	}

	fn num_mul(&self, rhs: &Number) -> Number {
		match &self {
			Number::Integer(left) => match rhs {
				Number::Integer(right) => Number::Integer(left * right),
				Number::Decimal(right) => Number::Decimal(&self.to_decimal() * right),
			},
			Number::Decimal(left) => Number::Decimal(left * &rhs.to_decimal()),
		}
	}

	fn num_div(&self, rhs: &Number) -> Number {
		match &self {
			Number::Integer(left) => match rhs {
				Number::Integer(right) => {
					let zero: BigInt = 0.into();
					if right == &zero {
						// Divide by zero, use float to get the right inf/NaN
						return Number::Decimal(self.to_decimal() / rhs.to_decimal());
					}
					if (left % right) == zero {
						// Evenly divisible, use integer
						Number::Integer(left / right)
					} else {
						// Not evenly divisible, fall back to float
						Number::Decimal(self.to_decimal() / rhs.to_decimal())
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
			mode: NumberFormatMode::Normal,
			separator: NumberSeparatorMode::Comma,
			precision: 12,
			trailing_zeros: false,
			integer_radix: 10,
		}
	}

	pub fn format_number(&self, num: &Number) -> String {
		match num {
			Number::Integer(int) => self.format_bigint(int),
			Number::Decimal(value) => value.to_str(),
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
		let zero: BigUint = 0u32.into();
		let radix: BigUint = self.integer_radix.into();

		let mut digits = 0;
		while val != zero {
			// Check for thousands separator
			if digits % 3 == 0 && digits > 0 && self.integer_radix == 10 {
				match self.separator {
					NumberSeparatorMode::Comma => result.push(',' as u32 as u8),
					NumberSeparatorMode::Period => result.push('.' as u32 as u8),
					_ => (),
				}
			}

			// Get the lowest digit for the current radix and push it
			// onto the result.
			let digit: u8 = (&val % &radix).try_into().unwrap();
			if digit >= 10 {
				result.push('A' as u32 as u8 + digit - 10);
			} else {
				result.push('0' as u32 as u8 + digit);
			}

			// Update value to exclude this digit
			val /= &radix;
			digits += 1;
		}

		// If value was zero, ensure the string isn't blank
		if result.len() == 0 {
			result.push('0' as u32 as u8);
		}

		// Add prefixes for hex and oct modes
		if self.integer_radix == 16 && result.len() > 1 {
			result.push('x' as u32 as u8);
			result.push('0' as u32 as u8);
		}
		if self.integer_radix == 8 && result.len() > 1 {
			result.push('0' as u32 as u8);
		}

		// Add in sign
		if int.sign() == Sign::Minus {
			result.push('-' as u32 as u8);
		}

		// Create string
		result.reverse();
		String::from_utf8(result).unwrap()
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
