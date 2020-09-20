use crate::edit::NumberEditor;
use crate::error::{Error, Result};
use crate::font::SANS_16;
use crate::num_bigint::ToBigInt;
use crate::number::{IntegerMode, Number, NumberFormat};
use crate::screen::{Color, Rect, Screen};
use crate::storage::store;
use crate::undo::{push_undo_action, UndoAction};
use crate::value::{Value, ValueRef};
use alloc::string::ToString;
use alloc::vec::Vec;

pub const MAX_STACK_ENTRIES: usize = 1000;
pub const MAX_STACK_INDEX_DIGITS: usize = 3;

pub struct Stack {
	zero: ValueRef,
	entries: Vec<ValueRef>,
	editor: Option<NumberEditor>,
	push_new_entry: bool,
	empty: bool,
}

impl Stack {
	pub fn new() -> Self {
		let mut entries = Vec::new();
		let zero = store(0.into()).unwrap();
		entries.push(zero.clone());
		Stack {
			zero,
			entries,
			editor: None,
			push_new_entry: false,
			empty: true,
		}
	}

	pub fn len(&self) -> usize {
		self.entries.len()
	}

	pub fn editing(&self) -> bool {
		self.editor.is_some()
	}

	pub fn value_for_integer_mode(mode: &IntegerMode, value: Value) -> Value {
		match mode {
			IntegerMode::Float => value,
			IntegerMode::BigInteger => {
				if let Ok(int) = value.to_int_value() {
					int.into_owned()
				} else {
					value
				}
			}
			IntegerMode::SizedInteger(size, signed) => {
				if let Ok(int) = value.to_int() {
					let mask = 2.to_bigint().unwrap().pow(*size as u32) - 1.to_bigint().unwrap();
					let mut int = &*int & &mask;
					if *signed {
						let sign_bit = 2.to_bigint().unwrap().pow((*size - 1) as u32);
						if (&int & &sign_bit) != 0.to_bigint().unwrap() {
							int = -((int ^ mask) + 1.to_bigint().unwrap());
						}
					}
					Value::Number(Number::Integer(int))
				} else {
					value
				}
			}
		}
	}

	fn push_internal(&mut self, value: Value) -> Result<()> {
		if self.entries.len() >= MAX_STACK_ENTRIES {
			return Err(Error::StackOverflow);
		}
		self.entries.push(store(value)?);
		self.push_new_entry = true;
		self.empty = false;
		self.editor = None;
		Ok(())
	}

	pub fn push(&mut self, value: Value) -> Result<()> {
		self.push_internal(value)?;
		push_undo_action(UndoAction::Push);
		Ok(())
	}

	pub fn entry(&self, idx: usize) -> Result<Value> {
		let value_ref = self.entry_ref(idx)?;
		Ok(value_ref.get()?)
	}

	fn entry_ref(&self, idx: usize) -> Result<&ValueRef> {
		if idx >= self.entries.len() {
			return Err(Error::NotEnoughValues);
		}
		Ok(&self.entries[(self.entries.len() - 1) - idx])
	}

	fn entry_mut(&mut self, idx: usize) -> Result<&mut ValueRef> {
		if idx >= self.entries.len() {
			return Err(Error::NotEnoughValues);
		}
		let len = self.entries.len();
		Ok(&mut self.entries[(len - 1) - idx])
	}

	fn set_entry_internal(&mut self, idx: usize, value: Value) -> Result<()> {
		if idx >= self.entries.len() {
			return Err(Error::NotEnoughValues);
		}
		let len = self.entries.len();
		let value_ref = store(value)?;
		self.entries[(len - 1) - idx] = value_ref;
		self.empty = false;
		Ok(())
	}

	pub fn set_entry(&mut self, idx: usize, value: Value) -> Result<()> {
		if idx >= self.entries.len() {
			return Err(Error::NotEnoughValues);
		}
		let len = self.entries.len();
		let value_ref = store(value)?;
		push_undo_action(UndoAction::SetStackEntry(
			idx,
			self.entries[(len - 1) - idx].clone(),
		));
		self.entries[(len - 1) - idx] = value_ref;
		self.empty = false;
		Ok(())
	}

	pub fn top(&self) -> Value {
		self.entry(0).unwrap()
	}

