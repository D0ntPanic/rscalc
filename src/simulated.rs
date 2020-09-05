use crate::calc_main;
use crate::input::{InputQueue, Key, KeyEvent};
use crate::screen::{Color, Rect, Screen};
use gdk_pixbuf::{Colorspace, Pixbuf};
use glib::source::{timeout_add_local, Continue};
use gtk::*;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

const WIDTH: i32 = 400;
const HEIGHT: i32 = 240;
const WIDTH_BYTES: usize = WIDTH as usize / 8;

pub struct Refresh {
	screen: Option<VirtualDM42Screen>,
}

pub struct App {
	window: Window,
}

struct Content {
	container: Box,
	image: Image,
}

impl App {
	fn new() -> App {
		let refresh = Arc::new(Mutex::new(Refresh { screen: None }));
		let input_queue = Arc::new(Mutex::new(Vec::new()));
		let input_event = Arc::new(Condvar::new());
		let screen = VirtualDM42Screen::new(refresh.clone());
		let input = VirtualInputQueue::new(input_queue.clone(), input_event.clone());
		let content = Content::new(&screen, input_queue, input_event);
		thread::spawn(move || {
			calc_main(screen, input);
			std::process::exit(0);
		});

		let window = Window::new(WindowType::Toplevel);

		window.set_title("Calc");
		window.add(&content.container);

		window.connect_delete_event(move |_, _| {
			main_quit();
			Inhibit(false)
		});

		let timeout_refresh = refresh.clone();
		timeout_add_local(33, move || {
			let mut refresh = timeout_refresh.lock().unwrap();
			if let Some(screen) = &refresh.screen {
				let pixbuf = screen.to_pixbuf();
				content.image.set_from_pixbuf(Some(&pixbuf));
				refresh.screen = None;
			}
			Continue(true)
		});

		App { window }
	}

	pub fn run() {
		if gtk::init().is_err() {
			eprintln!("failed to initialize GTK Application");
			std::process::exit(1);
		}

		let app = App::new();
		app.window.show_all();
		gtk::main();
	}
}

