use crate::number::{Number, NumberFormat, NumberFormatMode, MAX_SHORT_DISPLAY_BITS};
use crate::screen::{Color, Font, Rect, Screen};
use crate::value::Value;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use intel_dfp::Decimal;
use num_bigint::{BigInt, BigUint, ToBigInt};

#[derive(Clone)]
pub enum Layout {
	Text(String, &'static Font, Color),
	StaticText(&'static str, &'static Font, Color),
	EditText(String, &'static Font, Color),
	Horizontal(Vec<Layout>),
	Vertical(Vec<Layout>),
	Fraction(Box<Layout>, Box<Layout>, Color),
	Power(Box<Layout>, Box<Layout>),
	HorizontalSpace(i32),
	VerticalSpace(i32),
	HorizontalRule,
	LeftAlign(Box<Layout>),
	UsageGraph(usize, usize, usize),
	UsageGraphUsedLegend,
	UsageGraphReclaimableLegend,
	UsageGraphFreeLegend,
	LeftMatrixBracket,
	RightMatrixBracket,
}

impl Layout {
	pub fn single_line_string_layout(
		string: &str,
		font: &'static Font,
		color: Color,
		max_width: i32,
		editor: bool,
	) -> Option<Self> {
		if font.width(string) <= max_width {
			Some(Layout::editable_text(
				string.to_string(),
				font,
				color,
				editor,
			))
		} else {
			None
		}
	}

	pub fn double_line_string_layout(
		string: &str,
		default_font: &'static Font,
		small_font: &'static Font,
		color: Color,
		max_width: i32,
		editor: bool,
	) -> Option<Self> {
		// Check width to see if it is possible to fit within two lines
		if small_font.width(string) > max_width * 2 {
			return None;
		}

		// Check layout width for a single line in the normal font
		if let Some(layout) =
			Self::single_line_string_layout(string, default_font, color, max_width, editor)
		{
			return Some(layout);
		}

		// Check layout width for a single line in the smaller font
		if let Some(layout) =
			Self::single_line_string_layout(string, small_font, color, max_width, editor)
		{
			return Some(layout);
		}

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
				width += small_font.width(&char_str);
			} else {
				width += small_font.advance(&char_str);
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
		layout_items.push(Layout::Text(first_str, &small_font, Color::ContentText));
		layout_items.push(Layout::editable_text(
			second_str,
			&small_font,
			Color::ContentText,
			editor,
		));
		let split_layout = Layout::Vertical(layout_items);
		if split_layout.width() <= max_width {
			Some(split_layout)
		} else {
			None
		}
	}

	pub fn single_line_decimal_layout(
		value: &Decimal,
		format: &NumberFormat,
		prefix: &str,
		suffix: &str,
		font: &'static Font,
		color: Color,
		max_width: i32,
	) -> Layout {
		let mut format = format.clone();
		loop {
			// Try to format the string and see if it fits
			let string = prefix.to_string() + &format.format_decimal(value) + suffix;
			if font.width(&string) <= max_width {
				// This string fits, return final layout
				return Layout::Text(string, font, color);
			}

			// Try a reduced precision. If the precision is already 3 or less, just use the
			// existing string.
			if format.precision <= 3 {
				return Layout::Text(string, font, color);
			}
			format = format.with_max_precision(format.precision - 1);
		}
	}

	pub fn rational_layout(
		num: &BigInt,
		denom: &BigUint,
		format: &NumberFormat,
		int_font: &'static Font,
		frac_font: &'static Font,
		color: Color,
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
			rational_horizontal_items.push(Layout::Text(int_str, int_font, color));
			rational_horizontal_items.push(Layout::HorizontalSpace(4));
			rational_horizontal_items.push(Layout::Fraction(
				Box::new(Layout::Text(num_str, frac_font, color)),
				Box::new(Layout::Text(denom_str, frac_font, color)),
				color,
			));
			let layout = Layout::Horizontal(rational_horizontal_items);
			if layout.width() <= max_width {
				Some(layout)
			} else {
				None
			}
		} else {
			None
		}
	}

	pub fn single_line_number_layout(
		value: &Number,
		format: &NumberFormat,
		default_font: &'static Font,
		small_font: &'static Font,
		color: Color,
		max_width: i32,
	) -> Option<Layout> {
		if let Number::Rational(num, denom) = value {
			if format.mode == NumberFormatMode::Rational {
				// Rational number, try to lay out as a fraction
				if let Some(layout) = Layout::rational_layout(
					num,
					denom,
					format,
					default_font,
					small_font,
					color,
					max_width,
				) {
					return Some(layout);
				}
			}
		}

		// Render full string of value and see if it fits
		let string = format.format_number(value);
		Layout::single_line_string_layout(&string, default_font, color, max_width, false)
	}

	pub fn double_line_number_layout(
		value: &Number,
		format: &NumberFormat,
		default_font: &'static Font,
		small_font: &'static Font,
		color: Color,
		max_width: i32,
	) -> Option<(Layout, bool)> {
		if let Number::Rational(num, denom) = value {
			if format.mode == NumberFormatMode::Rational {
				// Rational number, try to lay out as a fraction
				if let Some(layout) = Layout::rational_layout(
					num,
					denom,
					format,
					default_font,
					small_font,
					color,
					max_width,
				) {
					return Some((layout, true));
				}
			}
		}

		// Render full string of value and see if it fits
		let string = format.format_number(value);
		if let Some(layout) = Layout::double_line_string_layout(
			&string,
			default_font,
			small_font,
			color,
			max_width,
			false,
		) {
			Some((layout, false))
		} else {
			None
		}
	}

	pub fn single_line_numerical_value_layout(
		value: &Value,
		format: &NumberFormat,
		int_font: &'static Font,
		frac_font: &'static Font,
		max_width: i32,
		sign_spacing: bool,
	) -> Option<Layout> {
		match value {
			Value::Number(value) | Value::NumberWithUnit(value, _) => {
				Layout::single_line_number_layout(
					value,
					format,
					int_font,
					frac_font,
					Color::ContentText,
					max_width,
				)
			}
			Value::Complex(value) => {
				// Complex number, try to render the full representation of both real and
				// imaginary parts.
				let format = format.decimal_format();
				if let Some(real_layout) = Layout::single_line_number_layout(
					value.real_part(),
					&format,
					int_font,
					frac_font,
					Color::ContentText,
					max_width,
				) {
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

					if let Some(imaginary_layout) = Layout::single_line_number_layout(
						&*imaginary_part,
						&format,
						int_font,
						frac_font,
						Color::ContentText,
						max_width,
					) {
						// Both parts have a representation, construct final layout
						let mut horizontal_items = Vec::new();
						horizontal_items.push(real_layout);
						horizontal_items.push(Layout::StaticText(
							sign_text,
							int_font,
							Color::ContentText,
						));
						horizontal_items.push(imaginary_layout);
						horizontal_items.push(Layout::StaticText(
							"ℹ",
							int_font,
							Color::ContentText,
						));
						let layout = Layout::Horizontal(horizontal_items);
						if layout.width() <= max_width {
							return Some(layout);
						}
					}
				}

				// Try to render the floating point representation on a single line
				let string = value.format(&format);
				Layout::single_line_string_layout(
					&string,
					int_font,
					Color::ContentText,
					max_width,
					false,
				)
			}
			_ => None,
		}
	}

	pub fn single_line_simple_value_layout(
		value: &Value,
		format: &NumberFormat,
		font: &'static Font,
		max_width: i32,
	) -> Layout {
		match value {
			Value::Number(value) | Value::NumberWithUnit(value, _) => {
				// Render real numbers as a decimal of a precision that will fit
				Layout::single_line_decimal_layout(
					&value.to_decimal(),
					format,
					"",
					"",
					font,
					Color::ContentText,
					max_width,
				)
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
				let real_layout = Layout::single_line_decimal_layout(
					&value.real_part().to_decimal(),
					&format,
					"",
					"",
					font,
					Color::ContentText,
					(max_width - font.width(sign_text)) / 2,
				);
				let imaginary_layout = Layout::single_line_decimal_layout(
					&imaginary_part.to_decimal(),
					&format,
					sign_text,
					"ℹ",
					font,
					Color::ContentText,
					(max_width - font.width(sign_text)) / 2,
				);

				let mut horizontal_layout_items = Vec::new();
				horizontal_layout_items.push(real_layout);
				horizontal_layout_items.push(imaginary_layout);
				Layout::Horizontal(horizontal_layout_items)
			}
			_ => {
				// Other type of value, just display as a string
				// TODO: Use truncatable rendering here so that it will never fail
				let string = value.to_string();
				if let Some(layout) = Layout::single_line_string_layout(
					&string,
					font,
					Color::ContentText,
					max_width,
					false,
				) {
					layout
				} else {
					Layout::StaticText("⟪Render error⟫", font, Color::ContentText)
				}
			}
		}
	}

	pub fn double_line_simple_value_layout(
		value: &Value,
		format: &NumberFormat,
		default_font: &'static Font,
		small_font: &'static Font,
		max_width: i32,
	) -> Layout {
		match value {
			Value::Number(value) | Value::NumberWithUnit(value, _) => {
				// Render real numbers as a decimal of a precision that will fit
				Layout::single_line_decimal_layout(
					&value.to_decimal(),
					format,
					"",
					"",
					default_font,
					Color::ContentText,
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
				let real_layout = Layout::single_line_decimal_layout(
					&value.real_part().to_decimal(),
					&format,
					"",
					"",
					small_font,
					Color::ContentText,
					max_width,
				);
				let imaginary_layout = Layout::single_line_decimal_layout(
					&imaginary_part.to_decimal(),
					&format,
					sign_text,
					"ℹ",
					small_font,
					Color::ContentText,
					max_width,
				);

				let mut vertical_layout_items = Vec::new();
				vertical_layout_items.push(real_layout);
				vertical_layout_items.push(imaginary_layout);
				Layout::Vertical(vertical_layout_items)
			}
			_ => {
				// Other type of value, just display as a string
				// TODO: Use truncatable rendering here so that it will never fail
				let string = value.to_string();
				if let Some(layout) = Layout::double_line_string_layout(
					&string,
					default_font,
					small_font,
					Color::ContentText,
					max_width,
					false,
				) {
					layout
				} else {
					Layout::StaticText("⟪Render error⟫", default_font, Color::ContentText)
				}
			}
		}
	}

	pub fn editable_text(string: String, font: &'static Font, color: Color, editor: bool) -> Self {
		if editor {
			Layout::EditText(string, font, color)
		} else {
			Layout::Text(string, font, color)
		}
	}

	pub fn full_width(&self) -> bool {
		match self {
			Layout::LeftAlign(_) | Layout::HorizontalRule | Layout::UsageGraph(_, _, _) => true,
			Layout::Vertical(items) => {
				for item in items {
					if item.full_width() {
						return true;
					}
				}
				false
			}
			_ => false,
		}
	}

	pub fn full_height(&self) -> bool {
		match self {
			Layout::LeftMatrixBracket | Layout::RightMatrixBracket => true,
			_ => false,
		}
	}

	pub fn width(&self) -> i32 {
		match self {
			Layout::Text(string, font, _) => font.width(string),
			Layout::StaticText(string, font, _) => font.width(string),
			Layout::EditText(string, font, _) => font.width(string),
			Layout::Horizontal(items) => items.iter().fold(0, |width, item| width + item.width()),
			Layout::Vertical(items) => items
				.iter()
				.fold(0, |width, item| core::cmp::max(width, item.width())),
			Layout::Fraction(numer, denom, _) => core::cmp::max(numer.width(), denom.width()),
			Layout::Power(base, power) => base.width() + power.width(),
			Layout::HorizontalSpace(width) => *width,
			Layout::VerticalSpace(_) => 0,
			Layout::HorizontalRule => 0,
			Layout::LeftAlign(item) => item.width(),
			Layout::UsageGraph(_, _, _) => 0,
			Layout::UsageGraphUsedLegend
			| Layout::UsageGraphReclaimableLegend
			| Layout::UsageGraphFreeLegend => 11,
			Layout::LeftMatrixBracket | Layout::RightMatrixBracket => 10,
		}
	}

	pub fn height(&self) -> i32 {
		match self {
			Layout::Text(_, font, _) => font.height,
			Layout::StaticText(_, font, _) => font.height,
			Layout::EditText(_, font, _) => font.height,
			Layout::Horizontal(items) => items
				.iter()
				.fold(0, |height, item| core::cmp::max(height, item.height())),
			Layout::Vertical(items) => items.iter().fold(0, |height, item| height + item.height()),
			Layout::Fraction(numer, denom, _) => numer.height() + denom.height(),
			Layout::Power(base, power) => core::cmp::max(base.height(), power.height()),
			Layout::HorizontalSpace(_) => 0,
			Layout::VerticalSpace(height) => *height,
			Layout::HorizontalRule => 1,
			Layout::LeftAlign(item) => item.height(),
			Layout::UsageGraph(_, _, _) => 31,
			Layout::UsageGraphUsedLegend
			| Layout::UsageGraphReclaimableLegend
			| Layout::UsageGraphFreeLegend => 11,
			Layout::LeftMatrixBracket | Layout::RightMatrixBracket => 16,
		}
	}

	fn overridden_color(color: &Color, color_override: &Option<Color>) -> Color {
		if let Some(color_override) = color_override {
			*color_override
		} else {
			*color
		}
	}

	pub fn render(
		&self,
		screen: &mut dyn Screen,
		rect: Rect,
		clip_rect: &Rect,
		color_override: Option<Color>,
	) {
		// Determine the size of the layout and render it right jusitified
		// and centered vertically.
		let mut width = self.width();
		let mut height = self.height();
		if self.full_width() {
			width = rect.w;
		}
		if self.full_height() {
			height = rect.h;
		}
		let mut rect = Rect {
			x: rect.x + rect.w - width,
			y: rect.y + (rect.h - height) / 2,
			w: width,
			h: height,
		};

		// Check to see if the layout is entirely out of the clipping bounds
		if width > 0 && height > 0 {
			let clipped_rect = rect.clipped_to(clip_rect);
			if clipped_rect.w == 0 || clipped_rect.h == 0 {
				return;
			}
		}

		// Render the layout to the screen
		match self {
			Layout::Text(string, font, color) => font.draw_clipped(
				screen,
				clip_rect,
				rect.x,
				rect.y,
				string,
				Self::overridden_color(color, &color_override),
			),
			Layout::StaticText(string, font, color) => font.draw_clipped(
				screen,
				clip_rect,
				rect.x,
				rect.y,
				string,
				Self::overridden_color(color, &color_override),
			),
			Layout::EditText(string, font, color) => {
				font.draw_clipped(
					screen,
					clip_rect,
					rect.x,
					rect.y,
					string,
					Self::overridden_color(color, &color_override),
				);
				screen.fill(
					Rect {
						x: rect.x + rect.w - 1,
						y: rect.y,
						w: 3,
						h: rect.h,
					}
					.clipped_to(clip_rect),
					Self::overridden_color(color, &color_override),
				);
			}
			Layout::Horizontal(items) => {
				// Layout items from left to right, letting the individual layouts handle the
				// vertical space allocated.
				for item in items {
					let item_width = item.width();
					item.render(
						screen,
						Rect {
							x: rect.x,
							y: rect.y,
							w: item_width,
							h: rect.h,
						},
						clip_rect,
						color_override,
					);
					rect.x += item_width;
					rect.w -= item_width;
				}
			}
			Layout::Vertical(items) => {
				// Layout items from top to bottom, letting the individual layouts handle the
				// horizontal space allocated.
				for item in items {
					let item_height = item.height();
					item.render(
						screen,
						Rect {
							x: rect.x,
							y: rect.y,
							w: rect.w,
							h: item_height,
						},
						clip_rect,
						color_override,
					);
					rect.y += item_height;
					rect.h -= item_height;
				}
			}
			Layout::HorizontalRule => {
				screen.horizontal_pattern(
					rect.x,
					rect.w,
					rect.y,
					0b100100100100100100100100,
					24,
					Color::StackSeparator,
				);
			}
			Layout::Fraction(numer, denom, color) => {
				// Determine the sizes of the numerator and denominator
				let numer_width = numer.width();
				let numer_height = numer.height();
				let denom_width = denom.width();
				let denom_height = denom.height();

				// Render the numerator centered at the top
				numer.render(
					screen,
					Rect {
						x: rect.x + (width - numer_width) / 2,
						y: rect.y + (height - (numer_height + denom_height)) / 2,
						w: numer_width,
						h: numer_height,
					},
					clip_rect,
					color_override,
				);

				// Render the denominator cenetered at the bottom
				denom.render(
					screen,
					Rect {
						x: rect.x + (width - denom_width) / 2,
						y: rect.y + numer_height + (height - (numer_height + denom_height)) / 2,
						w: denom_width,
						h: denom_height,
					},
					clip_rect,
					color_override,
				);

				// Render the line separating the numerator and the denominator
				screen.fill(
					Rect {
						x: rect.x,
						y: rect.y + numer_height + (height - (numer_height + denom_height)) / 2,
						w: rect.w,
						h: 1,
					}
					.clipped_to(clip_rect),
					Self::overridden_color(color, &color_override),
				);
			}
			Layout::Power(base, power) => {
				// Determine the sizes of the base and the power
				let base_width = base.width();
				let base_height = base.height();
				let power_width = power.width();
				let power_height = power.height();

				// Render the base
				base.render(
					screen,
					Rect {
						x: rect.x,
						y: rect.y + rect.h - base_height,
						w: base_width,
						h: base_height,
					},
					clip_rect,
					color_override,
				);

				// Render the power
				power.render(
					screen,
					Rect {
						x: rect.x + rect.w - power_width,
						y: rect.y,
						w: power_width,
						h: power_height,
					},
					clip_rect,
					color_override,
				);
			}
			Layout::VerticalSpace(_) | Layout::HorizontalSpace(_) => (),
			Layout::LeftAlign(item) => {
				item.render(
					screen,
					Rect {
						x: rect.x,
						y: rect.y,
						w: item.width(),
						h: rect.h,
					},
					clip_rect,
					color_override,
				);
			}
			Layout::UsageGraph(used, reclaimable, free) => {
				// Calculate pixel sizes of the parts of the graph
				let total = used + reclaimable + free;
				let used_pixels = ((*used as u64 * (rect.w - 2) as u64) / total as u64) as i32;
				let reclaimable_pixels =
					((*reclaimable as u64 * (rect.w - 2) as u64) / total as u64) as i32;

				// Fill graph
				screen.fill(
					Rect {
						x: rect.x,
						y: rect.y,
						w: rect.w,
						h: rect.h,
					},
					Color::ContentText,
				);

				// Empty out available area
				screen.fill(
					Rect {
						x: rect.x + 1 + used_pixels,
						y: rect.y + 1,
						w: rect.w - (used_pixels + 2),
						h: rect.h - 2,
					},
					Color::ContentBackground,
				);

				// Draw reclaimable pattern
				for y_offset in 1..rect.h - 1 {
					screen.horizontal_pattern(
						rect.x + 1 + used_pixels,
						reclaimable_pixels,
						rect.y + y_offset,
						match y_offset & 3 {
							0 => 0b000100010001000100010001,
							2 => 0b010001000100010001000100,
							_ => 0b000000000000000000000000,
						},
						24,
						Color::ContentText,
					);
				}

				// Draw line between reclaimable and free
				screen.fill(
					Rect {
						x: rect.x + used_pixels + reclaimable_pixels,
						y: rect.y,
						w: 1,
						h: rect.h,
					},
					Color::ContentText,
				);
			}
			Layout::UsageGraphUsedLegend => {
				screen.fill(rect, Color::ContentText);
			}
			Layout::UsageGraphReclaimableLegend => {
				screen.fill(rect.clone(), Color::ContentText);
				screen.fill(
					Rect {
						x: rect.x + 1,
						y: rect.y + 1,
						w: rect.w - 2,
						h: rect.h - 2,
					},
					Color::ContentBackground,
				);
				for y_offset in 1..rect.h - 1 {
					screen.horizontal_pattern(
						rect.x + 1,
						rect.w - 2,
						rect.y + y_offset,
						match y_offset & 3 {
							0 => 0b000100010001000100010001,
							2 => 0b010001000100010001000100,
							_ => 0b000000000000000000000000,
						},
						24,
						Color::ContentText,
					);
				}
			}
			Layout::UsageGraphFreeLegend => {
				screen.fill(rect.clone(), Color::ContentText);
				screen.fill(
					Rect {
						x: rect.x + 1,
						y: rect.y + 1,
						w: rect.w - 2,
						h: rect.h - 2,
					},
					Color::ContentBackground,
				);
			}
			Layout::LeftMatrixBracket => {
				screen.fill(
					Rect {
						x: rect.x,
						y: rect.y + 1,
						w: 2,
						h: rect.h - 2,
					},
					Color::ContentText,
				);
				screen.fill(
					Rect {
						x: rect.x,
						y: rect.y + 1,
						w: rect.w - 4,
						h: 2,
					},
					Color::ContentText,
				);
				screen.fill(
					Rect {
						x: rect.x,
						y: rect.y + rect.h - 3,
						w: rect.w - 4,
						h: 2,
					},
					Color::ContentText,
				);
			}
			Layout::RightMatrixBracket => {
				screen.fill(
					Rect {
						x: rect.x + rect.w - 2,
						y: rect.y + 1,
						w: 2,
						h: rect.h - 2,
					},
					Color::ContentText,
				);
				screen.fill(
					Rect {
						x: rect.x + 4,
						y: rect.y + 1,
						w: rect.w - 4,
						h: 2,
					},
					Color::ContentText,
				);
				screen.fill(
					Rect {
						x: rect.x + 4,
						y: rect.y + rect.h - 3,
						w: rect.w - 4,
						h: 2,
					},
					Color::ContentText,
				);
			}
		}
	}
}
