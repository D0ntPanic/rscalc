use crate::edit::NumberEditor;
use crate::error::{Error, Result};
use crate::font::SANS_16;
use crate::num_bigint::ToBigInt;
use crate::number::{IntegerMode, Number, NumberFormat};
use crate::screen::{Color, Rect, Screen};
use crate::value::Value;
use alloc::borrow::Cow;
use alloc::string::ToString;
use alloc::vec::Vec;

pub struct Stack {
	entries: Vec<Value>,
	editor: Option<NumberEditor>,
	push_new_entry: bool,
}

impl Stack {
	pub fn new() -> Self {
		let mut entries = Vec::new();
		entries.push(0.into());
		Stack {
			entries,
			editor: None,
			push_new_entry: false,
		}
	}

	pub fn len(&self) -> usize {
		self.entries.len()
	}

	pub fn editing(&self) -> bool {
		self.editor.is_some()
	}

	pub fn value_for_integer_mode<'a>(mode: &IntegerMode, value: &'a Value) -> Cow<'a, Value> {
		match mode {
			IntegerMode::Float => Cow::Borrowed(value),
			IntegerMode::BigInteger => {
				if let Ok(int) = value.to_int_value() {
					int
				} else {
					Cow::Borrowed(value)
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
					Cow::Owned(Value::Number(Number::Integer(int)))
				} else {
					Cow::Borrowed(value)
				}
			}
		}
	}

	pub fn push(&mut self, value: Value) {
		self.entries.push(value);
		self.push_new_entry = true;
		self.editor = None;
	}

	pub fn entry(&self, idx: usize) -> Result<&Value> {
		if idx >= self.entries.len() {
			return Err(Error::NotEnoughValues);
		}
		Ok(&self.entries[(self.entries.len() - 1) - idx])
	}

	pub fn entry_mut(&mut self, idx: usize) -> Result<&mut Value> {
		if idx >= self.entries.len() {
			return Err(Error::NotEnoughValues);
		}
		let len = self.entries.len();
		Ok(&mut self.entries[(len - 1) - idx])
	}

	pub fn top(&self) -> &Value {
		self.entry(0).unwrap()
	}

	pub fn top_mut(&mut self) -> &mut Value {
		self.entry_mut(0).unwrap()
	}

	pub fn set_top(&mut self, value: Value) {
		*self.top_mut() = value;
		self.push_new_entry = true;
		self.editor = None;
	}

	pub fn replace_entries(&mut self, count: usize, value: Value) -> Result<()> {
		if count > self.entries.len() {
			return Err(Error::NotEnoughValues);
		}
		for _ in 1..count {
			self.pop();
		}
		self.set_top(value);
		self.push_new_entry = true;
		self.editor = None;
		Ok(())
	}

	pub fn pop(&mut self) -> Value {
		let result = self.entries.pop().unwrap();
		if self.entries.len() == 0 {
			self.entries.push(0.into());
		}
		self.push_new_entry = true;
		self.editor = None;
		result
	}

	pub fn swap(&mut self, a_idx: usize, b_idx: usize) -> Result<()> {
		let a = self.entry(a_idx)?.clone();
		let b = self.entry(b_idx)?.clone();
		*self.entry_mut(a_idx)? = b;
		*self.entry_mut(b_idx)? = a;
		self.end_edit();
		self.push_new_entry = true;
		self.editor = None;
		Ok(())
	}

	pub fn rotate_down(&mut self) {
		if self.entries.len() > 1 {
			let top = self.top().clone();
			self.pop();
			self.entries.insert(0, top);
		}
	}

	pub fn enter(&mut self) {
		self.push(self.top().clone());
		self.push_new_entry = false;
	}

	pub fn input_value(&mut self, value: Value) {
		if self.push_new_entry {
			self.push(value);
		} else {
			self.set_top(value);
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
				self.push(0.into());
			} else {
				self.set_top(0.into());
			}
			self.editor = Some(NumberEditor::new(format));
			self.push_new_entry = false;
		}
		if let Some(cur_editor) = &mut self.editor {
			cur_editor.push_char(ch)?;
			let value = cur_editor.number();
			*self.top_mut() = Value::Number(value);
		}
		Ok(())
	}

	pub fn exponent(&mut self) {
		if let Some(cur_editor) = &mut self.editor {
			cur_editor.exponent();
			let value = cur_editor.number();
			*self.top_mut() = Value::Number(value);
		}
	}

	pub fn backspace(&mut self) {
		if let Some(cur_editor) = &mut self.editor {
			if cur_editor.backspace() {
				let value = cur_editor.number();
				*self.top_mut() = Value::Number(value);
			} else {
				self.set_top(0.into());
				self.push_new_entry = false;
			}
		} else {
			let mut new_entry = self.entries.len() > 1;
			self.pop();
			if let Ok(Number::Integer(int)) = self.top().number() {
				if int == &0.to_bigint().unwrap() {
					new_entry = false;
				}
			}
			self.push_new_entry = new_entry;
		}
	}

	pub fn neg(&mut self) -> Result<()> {
		if let Some(cur_editor) = &mut self.editor {
			cur_editor.neg();
			let value = cur_editor.number();
			*self.top_mut() = Value::Number(value);
		} else {
			self.set_top((-self.top())?);
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
				_ => Number::Integer(idx.into()).to_str(),
			};
			let label = label + ": ";
			let label_width = 4 + SANS_16.width(&label);

			// Render stack entry to a layout
			let entry =
				Self::value_for_integer_mode(&format.integer_mode, self.entry(idx).unwrap());
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
		}
	}
}
