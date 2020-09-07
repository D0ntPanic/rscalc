use crate::edit::NumberEditor;
use crate::font::{SANS_16, SANS_20, SANS_24};
use crate::layout::Layout;
use crate::number::{Number, NumberFormat, NumberFormatMode};
use crate::screen::Color;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use num_bigint::{BigInt, ToBigInt};

#[derive(Clone)]
pub enum Value {
	Number(Number),
}

impl Value {
	pub fn is_numeric(&self) -> bool {
		match self {
			Value::Number(_) => true,
		}
	}

	pub fn number(&self) -> Option<&Number> {
		match self {
			Value::Number(num) => Some(num),
		}
	}

	pub fn to_int(&self) -> Option<BigInt> {
		match self {
			Value::Number(num) => num.to_int(),
		}
	}

	pub fn to_str(&self) -> String {
		match self {
			Value::Number(num) => num.to_str(),
		}
	}

	pub fn format(&self, format: &NumberFormat) -> String {
		match self {
			Value::Number(num) => format.format_number(num),
		}
	}

	pub fn pow(&self, power: &Value) -> Option<Value> {
		if let (Some(value), Some(power)) = (self.number(), power.number()) {
			Some(Value::Number(value.pow(power)))
		} else {
			None
		}
	}

	pub fn sqrt(&self) -> Option<Value> {
		if let Some(value) = self.number() {
			Some(Value::Number(value.sqrt()))
		} else {
			None
		}
	}

	pub fn log(&self) -> Option<Value> {
		if let Some(value) = self.number() {
			Some(Value::Number(value.log()))
		} else {
			None
		}
	}

	pub fn exp10(&self) -> Option<Value> {
		if let Some(value) = self.number() {
			Some(Value::Number(value.exp10()))
		} else {
			None
		}
	}

	pub fn ln(&self) -> Option<Value> {
		if let Some(value) = self.number() {
			Some(Value::Number(value.ln()))
		} else {
			None
		}
	}

	pub fn exp(&self) -> Option<Value> {
		if let Some(value) = self.number() {
			Some(Value::Number(value.exp()))
		} else {
			None
		}
	}

	pub fn sin(&self) -> Option<Value> {
		if let Some(value) = self.number() {
			Some(Value::Number(value.sin()))
		} else {
			None
		}
	}

	pub fn cos(&self) -> Option<Value> {
		if let Some(value) = self.number() {
			Some(Value::Number(value.cos()))
		} else {
			None
		}
	}

	pub fn tan(&self) -> Option<Value> {
		if let Some(value) = self.number() {
			Some(Value::Number(value.tan()))
		} else {
			None
		}
	}

	pub fn asin(&self) -> Option<Value> {
		if let Some(value) = self.number() {
			Some(Value::Number(value.asin()))
		} else {
			None
		}
	}

	pub fn acos(&self) -> Option<Value> {
		if let Some(value) = self.number() {
			Some(Value::Number(value.acos()))
		} else {
			None
		}
	}

	pub fn atan(&self) -> Option<Value> {
		if let Some(value) = self.number() {
			Some(Value::Number(value.atan()))
		} else {
			None
		}
	}

	fn value_add(&self, rhs: &Value) -> Option<Value> {
		match self {
			Value::Number(left) => match rhs {
				Value::Number(right) => Some(Value::Number(left + right)),
			},
		}
	}

	fn value_sub(&self, rhs: &Value) -> Option<Value> {
		match self {
			Value::Number(left) => match rhs {
				Value::Number(right) => Some(Value::Number(left - right)),
			},
		}
	}

	fn value_mul(&self, rhs: &Value) -> Option<Value> {
		match self {
			Value::Number(left) => match rhs {
				Value::Number(right) => Some(Value::Number(left * right)),
			},
		}
	}

	fn value_div(&self, rhs: &Value) -> Option<Value> {
		match self {
			Value::Number(left) => match rhs {
				Value::Number(right) => Some(Value::Number(left / right)),
			},
		}
	}

