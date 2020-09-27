use crate::error::{Error, Result};
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
		self.array.set(idx, store(value)?)?;
		Ok(())
	}

	/// Sets a reference to a value directly. Do not call with a reference to a vector
	/// or matrix, as this can cause circular references and crash. This is not checked,
	/// so is marked unsafe.
	unsafe fn set_ref(&mut self, idx: usize, value: ValueRef) -> Result<()> {
		if idx >= self.len {
			return Err(Error::IndexOutOfRange);
		}
		self.array.set(idx, value)?;
		Ok(())
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
			let value = Value::deep_copy_value(self.get_ref(i)?)?;
			unsafe {
				// Assume values that are already in the vector are safe for the vector
				self.set_ref(i, value)?;
			}
		}
		Ok(())
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
