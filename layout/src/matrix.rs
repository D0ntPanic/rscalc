use crate::font::{Font, FontMetrics};
use crate::layout::Layout;
use crate::value::ValueLayout;
use rscalc_math::format::Format;
use rscalc_math::matrix::Matrix;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub trait MatrixLayout {
	fn layout(
		&self,
		format: &Format,
		font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Option<Layout>;
}

impl MatrixLayout for Matrix {
	fn layout(
		&self,
		format: &Format,
		font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Option<Layout> {
		let mut col_items = Vec::new();
		let left_bracket = Layout::LeftMatrixBracket;
		let right_bracket = Layout::RightMatrixBracket;
		let col_width = max_width.checked_sub(
			left_bracket.width(metrics)
				+ right_bracket.width(metrics)
				+ (self.cols() as i32 - 1) * 20,
		)? / (self.cols() as i32);
		col_items.push(left_bracket);

		for col in 0..self.cols() {
			if col != 0 {
				col_items.push(Layout::HorizontalSpace(20));
			}
			let mut row_items = Vec::new();
			for row in 0..self.rows() {
				let value = if let Ok(value) = self.get(row, col) {
					value
				} else {
					return None;
				};

				row_items.push(value.single_line_simple_layout(format, font, metrics, col_width));
			}
			col_items.push(Layout::Vertical(row_items));
		}

		col_items.push(right_bracket);
		let layout = Layout::Horizontal(col_items);
		if layout.width(metrics) <= max_width {
			let mut result_items = Vec::new();
			result_items.push(Layout::VerticalSpace(2));
			result_items.push(layout);
			result_items.push(Layout::VerticalSpace(2));
			Some(Layout::Vertical(result_items))
		} else {
			None
		}
	}
}
