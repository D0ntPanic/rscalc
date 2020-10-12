use crate::font::{Font, FontMetrics};

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct Rect {
	pub x: i32,
	pub y: i32,
	pub w: i32,
	pub h: i32,
}

impl Rect {
	pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
		Rect { x, y, w, h }
	}

	pub fn clipped_to(&self, area: &Rect) -> Self {
		let Rect {
			mut x,
			mut y,
			mut w,
			mut h,
		} = self.clone();
		if x < area.x {
			w += x - area.x;
			x = area.x;
		}
		if y < area.y {
			h += y - area.y;
			y = area.y;
		}
		if w <= 0 || h <= 0 {
			return Rect {
				x: area.x,
				y: area.y,
				w: 0,
				h: 0,
			};
		}
		if (x + w) > (area.x + area.w) {
			w = (area.x + area.w) - x;
		}
		if (y + h) > (area.y + area.h) {
			h = (area.y + area.h) - y;
		}
		if w <= 0 || h <= 0 {
			return Rect {
				x: area.x,
				y: area.y,
				w: 0,
				h: 0,
			};
		}
		Rect { x, y, w, h }
	}
}

#[derive(Clone, PartialEq, Eq, Copy)]
pub enum TokenType {
	Text,
	Integer,
	Float,
	Object,
	Keyword,
	Symbol,
	Complex,
	Unit,
	Label,
	Separator,
	Error,
}

pub trait LayoutRenderer {
	fn fill(&mut self, rect: &Rect, token_type: TokenType);
	fn erase(&mut self, rect: &Rect);
	fn horizontal_pattern(
		&mut self,
		x: i32,
		width: i32,
		y: i32,
		pattern: u32,
		pattern_width: u8,
		token_type: TokenType,
	);

	fn draw_text(
		&mut self,
		x: i32,
		y: i32,
		text: &str,
		font: Font,
		token_type: TokenType,
		clip_rect: &Rect,
	);

	fn metrics(&self) -> &dyn FontMetrics;
	fn set_selection_state(&mut self, selected: bool);
}

