use crate::font::Font;
use crate::layout::{Layout, LayoutRenderer, Rect, TokenType};
use crate::value::ValueLayout;
use rscalc_math::format::Format;
use rscalc_math::number::Number;
use rscalc_math::stack::{Stack, StackEvent};

#[cfg(feature = "std")]
use std::cell::RefCell;
#[cfg(feature = "std")]
use std::collections::BTreeMap;
#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap;
#[cfg(not(feature = "std"))]
use alloc::rc::Rc;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
use core::cell::RefCell;

#[derive(Clone)]
struct CachedStackEntryLayout {
	width: i32,
	idx: usize,
	bottom: i32,
	layout: Layout,
}

pub struct StackRenderer {
	render_cache: BTreeMap<usize, CachedStackEntryLayout>,
	prev_render_top: i32,
	prev_render_area_top: i32,
}

impl StackRenderer {
	pub fn new(stack: &mut Stack) -> Rc<RefCell<StackRenderer>> {
		let renderer = Rc::new(RefCell::new(StackRenderer {
			render_cache: BTreeMap::new(),
			prev_render_top: 0,
			prev_render_area_top: 0,
		}));

		// Register to get notifications for stack changes
		let renderer_copy = renderer.clone();
		stack.add_event_notify(move |event| {
			renderer_copy.borrow_mut().event(event);
		});

		renderer
	}

	fn event(&mut self, event: &StackEvent) {
		// Update rendering cache for stack changes
		match event {
			StackEvent::ValuePushed => {
				let mut new_cache = BTreeMap::new();
				for (key, value) in &self.render_cache {
					new_cache.insert(key + 1, value.clone());
				}
				self.render_cache = new_cache;
			}
			StackEvent::ValuePopped => {
				let mut new_cache = BTreeMap::new();
				for (key, value) in &self.render_cache {
					if key > &0 {
						new_cache.insert(key - 1, value.clone());
					}
				}
				self.render_cache = new_cache;
			}
			StackEvent::ValueChanged(idx) => {
				self.render_cache.remove(idx);
			}
			StackEvent::TopReplacedWithEntries(count) => {
				let mut new_cache = BTreeMap::new();
				for (key, value) in &self.render_cache {
					new_cache.insert(key + count - 1, value.clone());
				}
				self.render_cache = new_cache;
			}
			StackEvent::RotateUp => {
				let mut new_cache = BTreeMap::new();
				for (key, value) in &self.render_cache {
					new_cache.insert(key + 1, value.clone());
				}
				self.render_cache = new_cache;
			}
			StackEvent::Invalidate => {
				self.render_cache.clear();
			}
		}
	}

	pub fn force_refresh(&mut self) {
		for (_, value) in self.render_cache.iter_mut() {
			// Set bottom coordinate to an invalid position to force
			// rerendering the entry
			value.bottom = i32::MAX;
		}
		self.prev_render_top = 0;
		self.prev_render_area_top = 0;
	}

	pub fn invalidate_rendering(&mut self) {
		// Clear everything as values may have changed representation
		self.render_cache.clear();
		self.prev_render_top = 0;
		self.prev_render_area_top = 0;
	}

	pub fn render(
		&mut self,
		stack: &Stack,
		renderer: &mut dyn LayoutRenderer,
		format: &Format,
		base_font: Font,
		area: Rect,
		label_offset: usize,
	) {
		let mut bottom = area.y + area.h;
		let mut new_cache = BTreeMap::new();

		if stack.len() == 0 && label_offset == 0 {
			// Stack is empty, display a message instead of leaving the entire area blank
			let layout = Layout::HorizontalCenter(Box::new(Layout::StaticText(
				"⋘ Stack is empty ⋙",
				Font::Small,
				TokenType::Label,
			)));

			let height = layout.height(renderer.metrics());
			renderer.erase(
				&Rect {
					x: area.x,
					y: bottom - height,
					w: area.w,
					h: height,
				}
				.clipped_to(&area),
			);

			layout.render(
				renderer,
				Rect {
					x: area.x,
					y: bottom - height,
					w: area.w,
					h: height,
				},
				&area,
			);

			bottom -= height;
		}

		for idx in 0..stack.len() {
			if bottom < area.y {
				break;
			}

			// Construct and measure stack entry label
			let label = if format.stack_xyz {
				match idx + label_offset {
					0 => "x".to_string(),
					1 => "y".to_string(),
					2 => "z".to_string(),
					_ => Number::Integer((idx + label_offset + 1).into()).to_string(),
				}
			} else {
				Number::Integer((idx + label_offset + 1).into()).to_string()
			};
			let label = label + ": ";
			let label_width = 4 + renderer.metrics().width(Font::Small, &label);
			let width = area.w - label_width - 8;

			let layout = if let Some(cache) = self.render_cache.get(&idx) {
				// Check to see if this stack entry already been rendered to the screen in the
				// correct position with the same index
				let height = cache.layout.height(renderer.metrics());
				if idx == cache.idx
					&& bottom == cache.bottom
					&& (bottom - height >= core::cmp::max(area.y, self.prev_render_area_top)
						|| area.y == self.prev_render_area_top)
				{
					// Entry is already onscreen, no need to rerender
					bottom -= height;
					new_cache.insert(idx, cache.clone());
					continue;
				}

				if cache.width == width {
					Some(cache.layout.clone())
				} else {
					None
				}
			} else {
				None
			};

			let layout = if let Some(layout) = layout {
				layout
			} else {
				// Render stack entry to a layout
				let entry = match stack.entry(idx) {
					Ok(entry) => entry,
					Err(_) => continue,
				};
				let entry = Stack::value_for_integer_mode(&format.integer_mode, entry);
				entry.layout(format, base_font, renderer.metrics(), width)
			};

			// Clear the area of the stack entry
			let height = layout.height(renderer.metrics());
			renderer.erase(
				&Rect {
					x: area.x,
					y: bottom - height,
					w: area.w,
					h: height,
				}
				.clipped_to(&area),
			);

			// Render stack entry separator
			if bottom - height >= area.y {
				renderer.horizontal_pattern(
					area.x,
					area.w,
					bottom - height,
					0b100100100100100100100100,
					24,
					TokenType::Separator,
				);
			}

			// Draw the entry
			layout.render(
				renderer,
				Rect {
					x: area.x + label_width + 4,
					y: bottom - height,
					w: width,
					h: height,
				},
				&area,
			);

			// Draw the label
			let font_height = renderer.metrics().height(Font::Small);
			renderer.draw_text(
				4,
				(bottom - height) + (height - font_height) / 2,
				&label,
				Font::Small,
				TokenType::Label,
				&area,
			);

			// Insert rendered entry into rendering cache so that it can be quickly rendered next
			// time the screen is updated.
			new_cache.insert(
				idx,
				CachedStackEntryLayout {
					width,
					idx,
					bottom,
					layout,
				},
			);

			bottom -= height;
		}

		self.render_cache = new_cache;

		// If there is empty space above the stack, clear it now
		if (bottom > area.y && bottom > self.prev_render_top) || area.y < self.prev_render_area_top
		{
			let mut top = core::cmp::max(self.prev_render_top, area.y);
			if area.y < self.prev_render_area_top {
				top = area.y;
			}
			renderer.erase(&Rect {
				x: area.x,
				y: top,
				w: area.w,
				h: bottom - top,
			});
		}
		self.prev_render_top = bottom;
		self.prev_render_area_top = area.y;
	}
}
