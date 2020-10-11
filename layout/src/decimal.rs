use crate::font::{Font, FontMetrics};
use crate::layout::{Layout, TokenType};
use intel_dfp::Decimal;
use rscalc_math::format::Format;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

pub trait DecimalLayout {
	fn single_line_layout(
		&self,
		format: &Format,
		prefix: &str,
		suffix: &str,
		font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Layout;
}

impl DecimalLayout for Decimal {
	fn single_line_layout(
		&self,
		format: &Format,
		prefix: &str,
		suffix: &str,
		font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Layout {
		let mut format = format.clone();
		loop {
			// Try to format the string and see if it fits
			let string = prefix.to_string() + &format.format_decimal(self) + suffix;
			if metrics.width(font, &string) <= max_width {
				// This string fits, return final layout
				return Layout::Text(string, font, TokenType::Float);
			}

			// Try a reduced precision. If the precision is already 3 or less, just use the
			// existing string.
			if format.precision <= 3 {
				return Layout::Text(string, font, TokenType::Float);
			}
			format = format.with_max_precision(format.precision - 1);
		}
	}
}