	pub fn render(
		&self,
		format: &NumberFormat,
		editor: &Option<NumberEditor>,
		max_width: i32,
	) -> Layout {
		// Get string for number. If there is an editor, use editor state instead.
		let string = match editor {
			Some(editor) => editor.to_str(format),
			None => self.format(&format),
		};

		// Check for alternate representation strings
		let mut alt_string = match self.number() {
			Some(Number::Integer(int)) => {
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
				{
					if format.integer_radix == 10 {
						Some(self.format(&format.hex_format()))
					} else {
						Some(self.format(&format.decimal_format()))
					}
				} else {
					None
				}
			}
			Some(Number::Rational(_, _)) => {
				// Rational, show floating point as alternate form if enabled
				if format.show_alt_float && format.mode == NumberFormatMode::Rational {
					if let Some(number) = self.number() {
						Some(format.decimal_format().format_decimal(&number.to_decimal()))
					} else {
						None
					}
				} else {
					None
				}
			}
			_ => None,
		};

		// If alternate representation is too wide, don't display it
		if let Some(alt) = &alt_string {
			let width = SANS_16.width(alt) + 4;
			if width > max_width {
				alt_string = None;
			}
		}

		// Create layout for the default single line string rendering
		let mut layout = Layout::editable_text(
			string.clone(),
			&SANS_24,
			Color::ContentText,
			editor.is_some(),
		);

		// Check for more complex renderings
		let mut rational = false;
		if format.mode == NumberFormatMode::Rational {
			if let Some(Number::Rational(num, denom)) = self.number() {
				// Rational number, display as an integer and fraction. Break rational
				// into an integer part and fractional part.
				let int = num / denom.to_bigint().unwrap();
				let mut num = if &int < &0.to_bigint().unwrap() {
					-num - -&int * &denom.to_bigint().unwrap()
				} else {
					num - &int * &denom.to_bigint().unwrap()
				};

				// Get strings for the parts of the rational
				let int_str = if int == 0.to_bigint().unwrap() {
					if &num < &0.to_bigint().unwrap() {
						num = -num;
						"-".to_string()
					} else {
						"".to_string()
					}
				} else {
					format.format_bigint(&int)
				};
				let num_str = format.format_bigint(&num);
				let denom_str = format.format_bigint(&denom.to_bigint().unwrap());

				// Construct a layout for the rational
				let mut rational_horizontal_items = Vec::new();
				rational_horizontal_items.push(Layout::Text(int_str, &SANS_24, Color::ContentText));
				rational_horizontal_items.push(Layout::HorizontalSpace(4));
				rational_horizontal_items.push(Layout::Fraction(
					Box::new(Layout::Text(num_str, &SANS_20, Color::ContentText)),
					Box::new(Layout::Text(denom_str, &SANS_20, Color::ContentText)),
					Color::ContentText,
				));
				let rational_layout = Layout::Horizontal(rational_horizontal_items);

				// Check fractional representation width
				if rational_layout.width() <= max_width {
					// Fractional representation fits, use it
					layout = rational_layout;
					rational = true;
				} else {
					// Fractional representation is too wide, represent as float
					alt_string = None;
				}
			}
		}

		if !rational {
			// Integer or decimal float, first create a layout of the default
			// representation with a smaller font. If the default layout is too
			// wide, we will first reduce font size before splitting to multiple
			// lines.
			let min_layout = Layout::editable_text(
				string.clone(),
				&SANS_20,
				Color::ContentText,
				editor.is_some(),
			);

			if min_layout.width() > max_width * 2 {
				// String cannot fit onto two lines, render as decimal float
				if let Some(number) = self.number() {
					let string = format.format_decimal(&number.to_decimal());
					if let Some(alt) = &alt_string {
						if alt == &string {
							// Don't display the same representation as an alternate
							alt_string = None;
						}
					}

					layout = Layout::editable_text(
						string,
						&SANS_24,
						Color::ContentText,
						editor.is_some(),
					);
				} else {
					// TODO: Truncate non-numeric that doesn't fit
				}
			} else if min_layout.width() > max_width {
				// String does not fit, try to split it to two lines
				let chars: Vec<char> = string.chars().collect();
				let mut split_point = 0;
				let mut width = 0;
				for i in 0..chars.len() {
					let mut char_str = String::new();
					char_str.push(chars[(chars.len() - 1) - i]);
					split_point = i;
					// Add in the width of this character
					if i == 0 {
						width += SANS_20.width(&char_str);
					} else {
						width += SANS_20.advance(&char_str);
					}
					if width > max_width {
						break;
					}
				}

				// Check for a puncuation point near the split point, and move the split
				// there if there is one.
				for i in 0..5 {
					if i > split_point {
						break;
					}
					match chars[(chars.len() - 1) - (split_point - i)] {
						',' | '.' | 'x' | ' ' | '\'' => {
							split_point -= i;
							break;
						}
						_ => (),
					}
				}

				// Split the line into two lines
				let (first, second) = chars.split_at(chars.len() - split_point);
				let first_str: String = first.iter().collect();
				let second_str: String = second.iter().collect();
				let mut layout_items = Vec::new();
				layout_items.push(Layout::Text(first_str, &SANS_20, Color::ContentText));
				layout_items.push(Layout::editable_text(
					second_str,
					&SANS_20,
					Color::ContentText,
					editor.is_some(),
				));
				let split_layout = Layout::Vertical(layout_items);
				if split_layout.width() > max_width {
					// String cannot fit onto two lines, render as decimal float
					if let Some(number) = self.number() {
						let string = format.format_decimal(&number.to_decimal());
						if let Some(alt) = &alt_string {
							if alt == &string {
								// Don't display the same representation as an alternate
								alt_string = None;
							}
						}

						layout = Layout::editable_text(
							string,
							&SANS_24,
							Color::ContentText,
							editor.is_some(),
						);
					} else {
						// TODO: Truncate non-numeric that doesn't fit
					}
				} else {
					// String fits onto two lines
					layout = split_layout;
				}
			} else if layout.width() > max_width {
				layout = min_layout;
			}
		}

		// Add alternate string to layout if there was one
		if let Some(alt_string) = alt_string {
			let mut alt_layout_items = Vec::new();
			alt_layout_items.push(layout);
			alt_layout_items.push(Layout::Text(alt_string, &SANS_16, Color::ContentText));
			layout = Layout::Vertical(alt_layout_items);
		}

		layout
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

impl core::ops::Add for Value {
	type Output = Option<Value>;

	fn add(self, rhs: Self) -> Self::Output {
		self.value_add(&rhs)
	}
}

impl core::ops::Add for &Value {
	type Output = Option<Value>;

	fn add(self, rhs: Self) -> Self::Output {
		self.value_add(rhs)
	}
}

impl core::ops::Sub for Value {
	type Output = Option<Value>;

	fn sub(self, rhs: Self) -> Self::Output {
		self.value_sub(&rhs)
	}
}

impl core::ops::Sub for &Value {
	type Output = Option<Value>;

	fn sub(self, rhs: Self) -> Self::Output {
		self.value_sub(rhs)
	}
}

impl core::ops::Mul for Value {
	type Output = Option<Value>;

	fn mul(self, rhs: Self) -> Self::Output {
		self.value_mul(&rhs)
	}
}

impl core::ops::Mul for &Value {
	type Output = Option<Value>;

	fn mul(self, rhs: Self) -> Self::Output {
		self.value_mul(rhs)
	}
}

impl core::ops::Div for Value {
	type Output = Option<Value>;

	fn div(self, rhs: Self) -> Self::Output {
		self.value_div(&rhs)
	}
}

impl core::ops::Div for &Value {
	type Output = Option<Value>;

	fn div(self, rhs: Self) -> Self::Output {
		self.value_div(rhs)
	}
}

impl core::ops::Neg for Value {
	type Output = Option<Value>;

	fn neg(self) -> Self::Output {
		let zero: Value = 0.into();
		zero.value_sub(&self)
	}
}

impl core::ops::Neg for &Value {
	type Output = Option<Value>;

	fn neg(self) -> Self::Output {
		let zero: Value = 0.into();
		zero.value_sub(self)
	}
}
