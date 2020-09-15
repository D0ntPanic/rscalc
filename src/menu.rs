use crate::font::{SANS_13, SANS_16};
use crate::functions::Function;
use crate::layout::Layout;
use crate::number::Number;
use crate::screen::{Color, Rect, Screen};
use crate::storage::{available_bytes, free_bytes, reclaimable_bytes, used_bytes};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub struct MenuItem {
	pub layout: Layout,
	pub function: Function,
}

impl MenuItem {
	pub fn string_layout(text: String) -> Layout {
		Layout::LeftAlign(Box::new(Layout::Text(text, &SANS_16, Color::ContentText)))
	}
}

pub struct Menu {
	title: String,
	items: Vec<MenuItem>,
	bottom: Option<Layout>,
	selection: usize,
	initial_render: bool,
	rendered_selection: Option<usize>,
}

impl Menu {
	pub fn new(title: String, items: Vec<MenuItem>) -> Self {
		Menu {
			title,
			items,
			bottom: None,
			selection: 0,
			initial_render: true,
			rendered_selection: None,
		}
	}

	pub fn new_with_bottom(title: String, items: Vec<MenuItem>, bottom: Layout) -> Self {
		Menu {
			title,
			items,
			bottom: Some(bottom),
			selection: 0,
			initial_render: true,
			rendered_selection: None,
		}
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

	pub fn selected_function(&self) -> Function {
		self.items[self.selection].function
	}

	pub fn specific_function(&mut self, idx: usize) -> Option<Function> {
		if let Some(item) = self.items.get(idx) {
			self.selection = idx;
			Some(item.function)
		} else {
			None
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

		let mut i = 0;
		let mut y = SANS_16.height + 3;

		for item in &self.items {
			// Get height of item
			let height = item.layout.height();

			// Render item if it has been updated
			if self.initial_render
				|| i == self.selection && Some(i) != self.rendered_selection
				|| Some(i) == self.rendered_selection
			{
				// Get label for item
				let label = Number::Integer((i + 1).into()).to_str() + ". ";

				// Render item background
				screen.fill(
					Rect {
						x: 0,
						y,
						w: screen.width(),
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
					4,
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
					x: label_width,
					y,
					w: screen.width() - (label_width + 4),
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
			y += height;
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
		layout: MenuItem::string_layout("System Settings >".to_string()),
		function: Function::SystemMenu,
	});

	// Create memory usage indicator on bottom, start with text with bytes available
	let mut bottom_items = Vec::new();
	bottom_items.push(Layout::LeftAlign(Box::new(Layout::Text(
		"Memory: ".to_string()
			+ &Number::Integer(available_bytes().into()).to_str()
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
	legend_items.push(Layout::Text(
		" Used   ".to_string(),
		&SANS_13,
		Color::ContentText,
	));
	legend_items.push(Layout::UsageGraphReclaimableLegend);
	legend_items.push(Layout::Text(
		" Reclaimable   ".to_string(),
		&SANS_13,
		Color::ContentText,
	));
	legend_items.push(Layout::UsageGraphFreeLegend);
	legend_items.push(Layout::Text(
		" Free".to_string(),
		&SANS_13,
		Color::ContentText,
	));
	bottom_items.push(Layout::LeftAlign(Box::new(Layout::Horizontal(
		legend_items,
	))));

	// Add temporary memory available
	#[cfg(feature = "dm42")]
	bottom_items.push(Layout::LeftAlign(Box::new(Layout::Text(
		Number::Integer(crate::dm42::sys_free_mem().into()).to_str() + " bytes temporary memory",
		&SANS_13,
		Color::ContentText,
	))));

	// Return the menu object
	Menu::new_with_bottom("Setup".to_string(), items, Layout::Vertical(bottom_items))
}
