use crate::font::{Font, FontMetrics};
use crate::layout::{Layout, TokenType};

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub trait StringLayout: Sized {
	fn single_line_layout(
		&self,
		font: Font,
		token_type: TokenType,
		metrics: &dyn FontMetrics,
		max_width: i32,
		cursor: Option<usize>,
	) -> Option<Layout>;

	fn double_line_layout(
		&self,
		default_font: Font,
		small_font: Font,
		token_type: TokenType,
		metrics: &dyn FontMetrics,
		max_width: i32,
		cursor: Option<usize>,
	) -> Option<Layout>;
}

fn cursor_layout(string: &str, font: Font, token_type: TokenType, cursor: usize) -> Layout {
	let mut items = Vec::new();
	if cursor == 0 {
		items.push(Layout::EditCursor(font));
		items.push(Layout::Text(string.to_string(), font, token_type));
		Layout::Horizontal(items)
	} else if cursor >= string.len() {
		items.push(Layout::Text(string.to_string(), font, token_type));
		items.push(Layout::EditCursor(font));
		Layout::Horizontal(items)
	} else {
		Layout::Text(string.to_string(), font, token_type)
	}
}

impl StringLayout for String {
	fn single_line_layout(
		&self,
		font: Font,
		token_type: TokenType,
		metrics: &dyn FontMetrics,
		max_width: i32,
		cursor: Option<usize>,
	) -> Option<Layout> {
		if metrics.width(font, self) <= max_width {
			if let Some(cursor) = cursor {
				Some(cursor_layout(&self, font, token_type, cursor))
			} else {
				Some(Layout::Text(self.clone(), font, token_type))
			}
		} else {
			None
		}
	}

	fn double_line_layout(
		&self,
		default_font: Font,
		small_font: Font,
		token_type: TokenType,
		metrics: &dyn FontMetrics,
		max_width: i32,
		cursor: Option<usize>,
	) -> Option<Layout> {
		// Check width to see if it is possible to fit within two lines
		if metrics.width(small_font, &self) > max_width * 2 {
			return None;
		}

		// Check layout width for a single line in the normal font
		if let Some(layout) =
			Self::single_line_layout(self, default_font, token_type, metrics, max_width, cursor)
		{
			return Some(layout);
		}

		// Check layout width for a single line in the smaller font
		if let Some(layout) =
			Self::single_line_layout(self, small_font, token_type, metrics, max_width, cursor)
		{
			return Some(layout);
		}

		// String does not fit, try to split it to two lines
		let chars: Vec<char> = self.chars().collect();
		let mut split_point = 0;
		let mut width = 0;
		for i in 0..chars.len() {
			let mut char_str = String::new();
			char_str.push(chars[(chars.len() - 1) - i]);
			split_point = i;
			// Add in the width of this character
			if i == 0 {
				width += metrics.width(small_font, &char_str);
			} else {
				width += metrics.advance(small_font, &char_str);
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
		if let Some(cursor) = cursor {
			if cursor <= first_str.len() {
				layout_items.push(cursor_layout(&first_str, small_font, token_type, cursor));
				layout_items.push(Layout::Text(second_str, small_font, token_type));
			} else {
				let first_str_len = first_str.len();
				layout_items.push(Layout::Text(first_str, small_font, token_type));
				layout_items.push(cursor_layout(
					&second_str,
					small_font,
					token_type,
					cursor - first_str_len,
				));
			}
		} else {
			layout_items.push(Layout::Text(first_str, small_font, token_type));
			layout_items.push(Layout::Text(second_str, small_font, token_type));
		}
		let split_layout = Layout::Vertical(layout_items);
		if split_layout.width(metrics) <= max_width {
			Some(split_layout)
		} else {
			None
		}
	}
}
