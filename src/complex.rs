use crate::number::{Number, NumberFormat, ToNumber};
use alloc::string::String;

// Maximum integer size before it is converted into a floating point number.
pub const MAX_COMPLEX_INTEGER_BITS: u64 = 1024;

// Maximum denominator size. This will keep the maximum possible precision of the
// 128-bit float available in fractional form.
pub const MAX_COMPLEX_DENOMINATOR_BITS: u64 = 128;

#[derive(Clone)]
pub struct ComplexNumber {
	real: Number,
	imaginary: Number,
}

impl ComplexNumber {
	pub fn from_real(real: Number) -> Self {
		ComplexNumber {
			real: Self::check_int_bounds(real),
			imaginary: 0.into(),
		}
	}

	pub fn from_parts(real: Number, imaginary: Number) -> Self {
		ComplexNumber {
			real: Self::check_int_bounds(real),
			imaginary: Self::check_int_bounds(imaginary),
		}
	}

	fn check_int_bounds(value: Number) -> Number {
		Number::check_int_bounds_with_bit_count(
			value,
			MAX_COMPLEX_INTEGER_BITS,
			MAX_COMPLEX_DENOMINATOR_BITS,
		)
	}

	pub fn real_part(&self) -> &Number {
		&self.real
	}

	pub fn take_real_part(self) -> Number {
		self.real
	}

	pub fn set_real_part(&mut self, real: Number) {
		self.real = real;
	}

	pub fn imaginary_part(&self) -> &Number {
		&self.imaginary
	}

	pub fn set_imaginary_part(&mut self, imaginary: Number) {
		self.imaginary = imaginary;
	}

	pub fn is_real(&self) -> bool {
		self.imaginary.is_zero()
	}

	pub fn to_string(&self) -> String {
		if self.imaginary.is_negative() {
			self.real.to_string() + " - " + &(-&self.imaginary).to_string() + "ℹ"
		} else {
			self.real.to_string() + " + " + &self.imaginary.to_string() + "ℹ"
		}
	}

	pub fn format(&self, format: &NumberFormat) -> String {
		if self.imaginary.is_negative() {
			format.format_number(&self.real)
				+ " - " + &format.format_number(&-&self.imaginary)
				+ "ℹ"
		} else {
			format.format_number(&self.real) + " + " + &format.format_number(&self.imaginary) + "ℹ"
		}
	}

	pub fn sqrt(&self) -> Self {
		let magnitude = (&self.real * &self.real + &self.imaginary * &self.imaginary).sqrt();
		let imaginary = ((&magnitude - &self.real) / 2.to_number()).sqrt();
		ComplexNumber {
			real: Self::check_int_bounds(((&self.real + &magnitude) / 2.to_number()).sqrt()),
			imaginary: Self::check_int_bounds(if self.imaginary.is_negative() {
				-imaginary
			} else {
				imaginary
			}),
		}
	}

	fn complex_add(&self, other: &Self) -> Self {
		ComplexNumber {
			real: Self::check_int_bounds(&self.real + &other.real),
			imaginary: Self::check_int_bounds(&self.imaginary + &other.imaginary),
		}
	}

	fn complex_sub(&self, other: &Self) -> Self {
		ComplexNumber {
			real: Self::check_int_bounds(&self.real - &other.real),
			imaginary: Self::check_int_bounds(&self.imaginary - &other.imaginary),
		}
	}

	fn complex_mul(&self, other: &Self) -> Self {
		ComplexNumber {
			real: Self::check_int_bounds(
				&self.real * &other.real - &self.imaginary * &other.imaginary,
			),
			imaginary: Self::check_int_bounds(
				&self.real * &other.imaginary + &self.imaginary * &other.real,
			),
		}
	}

	fn complex_div(&self, other: &Self) -> Self {
		let divisor = &other.real * &other.real + &other.imaginary * &other.imaginary;
		ComplexNumber {
			real: Self::check_int_bounds(
				&(&self.real * &other.real + &self.imaginary * &other.imaginary) / &divisor,
			),
			imaginary: Self::check_int_bounds(
				&(&self.imaginary * &other.real - &self.real * &other.imaginary) / &divisor,
			),
		}
	}
}

impl core::ops::Add for ComplexNumber {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		self.complex_add(&rhs)
	}
}

impl core::ops::Add for &ComplexNumber {
	type Output = ComplexNumber;

	fn add(self, rhs: Self) -> Self::Output {
		self.complex_add(rhs)
	}
}

impl core::ops::AddAssign for ComplexNumber {
	fn add_assign(&mut self, rhs: Self) {
		*self = self.complex_add(&rhs);
	}
}

impl core::ops::Sub for ComplexNumber {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		self.complex_sub(&rhs)
	}
}

impl core::ops::Sub for &ComplexNumber {
	type Output = ComplexNumber;

	fn sub(self, rhs: Self) -> Self::Output {
		self.complex_sub(rhs)
	}
}

impl core::ops::SubAssign for ComplexNumber {
	fn sub_assign(&mut self, rhs: Self) {
		*self = self.complex_sub(&rhs);
	}
}

impl core::ops::Mul for ComplexNumber {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		self.complex_mul(&rhs)
	}
}

impl core::ops::Mul for &ComplexNumber {
	type Output = ComplexNumber;

	fn mul(self, rhs: Self) -> Self::Output {
		self.complex_mul(rhs)
	}
}

impl core::ops::MulAssign for ComplexNumber {
	fn mul_assign(&mut self, rhs: Self) {
		*self = self.complex_mul(&rhs);
	}
}

impl core::ops::Div for ComplexNumber {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		self.complex_div(&rhs)
	}
}

impl core::ops::Div for &ComplexNumber {
	type Output = ComplexNumber;

	fn div(self, rhs: Self) -> Self::Output {
		self.complex_div(rhs)
	}
}

impl core::ops::DivAssign for ComplexNumber {
	fn div_assign(&mut self, rhs: Self) {
		*self = self.complex_div(&rhs);
	}
}
