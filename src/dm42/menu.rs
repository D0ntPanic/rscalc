use crate::dm42::functions::Function;
use crate::dm42::screen::{RenderMode, Screen};
use crate::dm42::state::{State, StatusBarLeftDisplayType};
use core::cell::RefCell;
use rscalc_layout::font::Font;
use rscalc_layout::layout::{Layout, LayoutRenderer, Rect, TokenType};
use rscalc_math::format::AlternateFormatMode;
use rscalc_math::number::Number;
use rscalc_math::storage::{available_bytes, free_bytes, reclaimable_bytes, used_bytes};

#[cfg(not(feature = "dm42"))]
use std::borrow::Cow;

#[cfg(feature = "dm42")]
use alloc::borrow::Cow;
#[cfg(feature = "dm42")]
use alloc::boxed::Box;
#[cfg(feature = "dm42")]
use alloc::string::{String, ToString};
#[cfg(feature = "dm42")]
use alloc::vec::Vec;

#[derive(PartialEq, Eq, Clone)]
pub enum MenuItemFunction {
	Action(Function),
	InMenuAction(Function),
	InMenuActionWithDelete(Function, Function),
	ConversionAction(Function, Function, Function),
}

pub enum MenuItemLayout {
	Static(Layout),
	Dynamic(Box<dyn Fn(&State, &dyn Screen) -> Layout>),
}

pub struct MenuItem {
	pub layout: MenuItemLayout,
	pub function: MenuItemFunction,
}

impl MenuItem {
	pub fn string_layout(text: String) -> Layout {
		Layout::LeftAlign(Box::new(Layout::Text(text, Font::Small, TokenType::Text)))
	}

	pub fn string_layout_small(text: String) -> Layout {
		Layout::LeftAlign(Box::new(Layout::Text(
			text,
			Font::Smallest,
			TokenType::Text,
		)))
	}

	pub fn static_string_layout(text: &'static str) -> Layout {
		Layout::LeftAlign(Box::new(Layout::StaticText(
			text,
			Font::Small,
			TokenType::Text,
		)))
	}

	pub fn static_string_layout_small(text: &'static str) -> Layout {
		Layout::LeftAlign(Box::new(Layout::StaticText(
			text,
			Font::Smallest,
			TokenType::Text,
		)))
	}
}

pub struct MenuRenderCache {
	initial_render: bool,
	rendered_selection: Option<usize>,
}

pub struct Menu {
	title: String,
	items: Vec<MenuItem>,
	bottom: Option<Box<dyn Fn(&State, &dyn Screen) -> Layout>>,
	selection: usize,
	columns: usize,
	cache: RefCell<MenuRenderCache>,
}

impl Menu {
	pub fn new(title: &str, items: Vec<MenuItem>) -> Self {
		Menu {
			title: title.to_string(),
			items,
			bottom: None,
			selection: 0,
			columns: 1,
			cache: RefCell::new(MenuRenderCache {
				initial_render: true,
				rendered_selection: None,
			}),
		}
	}

	pub fn new_with_bottom(
		title: &str,
		items: Vec<MenuItem>,
		bottom: Box<dyn Fn(&State, &dyn Screen) -> Layout>,
	) -> Self {
		Menu {
			title: title.to_string(),
			items,
			bottom: Some(bottom),
			selection: 0,
			columns: 1,
			cache: RefCell::new(MenuRenderCache {
				initial_render: true,
				rendered_selection: None,
			}),
		}
	}

	pub fn set_columns(&mut self, cols: usize) {
		self.columns = cols;
	}

	pub fn up(&mut self) {
		self.selection = if self.selection == 0 {
			self.items.len() - 1
		} else {
			self.selection - 1
		};
	}

	pub fn down(&mut self) {
		self.selection = if (self.selection + 1) >= self.items.len() {
			0
		} else {
			self.selection + 1
		};
	}

	pub fn selected_function(&self) -> MenuItemFunction {
		self.items[self.selection].function.clone()
	}

	pub fn specific_function(&mut self, idx: usize) -> Option<MenuItemFunction> {
		if let Some(item) = self.items.get(idx) {
			self.selection = idx;
			Some(item.function.clone())
		} else {
			None
		}
	}

	pub fn set_selection(&mut self, idx: usize) {
		if idx < self.items.len() {
			self.selection = idx;
		}
	}

	pub fn force_refresh(&self) {
		self.cache.borrow_mut().initial_render = true;
	}

