#[derive(Debug, Clone)]
pub struct Rect {
	pub x: i32,
	pub y: i32,
	pub w: i32,
	pub h: i32
}

impl Rect {
	pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
		Rect {
			x, y, w, h
		}
	}

	pub fn clipped_to(&self, area: &Rect) -> Self {
		let Rect { mut x, mut y, mut w, mut h } = self.clone();
		if x < area.x {
			w += x - area.x;
			x = area.x;
		}
		if y < area.y {
			h += y - area.y;
			y = area.y;
		}
		if w <= 0 || h <= 0 {
			return Rect { x: area.x, y: area.y, w: 0, h: 0 };
		}
		if (x + w) > (area.x + area.w) {
			w = (area.x + area.w) - x;
		}
		if (y + h) > (area.y + area.h) {
			h = (area.y + area.h) - y;
		}
		if w <= 0 || h <= 0 {
			return Rect { x: area.x, y: area.y, w: 0, h: 0 };
		}
		Rect { x, y, w, h }
	}

	pub fn clipped_to_screen<ScreenT: Screen>(&self, screen: &ScreenT) -> Self {
		self.clipped_to(&Rect { x: 0, y: 0, w: screen.width(), h: screen.height() })
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Color {
	StatusBarBackground,
	StatusBarText,
	ContentBackground,
	ContentText,
	IntegerText,
	FloatText,
	ObjectText,
	KeywordText,
	SelectionBackground,
	SelectionText,
	MenuBackground,
	MenuText
}

impl Color {
	pub fn to_bw(&self) -> bool {
		match self {
			Color::StatusBarBackground | Color::ContentText |
				Color::IntegerText | Color::FloatText |
				Color::ObjectText | Color::KeywordText |
				Color::SelectionBackground | Color::MenuBackground => true,
			Color::StatusBarText | Color::ContentBackground |
				Color::SelectionText | Color::MenuText => false
		}
	}
}

pub struct Font {
	pub height: i32,
	pub chars: &'static [&'static [u8]],
	pub width: &'static [u8],
	pub advance: &'static [u8]
}

impl Font {
	pub fn draw<T: Screen>(&self, screen: &mut T, x: i32, y: i32, text: &str, color: Color) {
		let mut cur_x = x;
		for ch in text.bytes() {
			if ch < 0x20 || ch > 0x7e {
				continue;
			}
			let idx = (ch - 0x20) as usize;

			let width = self.width[idx];
			let bytes = (width + 7) / 8;
			let mut offset = 0;
			for line in 0..self.height {
				let mut remain = width;
				for byte in 0..bytes {
					let cur_width = if remain >= 8 { 8 } else { remain };
					screen.draw_bits(cur_x + byte as i32 * 8, y + line,
						self.chars[idx][offset] as u32, cur_width, color);
					remain = remain.saturating_sub(8);
					offset += 1;
				}
			}

			cur_x += self.advance[idx] as i32;
		}
	}

	pub fn width(&self, text: &str) -> i32 {
		let mut result = 0;
		let mut extra = 0;
		for ch in text.bytes() {
			if ch < 0x20 || ch > 0x7e {
				continue;
			}
			let idx = (ch - 0x20) as usize;
			let width = core::cmp::max(self.width[idx], self.advance[idx]) as i32;
			extra = width - self.advance[idx] as i32;
			result += self.advance[idx] as i32;
		}
		result + extra
	}

	pub fn advance(&self, text: &str) -> i32 {
		let mut result = 0;
		for ch in text.bytes() {
			if ch < 0x20 || ch > 0x7e {
				continue;
			}
			let idx = (ch - 0x20) as usize;
			result += self.advance[idx] as i32;
		}
		result
	}
}

pub trait Screen {
	fn width(&self) -> i32;
	fn height(&self) -> i32;
	fn clear(&mut self);
	fn refresh(&mut self);

	fn fill(&mut self, rect: Rect, color: Color);

	fn draw_bits(&mut self, x: i32, y: i32, bits: u32, width: u8, color: Color);
}
