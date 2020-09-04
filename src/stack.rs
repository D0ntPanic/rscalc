use crate::font::{SANS_16, SANS_20, SANS_24};
use crate::number::{Number, NumberFormat};
use crate::screen::{Color, Rect, Screen};
use alloc::string::ToString;
use alloc::vec::Vec;

pub struct Stack {
	entries: Vec<Number>,
}

impl Stack {
	pub fn new() -> Self {
		let zero: Number = 0.into();
		let mut entries = Vec::new();
		entries.push(zero);
		Stack { entries }
	}

	pub fn len(&self) -> usize {
		self.entries.len()
	}

	pub fn push(&mut self, num: Number) {
		self.entries.push(num);
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
	}

	pub fn replace_entries(&mut self, count: usize, num: Number) {
		for _ in 1..count {
			self.pop();
		}
		self.set_top(num);
	}

	pub fn pop(&mut self) -> Number {
		let result = self.entries.pop().unwrap();
		if self.entries.len() == 0 {
			self.entries.push(0.into());
		}
		result
	}

	pub fn render<ScreenT: Screen>(&self, screen: &mut ScreenT, format: &NumberFormat, area: Rect) {
		let mut bottom = area.y + area.h;

		for idx in 0..self.len() {
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
				entry,
				area.x + label_width,
				area.w - label_width,
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
	value: &Number,
	x: i32,
	w: i32,
	bottom: i32,
) -> i32 {
	let string = format.format_number(value);
	let width = SANS_24.width(&string) + 4;
	SANS_24.draw(
		screen,
		x + w - width,
		bottom - SANS_24.height,
		&string,
		Color::ContentText,
	);
	SANS_24.height
}
