use crate::complex::ComplexNumber;
use crate::constant::Constant;
use crate::error::{Error, Result};
use crate::format::{DecimalPointMode, Format, FormatMode, IntegerMode};
use crate::matrix::Matrix;
use crate::number::{Number, MAX_INTEGER_BITS};
use crate::stack::Stack;
use crate::storage::store;
use crate::time::Now;
use crate::unit::{AngleUnit, Unit};
use crate::value::{Value, ValueRef};
use crate::vector::Vector;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use num_bigint::ToBigInt;

#[cfg(feature = "std")]
use std::borrow::Cow;
#[cfg(feature = "std")]
use std::collections::BTreeMap;
#[cfg(feature = "std")]
use std::convert::TryFrom;

#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use core::convert::TryFrom;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Location {
	Integer(usize),
	StackOffset(usize),
	Variable(char),
}

pub struct Context {
	stack: Stack,
	format: Format,
	default_integer_format: IntegerMode,
	prev_decimal_integer_mode: IntegerMode,
	angle_mode: AngleUnit,
	memory: BTreeMap<Location, ValueRef>,
}

impl Context {
	pub fn new() -> Self {
		Context {
			stack: Stack::new(),
			format: Format::new(),
			default_integer_format: IntegerMode::BigInteger,
			prev_decimal_integer_mode: IntegerMode::Float,
			angle_mode: AngleUnit::Degrees,
			memory: BTreeMap::new(),
		}
	}

	pub fn new_with_undo() -> Self {
		Context {
			stack: Stack::new_with_undo(),
			format: Format::new(),
			default_integer_format: IntegerMode::BigInteger,
			prev_decimal_integer_mode: IntegerMode::Float,
			angle_mode: AngleUnit::Degrees,
			memory: BTreeMap::new(),
		}
	}

	pub fn stack(&self) -> &Stack {
		&self.stack
	}

	pub fn stack_mut(&mut self) -> &mut Stack {
		&mut self.stack
	}

	pub fn format(&self) -> &Format {
		&self.format
	}

	pub fn format_mut(&mut self) -> &mut Format {
		self.stack.invalidate_caches();
		&mut self.format
	}

	pub fn set_format_mode(&mut self, mode: FormatMode) {
		self.format.mode = mode;
		self.stack.invalidate_caches();
	}

	pub fn toggle_alt_hex(&mut self) {
		self.format.show_alt_hex = !self.format.show_alt_hex;
		self.stack.invalidate_caches();
	}

	pub fn toggle_alt_float(&mut self) {
		self.format.show_alt_float = !self.format.show_alt_float;
		self.stack.invalidate_caches();
	}

	pub fn set_thousands_separator(&mut self, state: bool) {
		self.format.thousands = state;
		self.stack.invalidate_caches();
	}

	pub fn set_decimal_point_mode(&mut self, mode: DecimalPointMode) {
		self.format.decimal_point = mode;
		self.stack.invalidate_caches();
	}

	pub fn set_float_mode(&mut self) -> Result<()> {
		if self.format.integer_radix == 10 {
			self.format.integer_mode = IntegerMode::Float;
			self.stack.invalidate_caches();
			Ok(())
		} else {
			Err(Error::FloatRequiresDecimalMode)
		}
	}

	pub fn set_integer_mode(&mut self, mode: IntegerMode) {
		self.format.integer_mode = mode;
		self.default_integer_format = mode;
		self.stack.invalidate_caches();
	}

	pub fn set_integer_radix(&mut self, radix: u8) {
		if radix == 10 {
			if self.format.integer_radix != 10 {
				self.format.integer_mode = self.prev_decimal_integer_mode;
			}
			self.format.integer_radix = radix;
		} else {
			if self.format.integer_radix == 10 {
				self.prev_decimal_integer_mode = self.format.integer_mode;
				self.format.integer_mode = self.default_integer_format;
			}
			self.format.integer_radix = radix;
		}
		self.stack.invalidate_caches();
	}

	pub fn toggle_integer_radix(&mut self) {
		if self.format.integer_radix == 10 {
			self.set_integer_radix(16);
		} else {
			self.set_integer_radix(10);
		}
	}

