use crate::complex::ComplexNumber;
use crate::edit::NumberEditor;
use crate::error::{Error, Result};
use crate::font::{SANS_13, SANS_16, SANS_20, SANS_24};
use crate::layout::Layout;
use crate::matrix::Matrix;
use crate::number::{Number, NumberFormat, NumberFormatMode, ToNumber, MAX_SHORT_DISPLAY_BITS};
use crate::screen::Color;
use crate::storage::{
	store, DeserializeInput, SerializeOutput, StorageObject, StorageRef, StorageRefSerializer,
};
use crate::time::{SimpleDateTimeFormat, SimpleDateTimeToString};
use crate::unit::{AngleUnit, CompositeUnit, TimeUnit, Unit};
use crate::vector::Vector;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use core::convert::TryFrom;
use core::ops::Add;
use num_bigint::{BigInt, ToBigInt};

#[derive(Clone)]
pub enum Value {
	Number(Number),
	NumberWithUnit(Number, CompositeUnit),
	Complex(ComplexNumber),
	DateTime(NaiveDateTime),
	Date(NaiveDate),
	Time(NaiveTime),
	Vector(Vector),
	Matrix(Matrix),
}

pub type ValueRef = StorageRef<Value>;

impl Value {
	/// Deep copies a value onto the non-reclaimable heap. This is used when pulling values out
	/// of reclaimable memory.
	pub fn deep_copy_value(value: ValueRef) -> Result<ValueRef> {
		let mut value = value.get()?;
		match &mut value {
			Value::Vector(vector) => vector.deep_copy_values()?,
			Value::Matrix(matrix) => matrix.deep_copy_values()?,
			_ => (),
		};
		store(value)
	}

	pub fn real_number(&self) -> Result<&Number> {
		match self {
			Value::Number(num) => Ok(num),
			Value::NumberWithUnit(num, _) => Ok(num),
			_ => Err(Error::NotARealNumber),
		}
	}

