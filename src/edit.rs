use crate::error::{Error, Result};
use crate::number::{Number, NumberDecimalPointMode, NumberFormat};
use alloc::string::String;
use alloc::vec::Vec;
use intel_dfp::Decimal;
use num_bigint::{BigInt, ToBigInt};

const MAX_FRACTION_DIGITS: usize = 34;
const MAX_EXPONENT: i32 = 9999;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NumberEditorState {
	Integer,
	Fraction,
	Exponent,
}

pub struct NumberEditor {
	sign: bool,
	integer: BigInt,
	fraction_digits: Vec<u8>,
	exponent_sign: bool,
	exponent: Option<i32>,
	radix: u8,
	state: NumberEditorState,
}

impl NumberEditor {
	pub fn new(format: &NumberFormat) -> Self {
		NumberEditor {
			sign: false,
			integer: 0.into(),
			fraction_digits: Vec::new(),
			exponent_sign: false,
			exponent: None,
			radix: format.integer_radix,
			state: NumberEditorState::Integer,
		}
	}

	fn push_digit(&mut self, digit: u8) -> Result<()> {
		if digit >= self.radix {
			return Err(Error::InvalidEntry);
		}
		match self.state {
			NumberEditorState::Integer => {
				self.integer *= self.radix.to_bigint().unwrap();
				self.integer += digit;
			}
			NumberEditorState::Fraction => {
				if self.fraction_digits.len() < MAX_FRACTION_DIGITS {
					self.fraction_digits.push(digit);
				}
			}
			NumberEditorState::Exponent => {
				let new_exponent = match self.exponent {
					Some(exponent) => (exponent * 10) + digit as i32,
					None => digit as i32,
				};
				if new_exponent <= MAX_EXPONENT {
					self.exponent = Some(new_exponent);
				}
			}
		}
		Ok(())
	}

	pub fn push_char(&mut self, ch: char) -> Result<()> {
		match ch {
			'0'..='9' => self.push_digit(ch as u32 as u8 - '0' as u32 as u8),
			'A'..='Z' => self.push_digit(ch as u32 as u8 - 'A' as u32 as u8 + 10),
			'a'..='z' => self.push_digit(ch as u32 as u8 - 'a' as u32 as u8 + 10),
			'.' => {
				if self.state == NumberEditorState::Integer && self.radix == 10 {
					self.state = NumberEditorState::Fraction;
					Ok(())
				} else {
					Err(Error::InvalidEntry)
				}
			}
			_ => Err(Error::InvalidEntry),
		}
	}

	pub fn exponent(&mut self) {
		if self.state != NumberEditorState::Exponent && self.radix == 10 {
			self.state = NumberEditorState::Exponent;
		}
	}

	pub fn neg(&mut self) {
		match self.state {
			NumberEditorState::Integer | NumberEditorState::Fraction => {
				self.sign = !self.sign;
			}
			NumberEditorState::Exponent => {
				self.exponent_sign = !self.exponent_sign;
			}
		}
	}

	pub fn backspace(&mut self) -> bool {
		match self.state {
			NumberEditorState::Integer => {
				self.integer /= self.radix.to_bigint().unwrap();
				if self.integer == 0.to_bigint().unwrap() {
					return false;
				}
			}
			NumberEditorState::Fraction => {
				if self.fraction_digits.len() == 0 {
					self.state = NumberEditorState::Integer;
				} else {
					self.fraction_digits.pop();
				}
			}
			NumberEditorState::Exponent => {
				if let Some(exponent) = self.exponent {
					let new_exponent = exponent / 10;
					if new_exponent == 0 {
						self.exponent = None;
					} else {
						self.exponent = Some(new_exponent);
					}
				} else if self.fraction_digits.len() == 0 {
					self.state = NumberEditorState::Integer;
				} else {
					self.state = NumberEditorState::Fraction;
				}
			}
		}
		true
	}

	pub fn to_str(&self, format: &NumberFormat) -> String {
		let mut result = String::new();
		if self.sign {
			result += "-";
		}
		result += format.format_bigint(&self.integer).as_str();
		if self.state != NumberEditorState::Integer {
			result += match format.decimal_point {
				NumberDecimalPointMode::Period => ".",
				NumberDecimalPointMode::Comma => ",",
			};
			let mut decimal_chars = Vec::new();
			for digit in &self.fraction_digits {
				decimal_chars.push(digit + '0' as u32 as u8);
			}
			result += String::from_utf8(decimal_chars).unwrap().as_str();
		}
		if self.state == NumberEditorState::Exponent {
			result += "ᴇ";
			if self.exponent_sign {
				result += "-";
			}
			if let Some(exponent) = self.exponent {
				result += format
					.exponent_format()
					.format_bigint(&exponent.to_bigint().unwrap())
					.as_str();
			}
		}

		result
	}

	pub fn number(&self) -> Number {
		if self.state == NumberEditorState::Integer {
			if self.sign {
				return Number::Integer(-self.integer.clone());
			} else {
				return Number::Integer(self.integer.clone());
			}
		}

		let mut result = Number::bigint_to_decimal(&self.integer);

		let one: Decimal = 1.into();
		let ten: Decimal = 10.into();
		let mut factor = &one / &ten;
		for digit in &self.fraction_digits {
			let digit: Decimal = (*digit as u32).into();
			result += &digit * &factor;
			factor = &factor / &ten;
		}

		let exponent: Decimal = match self.exponent {
			Some(exponent) => {
				if self.exponent_sign {
					-exponent
				} else {
					exponent
				}
			}
			None => 0,
		}
		.into();

		result *= exponent.exp10();
		if self.sign {
			Number::Decimal(-result)
		} else {
			Number::Decimal(result)
		}
	}
}