	pub fn default_integer_format(&self) -> &IntegerMode {
		&self.default_integer_format
	}

	pub fn set_default_integer_format(&mut self, mode: IntegerMode) {
		self.default_integer_format = mode;
	}

	pub fn prev_decimal_integer_mode(&self) -> &IntegerMode {
		&self.prev_decimal_integer_mode
	}

	pub fn set_prev_decimal_integer_mode(&mut self, mode: IntegerMode) {
		self.prev_decimal_integer_mode = mode;
	}

	pub fn angle_mode(&self) -> &AngleUnit {
		&self.angle_mode
	}

	pub fn set_angle_mode(&mut self, unit: AngleUnit) {
		self.angle_mode = unit;
	}

	pub fn stack_len(&self) -> usize {
		self.stack.len()
	}

	pub fn top(&self) -> Result<Value> {
		Ok(Stack::value_for_integer_mode(
			&self.format.integer_mode,
			self.stack.top()?,
		))
	}

	pub fn entry(&self, idx: usize) -> Result<Value> {
		Ok(Stack::value_for_integer_mode(
			&self.format.integer_mode,
			self.stack.entry(idx)?,
		))
	}

	pub fn replace_entries(&mut self, count: usize, value: Value) -> Result<()> {
		let value = Stack::value_for_integer_mode(&self.format.integer_mode, value);
		self.stack.replace_entries(count, value)?;
		Ok(())
	}

	pub fn replace_top_with_multiple(&mut self, items: Vec<ValueRef>) -> Result<()> {
		self.stack.replace_top_with_multiple(items)
	}

	pub fn set_top(&mut self, value: Value) -> Result<()> {
		let value = Stack::value_for_integer_mode(&self.format.integer_mode, value);
		self.stack.set_top(value)
	}

	pub fn set_entry(&mut self, offset: usize, value: Value) -> Result<()> {
		let value = Stack::value_for_integer_mode(&self.format.integer_mode, value);
		self.stack.set_entry(offset, value)?;
		Ok(())
	}

	pub fn push(&mut self, value: Value) -> Result<()> {
		let value = Stack::value_for_integer_mode(&self.format.integer_mode, value);
		self.stack.push(value)
	}

	pub fn push_constant(&mut self, constant: Constant) -> Result<()> {
		self.push(constant.value())
	}

	pub fn pop(&mut self) -> Result<Value> {
		Ok(Stack::value_for_integer_mode(
			&self.format.integer_mode,
			self.stack.pop()?,
		))
	}

	pub fn rotate_down(&mut self) {
		self.stack.rotate_down();
	}

	pub fn swap(&mut self, a_idx: usize, b_idx: usize) -> Result<()> {
		self.stack.swap(a_idx, b_idx)
	}

	pub fn clear_stack(&mut self) {
		self.stack.clear();
	}

	pub fn clear_undo_buffer(&mut self) {
		self.stack.clear_undo_buffer();
	}

