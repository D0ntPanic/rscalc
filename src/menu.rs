use crate::font::{SANS_13, SANS_16};
use crate::functions::Function;
use crate::layout::Layout;
use crate::number::Number;
use crate::screen::{Color, Rect, Screen};
use crate::state::{State, StatusBarLeftDisplayType};
use crate::storage::{available_bytes, free_bytes, reclaimable_bytes, used_bytes};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;

#[cfg(feature = "dm42")]
use crate::dm42::time_24_hour;
#[cfg(not(feature = "dm42"))]
use crate::time::time_24_hour;

#[derive(PartialEq, Eq, Clone, Copy)]
#[allow(dead_code)]
pub enum MenuItemFunction {
	Action(Function),
	InMenuAction(Function),
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
		Layout::LeftAlign(Box::new(Layout::Text(text, &SANS_16, Color::ContentText)))
	}

	pub fn string_layout_small(text: String) -> Layout {
		Layout::LeftAlign(Box::new(Layout::Text(text, &SANS_13, Color::ContentText)))
	}

	pub fn static_string_layout(text: &'static str) -> Layout {
		Layout::LeftAlign(Box::new(Layout::StaticText(
			text,
			&SANS_16,
			Color::ContentText,
		)))
	}

	pub fn static_string_layout_small(text: &'static str) -> Layout {
		Layout::LeftAlign(Box::new(Layout::StaticText(
			text,
			&SANS_13,
			Color::ContentText,
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
		self.items[self.selection].function
	}

	pub fn specific_function(&mut self, idx: usize) -> Option<MenuItemFunction> {
		if let Some(item) = self.items.get(idx) {
			self.selection = idx;
			Some(item.function)
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

			screen.fill(
				Rect {
					x: 0,
					y: 0,
					w: screen.width(),
					h: SANS_16.height,
				},
				Color::StatusBarBackground,
			);
			SANS_16.draw(screen, 4, 0, &self.title, Color::StatusBarText);

			// Draw bottom layout if present
			if let Some(bottom) = &self.bottom {
				let bottom = bottom(state, screen);
				let height = bottom.height();
				let rect = Rect {
					x: 4,
					y: screen.height() - height,
					w: screen.width() - 8,
					h: height,
				};
				bottom.render(screen, rect.clone(), &rect, None);
			}
		}

		let rows = (self.items.len() + self.columns - 1) / self.columns;
		let col_width = screen.width() / self.columns as i32;

		let mut i = 0;
		let mut row = 0;
		let top = SANS_16.height + 3;
		let mut x = 0;
		let mut y = top;

		for item in &self.items {
			let layout = match &item.layout {
				MenuItemLayout::Static(layout) => Cow::Borrowed(layout),
				MenuItemLayout::Dynamic(func) => Cow::Owned(func(state, screen)),
			};

			// Get height of item
			let height = layout.height();

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
				screen.fill(
					Rect {
						x: x,
						y,
						w: col_width,
						h: height,
					},
					if i == self.selection {
						Color::SelectionBackground
					} else {
						Color::ContentBackground
					},
				);

				// Render item label
				let label_width = SANS_16.width(&label);
				SANS_16.draw(
					screen,
					x + 4,
					y + (height / 2) - (SANS_16.height / 2),
					&label,
					if i == self.selection {
						Color::SelectionText
					} else {
						Color::ContentText
					},
				);

				// Render item contents
				let rect = Rect {
					x: x + label_width,
					y,
					w: col_width - (label_width + 4),
					h: height,
				};
				layout.render(
					screen,
					rect.clone(),
					&rect,
					if i == self.selection {
						Some(Color::SelectionText)
					} else {
						None
					},
				);
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
				&SANS_16,
				Color::ContentText,
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
			legend_items.push(Layout::StaticText(" Used   ", &SANS_13, Color::ContentText));
			legend_items.push(Layout::UsageGraphReclaimableLegend);
			legend_items.push(Layout::StaticText(
				" Reclaimable   ",
				&SANS_13,
				Color::ContentText,
			));
			legend_items.push(Layout::UsageGraphFreeLegend);
			legend_items.push(Layout::StaticText(" Free", &SANS_13, Color::ContentText));
			bottom_items.push(Layout::LeftAlign(Box::new(Layout::Horizontal(
				legend_items,
			))));

			// Add temporary memory available
			#[cfg(feature = "dm42")]
			bottom_items.push(Layout::LeftAlign(Box::new(Layout::Text(
				Number::Integer(crate::dm42::sys_free_mem().into()).to_string()
					+ " bytes temporary memory",
				&SANS_13,
				Color::ContentText,
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
		layout: MenuItemLayout::Dynamic(Box::new(|_state, _screen| {
			MenuItem::string_layout(
				"24-hour Clock   ".to_string() + if time_24_hour() { "[On]" } else { "[Off]" },
			)
		})),
		function: MenuItemFunction::InMenuAction(Function::Time24HourToggle),
	});

	// Return the menu object
	Menu::new("Settings", items)
}