	pub fn render(&self, state: &State, screen: &mut dyn Screen) {
		let initial_render = self.cache.borrow().initial_render;
		let rendered_selection = self.cache.borrow().rendered_selection;

		if initial_render {
			// On initial render, clear screen and draw title
			screen.clear();

			let screen_rect = screen.screen_rect();
			let mut renderer = screen.renderer(RenderMode::StatusBar);
			renderer.erase(&Rect {
				x: 0,
				y: 0,
				w: screen_rect.w,
				h: renderer.metrics().height(Font::Small),
			});
			renderer.draw_text(
				4,
				0,
				&self.title,
				Font::Small,
				TokenType::Text,
				&screen_rect,
			);

			// Draw bottom layout if present
			if let Some(bottom) = &self.bottom {
				let bottom = bottom(state, screen);
				let height = bottom.height(screen.metrics());
				let rect = Rect {
					x: 4,
					y: screen.height() - height,
					w: screen.width() - 8,
					h: height,
				};
				let mut renderer = screen.renderer(RenderMode::Normal);
				bottom.render(&mut renderer, rect.clone(), &rect);
			}
		}

		let rows = (self.items.len() + self.columns - 1) / self.columns;
		let col_width = screen.width() / self.columns as i32;

		let mut i = 0;
		let mut row = 0;
		let top = screen.metrics().height(Font::Small) + 3;
		let mut x = 0;
		let mut y = top;

		for item in &self.items {
			let layout = match &item.layout {
				MenuItemLayout::Static(layout) => Cow::Borrowed(layout),
				MenuItemLayout::Dynamic(func) => Cow::Owned(func(state, screen)),
			};

			let screen_rect = screen.screen_rect();
			let mut renderer = screen.renderer(if i == self.selection {
				RenderMode::Selected
			} else {
				RenderMode::Normal
			});

			// Get height of item
			let height = layout.height(renderer.metrics());

			// Render item if it has been updated
			if initial_render
				|| i == self.selection && Some(i) != rendered_selection
				|| Some(i) == rendered_selection
			{
				// Get label for item
				let label = match i + 1 {
					1..=9 => Number::Integer((i + 1).into()).to_string(),
					10 => "0".to_string(),
					_ => {
						let mut string = String::new();
						string.push(char::from_u32('A' as u32 + i as u32 - 10).unwrap());
						string
					}
				} + ". ";

				// Render item background
				renderer.erase(&Rect {
					x: x,
					y,
					w: col_width,
					h: height,
				});

				// Render item label
				let label_width = renderer.metrics().width(Font::Small, &label);
				renderer.draw_text(
					x + 4,
					y + (height / 2) - (renderer.metrics().height(Font::Small) / 2),
					&label,
					Font::Small,
					TokenType::Label,
					&screen_rect,
				);

				// Render item contents
				let rect = Rect {
					x: x + label_width,
					y,
					w: col_width - (label_width + 4),
					h: height,
				};
				layout.render(&mut renderer, rect.clone(), &rect);
			}

			i += 1;
			row += 1;
			y += height;

			if row >= rows {
				x += col_width;
				row = 0;
				y = top;
			}
		}

		screen.refresh();

		self.cache.borrow_mut().rendered_selection = Some(self.selection);
		self.cache.borrow_mut().initial_render = false;
	}
}

pub fn setup_menu() -> Menu {
	let mut items = Vec::new();

	// Create setup menu items
	items.push(MenuItem {
		layout: MenuItemLayout::Static(MenuItem::static_string_layout("Display Settings >")),
		function: MenuItemFunction::InMenuAction(Function::SettingsMenu),
	});

	#[cfg(feature = "dm42")]
	items.push(MenuItem {
		layout: MenuItemLayout::Static(MenuItem::static_string_layout("System Settings >")),
		function: MenuItemFunction::Action(Function::SystemMenu),
	});

	// Return the menu object
	Menu::new_with_bottom(
		"Setup",
		items,
		Box::new(|_state, _screen| {
			// Create memory usage indicator on bottom, start with text with bytes available
			let mut bottom_items = Vec::new();
			bottom_items.push(Layout::LeftAlign(Box::new(Layout::Text(
				"Memory: ".to_string()
					+ &Number::Integer(available_bytes().into()).to_string()
					+ " bytes available",
				Font::Small,
				TokenType::Text,
			))));

			// Add memory usage graph
			bottom_items.push(Layout::UsageGraph(
				used_bytes() - reclaimable_bytes(),
				reclaimable_bytes(),
				free_bytes(),
			));

			// Add legend for the graph
			let mut legend_items = Vec::new();
			legend_items.push(Layout::UsageGraphUsedLegend);
			legend_items.push(Layout::StaticText(
				" Used   ",
				Font::Smallest,
				TokenType::Text,
			));
			legend_items.push(Layout::UsageGraphReclaimableLegend);
			legend_items.push(Layout::StaticText(
				" Reclaimable   ",
				Font::Smallest,
				TokenType::Text,
			));
			legend_items.push(Layout::UsageGraphFreeLegend);
			legend_items.push(Layout::StaticText(" Free", Font::Smallest, TokenType::Text));
			bottom_items.push(Layout::LeftAlign(Box::new(Layout::Horizontal(
				legend_items,
			))));

			// Add temporary memory available
			#[cfg(feature = "dm42")]
			bottom_items.push(Layout::LeftAlign(Box::new(Layout::Text(
				Number::Integer(crate::dm42::device::sys_free_mem().into()).to_string()
					+ " bytes temporary memory",
				Font::Smallest,
				TokenType::Text,
			))));

			Layout::Vertical(bottom_items)
		}),
	)
}

