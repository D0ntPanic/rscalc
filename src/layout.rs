use crate::screen::{Color, Font, Rect, Screen};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

pub enum Layout {
	Text(String, &'static Font, Color),
	EditText(String, &'static Font, Color),
	Horizontal(Vec<Layout>),
	Vertical(Vec<Layout>),
	Fraction(Box<Layout>, Box<Layout>, Color),
	HorizontalSpace(i32),
	VerticalSpace(i32),
}

impl Layout {
	pub fn editable_text(string: String, font: &'static Font, color: Color, editor: bool) -> Self {
		if editor {
			Layout::EditText(string, font, color)
		} else {
			Layout::Text(string, font, color)
		}
	}

	pub fn width(&self) -> i32 {
		match self {
			Layout::Text(string, font, _) => font.width(string),
			Layout::EditText(string, font, _) => font.width(string),
			Layout::Horizontal(items) => items.iter().fold(0, |width, item| width + item.width()),
			Layout::Vertical(items) => items
				.iter()
				.fold(0, |width, item| core::cmp::max(width, item.width())),
			Layout::Fraction(numer, denom, _) => core::cmp::max(numer.width(), denom.width()),
			Layout::HorizontalSpace(width) => *width,
			Layout::VerticalSpace(_) => 0,
		}
	}

	pub fn height(&self) -> i32 {
		match self {
			Layout::Text(_, font, _) => font.height,
			Layout::EditText(_, font, _) => font.height,
			Layout::Horizontal(items) => items
				.iter()
				.fold(0, |height, item| core::cmp::max(height, item.height())),
			Layout::Vertical(items) => items.iter().fold(0, |height, item| height + item.height()),
			Layout::Fraction(numer, denom, _) => numer.height() + denom.height(),
			Layout::HorizontalSpace(_) => 0,
			Layout::VerticalSpace(height) => *height,
		}
	}

	pub fn render<ScreenT: Screen>(&self, screen: &mut ScreenT, rect: Rect) {
		// Determine the size of the layout and render it right jusitified
		// and centered vertically.
		let width = self.width();
		let height = self.height();
		let mut rect = Rect {
			x: rect.x + rect.w - width,
			y: rect.y + (rect.h - height) / 2,
			w: width,
			h: height,
		};

		// Render the layout to the screen
		match self {
			Layout::Text(string, font, color) => font.draw(screen, rect.x, rect.y, string, *color),
			Layout::EditText(string, font, color) => {
				font.draw(screen, rect.x, rect.y, string, *color);
				screen.fill(
					Rect {
						x: rect.x + rect.w - 1,
						y: rect.y,
						w: 3,
						h: rect.h,
					},
					*color,
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
					);
					rect.y += item_height;
					rect.h -= item_height;
				}
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
				);

				// Render the line separating the numerator and the denominator
				screen.fill(
					Rect {
						x: rect.x,
						y: rect.y + numer_height + (height - (numer_height + denom_height)) / 2,
						w: rect.w,
						h: 1,
					},
					*color,
				);
			}
			Layout::VerticalSpace(_) | Layout::HorizontalSpace(_) => (),
		}
	}
}