	pub fn complex_number<'a>(&'a self) -> Result<Cow<'a, ComplexNumber>> {
		match self {
			Value::Number(num) => Ok(Cow::Owned(ComplexNumber::from_real(num.clone()))),
			Value::NumberWithUnit(num, _) => Ok(Cow::Owned(ComplexNumber::from_real(num.clone()))),
			Value::Complex(value) => Ok(Cow::Borrowed(value)),
			_ => Err(Error::DataTypeMismatch),
		}
	}

	pub fn to_int<'a>(&'a self) -> Result<Cow<'a, BigInt>> {
		match self {
			Value::Number(num) => num.to_int(),
			Value::NumberWithUnit(num, _) => num.to_int(),
			_ => Err(Error::NotARealNumber),
		}
	}

	pub fn to_int_value<'a>(&'a self) -> Result<Cow<'a, Value>> {
		match self {
			Value::Number(Number::Integer(_)) => Ok(Cow::Borrowed(self)),
			Value::NumberWithUnit(Number::Integer(_), _) => Ok(Cow::Borrowed(self)),
			Value::Number(num) => Ok(Cow::Owned(Value::Number(Number::Integer(
				num.to_int()?.into_owned(),
			)))),
			Value::NumberWithUnit(num, unit) => Ok(Cow::Owned(Value::NumberWithUnit(
				Number::Integer(num.to_int()?.into_owned()),
				unit.clone(),
			))),
			_ => Err(Error::NotARealNumber),
		}
	}

	pub fn to_string(&self) -> String {
		match self {
			Value::Number(num) => num.to_string(),
			Value::NumberWithUnit(num, _) => num.to_string(),
			Value::Complex(num) => num.to_string(),
			Value::DateTime(dt) => dt.simple_format(&SimpleDateTimeFormat::full()),
			Value::Date(date) => date.simple_format(&SimpleDateTimeFormat::date()),
			Value::Time(time) => time.simple_format(&SimpleDateTimeFormat::time()),
			Value::Vector(vector) => {
				"⟪".to_string() + &vector.len().to_number().to_string() + " elem vector⟫"
			}
			Value::Matrix(matrix) => {
				"⟪".to_string()
					+ &matrix.rows().to_number().to_string()
					+ "×" + &matrix.cols().to_number().to_string()
					+ " matrix⟫"
			}
		}
	}

	pub fn format(&self, format: &NumberFormat) -> String {
		match self {
			Value::Number(num) => format.format_number(num),
			Value::NumberWithUnit(num, _) => format.format_number(num),
			Value::Complex(num) => num.format(format),
			_ => self.to_string(),
		}
	}

	pub fn is_vector_or_matrix(&self) -> bool {
		match self {
			Value::Vector(_) | Value::Matrix(_) => true,
			_ => false,
		}
	}

	pub fn pow(&self, power: &Value) -> Result<Value> {
		if let Value::Complex(value) = self {
			Self::check_complex(value.pow(&*power.complex_number()?))
		} else if let Value::Complex(power) = power {
			Self::check_complex(self.complex_number()?.pow(power))
		} else {
			Ok(Value::Number(self.real_number()?.pow(power.real_number()?)))
		}
	}

	pub fn sqrt(&self) -> Result<Value> {
		if let Value::Complex(value) = self {
			Self::check_complex(value.sqrt())
		} else {
			let value = self.real_number()?;
			if value.is_negative() {
				Self::check_complex(ComplexNumber::from_real(value.clone()).sqrt())
			} else {
				Ok(Value::Number(self.real_number()?.sqrt()))
			}
		}
	}

	pub fn log(&self) -> Result<Value> {
		if let Value::Complex(value) = self {
			Self::check_complex(value.log())
		} else if self.real_number()?.is_negative() {
			Self::check_complex(self.complex_number()?.log())
		} else {
			Ok(Value::Number(self.real_number()?.log()))
		}
	}

	pub fn exp10(&self) -> Result<Value> {
		if let Value::Complex(value) = self {
			Self::check_complex(value.exp10())
		} else {
			Ok(Value::Number(self.real_number()?.exp10()))
		}
	}

	pub fn ln(&self) -> Result<Value> {
		if let Value::Complex(value) = self {
			Self::check_complex(value.ln())
		} else if self.real_number()?.is_negative() {
			Self::check_complex(self.complex_number()?.ln())
		} else {
			Ok(Value::Number(self.real_number()?.ln()))
		}
	}

	pub fn exp(&self) -> Result<Value> {
		if let Value::Complex(value) = self {
			Self::check_complex(value.exp())
		} else {
			Ok(Value::Number(self.real_number()?.exp()))
		}
	}

	pub fn sin(&self, angle_mode: AngleUnit) -> Result<Value> {
		match self {
			Value::NumberWithUnit(num, unit) => {
				match unit
					.clone()
					.convert_single_unit(num, AngleUnit::Radians.into())
				{
					Ok(value) => Ok(Value::Number(value.sin())),
					_ => Ok(Value::Number(num.angle_to_radians(angle_mode).sin())),
				}
			}
			Value::Complex(value) => Self::check_complex(value.sin()),
			_ => Ok(Value::Number(
				self.real_number()?.angle_to_radians(angle_mode).sin(),
			)),
		}
	}

	pub fn cos(&self, angle_mode: AngleUnit) -> Result<Value> {
		match self {
			Value::NumberWithUnit(num, unit) => {
				match unit
					.clone()
					.convert_single_unit(num, AngleUnit::Radians.into())
				{
					Ok(value) => Ok(Value::Number(value.cos())),
					_ => Ok(Value::Number(num.angle_to_radians(angle_mode).cos())),
				}
			}
			Value::Complex(value) => Self::check_complex(value.cos()),
			_ => Ok(Value::Number(
				self.real_number()?.angle_to_radians(angle_mode).cos(),
			)),
		}
	}

	pub fn tan(&self, angle_mode: AngleUnit) -> Result<Value> {
		match self {
			Value::NumberWithUnit(num, unit) => {
				match unit
					.clone()
					.convert_single_unit(num, AngleUnit::Radians.into())
				{
					Ok(value) => Ok(Value::Number(value.tan())),
					_ => Ok(Value::Number(num.angle_to_radians(angle_mode).tan())),
				}
			}
			Value::Complex(value) => Self::check_complex(value.tan()),
			_ => Ok(Value::Number(
				self.real_number()?.angle_to_radians(angle_mode).tan(),
			)),
		}
	}

	pub fn asin(&self, angle_mode: AngleUnit) -> Result<Value> {
		if let Value::Complex(value) = self {
			Self::check_complex(value.asin())
		} else {
			let result = self.real_number()?.asin();
			if result.is_nan() {
				Self::check_complex(self.complex_number()?.asin())
			} else {
				Ok(Value::NumberWithUnit(
					result.angle_from_radians(angle_mode).into_owned(),
					CompositeUnit::single_unit(angle_mode.into()),
				))
			}
		}
	}

	pub fn acos(&self, angle_mode: AngleUnit) -> Result<Value> {
		if let Value::Complex(value) = self {
			Self::check_complex(value.acos())
		} else {
			let result = self.real_number()?.acos();
			if result.is_nan() {
				Self::check_complex(self.complex_number()?.acos())
			} else {
				Ok(Value::NumberWithUnit(
					result.angle_from_radians(angle_mode).into_owned(),
					CompositeUnit::single_unit(angle_mode.into()),
				))
			}
		}
	}

	pub fn atan(&self, angle_mode: AngleUnit) -> Result<Value> {
		if let Value::Complex(value) = self {
			Self::check_complex(value.atan())
		} else {
			let result = self.real_number()?.atan();
			if result.is_nan() {
				Self::check_complex(self.complex_number()?.atan())
			} else {
				Ok(Value::NumberWithUnit(
					result.angle_from_radians(angle_mode).into_owned(),
					CompositeUnit::single_unit(angle_mode.into()),
				))
			}
		}
	}

	pub fn sinh(&self) -> Result<Value> {
		match self {
			Value::Complex(value) => Self::check_complex(value.sinh()),
			_ => Ok(Value::Number(self.real_number()?.sinh())),
		}
	}

	pub fn cosh(&self) -> Result<Value> {
		match self {
			Value::Complex(value) => Self::check_complex(value.cosh()),
			_ => Ok(Value::Number(self.real_number()?.cosh())),
		}
	}

	pub fn tanh(&self) -> Result<Value> {
		match self {
			Value::Complex(value) => Self::check_complex(value.tanh()),
			_ => Ok(Value::Number(self.real_number()?.tanh())),
		}
	}

	pub fn asinh(&self) -> Result<Value> {
		match self {
			Value::Complex(value) => Self::check_complex(value.asinh()),
			_ => {
				let result = self.real_number()?.asinh();
				if result.is_nan() {
					Self::check_complex(self.complex_number()?.asinh())
				} else {
					Ok(Value::Number(result))
				}
			}
		}
	}

	pub fn acosh(&self) -> Result<Value> {
		match self {
			Value::Complex(value) => Self::check_complex(value.acosh()),
			_ => {
				let result = self.real_number()?.acosh();
				if result.is_nan() {
					Self::check_complex(self.complex_number()?.acosh())
				} else {
					Ok(Value::Number(result))
				}
			}
		}
	}

	pub fn atanh(&self) -> Result<Value> {
		match self {
			Value::Complex(value) => Self::check_complex(value.atanh()),
			_ => {
				let result = self.real_number()?.atanh();
				if result.is_nan() {
					Self::check_complex(self.complex_number()?.atanh())
				} else {
					Ok(Value::Number(result))
				}
			}
		}
	}

	pub fn add_unit(&self, unit: Unit) -> Result<Value> {
		match self {
			Value::Number(num) => Ok(Value::NumberWithUnit(
				num.clone(),
				CompositeUnit::single_unit(unit),
			)),
			Value::NumberWithUnit(num, existing_unit) => {
				let mut new_unit = existing_unit.clone();
				let new_num = new_unit.add_unit(num, unit);
				if new_unit.unitless() {
					Ok(Value::Number(new_num))
				} else {
					Ok(Value::NumberWithUnit(new_num, new_unit))
				}
			}
			_ => Err(Error::NotARealNumber),
		}
	}

	pub fn add_unit_inv(&self, unit: Unit) -> Result<Value> {
		match self {
			Value::Number(num) => Ok(Value::NumberWithUnit(
				num.clone(),
				CompositeUnit::single_unit_inv(unit),
			)),
			Value::NumberWithUnit(num, existing_unit) => {
				let mut new_unit = existing_unit.clone();
				let new_num = new_unit.add_unit_inv(num, unit);
				if new_unit.unitless() {
					Ok(Value::Number(new_num))
				} else {
					Ok(Value::NumberWithUnit(new_num, new_unit))
				}
			}
			_ => Err(Error::NotARealNumber),
		}
	}

	pub fn convert_single_unit(&self, unit: Unit) -> Result<Value> {
		match self {
			Value::NumberWithUnit(num, existing_unit) => {
				let mut new_unit = existing_unit.clone();
				let new_num = new_unit.convert_single_unit(num, unit)?;
				if new_unit.unitless() {
					Ok(Value::Number(new_num))
				} else {
					Ok(Value::NumberWithUnit(new_num, new_unit))
				}
			}
			Value::Number(_) => Err(Error::IncompatibleUnits),
			_ => Err(Error::NotARealNumber),
		}
	}

	fn datetime_add_secs(&self, dt: &NaiveDateTime, secs: &Number) -> Result<Value> {
		let nano = i64::try_from(&*(secs * &1_000_000_000.to_number()).to_int()?)?;
		Ok(Value::DateTime(dt.add(Duration::nanoseconds(nano))))
	}

	fn date_add_days(&self, date: &NaiveDate, days: &Number) -> Result<Value> {
		Ok(Value::Date(
			date.add(Duration::days(i64::try_from(&*days.to_int()?)?)),
		))
	}

	fn time_add_secs(&self, time: &NaiveTime, secs: &Number) -> Result<Value> {
		let nano = i64::try_from(&*(secs * &1_000_000_000.to_number()).to_int()?)?;
		Ok(Value::Time(time.add(Duration::nanoseconds(nano))))
	}

	pub fn check_complex(value: ComplexNumber) -> Result<Value> {
		if value.is_out_of_range() {
			Err(Error::ValueOutOfRange)
		} else if value.is_real() {
			// Use a pure real number if imaginary part is zero
			Ok(Value::Number(value.take_real_part()))
		} else {
			Ok(Value::Complex(value))
		}
	}

	fn value_add(&self, rhs: &Value) -> Result<Value> {
		match self {
			Value::Number(left) => match rhs {
				Value::Number(right) => Ok(Value::Number(left + right)),
				Value::NumberWithUnit(right, right_unit) => {
					Ok(Value::NumberWithUnit(left + right, right_unit.clone()))
				}
				Value::Complex(right) => {
					Self::check_complex(&ComplexNumber::from_real(left.clone()) + right)
				}
				Value::DateTime(right) => self.datetime_add_secs(right, left),
				Value::Date(right) => self.date_add_days(right, left),
				Value::Time(right) => self.time_add_secs(right, left),
				_ => Err(Error::DataTypeMismatch),
			},
			Value::NumberWithUnit(left, left_unit) => match rhs {
				Value::Number(right) => Ok(Value::NumberWithUnit(left + right, left_unit.clone())),
				Value::NumberWithUnit(right, right_unit) => Ok(Value::NumberWithUnit(
					&left_unit.coerce_to_other(left, right_unit)? + right,
					right_unit.clone(),
				)),
				Value::Complex(right) => {
					Self::check_complex(&ComplexNumber::from_real(left.clone()) + right)
				}
				Value::DateTime(right) => self.datetime_add_secs(
					right,
					&left_unit.coerce_to_other(
						left,
						&CompositeUnit::single_unit(TimeUnit::Seconds.into()),
					)?,
				),
				Value::Date(right) => self.date_add_days(
					right,
					&left_unit.coerce_to_other(
						left,
						&CompositeUnit::single_unit(TimeUnit::Days.into()),
					)?,
				),
				Value::Time(right) => self.time_add_secs(
					right,
					&left_unit.coerce_to_other(
						left,
						&CompositeUnit::single_unit(TimeUnit::Seconds.into()),
					)?,
				),
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Complex(left) => match rhs {
				Value::Number(right) => {
					Self::check_complex(left + &ComplexNumber::from_real(right.clone()))
				}
				Value::NumberWithUnit(right, _) => {
					Self::check_complex(left + &ComplexNumber::from_real(right.clone()))
				}
				Value::Complex(right) => Self::check_complex(left + right),
				_ => Err(Error::DataTypeMismatch),
			},
			Value::DateTime(left) => match rhs {
				Value::Number(right) => self.datetime_add_secs(left, right),
				Value::NumberWithUnit(right, right_unit) => self.datetime_add_secs(
					left,
					&right_unit.coerce_to_other(
						right,
						&CompositeUnit::single_unit(TimeUnit::Seconds.into()),
					)?,
				),
				Value::Complex(_) => Err(Error::NotARealNumber),
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Date(left) => match rhs {
				Value::Number(right) => self.date_add_days(left, right),
				Value::NumberWithUnit(right, right_unit) => self.date_add_days(
					left,
					&right_unit.coerce_to_other(
						right,
						&CompositeUnit::single_unit(TimeUnit::Days.into()),
					)?,
				),
				Value::Complex(_) => Err(Error::NotARealNumber),
				Value::Time(right) => Ok(Value::DateTime(NaiveDateTime::new(
					left.clone(),
					right.clone(),
				))),
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Time(left) => match rhs {
				Value::Number(right) => self.time_add_secs(left, right),
				Value::NumberWithUnit(right, right_unit) => self.time_add_secs(
					left,
					&right_unit.coerce_to_other(
						right,
						&CompositeUnit::single_unit(TimeUnit::Seconds.into()),
					)?,
				),
				Value::Complex(_) => Err(Error::NotARealNumber),
				Value::Date(right) => Ok(Value::DateTime(NaiveDateTime::new(
					right.clone(),
					left.clone(),
				))),
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Vector(left) => match rhs {
				Value::Vector(right) => {
					if left.len() != right.len() {
						return Err(Error::DimensionMismatch);
					}
					let mut result = left.clone();
					for i in 0..left.len() {
						let elem = (left.get(i)? + right.get(i)?)?;
						result.set(i, elem)?;
					}
					Ok(Value::Vector(result))
				}
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Matrix(left) => match rhs {
				Value::Matrix(right) => {
					if left.rows() != right.rows() || left.cols() != right.cols() {
						return Err(Error::DimensionMismatch);
					}
					let mut result = left.clone();
					for row in 0..left.rows() {
						for col in 0..left.cols() {
							let elem = (left.get(row, col)? + right.get(row, col)?)?;
							result.set(row, col, elem)?;
						}
					}
					Ok(Value::Matrix(result))
				}
				_ => Err(Error::DataTypeMismatch),
			},
		}
	}

	fn value_sub(&self, rhs: &Value) -> Result<Value> {
		match self {
			Value::Number(left) => match rhs {
				Value::Number(right) => Ok(Value::Number(left - right)),
				Value::NumberWithUnit(right, right_unit) => {
					Ok(Value::NumberWithUnit(left - right, right_unit.clone()))
				}
				Value::Complex(right) => {
					Self::check_complex(&ComplexNumber::from_real(left.clone()) - right)
				}
				_ => Err(Error::DataTypeMismatch),
			},
			Value::NumberWithUnit(left, left_unit) => match rhs {
				Value::Number(right) => Ok(Value::NumberWithUnit(left - right, left_unit.clone())),
				Value::NumberWithUnit(right, right_unit) => Ok(Value::NumberWithUnit(
					&left_unit.coerce_to_other(left, right_unit)? - right,
					right_unit.clone(),
				)),
				Value::Complex(right) => {
					Self::check_complex(&ComplexNumber::from_real(left.clone()) - right)
				}
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Complex(left) => match rhs {
				Value::Number(right) => {
					Self::check_complex(left - &ComplexNumber::from_real(right.clone()))
				}
				Value::NumberWithUnit(right, _) => {
					Self::check_complex(left - &ComplexNumber::from_real(right.clone()))
				}
				Value::Complex(right) => Self::check_complex(left - right),
				_ => Err(Error::DataTypeMismatch),
			},
			Value::DateTime(left) => match rhs {
				Value::Number(right) => self.datetime_add_secs(left, &-right),
				Value::NumberWithUnit(right, right_unit) => self.datetime_add_secs(
					left,
					&-right_unit.coerce_to_other(
						right,
						&CompositeUnit::single_unit(TimeUnit::Seconds.into()),
					)?,
				),
				Value::Complex(_) => Err(Error::NotARealNumber),
				Value::DateTime(right) => {
					let nanoseconds = left
						.signed_duration_since(*right)
						.num_nanoseconds()
						.ok_or(Error::ValueOutOfRange)?;
					Ok(Value::NumberWithUnit(
						nanoseconds.to_number() / 1_000_000_000.to_number(),
						CompositeUnit::single_unit(TimeUnit::Seconds.into()),
					))
				}
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Date(left) => match rhs {
				Value::Number(right) => self.date_add_days(left, &-right),
				Value::NumberWithUnit(right, right_unit) => self.date_add_days(
					left,
					&-right_unit.coerce_to_other(
						right,
						&CompositeUnit::single_unit(TimeUnit::Days.into()),
					)?,
				),
				Value::Complex(_) => Err(Error::NotARealNumber),
				Value::Date(right) => {
					let days: Number = left.signed_duration_since(*right).num_days().into();
					Ok(Value::NumberWithUnit(
						days,
						CompositeUnit::single_unit(TimeUnit::Days.into()),
					))
				}
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Time(left) => match rhs {
				Value::Number(right) => self.time_add_secs(left, &-right),
				Value::NumberWithUnit(right, right_unit) => self.time_add_secs(
					left,
					&-right_unit.coerce_to_other(
						right,
						&CompositeUnit::single_unit(TimeUnit::Seconds.into()),
					)?,
				),
				Value::Complex(_) => Err(Error::NotARealNumber),
				Value::Time(right) => {
					let nanoseconds = left
						.signed_duration_since(*right)
						.num_nanoseconds()
						.ok_or(Error::ValueOutOfRange)?;
					Ok(Value::NumberWithUnit(
						nanoseconds.to_number() / 1_000_000_000.to_number(),
						CompositeUnit::single_unit(TimeUnit::Seconds.into()),
					))
				}
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Vector(left) => match rhs {
				Value::Vector(right) => {
					if left.len() != right.len() {
						return Err(Error::DimensionMismatch);
					}
					let mut result = left.clone();
					for i in 0..left.len() {
						let elem = (left.get(i)? - right.get(i)?)?;
						result.set(i, elem)?;
					}
					Ok(Value::Vector(result))
				}
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Matrix(left) => match rhs {
				Value::Matrix(right) => {
					if left.rows() != right.rows() || left.cols() != right.cols() {
						return Err(Error::DimensionMismatch);
					}
					let mut result = left.clone();
					for row in 0..left.rows() {
						for col in 0..left.cols() {
							let elem = (left.get(row, col)? - right.get(row, col)?)?;
							result.set(row, col, elem)?;
						}
					}
					Ok(Value::Matrix(result))
				}
				_ => Err(Error::DataTypeMismatch),
			},
		}
	}

	fn value_neg(&self) -> Result<Value> {
		match self {
			Value::Vector(vector) => {
				let mut result = vector.clone();
				for i in 0..vector.len() {
					let elem = (-vector.get(i)?)?;
					result.set(i, elem)?;
				}
				Ok(Value::Vector(result))
			}
			Value::Matrix(matrix) => {
				let mut result = matrix.clone();
				for row in 0..matrix.rows() {
					for col in 0..matrix.cols() {
						let elem = (-matrix.get(row, col)?)?;
						result.set(row, col, elem)?;
					}
				}
				Ok(Value::Matrix(result))
			}
			_ => &Value::Number(0.to_number()) - self,
		}
	}

	fn value_mul(&self, rhs: &Value) -> Result<Value> {
		match self {
			Value::Number(left) => match rhs {
				Value::Number(right) => Ok(Value::Number(left * right)),
				Value::NumberWithUnit(right, right_unit) => {
					Ok(Value::NumberWithUnit(left * right, right_unit.clone()))
				}
				Value::Complex(right) => {
					Self::check_complex(&ComplexNumber::from_real(left.clone()) * right)
				}
				Value::Vector(right) => {
					let mut result = right.clone();
					for i in 0..right.len() {
						let elem = (self * &right.get(i)?)?;
						result.set(i, elem)?;
					}
					Ok(Value::Vector(result))
				}
				Value::Matrix(right) => {
					let mut result = right.clone();
					for row in 0..right.rows() {
						for col in 0..right.cols() {
							let elem = (self * &right.get(row, col)?)?;
							result.set(row, col, elem)?;
						}
					}
					Ok(Value::Matrix(result))
				}
				_ => Err(Error::DataTypeMismatch),
			},
			Value::NumberWithUnit(left, left_unit) => match rhs {
				Value::Number(right) => Ok(Value::NumberWithUnit(left * right, left_unit.clone())),
				Value::NumberWithUnit(right, right_unit) => {
					let mut unit = left_unit.clone();
					let left = unit.combine(left, right_unit);
					Ok(Value::NumberWithUnit(&left * right, unit))
				}
				Value::Complex(right) => {
					Self::check_complex(&ComplexNumber::from_real(left.clone()) * right)
				}
				Value::Vector(right) => {
					let mut result = right.clone();
					for i in 0..right.len() {
						let elem = (self * &right.get(i)?)?;
						result.set(i, elem)?;
					}
					Ok(Value::Vector(result))
				}
				Value::Matrix(right) => {
					let mut result = right.clone();
					for row in 0..right.rows() {
						for col in 0..right.cols() {
							let elem = (self * &right.get(row, col)?)?;
							result.set(row, col, elem)?;
						}
					}
					Ok(Value::Matrix(result))
				}
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Complex(left) => match rhs {
				Value::Number(right) => {
					Self::check_complex(left * &ComplexNumber::from_real(right.clone()))
				}
				Value::NumberWithUnit(right, _) => {
					Self::check_complex(left * &ComplexNumber::from_real(right.clone()))
				}
				Value::Complex(right) => Self::check_complex(left * right),
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Vector(left) => match rhs {
				Value::Number(_) | Value::NumberWithUnit(_, _) => {
					let mut result = left.clone();
					for i in 0..left.len() {
						let elem = (&left.get(i)? * rhs)?;
						result.set(i, elem)?;
					}
					Ok(Value::Vector(result))
				}
				Value::Vector(right) => {
					if left.len() != 1 || right.len() != 1 {
						return Err(Error::DimensionMismatch);
					}
					let mut result = Vector::new()?;
					result.push((left.get(0)? * right.get(0)?)?)?;
					Ok(Value::Vector(result))
				}
				Value::Matrix(right) => {
					if left.len() != right.rows() {
						return Err(Error::DimensionMismatch);
					}
					let mut result = Vector::new()?;
					for col in 0..right.cols() {
						let mut sum = Value::Number(0.into());
						for i in 0..left.len() {
							sum = (sum + (left.get(i)? * right.get(i, col)?)?)?;
						}
						result.push(sum)?;
					}
					Ok(Value::Vector(result))
				}
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Matrix(left) => match rhs {
				Value::Number(_) | Value::NumberWithUnit(_, _) => {
					let mut result = left.clone();
					for row in 0..left.rows() {
						for col in 0..left.cols() {
							let elem = (&left.get(row, col)? * rhs)?;
							result.set(row, col, elem)?;
						}
					}
					Ok(Value::Matrix(result))
				}
				Value::Matrix(right) => {
					if left.cols() != right.rows() {
						return Err(Error::DimensionMismatch);
					}
					let mut result = Matrix::new(left.rows(), right.cols())?;
					for row in 0..left.rows() {
						for col in 0..right.cols() {
							let mut sum = Value::Number(0.into());
							for i in 0..left.cols() {
								sum = (sum + (left.get(row, i)? * right.get(i, col)?)?)?;
							}
							result.set(row, col, sum)?;
						}
					}
					Ok(Value::Matrix(result))
				}
				Value::Vector(right) => {
					if left.cols() != 1 {
						return Err(Error::DimensionMismatch);
					}
					let mut result = Matrix::new(left.rows(), right.len())?;
					for row in 0..left.rows() {
						for col in 0..right.len() {
							let value = (left.get(row, 0)? * right.get(col)?)?;
							result.set(row, col, value)?;
						}
					}
					Ok(Value::Matrix(result))
				}
				_ => Err(Error::DataTypeMismatch),
			},
			_ => Err(Error::DataTypeMismatch),
		}
	}

	fn value_div(&self, rhs: &Value) -> Result<Value> {
		match self {
			Value::Number(left) => match rhs {
				Value::Number(right) => Ok(Value::Number(left / right)),
				Value::NumberWithUnit(right, right_unit) => {
					Ok(Value::NumberWithUnit(left / right, right_unit.inverse()))
				}
				Value::Complex(right) => {
					Self::check_complex(&ComplexNumber::from_real(left.clone()) / right)
				}
				_ => Err(Error::DataTypeMismatch),
			},
			Value::NumberWithUnit(left, left_unit) => match rhs {
				Value::Number(right) => Ok(Value::NumberWithUnit(left / right, left_unit.clone())),
				Value::NumberWithUnit(right, right_unit) => {
					let mut unit = left_unit.clone();
					let left = unit.combine(left, &right_unit.inverse());
					Ok(Value::NumberWithUnit(&left / right, unit))
				}
				Value::Complex(right) => {
					Self::check_complex(&ComplexNumber::from_real(left.clone()) / right)
				}
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Complex(left) => match rhs {
				Value::Number(right) => {
					Self::check_complex(left / &ComplexNumber::from_real(right.clone()))
				}
				Value::NumberWithUnit(right, _) => {
					Self::check_complex(left / &ComplexNumber::from_real(right.clone()))
				}
				Value::Complex(right) => Self::check_complex(left / right),
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Vector(left) => match rhs {
				Value::Number(_) | Value::NumberWithUnit(_, _) => {
					let mut result = left.clone();
					for i in 0..left.len() {
						let elem = (&left.get(i)? / rhs)?;
						result.set(i, elem)?;
					}
					Ok(Value::Vector(result))
				}
				_ => Err(Error::DataTypeMismatch),
			},
			Value::Matrix(left) => match rhs {
				Value::Number(_) | Value::NumberWithUnit(_, _) => {
					let mut result = left.clone();
					for row in 0..left.rows() {
						for col in 0..left.cols() {
							let elem = (&left.get(row, col)? / rhs)?;
							result.set(row, col, elem)?;
						}
					}
					Ok(Value::Matrix(result))
				}
				_ => Err(Error::DataTypeMismatch),
			},
			_ => Err(Error::DataTypeMismatch),
		}
	}

	fn render_units(&self) -> Option<Layout> {
		match self {
			Value::NumberWithUnit(_, units) => {
				// Font sizes are different depending on if the units have a fraction
				// representation or not, so keep track of both
				let mut numer_layout = Vec::new();
				let mut numer_only_layout = Vec::new();
				let mut denom_layout = Vec::new();
				let mut denom_only_layout = Vec::new();

				// Sort units into numerator and denominator layout lists
				for (_, unit) in &units.units {
					if unit.1 < 0 {
						// Power is negative, unit is in denominator
						if denom_layout.len() != 0 {
							// Add multiplication symbol to separate unit names
							denom_layout.push(Layout::StaticText(
								"∙",
								&SANS_20,
								Color::ContentText,
							));
							denom_only_layout.push(Layout::StaticText(
								"∙",
								&SANS_24,
								Color::ContentText,
							));
						}

						// Create layout in denomator of a fraction
						let unit_text =
							Layout::StaticText(unit.0.to_str(), &SANS_20, Color::ContentText);
						let layout = if unit.1 < -1 {
							Layout::Power(
								Box::new(unit_text),
								Box::new(Layout::Text(
									(-unit.1).to_number().to_string(),
									&SANS_13,
									Color::ContentText,
								)),
							)
						} else {
							unit_text
						};
						denom_layout.push(layout);

						// Create layout if there is no numerator
						denom_only_layout.push(Layout::Power(
							Box::new(Layout::StaticText(
								unit.0.to_str(),
								&SANS_24,
								Color::ContentText,
							)),
							Box::new(Layout::Text(
								unit.1.to_number().to_string(),
								&SANS_16,
								Color::ContentText,
							)),
						));
					} else if unit.1 > 0 {
						// Power is positive, unit is in numerator
						if numer_layout.len() != 0 {
							// Add multiplication symbol to separate unit names
							numer_layout.push(Layout::StaticText(
								"∙",
								&SANS_20,
								Color::ContentText,
							));
							numer_only_layout.push(Layout::StaticText(
								"∙",
								&SANS_24,
								Color::ContentText,
							));
						}

						// Create layout in numerator of a fraction
						let unit_text =
							Layout::StaticText(unit.0.to_str(), &SANS_20, Color::ContentText);
						let layout = if unit.1 > 1 {
							Layout::Power(
								Box::new(unit_text),
								Box::new(Layout::Text(
									unit.1.to_number().to_string(),
									&SANS_13,
									Color::ContentText,
								)),
							)
						} else {
							unit_text
						};
						numer_layout.push(layout);

						// Create layout if there is no denominator
						let unit_text =
							Layout::StaticText(unit.0.to_str(), &SANS_24, Color::ContentText);
						let layout = if unit.1 > 1 {
							Layout::Power(
								Box::new(unit_text),
								Box::new(Layout::Text(
									unit.1.to_number().to_string(),
									&SANS_16,
									Color::ContentText,
								)),
							)
						} else {
							unit_text
						};
						numer_only_layout.push(layout);
					}
				}

				// Create final layout
				if numer_layout.len() == 0 && denom_layout.len() == 0 {
					// No unit
					None
				} else if denom_layout.len() == 0 {
					// Numerator only
					numer_only_layout
						.insert(0, Layout::StaticText(" ", &SANS_24, Color::ContentText));
					Some(Layout::Horizontal(numer_only_layout))
				} else if numer_layout.len() == 0 {
					// Denominator only
					denom_only_layout
						.insert(0, Layout::StaticText(" ", &SANS_24, Color::ContentText));
					Some(Layout::Horizontal(denom_only_layout))
				} else {
					// Fraction
					let mut final_layout = Vec::new();
					final_layout.push(Layout::StaticText(" ", &SANS_24, Color::ContentText));
					final_layout.push(Layout::Fraction(
						Box::new(Layout::Horizontal(numer_layout)),
						Box::new(Layout::Horizontal(denom_layout)),
						Color::ContentText,
					));
					Some(Layout::Horizontal(final_layout))
				}
			}
			_ => None,
		}
	}

	fn alternate_hex_layout(&self, format: &NumberFormat, max_width: i32) -> Option<Layout> {
		match self.real_number() {
			Ok(Number::Integer(int)) => {
				// Integer, if number is ten or greater check for the
				// hexadecimal alternate form
				if format.show_alt_hex
					&& (format.integer_radix != 10
						|| format.mode == NumberFormatMode::Normal
						|| format.mode == NumberFormatMode::Rational)
					&& (int <= &-10.to_bigint().unwrap()
						|| int >= &10.to_bigint().unwrap()
						|| int <= &(-(format.integer_radix as i8)).to_bigint().unwrap()
						|| int >= &(format.integer_radix as i8).to_bigint().unwrap())
					&& int.bits() <= MAX_SHORT_DISPLAY_BITS
				{
					// There is an alternate form to display, try to generate a single
					// line layout for it.
					let string = if format.integer_radix == 10 {
						self.format(&format.hex_format())
					} else {
						self.format(&format.decimal_format())
					};
					Layout::single_line_string_layout(
						&string,
						&SANS_16,
						Color::ContentText,
						max_width,
						false,
					)
				} else {
					None
				}
			}
			_ => None,
		}
	}

	fn alternate_float_layout(&self, format: &NumberFormat, max_width: i32) -> Option<Layout> {
		match self {
			Value::Number(Number::Rational(_, _))
			| Value::NumberWithUnit(Number::Rational(_, _), _) => {
				// Real number in rational form
				if format.show_alt_float && format.mode == NumberFormatMode::Rational {
					if let Ok(number) = self.real_number() {
						let string = format.decimal_format().format_decimal(&number.to_decimal());
						Layout::single_line_string_layout(
							&string,
							&SANS_16,
							Color::ContentText,
							max_width,
							false,
						)
					} else {
						None
					}
				} else {
					None
				}
			}
			Value::Complex(value) => {
				if format.show_alt_float
					&& format.mode == NumberFormatMode::Rational
					&& (value.real_part().is_rational() || value.imaginary_part().is_rational())
				{
					// Complex number with at least one part in rational form
					let real_part = value.real_part().to_decimal();
					let imaginary_part = value.imaginary_part().to_decimal();
					let string = if imaginary_part.is_sign_negative() {
						format.with_max_precision(8).format_decimal(&real_part)
							+ " - " + &format
							.with_max_precision(8)
							.format_decimal(&-&*imaginary_part)
							+ "ℹ"
					} else {
						format.with_max_precision(8).format_decimal(&real_part)
							+ " + " + &format.with_max_precision(8).format_decimal(&imaginary_part)
							+ "ℹ"
					};
					Layout::single_line_string_layout(
						&string,
						&SANS_16,
						Color::ContentText,
						max_width,
						false,
					)
				} else {
					None
				}
			}
			_ => None,
		}
	}

	pub fn render(
		&self,
		format: &NumberFormat,
		editor: &Option<NumberEditor>,
		max_width: i32,
	) -> Layout {
		let mut max_width = max_width;

		// First check for an active editor. There can't be units at this stage so ignore
		// them at this point.
		if let Some(editor) = editor {
			// Currently editing number, format editor text
			let mut layout = if let Some(layout) = Layout::double_line_string_layout(
				&editor.to_string(format),
				&SANS_24,
				&SANS_20,
				Color::ContentText,
				max_width,
				true,
			) {
				// Full editor representation is OK, display it
				layout
			} else {
				// Editor text cannot fit in the layout constaints, display floating
				// point representation instead. Editor only operates on real numbers
				// so we assume that it is a real number.
				Layout::single_line_decimal_layout(
					&self.real_number().unwrap().to_decimal(),
					format,
					"",
					"",
					&SANS_24,
					Color::ContentText,
					max_width,
				)
			};

			// If the hex representation is enabled and valid, show it below
			if let Some(alt_layout) = self.alternate_hex_layout(format, max_width) {
				let mut alt_layout_items = Vec::new();
				alt_layout_items.push(layout);
				alt_layout_items.push(alt_layout);
				layout = Layout::Vertical(alt_layout_items);
			}
			return layout;
		}

		// Generate unit layout if there are units
		let mut unit_layout = self.render_units();
		if let Some(layout) = &unit_layout {
			let width = layout.width();
			if width > max_width / 2 {
				// Units take up too much room, don't display them
				unit_layout = None;
			} else {
				// Reduce remaining maximum width by width of units
				max_width -= width;
			}
		}

		// Check full detailed layout of value to see if it is valid and fits within the max size
		match self {
			Value::Number(value) | Value::NumberWithUnit(value, _) => {
				// Real number, try to render full representation
				if let Some((layout, is_rational)) = Layout::double_line_number_layout(
					value,
					format,
					&SANS_24,
					&SANS_20,
					Color::ContentText,
					max_width,
				) {
					// If units are present, add them to the layout
					let mut layout = if let Some(unit_layout) = unit_layout {
						let mut horizontal_items = Vec::new();
						horizontal_items.push(layout);
						horizontal_items.push(unit_layout);
						Layout::Horizontal(horizontal_items)
					} else {
						layout
					};

					// Check to see if alternate representations are available
					if let Some(alt_layout) = self.alternate_hex_layout(format, max_width) {
						let mut alt_layout_items = Vec::new();
						alt_layout_items.push(layout);
						alt_layout_items.push(alt_layout);
						layout = Layout::Vertical(alt_layout_items);
					} else if is_rational {
						if let Some(alt_layout) = self.alternate_float_layout(format, max_width) {
							let mut alt_layout_items = Vec::new();
							alt_layout_items.push(layout);
							alt_layout_items.push(alt_layout);
							layout = Layout::Vertical(alt_layout_items);
						}
					}
					return layout;
				}
			}
			Value::Complex(_) => {
				if let Some(mut layout) = Layout::single_line_numerical_value_layout(
					self, format, &SANS_24, &SANS_20, max_width, true,
				) {
					// Layout fits. Check to see if floating point alternate
					// representation is enabled
					if let Some(alt_layout) = self.alternate_float_layout(&format, max_width) {
						let mut alt_layout_items = Vec::new();
						alt_layout_items.push(layout);
						alt_layout_items.push(alt_layout);
						layout = Layout::Vertical(alt_layout_items);
					}
					return layout;
				}
			}
			Value::Vector(vector) => {
				// Vector, try to represent full form of vector entries in a single line. This is the
				// preferred form because it can show rationals.
				if let Some(layout) =
					vector.single_line_full_layout(format, &SANS_24, &SANS_20, max_width)
				{
					return layout;
				}

				// Try a three line layout with full precision decimal form
				if let Some(layout) = vector.multi_line_layout(format, &SANS_20, max_width, 3) {
					return layout;
				}

				// Try a three line layout with partial precision decimal form
				if let Some(layout) =
					vector.multi_line_layout(&format.with_max_precision(6), &SANS_20, max_width, 3)
				{
					return layout;
				}

				// Try a four line layout with smaller font
				if let Some(layout) =
					vector.multi_line_layout(&format.with_max_precision(6), &SANS_16, max_width, 4)
				{
					return layout;
				}
			}
			Value::Matrix(matrix) => {
				// Matrix, try to display all elements of a matrix of 4x4 or smaller.
				let largest_axis = core::cmp::max(matrix.rows(), matrix.cols());
				if largest_axis <= 4 {
					let mut max_font_size: i32 = if largest_axis == 1 {
						3
					} else if largest_axis <= 3 {
						2
					} else {
						1
					};

					while max_font_size >= 0 {
						let font = match max_font_size {
							3 => &SANS_24,
							2 => &SANS_20,
							1 => &SANS_16,
							0 => &SANS_13,
							_ => unreachable!(),
						};

						if let Some(layout) = matrix.layout(format, font, max_width) {
							return layout;
						}
						max_font_size -= 1;
					}
				}
			}
			_ => (),
		}

		// Generate simple layout that will always fit
		let layout =
			Layout::double_line_simple_value_layout(self, format, &SANS_24, &SANS_20, max_width);

		// If units are present, add them to the layout
		if let Some(unit_layout) = unit_layout {
			let mut horizontal_items = Vec::new();
			horizontal_items.push(layout);
			horizontal_items.push(unit_layout);
			Layout::Horizontal(horizontal_items)
		} else {
			layout
		}
	}
}

impl From<Number> for Value {
	fn from(num: Number) -> Self {
		Value::Number(num)
	}
}

impl From<u8> for Value {
	fn from(val: u8) -> Self {
		Value::Number(Number::Integer(val.into()))
	}
}

impl From<i8> for Value {
	fn from(val: i8) -> Self {
		Value::Number(Number::Integer(val.into()))
	}
}

impl From<u16> for Value {
	fn from(val: u16) -> Self {
		Value::Number(Number::Integer(val.into()))
	}
}

impl From<i16> for Value {
	fn from(val: i16) -> Self {
		Value::Number(Number::Integer(val.into()))
	}
}

impl From<u32> for Value {
	fn from(val: u32) -> Self {
		Value::Number(Number::Integer(val.into()))
	}
}

impl From<i32> for Value {
	fn from(val: i32) -> Self {
		Value::Number(Number::Integer(val.into()))
	}
}

impl From<u64> for Value {
	fn from(val: u64) -> Self {
		Value::Number(Number::Integer(val.into()))
	}
}

impl From<i64> for Value {
	fn from(val: i64) -> Self {
		Value::Number(Number::Integer(val.into()))
	}
}

impl From<u128> for Value {
	fn from(val: u128) -> Self {
		Value::Number(Number::Integer(val.into()))
	}
}

impl From<i128> for Value {
	fn from(val: i128) -> Self {
		Value::Number(Number::Integer(val.into()))
	}
}

impl From<f32> for Value {
	fn from(val: f32) -> Self {
		Value::Number(Number::Decimal(val.into()))
	}
}

impl From<f64> for Value {
	fn from(val: f64) -> Self {
		Value::Number(Number::Decimal(val.into()))
	}
}

impl From<ComplexNumber> for Value {
	fn from(val: ComplexNumber) -> Self {
		Value::Complex(val)
	}
}

impl core::ops::Add for Value {
	type Output = Result<Value>;

	fn add(self, rhs: Self) -> Self::Output {
		self.value_add(&rhs)
	}
}

impl core::ops::Add for &Value {
	type Output = Result<Value>;

	fn add(self, rhs: Self) -> Self::Output {
		self.value_add(rhs)
	}
}

impl core::ops::Sub for Value {
	type Output = Result<Value>;

	fn sub(self, rhs: Self) -> Self::Output {
		self.value_sub(&rhs)
	}
}

impl core::ops::Sub for &Value {
	type Output = Result<Value>;

	fn sub(self, rhs: Self) -> Self::Output {
		self.value_sub(rhs)
	}
}

impl core::ops::Mul for Value {
	type Output = Result<Value>;

	fn mul(self, rhs: Self) -> Self::Output {
		self.value_mul(&rhs)
	}
}

impl core::ops::Mul for &Value {
	type Output = Result<Value>;

	fn mul(self, rhs: Self) -> Self::Output {
		self.value_mul(rhs)
	}
}

impl core::ops::Div for Value {
	type Output = Result<Value>;

	fn div(self, rhs: Self) -> Self::Output {
		self.value_div(&rhs)
	}
}

impl core::ops::Div for &Value {
	type Output = Result<Value>;

	fn div(self, rhs: Self) -> Self::Output {
		self.value_div(rhs)
	}
}

impl core::ops::Neg for Value {
	type Output = Result<Value>;

	fn neg(self) -> Self::Output {
		self.value_neg()
	}
}

impl core::ops::Neg for &Value {
	type Output = Result<Value>;

	fn neg(self) -> Self::Output {
		self.value_neg()
	}
}

const VALUE_SERIALIZE_TYPE_NUMBER: u8 = 0;
const VALUE_SERIALIZE_TYPE_NUMBER_WITH_UNIT: u8 = 1;
const VALUE_SERIALIZE_TYPE_COMPLEX: u8 = 2;
const VALUE_SERIALIZE_TYPE_DATETIME: u8 = 3;
const VALUE_SERIALIZE_TYPE_DATE: u8 = 4;
const VALUE_SERIALIZE_TYPE_TIME: u8 = 5;
const VALUE_SERIALIZE_TYPE_VECTOR: u8 = 6;
const VALUE_SERIALIZE_TYPE_MATRIX: u8 = 7;

impl StorageObject for Value {
	fn serialize<Ref: StorageRefSerializer, Out: SerializeOutput>(
		&self,
		output: &mut Out,
		storage_refs: &mut Ref,
	) -> Result<()> {
		match self {
			Value::Number(num) => {
				output.write_u8(VALUE_SERIALIZE_TYPE_NUMBER)?;
				num.serialize(output, storage_refs)?;
			}
			Value::NumberWithUnit(num, unit) => {
				output.write_u8(VALUE_SERIALIZE_TYPE_NUMBER_WITH_UNIT)?;
				num.serialize(output, storage_refs)?;
				unit.serialize(output, storage_refs)?;
			}
			Value::Complex(num) => {
				output.write_u8(VALUE_SERIALIZE_TYPE_COMPLEX)?;
				num.real_part().serialize(output, storage_refs)?;
				num.imaginary_part().serialize(output, storage_refs)?;
			}
			Value::DateTime(dt) => {
				output.write_u8(VALUE_SERIALIZE_TYPE_DATETIME)?;
				output.write_i32(dt.year())?;
				output.write_u8(dt.month() as u8)?;
				output.write_u8(dt.day() as u8)?;
				output.write_u8(dt.hour() as u8)?;
				output.write_u8(dt.minute() as u8)?;
				output.write_u8(dt.second() as u8)?;
				output.write_u32(dt.nanosecond())?;
			}
			Value::Date(date) => {
				output.write_u8(VALUE_SERIALIZE_TYPE_DATE)?;
				output.write_i32(date.year())?;
				output.write_u8(date.month() as u8)?;
				output.write_u8(date.day() as u8)?;
			}
			Value::Time(time) => {
				output.write_u8(VALUE_SERIALIZE_TYPE_TIME)?;
				output.write_u8(time.hour() as u8)?;
				output.write_u8(time.minute() as u8)?;
				output.write_u8(time.second() as u8)?;
				output.write_u32(time.nanosecond())?;
			}
			Value::Vector(vector) => {
				output.write_u8(VALUE_SERIALIZE_TYPE_VECTOR)?;
				vector.serialize(output, storage_refs)?;
			}
			Value::Matrix(vector) => {
				output.write_u8(VALUE_SERIALIZE_TYPE_MATRIX)?;
				vector.serialize(output, storage_refs)?;
			}
		}
		Ok(())
	}

	unsafe fn deserialize<T: StorageRefSerializer>(
		input: &mut DeserializeInput,
		storage_refs: &T,
	) -> Result<Self> {
		match input.read_u8()? {
			VALUE_SERIALIZE_TYPE_NUMBER => {
				Ok(Value::Number(Number::deserialize(input, storage_refs)?))
			}
			VALUE_SERIALIZE_TYPE_NUMBER_WITH_UNIT => {
				let number = Number::deserialize(input, storage_refs)?;
				let unit = CompositeUnit::deserialize(input, storage_refs)?;
				Ok(Value::NumberWithUnit(number, unit))
			}
			VALUE_SERIALIZE_TYPE_COMPLEX => {
				let real = Number::deserialize(input, storage_refs)?;
				let imaginary = Number::deserialize(input, storage_refs)?;
				Ok(Value::Complex(ComplexNumber::from_parts(real, imaginary)))
			}
			VALUE_SERIALIZE_TYPE_DATETIME => {
				let year = input.read_i32()?;
				let month = input.read_u8()? as u32;
				let day = input.read_u8()? as u32;
				let hour = input.read_u8()? as u32;
				let minute = input.read_u8()? as u32;
				let second = input.read_u8()? as u32;
				let nanosecond = input.read_u32()?;
				let date = NaiveDate::from_ymd_opt(year, month, day).ok_or(Error::CorruptData)?;
				let time = NaiveTime::from_hms_nano_opt(hour, minute, second, nanosecond)
					.ok_or(Error::CorruptData)?;
				Ok(Value::DateTime(NaiveDateTime::new(date, time)))
			}
			VALUE_SERIALIZE_TYPE_DATE => {
				let year = input.read_i32()?;
				let month = input.read_u8()? as u32;
				let day = input.read_u8()? as u32;
				let date = NaiveDate::from_ymd_opt(year, month, day).ok_or(Error::CorruptData)?;
				Ok(Value::Date(date))
			}
			VALUE_SERIALIZE_TYPE_TIME => {
				let hour = input.read_u8()? as u32;
				let minute = input.read_u8()? as u32;
				let second = input.read_u8()? as u32;
				let nanosecond = input.read_u32()?;
				let time = NaiveTime::from_hms_nano_opt(hour, minute, second, nanosecond)
					.ok_or(Error::CorruptData)?;
				Ok(Value::Time(time))
			}
			VALUE_SERIALIZE_TYPE_VECTOR => {
				let vector = Vector::deserialize(input, storage_refs)?;
				Ok(Value::Vector(vector))
			}
			VALUE_SERIALIZE_TYPE_MATRIX => {
				let matrix = Matrix::deserialize(input, storage_refs)?;
				Ok(Value::Matrix(matrix))
			}
			_ => Err(Error::CorruptData),
		}
	}
}
