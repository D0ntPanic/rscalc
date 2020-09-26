use crate::number::{Number, NumberFormat, ToNumber};
use alloc::string::String;
use intel_dfp::Decimal;

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

pub trait ToComplex {
	fn to_complex(self) -> ComplexNumber;
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

	pub fn i() -> Self {
		ComplexNumber {
			real: 0.into(),
			imaginary: 1.into(),
		}
	}

	pub fn neg_i() -> Self {
		ComplexNumber {
			real: 0.into(),
			imaginary: (-1).into(),
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

	pub fn is_out_of_range(&self) -> bool {
		self.real.is_infinite()
			|| self.real.is_nan()
			|| self.imaginary.is_infinite()
			|| self.imaginary.is_nan()
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

	pub fn magnitude(&self) -> Number {
		(&self.real * &self.real + &self.imaginary * &self.imaginary).sqrt()
	}

	pub fn polar_angle(&self) -> Number {
		if self.real.is_zero() && self.imaginary.is_zero() {
			0.to_number()
		} else {
			let mut angle = Decimal::atan2(&self.imaginary.to_decimal(), &self.real.to_decimal());
			if angle.is_sign_negative() {
				angle += Decimal::pi() * Decimal::from(2);
			}
			Number::Decimal(angle)
		}
	}

	pub fn sqrt(&self) -> Self {
		let magnitude = (&self.real * &self.real + &self.imaginary * &self.imaginary).sqrt();
		let mut real_squared = (&self.real + &magnitude) / 2.to_number();
		let mut imaginary_squared = (&magnitude - &self.real) / 2.to_number();
		if real_squared.is_negative() {
			// Numerical error can cause this to be a small negative, coerce to zero if it happens
			real_squared = 0.to_number();
		}
		if imaginary_squared.is_negative() {
			// Numerical error can cause this to be a small negative, coerce to zero if it happens
			imaginary_squared = 0.to_number();
		}
		let imaginary = imaginary_squared.sqrt();
		ComplexNumber {
			real: Self::check_int_bounds(real_squared.sqrt()),
			imaginary: Self::check_int_bounds(if self.imaginary.is_negative() {
				-imaginary
			} else {
				imaginary
			}),
		}
	}

	pub fn exp(&self) -> Self {
		let real_exp = self.real.exp();
		let cos_imag = self.imaginary.cos();
		let sin_imag = self.imaginary.sin();
		ComplexNumber {
			real: &real_exp * &cos_imag,
			imaginary: &real_exp * &sin_imag,
		}
	}

	pub fn ln(&self) -> Self {
		ComplexNumber {
			real: self.magnitude().ln(),
			imaginary: self.polar_angle(),
		}
	}

	pub fn exp10(&self) -> Self {
		10.to_complex().pow(self)
	}

	pub fn log(&self) -> Self {
		self.ln() / 10.to_number().ln().to_complex()
	}

	pub fn pow(&self, power: &ComplexNumber) -> Self {
		(power * &self.ln()).exp()
	}

	pub fn sin(&self) -> Self {
		ComplexNumber {
			real: &self.real.sin() * &self.imaginary.cosh(),
			imaginary: &self.real.cos() * &self.imaginary.sinh(),
		}
	}

	pub fn cos(&self) -> Self {
		ComplexNumber {
			real: &self.real.cos() * &self.imaginary.cosh(),
			imaginary: &-self.real.sin() * &self.imaginary.sinh(),
		}
	}

	pub fn tan(&self) -> Self {
		self.sin() / self.cos()
	}

	pub fn asin(&self) -> Self {
		ComplexNumber::from_parts(0.to_number(), -1.to_number())
			* ((1.to_complex() - (self * self)).sqrt() + &ComplexNumber::i() * self).ln()
	}

	pub fn acos(&self) -> Self {
		ComplexNumber::neg_i()
			* (&(ComplexNumber::i() * (1.to_complex() - (self * self)).sqrt()) + self).ln()
	}

	pub fn atan(&self) -> Self {
		ComplexNumber::from_parts(0.to_number(), -1.to_number() / 2.to_number())
			* ((&ComplexNumber::i() - self) / (&ComplexNumber::i() + self)).ln()
	}

	pub fn sinh(&self) -> Self {
		(1.to_complex() - (&(-2).to_complex() * self).exp()) / (2.to_complex() * (-self).exp())
	}

	pub fn cosh(&self) -> Self {
		(1.to_complex() + (&(-2).to_complex() * self).exp()) / (2.to_complex() * (-self).exp())
	}

	pub fn tanh(&self) -> Self {
		let e_2x = (&2.to_complex() * self).exp();
		(&e_2x - &1.to_complex()) / (&e_2x + &1.to_complex())
	}

	pub fn asinh(&self) -> Self {
		(self + &(self * self + 1.to_complex()).sqrt()).ln()
	}

	pub fn acosh(&self) -> Self {
		(self + &(self * self - 1.to_complex()).sqrt()).ln()
	}

	pub fn atanh(&self) -> Self {
		((&1.to_complex() + self) / (&1.to_complex() - self)).ln() / 2.to_complex()
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

impl core::ops::Neg for ComplexNumber {
	type Output = Self;

	fn neg(self) -> Self::Output {
		0.to_complex().complex_sub(&self)
	}
}

impl core::ops::Neg for &ComplexNumber {
	type Output = ComplexNumber;

	fn neg(self) -> Self::Output {
		0.to_complex().complex_sub(self)
	}
}

impl<T: ToNumber> From<T> for ComplexNumber {
	fn from(val: T) -> Self {
		ComplexNumber::from_real(val.to_number())
	}
}

impl From<Number> for ComplexNumber {
	fn from(val: Number) -> Self {
		ComplexNumber::from_real(val)
	}
}

impl<T: ToNumber> ToComplex for T {
	fn to_complex(self) -> ComplexNumber {
		self.into()
	}
}

impl ToComplex for Number {
	fn to_complex(self) -> ComplexNumber {
		self.into()
	}
}
