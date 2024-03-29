use crate::error::{Error, Result};
use crate::number::ToNumber;
use crate::storage::{
	store, DeserializeInput, SerializeOutput, StorageObject, StorageRef, StorageRefArray,
	StorageRefSerializer,
};
use crate::value::{Value, ValueRef};

const MAX_CAPACITY: usize = 1000;
const EXTRA_CAPACITY: usize = 4;

#[derive(Clone)]
pub struct Vector {
	len: usize,
	array: StorageRefArray<Value>,
	zero: StorageRef<Value>,
}

impl Vector {
	pub fn new() -> Result<Self> {
		let zero = store(Value::Number(0.into()))?;
		Ok(Vector {
			len: 0,
			array: StorageRefArray::new(EXTRA_CAPACITY, zero.clone())?,
			zero,
		})
	}

	fn from_len_and_array(len: usize, array: StorageRefArray<Value>) -> Result<Self> {
		let zero = store(Value::Number(0.into()))?;
		Ok(Vector { len, array, zero })
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn get(&self, idx: usize) -> Result<Value> {
		if idx >= self.len {
			return Err(Error::IndexOutOfRange);
		}
		Ok(self.array.get(idx)?.get()?)
	}

	pub fn get_ref(&self, idx: usize) -> Result<ValueRef> {
		if idx >= self.len {
			return Err(Error::IndexOutOfRange);
		}
		Ok(self.array.get(idx)?)
	}

	pub fn set(&mut self, idx: usize, value: Value) -> Result<()> {
		if idx >= self.len {
			return Err(Error::IndexOutOfRange);
		}
		if value.is_vector_or_matrix() {
			return Err(Error::DataTypeMismatch);
		}
		self.array.set(idx, store(value)?)
	}

	pub fn insert(&mut self, idx: usize, value: Value) -> Result<()> {
		if idx > self.len {
			return Err(Error::IndexOutOfRange);
		}
		if (self.len + 1) > MAX_CAPACITY {
			return Err(Error::VectorTooLarge);
		}
		if value.is_vector_or_matrix() {
			return Err(Error::DataTypeMismatch);
		}

		if (self.len + 1) > self.array.len() {
			// Not enough storage space in backing array, need to resize it
			let new_array = self
				.array
				.with_size(self.len + 1 + EXTRA_CAPACITY, self.zero.clone())?;
			self.array = new_array;
		}

		// Move values past insertion index forward
		for i in (idx..self.len).rev() {
			self.array.set(i + 1, self.array.get(i)?)?;
		}

		// Place value at insertion point and update length
		self.array.set(idx, store(value)?)?;
		self.len += 1;
		Ok(())
	}

	pub fn push(&mut self, value: Value) -> Result<()> {
		self.insert(self.len(), value)
	}

	pub fn pop(&mut self) -> Result<Value> {
		if self.len == 0 {
			return Err(Error::NotEnoughValues);
		}
		let result = self.array.get(self.len - 1)?.get()?;
		self.len -= 1;
		Ok(result)
	}

	pub fn extend_with(&mut self, other: &Vector) -> Result<()> {
		if (self.len + other.len) > MAX_CAPACITY {
			return Err(Error::VectorTooLarge);
		}
		if (self.len + other.len) > self.array.len() {
			// Not enough storage space in backing array, need to resize it
			let new_array = self
				.array
				.with_size(self.len + other.len + EXTRA_CAPACITY, self.zero.clone())?;
			self.array = new_array;
		}

		for i in 0..other.len {
			self.array.set(self.len + i, other.array.get(i)?)?;
		}
		self.len += other.len;
		Ok(())
	}

	/// Deep copies all values in the vector onto the non-reclaimable heap. This is used
	/// when pulling values out of reclaimable memory.
	pub fn deep_copy_values(&mut self) -> Result<()> {
		for i in 0..self.len {
			let value = Value::deep_copy_value(self.array.get(i)?)?;

			// Assume values that are already in the vector are safe for the vector
			self.array.set(i, value)?;
		}
		Ok(())
	}

	pub fn sum(&self) -> Result<Value> {
		if self.len() == 0 {
			return Err(Error::NotEnoughValues);
		}
		let mut result = self.get(0)?;
		for i in 1..self.len() {
			result = (result + self.get(i)?)?;
		}
		Ok(result)
	}

	pub fn mean(&self) -> Result<Value> {
		self.sum()? / Value::Number(self.len().to_number())
	}

	pub fn magnitude(&self) -> Result<Value> {
		self.dot(self)?.sqrt()
	}

	pub fn normalize(&self) -> Result<Vector> {
		if self.len() == 0 {
			return Err(Error::NotEnoughValues);
		}
		let magnitude = self.magnitude()?;
		let mut result = self.clone();
		for i in 0..self.len() {
			let value = (&self.get(i)? / &magnitude)?;
			result.set(i, value)?;
		}
		Ok(result)
	}

	fn mul_members(a: &Vector, a_idx: usize, b: &Vector, b_idx: usize) -> Result<Value> {
		a.get(a_idx)? * b.get(b_idx)?
	}

	pub fn dot(&self, other: &Vector) -> Result<Value> {
		if self.len() == 0 {
			return Err(Error::NotEnoughValues);
		}
		if self.len() != other.len() {
			return Err(Error::DimensionMismatch);
		}
		let mut result = Value::Number(0.into());
		for i in 0..self.len() {
			result = (result + Self::mul_members(self, i, other, i)?)?;
		}
		Ok(result)
	}

	pub fn cross(&self, other: &Vector) -> Result<Vector> {
		if self.len() != 3 || other.len() != 3 {
			return Err(Error::DimensionMismatch);
		}
		let mut result = Vector::new()?;
		result.push(
			(Self::mul_members(self, 1, other, 2)? - Self::mul_members(self, 2, other, 1)?)?,
		)?;
		result.push(
			(Self::mul_members(self, 2, other, 0)? - Self::mul_members(self, 0, other, 2)?)?,
		)?;
		result.push(
			(Self::mul_members(self, 0, other, 1)? - Self::mul_members(self, 1, other, 0)?)?,
		)?;
		Ok(result)
	}
}

impl StorageObject for Vector {
	fn serialize<Ref: StorageRefSerializer, Out: SerializeOutput>(
		&self,
		output: &mut Out,
		storage_refs: &mut Ref,
	) -> Result<()> {
		output.write_u32(self.len as u32)?;
		storage_refs.serialize_array(&self.array, output)?;
		Ok(())
	}

	unsafe fn deserialize<T: StorageRefSerializer>(
		input: &mut DeserializeInput,
		storage_refs: &T,
	) -> Result<Self> {
		let len = input.read_u32()? as usize;
		let array = storage_refs.deserialize_array(input)?;
		Ok(Vector::from_len_and_array(len, array)?)
	}
}