	pub fn read<'a>(&'a self, location: &Location) -> Result<Value> {
		match location {
			Location::StackOffset(offset) => self.entry(*offset),
			location => {
				if let Some(value) = self.memory.get(location) {
					Ok(value.get()?)
				} else {
					Err(Error::ValueNotDefined)
				}
			}
		}
	}

	pub fn write(&mut self, location: Location, value: Value) -> Result<()> {
		match location {
			Location::StackOffset(offset) => self.set_entry(offset, value)?,
			location => {
				self.memory.insert(location, store(value)?);
			}
		}
		Ok(())
	}

	pub fn undo(&mut self) -> Result<()> {
		self.stack.undo()
	}

	pub fn add(&mut self) -> Result<()> {
		self.replace_entries(2, (self.entry(1)? + self.entry(0)?)?)
	}

	pub fn sub(&mut self) -> Result<()> {
		self.replace_entries(2, (self.entry(1)? - self.entry(0)?)?)
	}

	pub fn mul(&mut self) -> Result<()> {
		self.replace_entries(2, (self.entry(1)? * self.entry(0)?)?)
	}

	pub fn div(&mut self) -> Result<()> {
		self.replace_entries(2, (self.entry(1)? / self.entry(0)?)?)
	}

	pub fn recip(&mut self) -> Result<()> {
		self.set_top((Value::Number(1.into()) / self.top()?)?)
	}

	pub fn pow(&mut self) -> Result<()> {
		self.replace_entries(2, (self.entry(1)?).pow(&self.entry(0)?)?)
	}

	pub fn sqrt(&mut self) -> Result<()> {
		self.set_top(self.top()?.sqrt()?)
	}

	pub fn square(&mut self) -> Result<()> {
		let top = self.top()?;
		let square = (&top * &top)?;
		self.set_top(square)
	}

	pub fn percent(&mut self) -> Result<()> {
		let factor = (self.entry(0)? / Value::Number(100.into()))?;
		self.set_top((self.entry(1)? * factor)?)
	}

	pub fn log(&mut self) -> Result<()> {
		self.set_top(self.top()?.log()?)
	}

	pub fn exp10(&mut self) -> Result<()> {
		self.set_top(self.top()?.exp10()?)
	}

	pub fn ln(&mut self) -> Result<()> {
		self.set_top(self.top()?.ln()?)
	}

	pub fn exp(&mut self) -> Result<()> {
		self.set_top(self.top()?.exp()?)
	}

	pub fn sin(&mut self) -> Result<()> {
		self.set_top(self.top()?.sin(self.angle_mode)?)
	}

	pub fn cos(&mut self) -> Result<()> {
		self.set_top(self.top()?.cos(self.angle_mode)?)
	}

	pub fn tan(&mut self) -> Result<()> {
		self.set_top(self.top()?.tan(self.angle_mode)?)
	}

	pub fn asin(&mut self) -> Result<()> {
		self.set_top(self.top()?.asin(self.angle_mode)?)
	}

	pub fn acos(&mut self) -> Result<()> {
		self.set_top(self.top()?.acos(self.angle_mode)?)
	}

	pub fn atan(&mut self) -> Result<()> {
		self.set_top(self.top()?.atan(self.angle_mode)?)
	}

	pub fn sinh(&mut self) -> Result<()> {
		self.set_top(self.top()?.sinh()?)
	}

	pub fn cosh(&mut self) -> Result<()> {
		self.set_top(self.top()?.cosh()?)
	}

	pub fn tanh(&mut self) -> Result<()> {
		self.set_top(self.top()?.tanh()?)
	}

	pub fn asinh(&mut self) -> Result<()> {
		self.set_top(self.top()?.asinh()?)
	}

	pub fn acosh(&mut self) -> Result<()> {
		self.set_top(self.top()?.acosh()?)
	}

	pub fn atanh(&mut self) -> Result<()> {
		self.set_top(self.top()?.atanh()?)
	}

	pub fn and(&mut self) -> Result<()> {
		let value = Value::Number(Number::Integer(
			&*self.entry(1)?.to_int()? & &*self.entry(0)?.to_int()?,
		));
		self.replace_entries(2, value)
	}

	pub fn or(&mut self) -> Result<()> {
		let value = Value::Number(Number::Integer(
			&*self.entry(1)?.to_int()? | &*self.entry(0)?.to_int()?,
		));
		self.replace_entries(2, value)
	}

	pub fn xor(&mut self) -> Result<()> {
		let value = Value::Number(Number::Integer(
			&*self.entry(1)?.to_int()? ^ &*self.entry(0)?.to_int()?,
		));
		self.replace_entries(2, value)
	}

	pub fn not(&mut self) -> Result<()> {
		let value = Number::Integer(!&*self.top()?.to_int()?);
		self.set_top(Value::Number(value))
	}

	pub fn shl(&mut self) -> Result<()> {
		let x = self.entry(0)?;
		let mut x = x.to_int()?;
		if let IntegerMode::SizedInteger(size, _) = self.format.integer_mode {
			if size.is_power_of_two() {
				x = Cow::Owned(&*x & &(size - 1).to_bigint().unwrap());
			}
		}
		let x = u32::try_from(&*x)?;
		let y = self.entry(1)?;
		let y = y.to_int()?;
		if (y.bits() + x as u64) > MAX_INTEGER_BITS {
			return Err(Error::ValueOutOfRange);
		}
		let value = Value::Number(Number::Integer(&*y << x));
		self.replace_entries(2, value)
	}

	pub fn shr(&mut self) -> Result<()> {
		let x = self.entry(0)?;
		let mut x = x.to_int()?;
		if let IntegerMode::SizedInteger(size, _) = self.format.integer_mode {
			if size.is_power_of_two() {
				x = Cow::Owned(&*x & (size - 1).to_bigint().unwrap());
			}
		}
		let x = u32::try_from(&*x)?;
		let y = self.entry(1)?;
		let y = y.to_int()?;
		let value = Value::Number(Number::Integer(&*y >> x));
		self.replace_entries(2, value)
	}

	pub fn rotate_left(&mut self) -> Result<()> {
		if let IntegerMode::SizedInteger(size, _) = self.format.integer_mode {
			let x = self.entry(0)?;
			let mut x = x.to_int()?;
			if size.is_power_of_two() {
				x = Cow::Owned(&*x & (size - 1).to_bigint().unwrap());
			}
			if let Ok(x) = u32::try_from(&*x) {
				let y = self.entry(1)?;
				let y = y.to_int()?;
				let value = (&*y << x) | (&*y >> ((size as u32) - x));
				self.replace_entries(2, Value::Number(Number::Integer(value)))
			} else {
				Err(Error::ValueOutOfRange)
			}
		} else {
			Err(Error::RequiresSizedIntegerMode)
		}
	}

	pub fn rotate_right(&mut self) -> Result<()> {
		if let IntegerMode::SizedInteger(size, _) = self.format.integer_mode {
			let x = self.entry(0)?;
			let mut x = x.to_int()?;
			if size.is_power_of_two() {
				x = Cow::Owned(&*x & (size - 1).to_bigint().unwrap());
			}
			if let Ok(x) = u32::try_from(&*x) {
				let y = self.entry(1)?;
				let y = y.to_int()?;
				let value = (&*y >> x) | (&*y << ((size as u32) - x));
				self.replace_entries(2, Value::Number(Number::Integer(value)))
			} else {
				Err(Error::ValueOutOfRange)
			}
		} else {
			Err(Error::RequiresSizedIntegerMode)
		}
	}

	pub fn now(&mut self) -> Result<()> {
		self.push(Value::DateTime(NaiveDateTime::now()?))
	}

	pub fn date(&mut self) -> Result<()> {
		if let Value::DateTime(dt) = self.top()? {
			let date = dt.date();
			self.set_top(Value::Date(date))
		} else {
			let year = i32::try_from(&*self.entry(2)?.to_int()?)?;
			let month = u8::try_from(&*self.entry(1)?.to_int()?)?;
			let day = u8::try_from(&*self.entry(0)?.to_int()?)?;
			let date = NaiveDate::from_ymd_opt(year, month as u32, day as u32)
				.ok_or(Error::InvalidDate)?;
			self.replace_entries(3, Value::Date(date))
		}
	}

	pub fn time(&mut self) -> Result<()> {
		if let Value::DateTime(dt) = self.top()? {
			let time = dt.time();
			self.set_top(Value::Time(time))
		} else {
			let nano = (self.entry(0)?
				* Value::Number(Number::Integer(1_000_000_000.to_bigint().unwrap())))?;
			let hr = u8::try_from(&*self.entry(2)?.to_int()?)?;
			let min = u8::try_from(&*self.entry(1)?.to_int()?)?;
			let sec = u64::try_from(&*nano.to_int()?)?;
			let nsec = (sec % 1_000_000_000) as u32;
			let sec = (sec / 1_000_000_000) as u32;
			let time = NaiveTime::from_hms_nano_opt(hr as u32, min as u32, sec, nsec)
				.ok_or(Error::InvalidTime)?;
			self.replace_entries(3, Value::Time(time))
		}
	}

	pub fn clear_units(&mut self) -> Result<()> {
		let value = if let Value::NumberWithUnit(num, _) = self.top()? {
			Value::Number(num)
		} else {
			self.top()?
		};
		self.set_top(value)
	}

	pub fn add_unit(&mut self, unit: Unit) -> Result<()> {
		let value = self.top()?.add_unit(unit)?;
		self.set_top(value)
	}

	pub fn add_unit_squared(&mut self, unit: Unit) -> Result<()> {
		let value = self.top()?.add_unit(unit)?;
		let value = value.add_unit(unit)?;
		self.set_top(value)
	}

	pub fn add_unit_cubed(&mut self, unit: Unit) -> Result<()> {
		let value = self.top()?.add_unit(unit)?;
		let value = value.add_unit(unit)?;
		let value = value.add_unit(unit)?;
		self.set_top(value)
	}

	pub fn add_inv_unit(&mut self, unit: Unit) -> Result<()> {
		let value = self.top()?.add_inv_unit(unit)?;
		self.set_top(value)
	}

	pub fn add_inv_unit_squared(&mut self, unit: Unit) -> Result<()> {
		let value = self.top()?.add_inv_unit(unit)?;
		let value = value.add_inv_unit(unit)?;
		self.set_top(value)
	}

	pub fn add_inv_unit_cubed(&mut self, unit: Unit) -> Result<()> {
		let value = self.top()?.add_inv_unit(unit)?;
		let value = value.add_inv_unit(unit)?;
		let value = value.add_inv_unit(unit)?;
		self.set_top(value)
	}

	pub fn convert_to_unit(&mut self, unit: Unit) -> Result<()> {
		let value = self.top()?.convert_single_unit(unit)?;
		self.set_top(value)
	}

	pub fn sum(&mut self) -> Result<()> {
		if let Value::Vector(vector) = self.top()? {
			self.set_top(vector.sum()?)
		} else {
			Err(Error::DataTypeMismatch)
		}
	}

	pub fn mean(&mut self) -> Result<()> {
		if let Value::Vector(vector) = self.top()? {
			self.set_top(vector.mean()?)
		} else {
			Err(Error::DataTypeMismatch)
		}
	}

	pub fn dot_product(&mut self) -> Result<()> {
		let a = self.entry(1)?;
		let b = self.entry(0)?;
		if let Value::Vector(a_vector) = a {
			if let Value::Vector(b_vector) = b {
				self.replace_entries(2, a_vector.dot(&b_vector)?)
			} else {
				Err(Error::DataTypeMismatch)
			}
		} else {
			Err(Error::DataTypeMismatch)
		}
	}

	pub fn cross_product(&mut self) -> Result<()> {
		let a = self.entry(1)?;
		let b = self.entry(0)?;
		if let Value::Vector(a_vector) = a {
			if let Value::Vector(b_vector) = b {
				self.replace_entries(2, Value::Vector(a_vector.cross(&b_vector)?))
			} else {
				Err(Error::DataTypeMismatch)
			}
		} else {
			Err(Error::DataTypeMismatch)
		}
	}

	pub fn magnitude(&mut self) -> Result<()> {
		if let Value::Vector(vector) = self.top()? {
			self.set_top(vector.magnitude()?)
		} else {
			Err(Error::DataTypeMismatch)
		}
	}

	pub fn normalize(&mut self) -> Result<()> {
		if let Value::Vector(vector) = self.top()? {
			self.set_top(Value::Vector(vector.normalize()?))
		} else {
			Err(Error::DataTypeMismatch)
		}
	}

	pub fn to_matrix(&mut self) -> Result<()> {
		// Get the desired size of the matrix and create it
		let rows = usize::try_from(&*self.entry(1)?.to_int()?)?;
		let cols = usize::try_from(&*self.entry(0)?.to_int()?)?;
		if rows == 0 || cols == 0 {
			return Err(Error::ValueOutOfRange);
		}
		let mut result = Matrix::new(rows, cols)?;

		// Find the stack entry containing the start of the elements. Elements can
		// be placed as values on the stack or in vectors, or a mix of both.
		let mut remaining_size = rows * cols;
		let mut start_entry = 2;
		while remaining_size > 0 {
			match self.entry(start_entry)? {
				Value::Vector(vector) => {
					if vector.len() > remaining_size {
						return Err(Error::DimensionMismatch);
					}
					remaining_size -= vector.len();
				}
				Value::Matrix(_) => {
					return Err(Error::DataTypeMismatch);
				}
				_ => remaining_size -= 1,
			}
			if remaining_size == 0 {
				break;
			}
			start_entry += 1;
		}

		// Place the elements into the matrix
		let mut row = 0;
		let mut col = 0;
		for entry in (2..=start_entry).rev() {
			match self.entry(entry)? {
				Value::Vector(vector) => {
					for i in 0..vector.len() {
						result.set(row, col, vector.get(i)?)?;
						col += 1;
						if col >= cols {
							row += 1;
							col = 0;
						}
					}
				}
				entry => {
					result.set(row, col, entry)?;
					col += 1;
					if col >= cols {
						row += 1;
						col = 0;
					}
				}
			}
		}

		if rows == 1 {
			// Matrix of one row is always stored as a vector
			let mut vector = Vector::new()?;
			for col in 0..cols {
				vector.push(result.get(0, col)?)?;
			}
			self.replace_entries(start_entry + 1, Value::Vector(vector))
		} else {
			self.replace_entries(start_entry + 1, Value::Matrix(result))
		}
	}

	pub fn rows_to_matrix(&mut self) -> Result<()> {
		// Get the number of rows off the stack
		let rows = usize::try_from(&*self.entry(0)?.to_int()?)?;
		if rows == 0 {
			return Err(Error::ValueOutOfRange);
		}

		// Determine the number of columns based on the vectors on the stack
		let mut cols = None;
		for row in 0..rows {
			match self.entry(rows - row)? {
				Value::Vector(vector) => {
					if let Some(existing_cols) = cols {
						if vector.len() != existing_cols {
							return Err(Error::DimensionMismatch);
						}
					}
					cols = Some(vector.len());
				}
				_ => return Err(Error::DataTypeMismatch),
			}
		}
		let cols = cols.unwrap();

		if rows == 1 {
			// Matrix of one row is always stored as a vector, just put the vector
			// on the top of the stack by popping the row count after validating
			// that it is a valid vector.
			let value = self.entry(1)?;
			self.replace_entries(2, value)
		} else {
			// Create the matrix from the stack data
			let mut result = Matrix::new(rows, cols)?;
			for row in 0..rows {
				match self.entry(rows - row)? {
					Value::Vector(vector) => {
						for col in 0..cols {
							result.set(row, col, vector.get(col)?)?;
						}
					}
					_ => return Err(Error::DataTypeMismatch),
				}
			}

			self.replace_entries(rows + 1, Value::Matrix(result))
		}
	}

	pub fn cols_to_matrix(&mut self) -> Result<()> {
		// Get the number of columns off the stack
		let cols = usize::try_from(&*self.entry(0)?.to_int()?)?;
		if cols == 0 {
			return Err(Error::ValueOutOfRange);
		}

		// Determine the number of columns based on the vectors on the stack
		let mut rows = None;
		for col in 0..cols {
			match self.entry(cols - col)? {
				Value::Vector(vector) => {
					if let Some(existing_rows) = rows {
						if vector.len() != existing_rows {
							return Err(Error::DimensionMismatch);
						}
					}
					rows = Some(vector.len());
				}
				_ => return Err(Error::DataTypeMismatch),
			}
		}
		let rows = rows.unwrap();

		// Create the matrix from the stack data
		let mut result = Matrix::new(rows, cols)?;
		for col in 0..cols {
			match self.entry(cols - col)? {
				Value::Vector(vector) => {
					for row in 0..rows {
						result.set(row, col, vector.get(row)?)?;
					}
				}
				_ => return Err(Error::DataTypeMismatch),
			}
		}

		if rows == 1 {
			// Matrix of one row is always stored as a vector
			let mut vector = Vector::new()?;
			for col in 0..cols {
				vector.push(result.get(0, col)?)?;
			}
			self.replace_entries(cols + 1, Value::Vector(vector))
		} else {
			self.replace_entries(cols + 1, Value::Matrix(result))
		}
	}

	pub fn identity_matrix(&mut self) -> Result<()> {
		let size = usize::try_from(&*self.top()?.to_int()?)?;
		if size == 0 {
			Err(Error::ValueOutOfRange)
		} else if size == 1 {
			let mut result = Vector::new()?;
			result.push(1.into())?;
			self.set_top(Value::Vector(result))
		} else {
			let mut result = Matrix::new(size, size)?;
			for i in 0..size {
				result.set(i, i, 1.into())?;
			}
			self.set_top(Value::Matrix(result))
		}
	}

	pub fn transpose(&mut self) -> Result<()> {
		match self.top()? {
			Value::Vector(vector) => {
				if vector.len() != 1 {
					let mut result = Matrix::new(vector.len(), 1)?;
					for i in 0..vector.len() {
						result.set(i, 0, vector.get(i)?)?;
					}
					self.set_top(Value::Matrix(result))
				} else {
					Ok(())
				}
			}
			Value::Matrix(matrix) => {
				if matrix.cols() == 1 {
					let mut result = Vector::new()?;
					for i in 0..matrix.rows() {
						result.push(matrix.get(i, 0)?)?;
					}
					self.set_top(Value::Vector(result))
				} else {
					let mut result = Matrix::new(matrix.cols(), matrix.rows())?;
					for row in 0..matrix.cols() {
						for col in 0..matrix.rows() {
							result.set(row, col, matrix.get(col, row)?)?;
						}
					}
					self.set_top(Value::Matrix(result))
				}
			}
			_ => Err(Error::DataTypeMismatch),
		}
	}

	pub fn complex(&mut self) -> Result<()> {
		let top = self.entry(0)?;
		if let Value::Complex(value) = top {
			// If a complex number is on the top of the stack, break it into
			// real and imaginary parts.
			let mut items = Vec::new();
			items.push(store(Value::Number(value.real_part().clone()))?);
			items.push(store(Value::Number(value.imaginary_part().clone()))?);
			self.replace_top_with_multiple(items)
		} else {
			// Take the real and imaginary components on the top two entries
			// on the stack and create a complex number.
			let real = self.entry(1)?;
			let imaginary = top;
			self.replace_entries(
				2,
				Value::check_complex(ComplexNumber::from_parts(
					real.real_number()?.clone(),
					imaginary.real_number()?.clone(),
				))?
				.into(),
			)
		}
	}

	pub fn add_to_vector(&mut self) -> Result<()> {
		let top = self.entry(0)?;
		if let Value::Vector(existing_vector) = top {
			// Top entry is a vector. Check entry above it.
			let prev_value = self.entry(1)?;
			if let Value::Vector(prev_vector) = prev_value {
				// Top two entries are vectors. Merge the vectors.
				let mut new_vector = prev_vector.clone();
				new_vector.extend_with(&existing_vector)?;
				self.replace_entries(2, Value::Vector(new_vector))
			} else {
				// Fold the second entry into the vector.
				let mut new_vector = existing_vector.clone();
				new_vector.insert(0, prev_value)?;
				self.replace_entries(2, Value::Vector(new_vector))
			}
		} else {
			// Create a vector containing the value on the top of the stack.
			let mut vector = Vector::new()?;
			vector.push(top)?;
			self.set_top(Value::Vector(vector))
		}
	}

	pub fn decompose(&mut self) -> Result<()> {
		let top = self.entry(0)?;
		if let Value::Vector(vector) = top {
			// Top entry is a vector. Break apart the vector.
			let mut values: Vec<ValueRef> = Vec::new();
			for i in 0..vector.len() {
				values.push(vector.get_ref(i)?);
			}
			self.replace_top_with_multiple(values)
		} else if let Value::Matrix(matrix) = top {
			// Top entry is a matrix. Break apart the matrix.
			let mut values: Vec<ValueRef> = Vec::new();
			for row in 0..matrix.rows() {
				for col in 0..matrix.cols() {
					values.push(matrix.get_ref(row, col)?);
				}
			}
			self.replace_top_with_multiple(values)
		} else {
			// Batch create a vector from the entries on the stack. If there
			// is a vector or matrix on the stack, stop there.
			let mut vector = Vector::new()?;
			for i in 0..self.stack_len() {
				let value = self.entry(i)?;
				if value.is_vector_or_matrix() {
					break;
				}
				vector.insert(0, value)?;
			}
			if vector.len() == 0 {
				return Err(Error::DataTypeMismatch);
			}
			self.replace_entries(vector.len(), Value::Vector(vector))
		}
	}
}
