use crate::error::{Error, Result};
use crate::layout::Layout;
use crate::number::NumberFormat;
use crate::screen::Font;
use crate::storage::{
	store, DeserializeInput, SerializeOutput, StorageObject, StorageRef, StorageRefArray,
	StorageRefSerializer,
};
use crate::value::{Value, ValueRef};
use alloc::vec::Vec;

const MAX_CAPACITY: usize = 1024;

#[derive(Clone)]
pub struct Matrix {
	rows: usize,
	cols: usize,
	array: StorageRefArray<Value>,
	zero: StorageRef<Value>,
}

impl Matrix {
	pub fn new(rows: usize, cols: usize) -> Result<Self> {
		let size = rows.checked_mul(cols).ok_or(Error::MatrixTooLarge)?;
		if size > MAX_CAPACITY {
			return Err(Error::MatrixTooLarge);
		}
		let zero = store(Value::Number(0.into()))?;
		Ok(Matrix {
			rows,
			cols,
			array: StorageRefArray::new(size, zero.clone())?,
			zero,
		})
	}

	fn from_rows_cols_and_array(
		rows: usize,
		cols: usize,
		array: StorageRefArray<Value>,
	) -> Result<Self> {
		let zero = store(Value::Number(0.into()))?;
		Ok(Matrix {
			rows,
			cols,
			array,
			zero,
		})
	}

	pub fn rows(&self) -> usize {
		self.rows
	}

	pub fn cols(&self) -> usize {
		self.cols
	}

	pub fn get(&self, row: usize, col: usize) -> Result<Value> {
		if row >= self.rows || col >= self.cols {
			return Err(Error::IndexOutOfRange);
		}
		Ok(self.array.get((row * self.cols) + col)?.get()?)
	}

	pub fn get_ref(&self, row: usize, col: usize) -> Result<ValueRef> {
		if row >= self.rows || col >= self.cols {
			return Err(Error::IndexOutOfRange);
		}
		Ok(self.array.get((row * self.cols) + col)?)
	}

	pub fn set(&mut self, row: usize, col: usize, value: Value) -> Result<()> {
		if row >= self.rows || col >= self.cols {
			return Err(Error::IndexOutOfRange);
		}
		if value.is_vector_or_matrix() {
			return Err(Error::DataTypeMismatch);
		}
		self.array.set((row * self.cols) + col, store(value)?)
	}

	/// Deep copies all values in the matrix onto the non-reclaimable heap. This is used
	/// when pulling values out of reclaimable memory.
	pub fn deep_copy_values(&mut self) -> Result<()> {
		for i in 0..self.rows * self.cols {
			let value = Value::deep_copy_value(self.array.get(i)?)?;

			// Assume values that are already in the vector are safe for the vector
			self.array.set(i, value)?;
		}
		Ok(())
	}

	pub fn layout(
		&self,
		format: &NumberFormat,
		font: &'static Font,
		max_width: i32,
	) -> Option<Layout> {
		let mut col_items = Vec::new();
		let left_bracket = Layout::LeftMatrixBracket;
		let right_bracket = Layout::RightMatrixBracket;
		let col_width = max_width.checked_sub(
			left_bracket.width() + right_bracket.width() + (self.cols as i32 - 1) * 20,
		)? / (self.cols as i32);
		col_items.push(left_bracket);

		for col in 0..self.cols {
			if col != 0 {
				col_items.push(Layout::HorizontalSpace(20));
			}
			let mut row_items = Vec::new();
			for row in 0..self.rows {
				let value = if let Ok(value) = self.get(row, col) {
					value
				} else {
					return None;
				};

				row_items.push(Layout::single_line_simple_value_layout(
					&value, format, font, col_width,
				));
			}
			col_items.push(Layout::Vertical(row_items));
		}

		col_items.push(right_bracket);
		let layout = Layout::Horizontal(col_items);
		if layout.width() <= max_width {
			let mut result_items = Vec::new();
			result_items.push(Layout::VerticalSpace(2));
			result_items.push(layout);
			result_items.push(Layout::VerticalSpace(2));
			Some(Layout::Vertical(result_items))
		} else {
			None
		}
	}
}

impl StorageObject for Matrix {
	fn serialize<Ref: StorageRefSerializer, Out: SerializeOutput>(
		&self,
		output: &mut Out,
		storage_refs: &mut Ref,
	) -> Result<()> {
		output.write_u32(self.rows as u32)?;
		output.write_u32(self.cols as u32)?;
		storage_refs.serialize_array(&self.array, output)?;
		Ok(())
	}

	unsafe fn deserialize<T: StorageRefSerializer>(
		input: &mut DeserializeInput,
		storage_refs: &T,
	) -> Result<Self> {
		let rows = input.read_u32()? as usize;
		let cols = input.read_u32()? as usize;
		let array = storage_refs.deserialize_array(input)?;
		if rows.checked_mul(cols).ok_or(Error::MatrixTooLarge)? != array.len() {
			return Err(Error::CorruptData);
		}
		Ok(Matrix::from_rows_cols_and_array(rows, cols, array)?)
	}
}
