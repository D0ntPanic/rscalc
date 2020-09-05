use crate::edit::NumberEditor;
use crate::font::{SANS_16, SANS_20, SANS_24};
use crate::num_bigint::BigInt;
use crate::number::{Number, NumberFormat};
use crate::screen::{Color, Rect, Screen};
use alloc::string::ToString;
use alloc::vec::Vec;

pub struct Stack {
	entries: Vec<Number>,
	editor: Option<NumberEditor>,
	push_new_entry: bool,
}

impl Stack {
	pub fn new() -> Self {
		let zero: Number = 0.into();
		let mut entries = Vec::new();
		entries.push(zero);
		Stack {
			entries,
			editor: None,
			push_new_entry: false,
		}
	}

	pub fn len(&self) -> usize {
		self.entries.len()
	}

	pub fn push(&mut self, num: Number) {
		self.entries.push(num);
		self.push_new_entry = true;
		self.editor = None;
	}

	pub fn entry(&self, idx: usize) -> &Number {
		&self.entries[(self.entries.len() - 1) - idx]
	}

	pub fn entry_mut(&mut self, idx: usize) -> &mut Number {
		let len = self.entries.len();
		&mut self.entries[(len - 1) - idx]
	}

	pub fn top(&self) -> &Number {
		self.entry(0)
	}

	pub fn top_mut(&mut self) -> &mut Number {
		self.entry_mut(0)
	}

	pub fn set_top(&mut self, num: Number) {
		*self.top_mut() = num;
		self.push_new_entry = true;
		self.editor = None;
	}

	pub fn replace_entries(&mut self, count: usize, num: Number) {
		for _ in 1..count {
			self.pop();
		}
		self.set_top(num);
		self.push_new_entry = true;
		self.editor = None;
	}

	pub fn pop(&mut self) -> Number {
		let result = self.entries.pop().unwrap();
		if self.entries.len() == 0 {
			self.entries.push(0.into());
		}
		self.push_new_entry = true;
		self.editor = None;
		result
	}

	pub fn enter(&mut self) {
		self.push(self.top().clone());
		self.push_new_entry = false;
	}

	pub fn push_char(&mut self, ch: char) {
		if self.editor.is_none() {
			if self.push_new_entry {
				self.push(0.into());
			} else {
				self.set_top(0.into());
			}
			self.editor = Some(NumberEditor::new_decimal());
			self.push_new_entry = false;
		}
		if let Some(cur_editor) = &mut self.editor {
			if cur_editor.push_char(ch) {
				let value = cur_editor.number();
				*self.top_mut() = value;
			}
		}
	}

	pub fn exponent(&mut self) {
		if let Some(cur_editor) = &mut self.editor {
			cur_editor.exponent();
			let value = cur_editor.number();
			*self.top_mut() = value;
		}
	}

	pub fn backspace(&mut self) {
		if let Some(cur_editor) = &mut self.editor {
			if cur_editor.backspace() {
				let value = cur_editor.number();
				*self.top_mut() = value;
			} else {
				self.set_top(0.into());
				self.push_new_entry = false;
			}
		} else {
			let mut new_entry = self.entries.len() > 1;
			self.pop();
			if let Number::Integer(int) = self.top() {
				let zero: BigInt = 0.into();
				if int == &zero {
					new_entry = false;
				}
			}
			self.push_new_entry = new_entry;
		}
	}

	pub fn neg(&mut self) {
		if let Some(cur_editor) = &mut self.editor {
			cur_editor.neg();
			let value = cur_editor.number();
			*self.top_mut() = value;
		} else {
			let value = -self.top();
			self.set_top(value);
		}
	}

	pub fn render<ScreenT: Screen>(&self, screen: &mut ScreenT, format: &NumberFormat, area: Rect) {
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
				_ => Number::Integer((idx + 1).into()).to_str(),
			};
			let label = label + ": ";
			let label_width = 4 + SANS_16.width(&label);

			// Render stack entry
			let entry = self.entry(idx);
			let height = render_entry(
				screen,
				format,
				if idx == 0 { &self.editor } else { &None },
				entry,
				area.x + label_width,
				area.w - label_width - 4,
				bottom,
			);

			// Draw the label
			SANS_16.draw(
				screen,
				4,
				(bottom - height) + (height - SANS_16.height) / 2,
				&label,
				Color::StackLabelText,
			);

			bottom -= height;
		}
	}
}

fn render_entry<ScreenT: Screen>(
	screen: &mut ScreenT,
	format: &NumberFormat,
	editor: &Option<NumberEditor>,
	value: &Number,
	x: i32,
	w: i32,
	bottom: i32,
) -> i32 {
	// Get string for number. If there is an editor, use editor state instead.
	let string = match editor {
		Some(editor) => editor.to_str(format),
		None => format.format_number(value),
	};

	let mut top = bottom - SANS_24.height;

	// Check for alternate representation strings
	let alt_string = if let Number::Integer(int) = value {
		// Integer, if number is ten or greater check for the
		// hexadecimal alternate form
		let ten: BigInt = 10.into();
		let neg_ten = -&ten;
		if format.show_alt_hex && (int <= &neg_ten || int >= &ten) {
			if format.integer_radix == 10 {
				top -= SANS_16.height;
				Some(format.hex_format().format_number(value))
			} else if format.integer_radix == 16 {
				top -= SANS_16.height;
				Some(format.decimal_format().format_number(value))
			} else {
				None
			}
		} else {
			None
		}
	} else {
		None
	};

	// Render string
	let width = SANS_24.width(&string) + 4;
	let mut y = top;
	SANS_24.draw(screen, x + w - width, y, &string, Color::ContentText);

	if editor.is_some() {
		// If there is an editor, render cursor
		screen.fill(
			Rect {
				x: x + w - 3,
				y,
				w: 3,
				h: SANS_24.height,
			},
			Color::ContentText,
		);
	}

	y += SANS_24.height;

	if let Some(alt_string) = alt_string {
		let width = SANS_16.width(&alt_string) + 4;
		SANS_16.draw(screen, x + w - width, y, &alt_string, Color::ContentText);
	}

	bottom - top
}
