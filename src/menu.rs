use crate::font::{SANS_13, SANS_16};
use crate::functions::Function;
use crate::layout::Layout;
use crate::number::Number;
use crate::screen::{Color, Rect, Screen};
use crate::state::{State, StatusBarLeftDisplayType};
use crate::storage::{available_bytes, free_bytes, reclaimable_bytes, used_bytes};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[cfg(feature = "dm42")]
use crate::dm42::time_24_hour;
#[cfg(not(feature = "dm42"))]
use crate::time::time_24_hour;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MenuItemFunction {
	Action(Function),
	ConversionAction(Function, Function, Function),
}

pub struct MenuItem {
	pub layout: Layout,
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

pub struct Menu {
	title: String,
	items: Vec<MenuItem>,
	bottom: Option<Layout>,
	selection: usize,
	initial_render: bool,
	rendered_selection: Option<usize>,
	columns: usize,
}

impl Menu {
	pub fn new(title: &str, items: Vec<MenuItem>) -> Self {
		Menu {
			title: title.to_string(),
			items,
			bottom: None,
			selection: 0,
			initial_render: true,
			rendered_selection: None,
			columns: 1,
		}
	}

	pub fn new_with_bottom(title: &str, items: Vec<MenuItem>, bottom: Layout) -> Self {
		Menu {
			title: title.to_string(),
			items,
			bottom: Some(bottom),
			selection: 0,
			initial_render: true,
			rendered_selection: None,
			columns: 1,
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

	pub fn selection(&self) -> usize {
		self.selection
	}

	pub fn set_selection(&mut self, idx: usize) {
		if idx < self.items.len() {
			self.selection = idx;
		}
	}

	pub fn force_refresh(&mut self) {
		self.initial_render = true;
	}

	pub fn render<ScreenT: Screen>(&mut self, screen: &mut ScreenT) {
		if self.initial_render {
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
			// Get height of item
			let height = item.layout.height();

			// Render item if it has been updated
			if self.initial_render
				|| i == self.selection && Some(i) != self.rendered_selection
				|| Some(i) == self.rendered_selection
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
				item.layout.render(
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

		self.rendered_selection = Some(self.selection);
		self.initial_render = false;
	}
}

pub fn setup_menu() -> Menu {
	let mut items = Vec::new();

	// Create setup menu items
	items.push(MenuItem {
		layout: MenuItem::static_string_layout("Display Settings >"),
		function: MenuItemFunction::Action(Function::SettingsMenu),
	});

	#[cfg(feature = "dm42")]
	items.push(MenuItem {
		layout: MenuItem::static_string_layout("System Settings >"),
		function: MenuItemFunction::Action(Function::SystemMenu),
	});

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
		Number::Integer(crate::dm42::sys_free_mem().into()).to_string() + " bytes temporary memory",
		&SANS_13,
		Color::ContentText,
	))));

	// Return the menu object
	Menu::new_with_bottom("Setup", items, Layout::Vertical(bottom_items))
}

pub fn settings_menu(state: &State) -> Menu {
	let mut items = Vec::new();

	// Create settings menu items
	items.push(MenuItem {
		layout: MenuItem::string_layout(
			"Status Bar Text   ".to_string()
				+ match state.status_bar_left_display() {
					StatusBarLeftDisplayType::CurrentTime => "[Current Time]",
					StatusBarLeftDisplayType::FreeMemory => "[Free Memory]",
				},
		),
		function: MenuItemFunction::Action(Function::StatusBarLeftDisplayToggle),
	});

	items.push(MenuItem {
		layout: MenuItem::string_layout(
			"24-hour Clock   ".to_string() + if time_24_hour() { "[On]" } else { "[Off]" },
		),
		function: MenuItemFunction::Action(Function::Time24HourToggle),
	});

	// Return the menu object
	Menu::new("Settings", items)
}