impl Content {
	fn new(
		screen: &VirtualDM42Screen,
		input_queue: Arc<Mutex<Vec<KeyEvent>>>,
		input_event: Arc<Condvar>,
	) -> Content {
		let container = Box::new(Orientation::Vertical, 0);
		let image = Image::new();
		let pixbuf = screen.to_pixbuf();
		image.set_from_pixbuf(Some(&pixbuf));

		container.pack_start(&image, false, false, 0);

		let keyboard_top = Grid::new();
		keyboard_top.set_column_spacing(2);
		keyboard_top.set_row_spacing(2);
		keyboard_top.set_margin_start(8);
		keyboard_top.set_margin_end(8);
		keyboard_top.set_margin_top(8);

		let keyboard_bottom = Grid::new();
		keyboard_bottom.set_column_spacing(2);
		keyboard_bottom.set_row_spacing(2);
		keyboard_bottom.set_margin_start(8);
		keyboard_bottom.set_margin_end(8);
		keyboard_bottom.set_margin_top(4);
		keyboard_bottom.set_margin_bottom(8);

		macro_rules! f_key {
			($grid: expr, $x: expr, $y: expr, $key: expr) => {
				let key = Button::with_label("");
				$grid.attach(&key, $x, $y, 1, 1);
				let button_input_queue = input_queue.clone();
				let button_input_event = input_event.clone();
				key.connect_clicked(move |_| {
					let mut queue = button_input_queue.lock().unwrap();
					queue.push(KeyEvent::Press($key));
					queue.push(KeyEvent::Release);
					button_input_event.notify_one();
					});
			};
		}

		f_key!(keyboard_top, 0, 0, Key::F1);
		f_key!(keyboard_top, 2, 0, Key::F2);
		f_key!(keyboard_top, 4, 0, Key::F3);
		f_key!(keyboard_top, 6, 0, Key::F4);
		f_key!(keyboard_top, 8, 0, Key::F5);
		f_key!(keyboard_top, 10, 0, Key::F6);

		macro_rules! key {
			($grid: expr, $button:expr, $shift:expr, $alpha: expr, $x: expr, $y: expr, $span: expr, $key: expr) => {
				let key_lbl = Label::new(Some($shift));
				let key = Button::with_label($button);
				let a = Label::new(Some($alpha));
				key.set_hexpand(true);
				key_lbl.set_margin_top(4);
				a.set_margin_end(4);
				let button_input_queue = input_queue.clone();
				let button_input_event = input_event.clone();
				key.connect_clicked(move |_| {
					let mut queue = button_input_queue.lock().unwrap();
					queue.push(KeyEvent::Press($key));
					queue.push(KeyEvent::Release);
					button_input_event.notify_one();
					});
				$grid.attach(&key_lbl, $x * 2, $y, $span, 1);
				$grid.attach(&key, $x * 2, $y + 1, $span, 1);
				$grid.attach(&a, $x * 2 + $span, $y + 1, $span, 1);
			};
		}

		key!(keyboard_top, "∑+", "∑-", "A", 0, 1, 1, Key::Sigma);
		key!(keyboard_top, "1/x", "y^x", "B", 1, 1, 1, Key::Recip);
		key!(keyboard_top, "sqrt", "x^2", "C", 2, 1, 1, Key::Sqrt);
		key!(keyboard_top, "log", "10^x", "D", 3, 1, 1, Key::Log);
		key!(keyboard_top, "ln", "e^x", "E", 4, 1, 1, Key::Ln);
		key!(keyboard_top, "XEQ", "GTO", "F", 5, 1, 1, Key::Xeq);

		key!(keyboard_top, "STO", "CPLX", "G", 0, 3, 1, Key::Sto);
		key!(keyboard_top, "RCL", "%", "H", 1, 3, 1, Key::Rcl);
		key!(keyboard_top, "R↓", "pi", "I", 2, 3, 1, Key::RotateDown);
		key!(keyboard_top, "sin", "asin", "J", 3, 3, 1, Key::Sin);
		key!(keyboard_top, "cos", "acos", "K", 4, 3, 1, Key::Cos);
		key!(keyboard_top, "tan", "atan", "L", 5, 3, 1, Key::Tan);

		key!(keyboard_top, "ENTER", "ALPHA", "", 0, 5, 3, Key::Enter);
		key!(keyboard_top, "x⇋y", "LAST x", "M", 2, 5, 1, Key::Swap);
		key!(keyboard_top, "+/-", "MODES", "N", 3, 5, 1, Key::Neg);
		key!(keyboard_top, "E", "DISP", "O", 4, 5, 1, Key::E);
		key!(keyboard_top, "←", "CLEAR", "", 5, 5, 1, Key::Backspace);

		key!(keyboard_bottom, "▲", "BST", "", 0, 0, 1, Key::Up);
		key!(keyboard_bottom, "7", "SOLVE", "P", 1, 0, 1, Key::Seven);
		key!(keyboard_bottom, "8", "INTEG", "Q", 2, 0, 1, Key::Eight);
		key!(keyboard_bottom, "9", "MATRIX", "R", 3, 0, 1, Key::Nine);
		key!(keyboard_bottom, "÷", "STAT", "S", 4, 0, 1, Key::Div);

		key!(keyboard_bottom, "▼", "SST", "", 0, 2, 1, Key::Down);
		key!(keyboard_bottom, "4", "BASE", "T", 1, 2, 1, Key::Four);
		key!(keyboard_bottom, "5", "CNVRT", "U", 2, 2, 1, Key::Five);
		key!(keyboard_bottom, "6", "FLAGS", "V", 3, 2, 1, Key::Six);
		key!(keyboard_bottom, "×", "PROB", "W", 4, 2, 1, Key::Mul);

		key!(keyboard_bottom, "SHIFT", "", "", 0, 4, 1, Key::Shift);
		key!(keyboard_bottom, "1", "ASSIGN", "X", 1, 4, 1, Key::One);
		key!(keyboard_bottom, "2", "CUSTOM", "Y", 2, 4, 1, Key::Two);
		key!(keyboard_bottom, "3", "PG.FCN", "Z", 3, 4, 1, Key::Three);
		key!(keyboard_bottom, "-", "PRINT", "-", 4, 4, 1, Key::Sub);

		key!(keyboard_bottom, "EXIT", "OFF", "", 0, 6, 1, Key::Exit);
		key!(keyboard_bottom, "0", "SETUP", ":", 1, 6, 1, Key::Zero);
		key!(keyboard_bottom, ".", "SHOW", ".", 2, 6, 1, Key::Dot);
		key!(keyboard_bottom, "R/S", "PRGM", "?", 3, 6, 1, Key::Run);
		key!(keyboard_bottom, "+", "CATLG", "⎵", 4, 6, 1, Key::Add);

		container.pack_start(&keyboard_top, false, false, 0);
		container.pack_start(&keyboard_bottom, false, false, 0);

		Content { container, image }
	}
}

