use crate::edit::NumberEditor;
use crate::error::{Error, Result};
use crate::font::{SANS_13, SANS_16, SANS_20, SANS_24};
use crate::layout::Layout;
use crate::number::{Number, NumberFormat, NumberFormatMode, ToNumber, MAX_SHORT_DISPLAY_BITS};
use crate::screen::Color;
use crate::time::{SimpleDateTimeFormat, SimpleDateTimeToString};
use crate::unit::{CompositeUnit, TimeUnit, Unit};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use core::convert::TryFrom;
use core::ops::Add;
use num_bigint::{BigInt, ToBigInt};

#[derive(Clone)]
pub enum Value {
	Number(Number),
	NumberWithUnit(Number, CompositeUnit),
	DateTime(NaiveDateTime),
	Date(NaiveDate),
	Time(NaiveTime),
}

impl Value {
	pub fn is_numeric(&self) -> bool {
		match self {
			Value::Number(_) => true,
			Value::NumberWithUnit(_, _) => true,
			Value::DateTime(_) | Value::Date(_) | Value::Time(_) => false,
		}
	}

	pub fn number(&self) -> Result<&Number> {
		match self {
			Value::Number(num) => Ok(num),
			Value::NumberWithUnit(num, _) => Ok(num),
			Value::DateTime(_) | Value::Date(_) | Value::Time(_) => Err(Error::NotANumber),
		}
	}

	pub fn to_int(&self) -> Result<BigInt> {
		match self {
			Value::Number(num) => num.to_int(),
			Value::NumberWithUnit(num, _) => num.to_int(),
			Value::DateTime(_) | Value::Date(_) | Value::Time(_) => Err(Error::NotANumber),
		}
	}

	pub fn to_str(&self) -> String {
		match self {
			Value::Number(num) => num.to_str(),
			Value::NumberWithUnit(num, _) => num.to_str(),
			Value::DateTime(dt) => dt.to_str(&SimpleDateTimeFormat::full()),
			Value::Date(date) => date.to_str(&SimpleDateTimeFormat::date()),
			Value::Time(time) => time.to_str(&SimpleDateTimeFormat::time()),
		}
	}

	pub fn format(&self, format: &NumberFormat) -> String {
		match self {
			Value::Number(num) => format.format_number(num),
			Value::NumberWithUnit(num, _) => format.format_number(num),
			Value::DateTime(_) | Value::Date(_) | Value::Time(_) => self.to_str(),
		}
	}

	pub fn pow(&self, power: &Value) -> Result<Value> {
		Ok(Value::Number(self.number()?.pow(power.number()?)))
	}

	pub fn sqrt(&self) -> Result<Value> {
		Ok(Value::Number(self.number()?.sqrt()))
	}

	pub fn log(&self) -> Result<Value> {
		Ok(Value::Number(self.number()?.log()))
	}

	pub fn exp10(&self) -> Result<Value> {
		Ok(Value::Number(self.number()?.exp10()))
	}

	pub fn ln(&self) -> Result<Value> {
		Ok(Value::Number(self.number()?.ln()))
	}

	pub fn exp(&self) -> Result<Value> {
		Ok(Value::Number(self.number()?.exp()))
	}

	pub fn sin(&self) -> Result<Value> {
		Ok(Value::Number(self.number()?.sin()))
	}

	pub fn cos(&self) -> Result<Value> {
		Ok(Value::Number(self.number()?.cos()))
	}

	pub fn tan(&self) -> Result<Value> {
		Ok(Value::Number(self.number()?.tan()))
	}

	pub fn asin(&self) -> Result<Value> {
		Ok(Value::Number(self.number()?.asin()))
	}

	pub fn acos(&self) -> Result<Value> {
		Ok(Value::Number(self.number()?.acos()))
	}