	fn top_ref(&self) -> &ValueRef {
		self.entry_ref(0).unwrap()
	}

	fn set_top_internal(&mut self, value: Value) -> Result<()> {
		self.set_entry_internal(0, value)?;
		self.push_new_entry = true;
		self.empty = false;
		self.editor = None;
		Ok(())
	}

	pub fn set_top(&mut self, value: Value) -> Result<()> {
		let old_value = self.top_ref().clone();
		self.set_top_internal(value)?;
		push_undo_action(UndoAction::Replace([old_value].to_vec()));
		Ok(())
	}

	fn set_top_edit(&mut self, value: Value) -> Result<()> {
		let len = self.entries.len();
		let value_ref = store(value)?;
		self.entries[len - 1] = value_ref;
		self.empty = false;
		Ok(())
	}

	fn set_top_ref(&mut self, value: ValueRef) {
		*self.entry_mut(0).unwrap() = value;
		self.push_new_entry = true;
		self.empty = false;
		self.editor = None;
	}

	pub fn replace_entries(&mut self, count: usize, value: Value) -> Result<()> {
		if count > self.entries.len() {
			return Err(Error::NotEnoughValues);
		}
		let old_values = self.entries[self.entries.len() - count..].to_vec();
		self.set_entry_internal(count - 1, value)?;
		for _ in 1..count {
			self.pop_internal();
		}
		push_undo_action(UndoAction::Replace(old_values));
		self.push_new_entry = true;
		self.editor = None;
		Ok(())
	}

	fn pop_internal(&mut self) -> ValueRef {
		let result = self.entries.pop().unwrap();
		if self.entries.len() == 0 {
			self.entries.push(self.zero.clone());
			self.empty = true;
		}
		self.push_new_entry = true;
		self.editor = None;
		result
	}

	fn swap_internal(&mut self, a_idx: usize, b_idx: usize) -> Result<()> {
		let a = self.entry_ref(a_idx)?.clone();
		let b = self.entry_ref(b_idx)?.clone();
		*self.entry_mut(a_idx)? = b;
		*self.entry_mut(b_idx)? = a;
		self.end_edit();
		self.push_new_entry = true;
		self.editor = None;
		Ok(())
	}

	pub fn swap(&mut self, a_idx: usize, b_idx: usize) -> Result<()> {
		self.swap_internal(a_idx, b_idx)?;
		push_undo_action(UndoAction::Swap(a_idx, b_idx));
		Ok(())
	}

	pub fn rotate_down(&mut self) {
		if self.entries.len() > 1 {
			push_undo_action(UndoAction::RotateDown);
			let top = self.top_ref().clone();
			self.pop_internal();
			self.entries.insert(0, top);
		}
	}

	fn rotate_up_internal(&mut self) {
		if self.entries.len() > 1 {
			let bottom = self.entries[0].clone();
			self.entries.remove(0);
			self.entries.push(bottom);
			self.push_new_entry = true;
			self.editor = None;
		}
	}

	pub fn clear(&mut self) {
		push_undo_action(UndoAction::Clear(self.entries.clone()));
		self.entries.clear();
		self.entries.push(self.zero.clone());
		self.push_new_entry = false;
		self.empty = true;
		self.editor = None;
	}

	pub fn enter(&mut self) -> Result<()> {
		self.push(self.top().clone())?;
		self.push_new_entry = false;
		Ok(())
	}

	pub fn input_value(&mut self, value: Value) -> Result<()> {
		if self.push_new_entry {
			self.push(value)
		} else {
			self.set_top(value)
		}
	}

	pub fn end_edit(&mut self) {
		if self.editor.is_some() {
			self.push_new_entry = true;
			self.editor = None;
		}
	}

	pub fn push_char(&mut self, ch: char, format: &NumberFormat) -> Result<()> {
		if self.editor.is_none() {
			if self.push_new_entry {
				self.push(0.into())?;
			} else {
				self.set_top_edit(0.into())?;
			}
			self.editor = Some(NumberEditor::new(format));
			self.push_new_entry = false;
		}
		if let Some(cur_editor) = &mut self.editor {
			cur_editor.push_char(ch)?;
			let value = cur_editor.number();
			self.set_top_edit(Value::Number(value))?;
		}
		Ok(())
	}