#[derive(Clone)]
pub struct VirtualDM42Screen {
	bitmap: [u8; WIDTH_BYTES * HEIGHT as usize],
	refresh: Arc<Mutex<Refresh>>,
}

impl VirtualDM42Screen {
	pub fn new(refresh: Arc<Mutex<Refresh>>) -> Self {
		VirtualDM42Screen {
			bitmap: [0; WIDTH_BYTES * HEIGHT as usize],
			refresh,
		}
	}

	fn pixel(&self, x: i32, y: i32) -> bool {
		if x < 0 || x >= WIDTH || y < 0 || y >= HEIGHT {
			return false;
		}
		self.bitmap[y as usize * WIDTH_BYTES + (x as usize / 8)] & (1 << (x & 7)) != 0
	}

	fn set_pixel(&mut self, x: i32, y: i32, color: bool) {
		if x < 0 || x >= WIDTH || y < 0 || y >= HEIGHT {
			return;
		}
		if color {
			self.bitmap[y as usize * WIDTH_BYTES + (x as usize / 8)] |= 1 << (x & 7);
		} else {
			self.bitmap[y as usize * WIDTH_BYTES + (x as usize / 8)] &= !(1 << (x & 7));
		}
	}

	fn to_pixbuf(&self) -> Pixbuf {
		let pixbuf = Pixbuf::new(Colorspace::Rgb, false, 8, WIDTH, HEIGHT).unwrap();

		for y in 0..HEIGHT {
			for x in 0..WIDTH {
				if self.pixel(x, y) {
					pixbuf.put_pixel(x as u32, y as u32, 0, 0, 0, 255);
				} else {
					pixbuf.put_pixel(x as u32, y as u32, 255, 255, 255, 255);
				}
			}
		}

		pixbuf
	}
}

impl Screen for VirtualDM42Screen {
	fn width(&self) -> i32 {
		WIDTH
	}

	fn height(&self) -> i32 {
		HEIGHT
	}

	fn clear(&mut self) {
		for i in 0..WIDTH_BYTES * HEIGHT as usize {
			self.bitmap[i] = 0;
		}
	}

	fn refresh(&mut self) {
		self.refresh.lock().unwrap().screen = Some(self.clone());
	}

	fn fill(&mut self, rect: Rect, color: Color) {
		let rect = rect.clipped_to_screen(self);
		let color = color.to_bw();
		for y in rect.y..rect.y + rect.h {
			for x in rect.x..rect.x + rect.w {
				self.set_pixel(x, y, color);
			}
		}
	}

	fn draw_bits(&mut self, x: i32, y: i32, bits: u32, width: u8, color: Color) {
		let color = color.to_bw();
		for i in 0..width {
			if bits & (1 << ((width - 1) - i)) != 0 {
				self.set_pixel(x + i as i32, y, color);
			}
		}
	}
}

pub struct VirtualInputQueue {
	queue: Arc<Mutex<Vec<KeyEvent>>>,
	event: Arc<Condvar>,
}

impl VirtualInputQueue {
	fn new(queue: Arc<Mutex<Vec<KeyEvent>>>, event: Arc<Condvar>) -> Self {
		VirtualInputQueue { queue, event }
	}
}

impl InputQueue for VirtualInputQueue {
	fn has_input(&self) -> bool {
		self.queue.lock().unwrap().len() != 0
	}

	fn pop_raw(&mut self) -> Option<KeyEvent> {
		let mut queue = self.queue.lock().unwrap();
		queue.pop()
	}

	fn wait_raw(&mut self) -> KeyEvent {
		let mut queue = self.queue.lock().unwrap();
		while queue.len() == 0 {
			queue = self.event.wait(queue).unwrap();
		}
		queue.pop().unwrap()
	}

	fn suspend(&self) {}
}