pub fn settings_menu() -> Menu {
	let mut items = Vec::new();

	// Create settings menu items
	items.push(MenuItem {
		layout: MenuItemLayout::Dynamic(Box::new(|state, _screen| {
			MenuItem::string_layout(
				"Status Bar Text   ".to_string()
					+ match state.status_bar_left_display() {
						StatusBarLeftDisplayType::CurrentTime => "[Current Time]",
						StatusBarLeftDisplayType::FreeMemory => "[Free Memory]",
					},
			)
		})),
		function: MenuItemFunction::InMenuAction(Function::StatusBarLeftDisplayToggle),
	});

	items.push(MenuItem {
		layout: MenuItemLayout::Dynamic(Box::new(|state, _screen| {
			MenuItem::string_layout(
				"24-hour Clock   ".to_string()
					+ if state.context().format().time_24_hour {
						"[On]"
					} else {
						"[Off]"
					},
			)
		})),
		function: MenuItemFunction::InMenuAction(Function::Time24HourToggle),
	});

	items.push(MenuItem {
		layout: MenuItemLayout::Dynamic(Box::new(|state, _screen| {
			MenuItem::string_layout(
				"Stack Labels   ".to_string()
					+ if state.context().format().stack_xyz {
						"[x,y,z,4]"
					} else {
						"[1,2,3,4]"
					},
			)
		})),
		function: MenuItemFunction::InMenuAction(Function::StackLabelXYZToggle),
	});

	items.push(MenuItem {
		layout: MenuItemLayout::Dynamic(Box::new(|state, _screen| {
			MenuItem::string_layout(
				"Show Empty Soft Keys   ".to_string()
					+ if state.function_keys().show_empty() {
						"[On]"
					} else {
						"[Off]"
					},
			)
		})),
		function: MenuItemFunction::InMenuAction(Function::ShowEmptySoftKeyToggle),
	});

	items.push(MenuItem {
		layout: MenuItemLayout::Dynamic(Box::new(|state, _screen| {
			MenuItem::string_layout(
				"Show Status Bar   ".to_string()
					+ if state.status_bar_enabled() {
						"[On]"
					} else {
						"[Off]"
					},
			)
		})),
		function: MenuItemFunction::InMenuAction(Function::StatusBarToggle),
	});

	items.push(MenuItem {
		layout: MenuItemLayout::Dynamic(Box::new(|state, _screen| {
			MenuItem::string_layout(
				"Font Size   ".to_string()
					+ match state.base_font() {
						Font::Smallest => "[Smallest]",
						Font::Small => "[Small]",
						Font::Medium => "[Medium]",
						Font::Large => "[Large]",
					},
			)
		})),
		function: MenuItemFunction::InMenuAction(Function::FontSizeToggle),
	});

	items.push(MenuItem {
		layout: MenuItemLayout::Dynamic(Box::new(|state, _screen| {
			MenuItem::string_layout(
				"Alternate Display   ".to_string()
					+ match state.context().format().alt_mode {
						AlternateFormatMode::Smart => "[Smart]",
						AlternateFormatMode::Bottom => "[Bottom]",
						AlternateFormatMode::Left => "[Left]",
					},
			)
		})),
		function: MenuItemFunction::InMenuAction(Function::AlternateFormatModeToggle),
	});

	// Return the menu object
	Menu::new("Settings", items)
}
