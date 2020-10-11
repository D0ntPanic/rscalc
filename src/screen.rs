use crate::font::{char_to_idx, SANS_13, SANS_16, SANS_20, SANS_24};
use rscalc_layout::font::{Font, FontMetrics};
use rscalc_layout::layout::{LayoutRenderer, Rect, TokenType};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Color {
	StatusBarBackground,
	StatusBarText,
	ContentBackground,
	LabelText,
	Separator,
	ContentText,
	IntegerText,
	FloatText,
	ObjectText,
	KeywordText,
	SymbolText,
	ComplexText,
	UnitText,
	ErrorText,
	SelectionBackground,
	SelectionText,
	MenuBackground,
	MenuText,
}

impl Color {
	pub fn to_bw(&self) -> bool {
		match self {
			Color::StatusBarBackground
			| Color::ContentText
			| Color::LabelText
			| Color::Separator
			| Color::IntegerText
			| Color::FloatText
			| Color::ObjectText
			| Color::KeywordText
			| Color::SymbolText
			| Color::ComplexText
			| Color::UnitText
			| Color::ErrorText
			| Color::SelectionBackground
			| Color::MenuBackground => true,
			Color::StatusBarText
			| Color::ContentBackground
			| Color::SelectionText
			| Color::MenuText => false,
		}
	}
}

pub struct BitmapFont {
	pub height: i32,
	pub chars: &'static [&'static [u8]],
	pub width: &'static [u8],
	pub advance: &'static [u8],
}

impl BitmapFont {
	pub fn draw(
		&self,
		screen: &mut dyn Screen,
		area: &Rect,
		x: i32,
		y: i32,
		text: &str,
		color: Color,
	) {
		// Check for completely out of bounds target
		if (y + self.height <= area.y) || (y >= area.y + area.h) {
			return;
		}

		// Render each character in the string
		let mut cur_x = x;
		for ch in text.chars() {
			// Check to see if we are past the right edge of the clip region. Once
			// there, no other characters can be in the region.
			if cur_x >= area.x + area.w {
				break;
			}

			// Decode character into font glyph index
			if let Some(idx) = char_to_idx(ch) {
				// Get width of character and determine the number of bytes per line
				// the glyph takes in the font data
				let width = self.width[idx];
				let bytes = (width + 7) / 8;

				// Render the character to the screen
				let mut offset = 0;
				for line in 0..self.height {
					// Check line to see if it is within the clip region
					if y + line < area.y {
						offset += bytes as usize;
						continue;
					}
					if y + line >= area.y + area.h {
						break;
					}

					// Render the character one byte of the glyph data at a time
					let mut remain = width;
					for byte in 0..bytes {
						// Determine the remaining pixels
						let mut cur_width = if remain >= 8 { 8 } else { remain };
						let mut data = self.chars[idx][offset];
						let mut x_offset = byte as i32 * 8;

						// Check this byte's target region to see if it is entirely
						// out of the clipping region
						if (cur_x + x_offset + cur_width as i32) <= area.x {
							// Clipped, advance to next byte's data without drawing it
							remain = remain.saturating_sub(8);
							offset += 1;
							continue;
						} else if cur_x + x_offset >= area.x + area.w {
							// Past right edge of clip area, nothing else in bounds
							offset += (bytes - byte) as usize;
							break;
						}

						// Clip the byte's data if it is partially clipped
						if cur_x + x_offset < area.x {
							let clipped = area.x - (cur_x + x_offset);
							cur_width -= clipped as u8;
							x_offset += clipped;
						}
						if (cur_x + x_offset + cur_width as i32) > area.x + area.w {
							let clipped = (cur_x + x_offset + cur_width as i32) - (area.x + area.w);
							data = data >> clipped;
							cur_width -= clipped as u8;
						}

						// Draw the data to the screen
						screen.draw_bits(cur_x + x_offset, y + line, data as u32, cur_width, color);

						// Advance to next byte's data
						remain = remain.saturating_sub(8);
						offset += 1;
					}
				}

				// Advance x coordinate to the next character
				cur_x += self.advance[idx] as i32;
			}
		}
	}

	pub fn width(&self, text: &str) -> i32 {
		let mut result = 0;
		let mut extra = 0;
		for ch in text.chars() {
			if let Some(idx) = char_to_idx(ch) {
				let width = core::cmp::max(self.width[idx], self.advance[idx]) as i32;
				extra = width - self.advance[idx] as i32;
				result += self.advance[idx] as i32;
			}
		}
		result + extra
	}

	pub fn advance(&self, text: &str) -> i32 {
		let mut result = 0;
		for ch in text.chars() {
			if let Some(idx) = char_to_idx(ch) {
				result += self.advance[idx] as i32;
			}
		}
		result
	}
}

pub trait Screen {
	fn width(&self) -> i32;
	fn height(&self) -> i32;

	fn screen_rect(&self) -> Rect {
		Rect {
			x: 0,
			y: 0,
			w: self.width(),
			h: self.height(),
		}
	}

	fn clear(&mut self);
	fn refresh(&mut self);

	fn fill(&mut self, rect: &Rect, color: Color);

	fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
		self.fill(&Rect { x, y, w: 1, h: 1 }, color);
	}

	fn horizontal_pattern(
		&mut self,
		x: i32,
		width: i32,
		y: i32,
		pattern: u32,
		pattern_width: u8,
		color: Color,
	) {
		let mut cur_x = x;
		let mut remaining = width;
		while remaining > 0 {
			let (cur_pattern, cur_width);
			if remaining >= pattern_width as i32 {
				cur_pattern = pattern;
				cur_width = pattern_width;
			} else {
				cur_pattern = pattern >> (pattern_width as i32 - remaining);
				cur_width = remaining as u8;
			}
			self.draw_bits(cur_x, y, cur_pattern, cur_width, color);
			cur_x += cur_width as i32;
			remaining -= cur_width as i32;
		}
	}

	fn draw_bits(&mut self, x: i32, y: i32, bits: u32, width: u8, color: Color);

	fn metrics(&self) -> &dyn FontMetrics {
		&ScreenFontMetrics
	}

	fn renderer(&mut self, render_mode: RenderMode) -> ScreenLayoutRenderer;
}

pub enum RenderMode {
	Normal,
	Selected,
	StatusBar,
	FunctionKeys,
}

impl RenderMode {
	fn color_for_token(&self, token_type: TokenType) -> Color {
		match self {
			RenderMode::Normal => match token_type {
				TokenType::Text => Color::ContentText,
				TokenType::Integer => Color::IntegerText,
				TokenType::Float => Color::FloatText,
				TokenType::Object => Color::ObjectText,
				TokenType::Keyword => Color::KeywordText,
				TokenType::Symbol => Color::SymbolText,
				TokenType::Complex => Color::ComplexText,
				TokenType::Unit => Color::UnitText,
				TokenType::Label => Color::LabelText,
				TokenType::Separator => Color::Separator,
				TokenType::Error => Color::ErrorText,
			},
			RenderMode::Selected => Color::SelectionText,
			RenderMode::StatusBar => Color::StatusBarText,
			RenderMode::FunctionKeys => Color::MenuText,
		}
	}

	fn color_for_background(&self) -> Color {
		match self {
			RenderMode::Normal => Color::ContentBackground,
			RenderMode::Selected => Color::SelectionBackground,
			RenderMode::StatusBar => Color::StatusBarBackground,
			RenderMode::FunctionKeys => Color::MenuBackground,
		}
	}
}

pub struct ScreenFontMetrics;

impl FontMetrics for ScreenFontMetrics {
	fn width(&self, font: Font, text: &str) -> i32 {
		match font {
			Font::Smallest => SANS_13.width(text),
			Font::Small => SANS_16.width(text),
			Font::Medium => SANS_20.width(text),
			Font::Large => SANS_24.width(text),
		}
	}

	fn advance(&self, font: Font, text: &str) -> i32 {
		match font {
			Font::Smallest => SANS_13.advance(text),
			Font::Small => SANS_16.advance(text),
			Font::Medium => SANS_20.advance(text),
			Font::Large => SANS_24.advance(text),
		}
	}

	fn height(&self, font: Font) -> i32 {
		match font {
			Font::Smallest => SANS_13.height,
			Font::Small => SANS_16.height,
			Font::Medium => SANS_20.height,
			Font::Large => SANS_24.height,
		}
	}
}

pub struct ScreenLayoutRenderer<'a> {
	screen: &'a mut dyn Screen,
	render_mode: RenderMode,
}

impl<'a> ScreenLayoutRenderer<'a> {
	pub fn new(screen: &'a mut dyn Screen, render_mode: RenderMode) -> Self {
		ScreenLayoutRenderer {
			screen,
			render_mode,
		}
	}
}

impl<'a> LayoutRenderer for ScreenLayoutRenderer<'a> {
	fn fill(&mut self, rect: &Rect, token_type: TokenType) {
		self.screen
			.fill(rect, self.render_mode.color_for_token(token_type));
	}

	fn erase(&mut self, rect: &Rect) {
		self.screen
			.fill(rect, self.render_mode.color_for_background());
	}

	fn horizontal_pattern(
		&mut self,
		x: i32,
		width: i32,
		y: i32,
		pattern: u32,
		pattern_width: u8,
		token_type: TokenType,
	) {
		self.screen.horizontal_pattern(
			x,
			width,
			y,
			pattern,
			pattern_width,
			self.render_mode.color_for_token(token_type),
		);
	}

	fn draw_text(
		&mut self,
		x: i32,
		y: i32,
		text: &str,
		font: Font,
		token_type: TokenType,
		clip_rect: &Rect,
	) {
		let font = match font {
			Font::Smallest => &SANS_13,
			Font::Small => &SANS_16,
			Font::Medium => &SANS_20,
			Font::Large => &SANS_24,
		};
		font.draw(
			self.screen,
			clip_rect,
			x,
			y,
			text,
			self.render_mode.color_for_token(token_type),
		);
	}

	fn metrics(&self) -> &dyn FontMetrics {
		self.screen.metrics()
	}
}