	pub fn atan(&self) -> Result<Value> {
		Ok(Value::Number(self.number()?.atan()))
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
			Value::DateTime(_) | Value::Date(_) | Value::Time(_) => Err(Error::NotANumber),
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
			Value::DateTime(_) | Value::Date(_) | Value::Time(_) => Err(Error::NotANumber),
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
			Value::DateTime(_) | Value::Date(_) | Value::Time(_) => Err(Error::NotANumber),
		}
	}

	fn datetime_add_secs(&self, dt: &NaiveDateTime, secs: &Number) -> Result<Value> {
		let nano = i64::try_from((secs * &1_000_000_000.to_number()).to_int()?)?;
		Ok(Value::DateTime(dt.add(Duration::nanoseconds(nano))))
	}

	fn date_add_days(&self, date: &NaiveDate, days: &Number) -> Result<Value> {
		Ok(Value::Date(
			date.add(Duration::days(i64::try_from(days.to_int()?)?)),
		))
	}

	fn time_add_secs(&self, time: &NaiveTime, secs: &Number) -> Result<Value> {
		let nano = i64::try_from((secs * &1_000_000_000.to_number()).to_int()?)?;
		Ok(Value::Time(time.add(Duration::nanoseconds(nano))))
	}

	fn value_add(&self, rhs: &Value) -> Result<Value> {
		match self {
			Value::Number(left) => match rhs {
				Value::Number(right) => Ok(Value::Number(left + right)),
				Value::NumberWithUnit(right, right_unit) => {
					Ok(Value::NumberWithUnit(left + right, right_unit.clone()))
				}
				Value::DateTime(right) => self.datetime_add_secs(right, left),
				Value::Date(right) => self.date_add_days(right, left),
				Value::Time(right) => self.time_add_secs(right, left),
			},
			Value::NumberWithUnit(left, left_unit) => match rhs {
				Value::Number(right) => Ok(Value::NumberWithUnit(left + right, left_unit.clone())),
				Value::NumberWithUnit(right, right_unit) => Ok(Value::NumberWithUnit(
					&left_unit.coerce_to_other(left, right_unit)? + right,
					right_unit.clone(),
				)),
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
				Value::Date(right) => Ok(Value::DateTime(NaiveDateTime::new(
					right.clone(),
					left.clone(),
				))),
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
				Value::DateTime(_) | Value::Date(_) | Value::Time(_) => {
					Err(Error::DataTypeMismatch)
				}
			},
			Value::NumberWithUnit(left, left_unit) => match rhs {
				Value::Number(right) => Ok(Value::NumberWithUnit(left - right, left_unit.clone())),
				Value::NumberWithUnit(right, right_unit) => Ok(Value::NumberWithUnit(
					&left_unit.coerce_to_other(left, right_unit)? - right,
					right_unit.clone(),
				)),
				Value::DateTime(_) | Value::Date(_) | Value::Time(_) => {
					Err(Error::DataTypeMismatch)
				}
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
		}
	}

	fn value_mul(&self, rhs: &Value) -> Result<Value> {
		match self {
			Value::Number(left) => match rhs {
				Value::Number(right) => Ok(Value::Number(left * right)),
				Value::NumberWithUnit(right, right_unit) => {
					Ok(Value::NumberWithUnit(left * right, right_unit.clone()))
				}
				Value::DateTime(_) | Value::Date(_) | Value::Time(_) => {
					Err(Error::DataTypeMismatch)
				}
			},
			Value::NumberWithUnit(left, left_unit) => match rhs {
				Value::Number(right) => Ok(Value::NumberWithUnit(left * right, left_unit.clone())),
				Value::NumberWithUnit(right, right_unit) => {
					let mut unit = left_unit.clone();
					let left = unit.combine(left, right_unit);
					Ok(Value::NumberWithUnit(&left * right, unit))
				}
				Value::DateTime(_) | Value::Date(_) | Value::Time(_) => {
					Err(Error::DataTypeMismatch)
				}
			},
			Value::DateTime(_) | Value::Date(_) | Value::Time(_) => Err(Error::DataTypeMismatch),
		}
	}

	fn value_div(&self, rhs: &Value) -> Result<Value> {
		match self {
			Value::Number(left) => match rhs {
				Value::Number(right) => Ok(Value::Number(left / right)),
				Value::NumberWithUnit(right, right_unit) => {
					Ok(Value::NumberWithUnit(left / right, right_unit.inverse()))
				}
				Value::DateTime(_) | Value::Date(_) | Value::Time(_) => {
					Err(Error::DataTypeMismatch)
				}
			},
			Value::NumberWithUnit(left, left_unit) => match rhs {
				Value::Number(right) => Ok(Value::NumberWithUnit(left / right, left_unit.clone())),
				Value::NumberWithUnit(right, right_unit) => {
					let mut unit = left_unit.clone();
					let left = unit.combine(left, &right_unit.inverse());
					Ok(Value::NumberWithUnit(&left / right, unit))
				}
				Value::DateTime(_) | Value::Date(_) | Value::Time(_) => {
					Err(Error::DataTypeMismatch)
				}
			},
			Value::DateTime(_) | Value::Date(_) | Value::Time(_) => Err(Error::DataTypeMismatch),
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
							denom_layout.push(Layout::Text(
								"∙".to_string(),
								&SANS_20,
								Color::ContentText,
							));
							denom_only_layout.push(Layout::Text(
								"∙".to_string(),
								&SANS_24,
								Color::ContentText,
							));
						}

						// Create layout in denomator of a fraction
						let unit_text = Layout::Text(unit.0.to_str(), &SANS_20, Color::ContentText);
						let layout = if unit.1 < -1 {
							Layout::Power(
								Box::new(unit_text),
								Box::new(Layout::Text(
									(-unit.1).to_number().to_str(),
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
							Box::new(Layout::Text(unit.0.to_str(), &SANS_24, Color::ContentText)),
							Box::new(Layout::Text(
								unit.1.to_number().to_str(),
								&SANS_16,
								Color::ContentText,
							)),
						));
					} else if unit.1 > 0 {
						// Power is positive, unit is in numerator
						if numer_layout.len() != 0 {
							// Add multiplication symbol to separate unit names
							numer_layout.push(Layout::Text(
								"∙".to_string(),
								&SANS_20,
								Color::ContentText,
							));
							numer_only_layout.push(Layout::Text(
								"∙".to_string(),
								&SANS_24,
								Color::ContentText,
							));
						}

						// Create layout in numerator of a fraction
						let unit_text = Layout::Text(unit.0.to_str(), &SANS_20, Color::ContentText);
						let layout = if unit.1 > 1 {
							Layout::Power(
								Box::new(unit_text),
								Box::new(Layout::Text(
									unit.1.to_number().to_str(),
									&SANS_13,
									Color::ContentText,
								)),
							)
						} else {
							unit_text
						};
						numer_layout.push(layout);

						// Create layout if there is no denominator
						let unit_text = Layout::Text(unit.0.to_str(), &SANS_24, Color::ContentText);
						let layout = if unit.1 > 1 {
							Layout::Power(
								Box::new(unit_text),
								Box::new(Layout::Text(
									unit.1.to_number().to_str(),
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
					numer_only_layout.insert(
						0,
						Layout::Text(" ".to_string(), &SANS_24, Color::ContentText),
					);
					Some(Layout::Horizontal(numer_only_layout))
				} else if numer_layout.len() == 0 {
					// Denominator only
					denom_only_layout.insert(
						0,
						Layout::Text(" ".to_string(), &SANS_24, Color::ContentText),
					);
					Some(Layout::Horizontal(denom_only_layout))
				} else {
					// Fraction
					let mut final_layout = Vec::new();
					final_layout.push(Layout::Text(" ".to_string(), &SANS_24, Color::ContentText));
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

	pub fn render(
		&self,
		format: &NumberFormat,
		editor: &Option<NumberEditor>,
		max_width: i32,
	) -> Layout {
		let mut max_width = max_width;

		// Get string for number. If there is an editor, use editor state instead.
		let string = match editor {
			Some(editor) => editor.to_str(format),
			None => self.format(&format),
		};

		// Check for alternate representation strings
		let mut alt_string = match self.number() {
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
			Ok(Number::Rational(_, _)) => {
				// Rational, show floating point as alternate form if enabled
				if format.show_alt_float && format.mode == NumberFormatMode::Rational {
					if let Ok(number) = self.number() {
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

		// If alternate representation is the same as normal representation, don't display it
		if let Some(alt) = &alt_string {
			if alt == &string {
				alt_string = None;
			}
		}

		// If alternate representation is too wide, don't display it
		if let Some(alt) = &alt_string {
			let width = SANS_16.width(alt) + 4;
			if width > max_width {
				alt_string = None;
			}
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
			if let Ok(Number::Rational(num, denom)) = self.number() {
				// Check to see if rational number has too much precision to display here
				if num.bits() <= MAX_SHORT_DISPLAY_BITS && denom.bits() <= MAX_SHORT_DISPLAY_BITS {
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
					rational_horizontal_items.push(Layout::Text(
						int_str,
						&SANS_24,
						Color::ContentText,
					));
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
				if let Ok(number) = self.number() {
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
					if let Ok(number) = self.number() {
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

		// Add units to layout
		if let Some(unit_layout) = unit_layout {
			let mut items = Vec::new();
			items.push(layout);
			items.push(unit_layout);
			layout = Layout::Horizontal(items);
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
		Value::Number(0.into()).value_sub(&self)
	}
}

impl core::ops::Neg for &Value {
	type Output = Result<Value>;

	fn neg(self) -> Self::Output {
		Value::Number(0.into()).value_sub(self)
	}
}
