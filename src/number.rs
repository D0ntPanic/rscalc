use intel_dfp::Decimal;
use num_bigint::{BigInt, BigUint, Sign};
use core::convert::TryInto;

#[cfg(feature = "dm42")]
use alloc::string::String;
#[cfg(feature = "dm42")]
use alloc::vec::Vec;

pub enum Number {
	Integer(BigInt),
	Decimal(Decimal),
}

impl Number {
	pub fn new() -> Self {
		Number::Integer(0.into())
	}

	pub fn to_decimal(&self) -> Decimal {
		match self {
			Number::Integer(int) => bigint_to_decimal(&int),
			Number::Decimal(value) => value.clone()
		}
	}

	pub fn to_str(&self) -> String {
		match self {
			Number::Integer(int) => bigint_to_str(int, 10),
			Number::Decimal(value) => value.to_str()
		}
	}

	fn num_add(&self, rhs: &Number) -> Number {
		match &self {
			Number::Integer(left) => {
				match rhs {
					Number::Integer(right) => Number::Integer(left + right),
					Number::Decimal(right) => Number::Decimal(&self.to_decimal() + right)
				}
			}
			Number::Decimal(left) => Number::Decimal(left + &rhs.to_decimal())
		}
	}

	fn num_sub(&self, rhs: &Number) -> Number {
		match &self {
			Number::Integer(left) => {
				match rhs {
					Number::Integer(right) => Number::Integer(left - right),
					Number::Decimal(right) => Number::Decimal(&self.to_decimal() - right)
				}
			}
			Number::Decimal(left) => Number::Decimal(left - &rhs.to_decimal())
		}
	}

	fn num_mul(&self, rhs: &Number) -> Number {
		match &self {
			Number::Integer(left) => {
				match rhs {
					Number::Integer(right) => Number::Integer(left * right),
					Number::Decimal(right) => Number::Decimal(&self.to_decimal() * right)
				}
			}
			Number::Decimal(left) => Number::Decimal(left * &rhs.to_decimal())
		}
	}
}

fn bigint_to_decimal(int: &BigInt) -> Decimal {
	let mut result: Decimal = 0.into();
	let mut digit_factor: Decimal = 0.into();
	let (sign, digits) = int.to_u32_digits();
	for digit in digits {
		let digit_decimal: Decimal = digit.into();
		result += &digit_decimal * &digit_factor;
		digit_factor *= (1u64 << 32).into();
	}
	if sign == Sign::Minus {
		result = -result;
	}
	result
}

fn bigint_to_str(int: &BigInt, radix: u8) -> String {
	assert!(radix > 1 && radix <= 36);
	let mut result = Vec::new();
	let mut val = int.magnitude().clone();
	let zero: BigUint = 0u32.into();
	let ten: BigUint = 10u32.into();
	let mut digits = 0;
	while val != zero {
		if digits % 3 == 0 && digits > 0 && radix == 10 {
			result.push(0x2c);
		}
		let digit: u8 = (&val % &ten).try_into().unwrap();
		if digit >= 10 {
			result.push(0x41 + digit - 10);
		} else {
			result.push(0x30 + digit);
		}
		val /= &ten;
		digits += 1;
	}
	if result.len() == 0 {
		result.push(0x30);
	}
	if int.sign() == Sign::Minus {
		result.push(0x2d);
	}
	result.reverse();
	String::from_utf8(result).unwrap()
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
