use crate::font::{Font, FontMetrics};
use crate::layout::{Layout, TokenType};
use crate::string::StringLayout;
use num_bigint::{BigInt, BigUint, ToBigInt};
use rscalc_math::format::{Format, FormatMode, FormatResult, MAX_SHORT_DISPLAY_BITS};
use rscalc_math::number::Number;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub trait NumberFormatResultToToken {
	fn token_type(&self) -> TokenType;
}

impl NumberFormatResultToToken for FormatResult {
	fn token_type(&self) -> TokenType {
		match self {
			FormatResult::Integer(_) => TokenType::Integer,
			FormatResult::Float(_) => TokenType::Float,
			FormatResult::Complex(_) => TokenType::Complex,
			FormatResult::Object(_) => TokenType::Object,
		}
	}
}

fn rational_layout(
	num: &BigInt,
	denom: &BigUint,
	format: &Format,
	int_font: Font,
	frac_font: Font,
	metrics: &dyn FontMetrics,
	max_width: i32,
) -> Option<Layout> {
	// Check to see if rational number has too much precision to display
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
		rational_horizontal_items.push(Layout::Text(int_str, int_font, TokenType::Integer));
		rational_horizontal_items.push(Layout::HorizontalSpace(4));
		rational_horizontal_items.push(Layout::Fraction(
			Box::new(Layout::Text(num_str, frac_font, TokenType::Integer)),
			Box::new(Layout::Text(denom_str, frac_font, TokenType::Integer)),
			TokenType::Integer,
		));
		let layout = Layout::Horizontal(rational_horizontal_items);
		if layout.width(metrics) <= max_width {
			Some(layout)
		} else {
			None
		}
	} else {
		None
	}
}

pub trait NumberLayout {
	fn single_line_layout(
		&self,
		format: &Format,
		default_font: Font,
		small_font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Option<Layout>;

	fn double_line_layout(
		&self,
		format: &Format,
		default_font: Font,
		small_font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Option<(Layout, bool)>;
}

impl NumberLayout for Number {
	fn single_line_layout(
		&self,
		format: &Format,
		default_font: Font,
		small_font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Option<Layout> {
		if let Number::Rational(num, denom) = self {
			if format.mode == FormatMode::Rational {
				// Rational number, try to lay out as a fraction
				if let Some(layout) = rational_layout(
					num,
					denom,
					format,
					default_font,
					small_font,
					metrics,
					max_width,
				) {
					return Some(layout);
				}
			}
		}

		// Render full string of value and see if it fits
		let format_result = format.format_number(self);
		let token_type = format_result.token_type();
		format_result.to_string().single_line_layout(
			default_font,
			token_type,
			metrics,
			max_width,
			None,
		)
	}

	fn double_line_layout(
		&self,
		format: &Format,
		default_font: Font,
		small_font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Option<(Layout, bool)> {
		if let Number::Rational(num, denom) = self {
			if format.mode == FormatMode::Rational {
				// Rational number, try to lay out as a fraction
				if let Some(layout) = rational_layout(
					num,
					denom,
					format,
					default_font,
					small_font,
					metrics,
					max_width,
				) {
					return Some((layout, true));
				}
			}
		}

		// Render full string of value and see if it fits
		let format_result = format.format_number(self);
		let token_type = format_result.token_type();
		if let Some(layout) = format_result.to_string().double_line_layout(
			default_font,
			small_font,
			token_type,
			metrics,
			max_width,
			None,
		) {
			Some((layout, false))
		} else {
			None
		}
	}
}