#[derive(Clone)]
pub enum Layout {
	Text(String, Font, TokenType),
	StaticText(&'static str, Font, TokenType),
	PartialText(String, Font, TokenType),
	PartialStaticText(&'static str, Font, TokenType),
	EditCursor(Font),
	Horizontal(Vec<Layout>),
	Vertical(Vec<Layout>),
	Fraction(Box<Layout>, Box<Layout>, TokenType),
	Power(Box<Layout>, Box<Layout>),
	HorizontalSpace(i32),
	VerticalSpace(i32),
	HorizontalRule,
	LeftAlign(Box<Layout>),
	HorizontalCenter(Box<Layout>),
	UsageGraph(usize, usize, usize),
	UsageGraphUsedLegend,
	UsageGraphReclaimableLegend,
	UsageGraphFreeLegend,
	LeftMatrixBracket,
	RightMatrixBracket,
}

impl Layout {
	pub fn full_width(&self) -> bool {
		match self {
			Layout::LeftAlign(_)
			| Layout::HorizontalCenter(_)
			| Layout::HorizontalRule
			| Layout::UsageGraph(_, _, _) => true,
			Layout::Horizontal(items) | Layout::Vertical(items) => {
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

	pub fn width(&self, metrics: &dyn FontMetrics) -> i32 {
		match self {
			Layout::Text(string, font, _) => metrics.width(*font, string),
			Layout::StaticText(string, font, _) => metrics.width(*font, string),
			Layout::PartialText(string, font, _) => metrics.advance(*font, string),
			Layout::PartialStaticText(string, font, _) => metrics.advance(*font, string),
			Layout::EditCursor(_) => 0,
			Layout::Horizontal(items) => items
				.iter()
				.fold(0, |width, item| width + item.width(metrics)),
			Layout::Vertical(items) => items
				.iter()
				.fold(0, |width, item| core::cmp::max(width, item.width(metrics))),
			Layout::Fraction(numer, denom, _) => {
				core::cmp::max(numer.width(metrics), denom.width(metrics))
			}
			Layout::Power(base, power) => base.width(metrics) + power.width(metrics),
			Layout::HorizontalSpace(width) => *width,
			Layout::VerticalSpace(_) => 0,
			Layout::HorizontalRule => 0,
			Layout::LeftAlign(item) | Layout::HorizontalCenter(item) => item.width(metrics),
			Layout::UsageGraph(_, _, _) => 0,
			Layout::UsageGraphUsedLegend
			| Layout::UsageGraphReclaimableLegend
			| Layout::UsageGraphFreeLegend => 11,
			Layout::LeftMatrixBracket | Layout::RightMatrixBracket => 10,
		}
	}

	pub fn height(&self, metrics: &dyn FontMetrics) -> i32 {
		match self {
			Layout::Text(_, font, _)
			| Layout::StaticText(_, font, _)
			| Layout::PartialText(_, font, _)
			| Layout::PartialStaticText(_, font, _)
			| Layout::EditCursor(font) => metrics.height(*font),
			Layout::Horizontal(items) => items.iter().fold(0, |height, item| {
				core::cmp::max(height, item.height(metrics))
			}),
			Layout::Vertical(items) => items
				.iter()
				.fold(0, |height, item| height + item.height(metrics)),
			Layout::Fraction(numer, denom, _) => numer.height(metrics) + denom.height(metrics),
			Layout::Power(base, power) => {
				let base_height = base.height(metrics);
				let power_height = power.height(metrics);
				let mut max_height = core::cmp::max(base_height, power_height);
				if max_height - power_height < 4 {
					max_height = core::cmp::max(base_height, power_height + 4);
				}
				max_height
			}
			Layout::HorizontalSpace(_) => 0,
			Layout::VerticalSpace(height) => *height,
			Layout::HorizontalRule => 1,
			Layout::LeftAlign(item) | Layout::HorizontalCenter(item) => item.height(metrics),
			Layout::UsageGraph(_, _, _) => 31,
			Layout::UsageGraphUsedLegend
			| Layout::UsageGraphReclaimableLegend
			| Layout::UsageGraphFreeLegend => 11,
			Layout::LeftMatrixBracket | Layout::RightMatrixBracket => 16,
		}
	}

	pub fn render(&self, renderer: &mut dyn LayoutRenderer, rect: Rect, clip_rect: &Rect) {
		// Determine the size of the layout and render it right jusitified
		// and centered vertically.
		let mut width = self.width(renderer.metrics());
		let mut height = self.height(renderer.metrics());
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
			Layout::Text(string, font, token_type)
			| Layout::PartialText(string, font, token_type) => {
				renderer.draw_text(rect.x, rect.y, &string, *font, *token_type, clip_rect)
			}
			Layout::StaticText(string, font, token_type)
			| Layout::PartialStaticText(string, font, token_type) => {
				renderer.draw_text(rect.x, rect.y, string, *font, *token_type, clip_rect)
			}
			Layout::EditCursor(_) => {
				renderer.fill(
					&Rect {
						x: rect.x - 1,
						y: rect.y,
						w: 3,
						h: rect.h,
					}
					.clipped_to(clip_rect),
					TokenType::Text,
				);
			}
			Layout::Horizontal(items) => {
				// Layout items from left to right, letting the individual layouts handle the
				// vertical space allocated.
				let natural_width = self.width(renderer.metrics());
				let mut full_width_count = 0;
				for item in items {
					if item.full_width() {
						full_width_count += 1;
					}
				}
				let extra_width = if full_width_count > 0 {
					(rect.w - natural_width) / full_width_count
				} else {
					0
				};

				for item in items {
					let mut item_width = item.width(renderer.metrics());
					if item.full_width() {
						item_width += extra_width;
					}
					item.render(
						renderer,
						Rect {
							x: rect.x,
							y: rect.y,
							w: item_width,
							h: rect.h,
						},
						clip_rect,
					);
					rect.x += item_width;
					rect.w -= item_width;
				}
			}
			Layout::Vertical(items) => {
				// Layout items from top to bottom, letting the individual layouts handle the
				// horizontal space allocated.
				for item in items {
					let item_height = item.height(renderer.metrics());
					item.render(
						renderer,
						Rect {
							x: rect.x,
							y: rect.y,
							w: rect.w,
							h: item_height,
						},
						clip_rect,
					);
					rect.y += item_height;
					rect.h -= item_height;
				}
			}
			Layout::HorizontalRule => {
				renderer.horizontal_pattern(
					rect.x,
					rect.w,
					rect.y,
					0b100100100100100100100100,
					24,
					TokenType::Separator,
				);
			}
			Layout::Fraction(numer, denom, token_type) => {
				// Determine the sizes of the numerator and denominator
				let numer_width = numer.width(renderer.metrics());
				let numer_height = numer.height(renderer.metrics());
				let denom_width = denom.width(renderer.metrics());
				let denom_height = denom.height(renderer.metrics());

				// Render the numerator centered at the top
				numer.render(
					renderer,
					Rect {
						x: rect.x + (width - numer_width) / 2,
						y: rect.y + (height - (numer_height + denom_height)) / 2,
						w: numer_width,
						h: numer_height,
					},
					clip_rect,
				);

				// Render the denominator cenetered at the bottom
				denom.render(
					renderer,
					Rect {
						x: rect.x + (width - denom_width) / 2,
						y: rect.y + numer_height + (height - (numer_height + denom_height)) / 2,
						w: denom_width,
						h: denom_height,
					},
					clip_rect,
				);

				// Render the line separating the numerator and the denominator
				renderer.fill(
					&Rect {
						x: rect.x,
						y: rect.y + numer_height + (height - (numer_height + denom_height)) / 2,
						w: rect.w,
						h: 1,
					}
					.clipped_to(clip_rect),
					*token_type,
				);
			}
			Layout::Power(base, power) => {
				// Determine the sizes of the base and the power
				let base_width = base.width(renderer.metrics());
				let base_height = base.height(renderer.metrics());
				let power_width = power.width(renderer.metrics());
				let power_height = power.height(renderer.metrics());

				// Render the base
				base.render(
					renderer,
					Rect {
						x: rect.x,
						y: rect.y + rect.h - base_height,
						w: base_width,
						h: base_height,
					},
					clip_rect,
				);

				// Render the power
				power.render(
					renderer,
					Rect {
						x: rect.x + rect.w - power_width,
						y: rect.y,
						w: power_width,
						h: power_height,
					},
					clip_rect,
				);
			}
			Layout::VerticalSpace(_) | Layout::HorizontalSpace(_) => (),
			Layout::LeftAlign(item) => {
				let width = item.width(renderer.metrics());
				item.render(
					renderer,
					Rect {
						x: rect.x,
						y: rect.y,
						w: width,
						h: rect.h,
					},
					clip_rect,
				);
			}
			Layout::HorizontalCenter(item) => {
				let width = item.width(renderer.metrics());
				item.render(
					renderer,
					Rect {
						x: rect.x + (rect.w / 2) - (width / 2),
						y: rect.y,
						w: width,
						h: rect.h,
					},
					clip_rect,
				);
			}
			Layout::UsageGraph(used, reclaimable, free) => {
				// Calculate pixel sizes of the parts of the graph
				let total = used + reclaimable + free;
				let used_pixels = ((*used as u64 * (rect.w - 2) as u64) / total as u64) as i32;
				let reclaimable_pixels =
					((*reclaimable as u64 * (rect.w - 2) as u64) / total as u64) as i32;

				// Fill graph
				renderer.fill(
					&Rect {
						x: rect.x,
						y: rect.y,
						w: rect.w,
						h: rect.h,
					},
					TokenType::Text,
				);

				// Empty out available area
				renderer.erase(&Rect {
					x: rect.x + 1 + used_pixels,
					y: rect.y + 1,
					w: rect.w - (used_pixels + 2),
					h: rect.h - 2,
				});

				// Draw reclaimable pattern
				for y_offset in 1..rect.h - 1 {
					renderer.horizontal_pattern(
						rect.x + 1 + used_pixels,
						reclaimable_pixels,
						rect.y + y_offset,
						match y_offset & 3 {
							0 => 0b000100010001000100010001,
							2 => 0b010001000100010001000100,
							_ => 0b000000000000000000000000,
						},
						24,
						TokenType::Text,
					);
				}

				// Draw line between reclaimable and free
				renderer.fill(
					&Rect {
						x: rect.x + used_pixels + reclaimable_pixels,
						y: rect.y,
						w: 1,
						h: rect.h,
					},
					TokenType::Text,
				);
			}
			Layout::UsageGraphUsedLegend => {
				renderer.fill(&rect, TokenType::Text);
			}
			Layout::UsageGraphReclaimableLegend => {
				renderer.fill(&rect, TokenType::Text);
				renderer.erase(&Rect {
					x: rect.x + 1,
					y: rect.y + 1,
					w: rect.w - 2,
					h: rect.h - 2,
				});
				for y_offset in 1..rect.h - 1 {
					renderer.horizontal_pattern(
						rect.x + 1,
						rect.w - 2,
						rect.y + y_offset,
						match y_offset & 3 {
							0 => 0b000100010001000100010001,
							2 => 0b010001000100010001000100,
							_ => 0b000000000000000000000000,
						},
						24,
						TokenType::Text,
					);
				}
			}
			Layout::UsageGraphFreeLegend => {
				renderer.fill(&rect, TokenType::Text);
				renderer.erase(&Rect {
					x: rect.x + 1,
					y: rect.y + 1,
					w: rect.w - 2,
					h: rect.h - 2,
				});
			}
			Layout::LeftMatrixBracket => {
				renderer.fill(
					&Rect {
						x: rect.x,
						y: rect.y + 1,
						w: 2,
						h: rect.h - 2,
					},
					TokenType::Symbol,
				);
				renderer.fill(
					&Rect {
						x: rect.x,
						y: rect.y + 1,
						w: rect.w - 4,
						h: 2,
					},
					TokenType::Symbol,
				);
				renderer.fill(
					&Rect {
						x: rect.x,
						y: rect.y + rect.h - 3,
						w: rect.w - 4,
						h: 2,
					},
					TokenType::Symbol,
				);
			}
			Layout::RightMatrixBracket => {
				renderer.fill(
					&Rect {
						x: rect.x + rect.w - 2,
						y: rect.y + 1,
						w: 2,
						h: rect.h - 2,
					},
					TokenType::Symbol,
				);
				renderer.fill(
					&Rect {
						x: rect.x + 4,
						y: rect.y + 1,
						w: rect.w - 4,
						h: 2,
					},
					TokenType::Symbol,
				);
				renderer.fill(
					&Rect {
						x: rect.x + 4,
						y: rect.y + rect.h - 3,
						w: rect.w - 4,
						h: 2,
					},
					TokenType::Symbol,
				);
			}
		}
	}
}
