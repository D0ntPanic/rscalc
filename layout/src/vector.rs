use crate::font::{Font, FontMetrics};
use crate::layout::{Layout, TokenType};
use crate::value::ValueLayout;
use rscalc_math::format::Format;
use rscalc_math::vector::Vector;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub trait VectorLayout {
	fn single_line_full_layout(
		&self,
		format: &Format,
		default_font: Font,
		small_font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Option<Layout>;

	fn multi_line_layout(
		&self,
		format: &Format,
		font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
		max_lines: usize,
	) -> Option<Layout>;
}

impl VectorLayout for Vector {
	fn single_line_full_layout(
		&self,
		format: &Format,
		default_font: Font,
		small_font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
	) -> Option<Layout> {
		let mut horizontal_items = Vec::new();
		let left_paren = Layout::StaticText("⦗", default_font, TokenType::Symbol);
		let right_paren = Layout::StaticText("⦘", default_font, TokenType::Symbol);
		let mut width = left_paren.width(metrics) + right_paren.width(metrics);
		horizontal_items.push(left_paren);

		for i in 0..self.len() {
			if i > 0 {
				let spacing = Layout::HorizontalSpace(24);
				width += spacing.width(metrics);
				horizontal_items.push(spacing);
			}

			if width >= max_width {
				return None;
			}

			let value = if let Ok(value) = self.get(i) {
				value
			} else {
				return None;
			};

			if let Some(layout) = value.single_line_numerical_layout(
				format,
				default_font,
				small_font,
				metrics,
				max_width - width,
				false,
			) {
				width += layout.width(metrics);
				horizontal_items.push(layout);
			} else {
				return None;
			};
		}

		horizontal_items.push(right_paren);
		Some(Layout::Horizontal(horizontal_items))
	}

	fn multi_line_layout(
		&self,
		format: &Format,
		font: Font,
		metrics: &dyn FontMetrics,
		max_width: i32,
		max_lines: usize,
	) -> Option<Layout> {
		let mut vertical_items = Vec::new();
		let mut horizontal_items = Vec::new();
		let left_paren = Layout::StaticText("⦗", font, TokenType::Symbol);
		let right_paren = Layout::StaticText("⦘", font, TokenType::Symbol);
		let mut width = left_paren.width(metrics) + right_paren.width(metrics);
		horizontal_items.push(right_paren);

		for i in (0..self.len()).rev() {
			if (i + 1) < self.len() && horizontal_items.len() > 0 {
				let spacing = Layout::HorizontalSpace(20);
				width += spacing.width(metrics);
				horizontal_items.push(spacing);
			}

			let value = if let Ok(value) = self.get(i) {
				value
			} else {
				return None;
			};

			let layout = value.single_line_simple_layout(
				format,
				font,
				metrics,
				max_width - left_paren.width(metrics),
			);
			let layout_width = layout.width(metrics);
			if width + layout_width > max_width {
				vertical_items.push(Layout::Horizontal(
					horizontal_items.drain(..).rev().collect(),
				));
				if vertical_items.len() >= max_lines {
					return None;
				}
				width = left_paren.width(metrics);
			}
			width += layout.width(metrics);
			horizontal_items.push(layout);
		}

		horizontal_items.push(left_paren);
		vertical_items.push(Layout::Horizontal(
			horizontal_items.drain(..).rev().collect(),
		));
		Some(Layout::Vertical(vertical_items.drain(..).rev().collect()))
	}
}
