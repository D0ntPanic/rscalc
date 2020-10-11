use crate::decimal::DecimalLayout;
use crate::font::{Font, FontMetrics};
use crate::layout::{Layout, TokenType};
use crate::matrix::MatrixLayout;
use crate::number::NumberLayout;
use crate::string::StringLayout;
use crate::unit::CompositeUnitLayout;
use crate::vector::VectorLayout;
use num_bigint::ToBigInt;
use rscalc_math::format::{Format, FormatMode, MAX_SHORT_DISPLAY_BITS};
use rscalc_math::number::Number;
use rscalc_math::value::Value;

#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub enum AlternateLayoutType {
	None,
	Left,
	Bottom,
}

pub trait ValueLayout {
	fn layout(
		&self,
		format: &Format,
		base_font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Layout;

	fn single_line_numerical_layout(
		&self,
		format: &Format,
		int_font: Font,
		frac_font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
		sign_spacing: bool,
	) -> Option<Layout>;

	fn single_line_simple_layout(
		&self,
		format: &Format,
		font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Layout;

	fn double_line_simple_layout(
		&self,
		format: &Format,
		default_font: Font,
		small_font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Layout;

	fn alternate_hex_layout(
		&self,
		format: &Format,
		font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Option<Layout>;

	fn alternate_float_layout(
		&self,
		format: &Format,
		font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Option<Layout>;

	fn add_alternate_layout(
		&self,
		layout: Layout,
		format: &Format,
		font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
		alt_hex: bool,
		alt_float: bool,
	) -> (Layout, AlternateLayoutType);
}

impl ValueLayout for Value {
	fn layout(
		&self,
		format: &Format,
		base_font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Layout {
		let mut max_width = max_width;

		// Generate unit layout if there are units
		let mut unit_layout = match self {
			Value::NumberWithUnit(_, units) => units.layout(base_font),
			_ => None,
		};

		if let Some(layout) = &unit_layout {
			let width = layout.width(metrics);
			if width > max_width / 2 {
				// Units take up too much room, don't display them
				unit_layout = None;
			} else {
				// Reduce remaining maximum width by width of units
				max_width -= width;
			}
		}

		// Check full detailed layout of value to see if it is valid and fits within the max size
		match self {
			Value::Number(value) | Value::NumberWithUnit(value, _) => {
				// Real number, try to render full representation
				if let Some((layout, is_rational)) = value.double_line_layout(
					format,
					base_font,
					base_font.smaller(),
					metrics,
					max_width,
				) {
					// If units are present, add them to the layout
					let layout = if let Some(unit_layout) = unit_layout {
						let mut horizontal_items = Vec::new();
						horizontal_items.push(layout);
						horizontal_items.push(unit_layout);
						Layout::Horizontal(horizontal_items)
					} else {
						layout
					};

					// Check to see if alternate representations are available
					return self
						.add_alternate_layout(
							layout,
							format,
							base_font.smaller().smaller(),
							metrics,
							max_width,
							true,
							is_rational,
						)
						.0;
				}
			}
			Value::Complex(_) => {
				if let Some(layout) = self.single_line_numerical_layout(
					format,
					base_font,
					base_font.smaller(),
					metrics,
					max_width,
					true,
				) {
					// Layout fits. Check to see if floating point alternate
					// representation is enabled
					return self
						.add_alternate_layout(
							layout,
							&format,
							base_font.smaller().smaller(),
							metrics,
							max_width,
							false,
							true,
						)
						.0;
				}
			}
			Value::Vector(vector) => {
				// Vector, try to represent full form of vector entries in a single line. This is the
				// preferred form because it can show rationals.
				if let Some(layout) = vector.single_line_full_layout(
					format,
					base_font,
					base_font.smaller(),
					metrics,
					max_width,
				) {
					return layout;
				}

				// Try a three line layout with full precision decimal form
				if let Some(layout) =
					vector.multi_line_layout(format, base_font.smaller(), metrics, max_width, 3)
				{
					return layout;
				}

				// Try a three line layout with partial precision decimal form
				if let Some(layout) = vector.multi_line_layout(
					&format.with_max_precision(6),
					base_font.smaller(),
					metrics,
					max_width,
					3,
				) {
					return layout;
				}

				// Try a four line layout with smaller font
				if let Some(layout) = vector.multi_line_layout(
					&format.with_max_precision(6),
					base_font.smaller().smaller(),
					metrics,
					max_width,
					4,
				) {
					return layout;
				}
			}
			Value::Matrix(matrix) => {
				// Matrix, try to display all elements of a matrix of 4x4 or smaller.
				let largest_axis = core::cmp::max(matrix.rows(), matrix.cols());
				if largest_axis <= 4 {
					let mut font = if largest_axis == 1 {
						base_font
					} else if largest_axis <= 3 {
						base_font.smaller()
					} else {
						base_font.smaller().smaller()
					};

					loop {
						if let Some(layout) = matrix.layout(format, font, metrics, max_width) {
							return layout;
						}
						if font.is_smallest() {
							break;
						}
						font = font.smaller();
					}
				}
			}
			_ => (),
		}

		// Generate simple layout that will always fit
		let layout = self.double_line_simple_layout(
			format,
			base_font,
			base_font.smaller(),
			metrics,
			max_width,
		);

		// If units are present, add them to the layout
		if let Some(unit_layout) = unit_layout {
			let mut horizontal_items = Vec::new();
			horizontal_items.push(layout);
			horizontal_items.push(unit_layout);
			Layout::Horizontal(horizontal_items)
		} else {
			layout
		}
	}

	fn single_line_numerical_layout(
		&self,
		format: &Format,
		int_font: Font,
		frac_font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
		sign_spacing: bool,
	) -> Option<Layout> {
		match self {
			Value::Number(value) | Value::NumberWithUnit(value, _) => {
				value.single_line_layout(format, int_font, frac_font, metrics, max_width)
			}
			Value::Complex(value) => {
				// Complex number, try to render the full representation of both real and
				// imaginary parts.
				let format = format.decimal_format();
				if let Some(real_layout) = value
					.real_part()
					.single_line_layout(&format, int_font, frac_font, metrics, max_width)
				{
					let (sign_text, imaginary_part) = if sign_spacing {
						if value.imaginary_part().is_negative() {
							(" - ", Cow::Owned(-value.imaginary_part()))
						} else {
							(" + ", Cow::Borrowed(value.imaginary_part()))
						}
					} else {
						if value.imaginary_part().is_negative() {
							("-", Cow::Owned(-value.imaginary_part()))
						} else {
							("+", Cow::Borrowed(value.imaginary_part()))
						}
					};

					if let Some(imaginary_layout) = (&*imaginary_part)
						.single_line_layout(&format, int_font, frac_font, metrics, max_width)
					{
						// Both parts have a representation, construct final layout
						let mut horizontal_items = Vec::new();
						horizontal_items.push(real_layout);
						horizontal_items.push(Layout::StaticText(
							sign_text,
							int_font,
							TokenType::Complex,
						));
						horizontal_items.push(imaginary_layout);
						horizontal_items.push(Layout::StaticText(
							"ℹ",
							int_font,
							TokenType::Complex,
						));
						let layout = Layout::Horizontal(horizontal_items);
						if layout.width(metrics) <= max_width {
							return Some(layout);
						}
					}
				}

				// Try to render the floating point representation on a single line
				value.format(&format).single_line_layout(
					int_font,
					TokenType::Complex,
					metrics,
					max_width,
					None,
				)
			}
			_ => None,
		}
	}

	fn single_line_simple_layout(
		&self,
		format: &Format,
		font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Layout {
		match self {
			Value::Number(value) | Value::NumberWithUnit(value, _) => {
				// Render real numbers as a decimal of a precision that will fit
				value
					.to_decimal()
					.single_line_layout(format, "", "", font, metrics, max_width)
			}
			Value::Complex(value) => {
				// Render complex number as two lines, one with the decimal real part, and
				// one with the decimal imaginary part.
				let format = format.decimal_format();
				let (sign_text, imaginary_part) = if value.imaginary_part().is_negative() {
					("-", Cow::Owned(-value.imaginary_part()))
				} else {
					("+", Cow::Borrowed(value.imaginary_part()))
				};
				let real_layout = value.real_part().to_decimal().single_line_layout(
					&format,
					"",
					"",
					font,
					metrics,
					(max_width - metrics.width(font, sign_text)) / 2,
				);
				let imaginary_layout = imaginary_part.to_decimal().single_line_layout(
					&format,
					sign_text,
					"ℹ",
					font,
					metrics,
					(max_width - metrics.width(font, sign_text)) / 2,
				);

				let mut horizontal_layout_items = Vec::new();
				horizontal_layout_items.push(real_layout);
				horizontal_layout_items.push(imaginary_layout);
				Layout::Horizontal(horizontal_layout_items)
			}
			_ => {
				// Other type of value, just display as a string
				// TODO: Use truncatable rendering here so that it will never fail
				let string = self.to_string();
				if let Some(layout) =
					string.single_line_layout(font, TokenType::Object, metrics, max_width, None)
				{
					layout
				} else {
					Layout::StaticText("⟪Render error⟫", font, TokenType::Object)
				}
			}
		}
	}

	fn double_line_simple_layout(
		&self,
		format: &Format,
		default_font: Font,
		small_font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Layout {
		match self {
			Value::Number(value) | Value::NumberWithUnit(value, _) => {
				// Render real numbers as a decimal of a precision that will fit
				value.to_decimal().single_line_layout(
					format,
					"",
					"",
					default_font,
					metrics,
					max_width,
				)
			}
			Value::Complex(value) => {
				// Render complex number as two lines, one with the decimal real part, and
				// one with the decimal imaginary part.
				let format = format.decimal_format();
				let (sign_text, imaginary_part) = if value.imaginary_part().is_negative() {
					("- ", Cow::Owned(-value.imaginary_part()))
				} else {
					("+ ", Cow::Borrowed(value.imaginary_part()))
				};
				let real_layout = value
					.real_part()
					.to_decimal()
					.single_line_layout(&format, "", "", small_font, metrics, max_width);
				let imaginary_layout = imaginary_part
					.to_decimal()
					.single_line_layout(&format, sign_text, "ℹ", small_font, metrics, max_width);

				let mut vertical_layout_items = Vec::new();
				vertical_layout_items.push(real_layout);
				vertical_layout_items.push(imaginary_layout);
				Layout::Vertical(vertical_layout_items)
			}
			_ => {
				// Other type of value, just display as a string
				// TODO: Use truncatable rendering here so that it will never fail
				let string = self.to_string();
				if let Some(layout) = string.double_line_layout(
					default_font,
					small_font,
					TokenType::Object,
					metrics,
					max_width,
					None,
				) {
					layout
				} else {
					Layout::StaticText("⟪Render error⟫", default_font, TokenType::Object)
				}
			}
		}
	}

	fn alternate_hex_layout(
		&self,
		format: &Format,
		font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Option<Layout> {
		if max_width <= 0 {
			return None;
		}
		match self.real_number() {
			Ok(Number::Integer(int)) => {
				// Integer, if number is ten or greater check for the
				// hexadecimal alternate form
				if format.show_alt_hex
					&& (format.integer_radix != 10
						|| format.mode == FormatMode::Normal
						|| format.mode == FormatMode::Rational)
					&& (int <= &-10.to_bigint().unwrap()
						|| int >= &10.to_bigint().unwrap()
						|| int <= &(-(format.integer_radix as i8)).to_bigint().unwrap()
						|| int >= &(format.integer_radix as i8).to_bigint().unwrap())
					&& int.bits() <= MAX_SHORT_DISPLAY_BITS
				{
					// There is an alternate form to display, try to generate a single
					// line layout for it.
					let string = if format.integer_radix == 10 {
						self.format(&format.hex_format())
					} else {
						self.format(&format.decimal_format())
					};
					string.to_string().single_line_layout(
						font,
						TokenType::Integer,
						metrics,
						max_width,
						None,
					)
				} else {
					None
				}
			}
			_ => None,
		}
	}

	fn alternate_float_layout(
		&self,
		format: &Format,
		font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Option<Layout> {
		if max_width <= 0 {
			return None;
		}
		match self {
			Value::Number(Number::Rational(_, _))
			| Value::NumberWithUnit(Number::Rational(_, _), _) => {
				// Real number in rational form
				if format.show_alt_float && format.mode == FormatMode::Rational {
					if let Ok(number) = self.real_number() {
						let string = format.decimal_format().format_decimal(&number.to_decimal());
						string.single_line_layout(font, TokenType::Float, metrics, max_width, None)
					} else {
						None
					}
				} else {
					None
				}
			}
			Value::Complex(value) => {
				if format.show_alt_float
					&& format.mode == FormatMode::Rational
					&& (value.real_part().is_rational() || value.imaginary_part().is_rational())
				{
					// Complex number with at least one part in rational form
					let real_part = value.real_part().to_decimal();
					let imaginary_part = value.imaginary_part().to_decimal();
					let string = if imaginary_part.is_sign_negative() {
						format.with_max_precision(8).format_decimal(&real_part)
							+ " - " + &format
							.with_max_precision(8)
							.format_decimal(&-&*imaginary_part)
							+ "ℹ"
					} else {
						format.with_max_precision(8).format_decimal(&real_part)
							+ " + " + &format.with_max_precision(8).format_decimal(&imaginary_part)
							+ "ℹ"
					};
					string.single_line_layout(font, TokenType::Complex, metrics, max_width, None)
				} else {
					None
				}
			}
			_ => None,
		}
	}

	fn add_alternate_layout(
		&self,
		layout: Layout,
		format: &Format,
		font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
		alt_hex: bool,
		alt_float: bool,
	) -> (Layout, AlternateLayoutType) {
		let left_alt_width = max_width - (layout.width(metrics) + 24);
		if alt_hex {
			if format.alt_mode.left_enabled() {
				if let Some(alt_layout) =
					self.alternate_hex_layout(format, font, metrics, left_alt_width)
				{
					let mut alt_layout_items = Vec::new();
					alt_layout_items.push(Layout::LeftAlign(Box::new(alt_layout)));
					alt_layout_items.push(Layout::HorizontalSpace(24));
					alt_layout_items.push(layout);
					return (
						Layout::Horizontal(alt_layout_items),
						AlternateLayoutType::Left,
					);
				}
			}
			if format.alt_mode.bottom_enabled() {
				if let Some(alt_layout) =
					self.alternate_hex_layout(format, font, metrics, max_width)
				{
					let mut alt_layout_items = Vec::new();
					alt_layout_items.push(layout);
					alt_layout_items.push(alt_layout);
					return (
						Layout::Vertical(alt_layout_items),
						AlternateLayoutType::Bottom,
					);
				}
			}
		}

		if alt_float {
			if format.alt_mode.left_enabled() {
				if let Some(alt_layout) =
					self.alternate_float_layout(format, font, metrics, left_alt_width)
				{
					let mut alt_layout_items = Vec::new();
					alt_layout_items.push(Layout::LeftAlign(Box::new(alt_layout)));
					alt_layout_items.push(Layout::HorizontalSpace(24));
					alt_layout_items.push(layout);
					return (
						Layout::Horizontal(alt_layout_items),
						AlternateLayoutType::Left,
					);
				}
			}
			if format.alt_mode.bottom_enabled() {
				if let Some(alt_layout) =
					self.alternate_float_layout(format, font, metrics, max_width)
				{
					let mut alt_layout_items = Vec::new();
					alt_layout_items.push(layout);
					alt_layout_items.push(alt_layout);
					return (
						Layout::Vertical(alt_layout_items),
						AlternateLayoutType::Bottom,
					);
				}
			}
		}

		(layout, AlternateLayoutType::None)
	}
}