	pub fn exponent(&mut self) -> Result<()> {
		if let Some(cur_editor) = &mut self.editor {
			cur_editor.exponent();
			let value = cur_editor.number();
			self.set_top_edit(Value::Number(value))?;
		}
		Ok(())
	}

	pub fn backspace(&mut self) -> Result<()> {
		if let Some(cur_editor) = &mut self.editor {
			if cur_editor.backspace() {
				let value = cur_editor.number();
				self.set_top_edit(Value::Number(value))?;
			} else {
				self.set_top_ref(self.zero.clone());
				self.push_new_entry = false;
			}
		} else if !self.empty {
			let mut new_entry = self.entries.len() > 1;
			let value = self.pop_internal();
			push_undo_action(UndoAction::Pop(value));
			if let Ok(Number::Integer(int)) = self.top().number() {
				if int == &0.to_bigint().unwrap() {
					new_entry = false;
				}
			}
			self.push_new_entry = new_entry;
		}
		Ok(())
	}

	pub fn neg(&mut self) -> Result<()> {
		if let Some(cur_editor) = &mut self.editor {
			cur_editor.neg();
			let value = cur_editor.number();
			self.set_top_edit(Value::Number(value))?;
		} else {
			self.set_top((-self.top())?)?;
		}
		Ok(())
	}

	pub fn undo(&mut self, action: UndoAction) -> Result<()> {
		match action {
			UndoAction::Push => {
				self.pop_internal();
			}
			UndoAction::Pop(value) => {
				if self.empty {
					self.set_top_internal(value.get()?)?;
				} else {
					self.push_internal(value.get()?)?;
				}
			}
			UndoAction::Replace(values) => {
				if values.len() == 0 {
					self.pop_internal();
				} else {
					self.set_top_internal(values[0].get()?)?;
					for value in &values[1..] {
						self.push_internal(value.get()?)?;
					}
				}
			}
			UndoAction::Swap(a, b) => {
				self.swap_internal(a, b)?;
			}
			UndoAction::Clear(values) => {
				let mut value_refs = Vec::new();
				for value in values.iter() {
					value_refs.push(store(value.get()?)?);
				}
				if !self.empty {
					value_refs.extend_from_slice(&self.entries);
				}
				self.entries = value_refs;
				self.push_new_entry = true;
				self.editor = None;
				self.empty = false;
			}
			UndoAction::RotateDown => {
				self.rotate_up_internal();
			}
			UndoAction::SetStackEntry(idx, value) => {
				self.set_entry_internal(idx, value.get()?)?;
			}
		}
		Ok(())
	}

	pub fn render<ScreenT: Screen>(&self, screen: &mut ScreenT, format: &NumberFormat, area: Rect) {
		screen.fill(area.clone(), Color::ContentBackground);

		let mut bottom = area.y + area.h;

		for idx in 0..self.len() {
			if idx > 0 {
				// Render stack entry separator
				screen.horizontal_pattern(
					area.x,
					area.w,
					bottom,
					0b100100100100100100100100,
					24,
					Color::StackSeparator,
				);
			}

			// Construct and measure stack entry label
			let label = match idx {
				0 => "x".to_string(),
				1 => "y".to_string(),
				2 => "z".to_string(),
				_ => Number::Integer(idx.into()).to_string(),
			};
			let label = label + ": ";
			let label_width = 4 + SANS_16.width(&label);

			// Render stack entry to a layout
			let entry = match self.entry(idx) {
				Ok(entry) => entry,
				Err(_) => continue,
			};
			let entry = Self::value_for_integer_mode(&format.integer_mode, entry);
			let width = area.w - label_width - 8;
			let layout = entry.render(format, if idx == 0 { &self.editor } else { &None }, width);

			// Draw the entry
			let height = layout.height();
			layout.render(
				screen,
				Rect {
					x: area.x + label_width + 4,
					y: bottom - height,
					w: width,
					h: height,
				},
				&area,
				None,
			);

			// Draw the label
			SANS_16.draw_clipped(
				screen,
				&area,
				4,
				(bottom - height) + (height - SANS_16.height) / 2,
				&label,
				Color::StackLabelText,
			);

			bottom -= height;
			if bottom < area.y {
				break;
			}
		}
	}
}
