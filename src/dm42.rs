use crate::calc_main;
use crate::font;
use crate::input::{InputQueue, Key, KeyEvent};
use crate::screen::{Color, Rect, Screen};
use alloc::alloc::Layout;
use core::alloc::GlobalAlloc;

struct Heap;

const LIBRARY_BASE: usize = 0x8000201;

const WIDTH: i32 = 400;
const HEIGHT: i32 = 240;

const BLT_OR: i32 = 0;
const BLT_ANDN: i32 = 1;
const BLT_NONE: i32 = 0;

const STAT_RUNNING: u32 = 1 << 1;
const STAT_SUSPENDED: u32 = 1 << 2;
const STAT_OFF: u32 = 1 << 4;
const STAT_PGM_END: u32 = 1 << 9;

extern "C" {
	#[link_name = "post_main"]
	fn post_main() -> !;
	#[link_name = "system_setup_menu"]
	fn system_setup_menu();
}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
	let mut screen = DM42Screen;
	screen.clear();
	font::SANS_24.draw(
		&mut screen,
		2,
		2,
		"Internal error - Panic",
		Color::ContentText,
	);
	font::SANS_16.draw(
		&mut screen,
		2,
		2 + font::SANS_24.height,
		"Press a key to restart...",
		Color::ContentText,
	);
	screen.refresh();
	wait_for_key_press();
	unsafe {
		post_main();
	}
}

#[alloc_error_handler]
fn alloc_error_handler(_layout: Layout) -> ! {
	let mut screen = DM42Screen;
	screen.clear();
	font::SANS_24.draw(&mut screen, 2, 2, "Out of memory", Color::ContentText);
	font::SANS_16.draw(
		&mut screen,
		2,
		2 + font::SANS_24.height,
		"Unhandled memory allocation error.",
		Color::ContentText,
	);
	font::SANS_16.draw(
		&mut screen,
		2,
		2 + font::SANS_24.height + font::SANS_16.height * 2,
		"Press a key to restart...",
		Color::ContentText,
	);
	screen.refresh();
	wait_for_key_press();
	unsafe {
		post_main();
	}
}

#[no_mangle]
pub fn __aeabi_unwind_cpp_pr0() -> ! {
	let mut screen = DM42Screen;
	screen.clear();
	font::SANS_24.draw(&mut screen, 2, 2, "Internal error", Color::ContentText);
	font::SANS_16.draw(
		&mut screen,
		2,
		2 + font::SANS_24.height,
		"Unhandled C++ exception.",
		Color::ContentText,
	);
	font::SANS_16.draw(
		&mut screen,
		2,
		2 + font::SANS_24.height + font::SANS_16.height * 2,
		"Press a key to restart...",
		Color::ContentText,
	);
	screen.refresh();
	wait_for_key_press();
	unsafe {
		post_main();
	}
}

#[no_mangle]
// This is missing from the linked C library but it can be provided by Rust
pub extern "C" fn __aeabi_d2f(value: f64) -> f32 {
	value as f32
}

#[global_allocator]
static ALLOCATOR: Heap = Heap;

unsafe impl GlobalAlloc for Heap {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		let func_ptr: usize = LIBRARY_BASE + 0;
		let malloc: extern "C" fn(size: usize) -> *mut u8 = core::mem::transmute(func_ptr);
		malloc(layout.size())
	}

	unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
		let func_ptr: usize = LIBRARY_BASE + 4;
		let free: extern "C" fn(ptr: *mut u8) -> *mut u8 = core::mem::transmute(func_ptr);
		free(ptr);
	}
}

#[allow(non_snake_case)]
fn LCD_power_on() {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 24;
		let func: extern "C" fn() = core::mem::transmute(func_ptr);
		func();
	}
}

#[allow(non_snake_case)]
fn LCD_power_off(clear: i32) {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 28;
		let func: extern "C" fn(i32) = core::mem::transmute(func_ptr);
		func(clear);
	}
}

fn bitblt24(x: u32, dx: u32, y: u32, val: u32, blt_op: i32, fill: i32) {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 36;
		let func: extern "C" fn(x: u32, dx: u32, y: u32, val: u32, blt_op: i32, fill: i32) =
			core::mem::transmute(func_ptr);
		func(x, dx, y, val, blt_op, fill);
	}
}

fn lcd_clear_buf() {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 44;
		let func: extern "C" fn() = core::mem::transmute(func_ptr);
		func();
	}
}

fn lcd_refresh() {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 48;
		let func: extern "C" fn() = core::mem::transmute(func_ptr);
		func();
	}
}

fn lcd_forced_refresh() {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 52;
		let func: extern "C" fn() = core::mem::transmute(func_ptr);
		func();
	}
}

fn lcd_fill_rect(x: u32, y: u32, dx: u32, dy: u32, val: i32) {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 60;
		let func: extern "C" fn(x: u32, y: u32, dx: u32, dy: u32, val: i32) =
			core::mem::transmute(func_ptr);
		func(x, y, dx, dy, val);
	}
}

fn lcd_set_buf_cleared(val: i32) {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 84;
		let func: extern "C" fn(i32) = core::mem::transmute(func_ptr);
		func(val);
	}
}

fn lcd_get_buf_cleared() -> bool {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 88;
		let func: extern "C" fn() -> i32 = core::mem::transmute(func_ptr);
		func() != 0
	}
}

fn rtc_wakeup_delay() {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 228;
		let func: extern "C" fn() = core::mem::transmute(func_ptr);
		func();
	}
}

pub fn read_power_voltage() -> u32 {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 232;
		let func: extern "C" fn() -> u32 = core::mem::transmute(func_ptr);
		func()
	}
}

pub fn usb_powered() -> bool {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 288;
		let func: extern "C" fn() -> i32 = core::mem::transmute(func_ptr);
		func() != 0
	}
}

fn key_empty() -> bool {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 380;
		let func: extern "C" fn() -> i32 = core::mem::transmute(func_ptr);
		func() != 0
	}
}

fn key_pop() -> i32 {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 392;
		let func: extern "C" fn() -> i32 = core::mem::transmute(func_ptr);
		func()
	}
}

fn wait_for_key_press() {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 408;
		let func: extern "C" fn() = core::mem::transmute(func_ptr);
		func();
	}
}

fn reset_auto_off() {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 440;
		let func: extern "C" fn() = core::mem::transmute(func_ptr);
		func();
	}
}

fn sys_sleep() {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 536;
		let func: extern "C" fn() = core::mem::transmute(func_ptr);
		func();
	}
}

fn draw_power_off_image(val: i32) {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 556;
		let func: extern "C" fn(i32) = core::mem::transmute(func_ptr);
		func(val);
	}
}

fn state(bit: u32) -> bool {
	unsafe {
		let ptr = 0x10002000usize as *const u32;
		core::ptr::read_volatile(ptr) & bit != 0
	}
}

fn set_state(bit: u32) {
	unsafe {
		let ptr = 0x10002000usize as *mut u32;
		core::ptr::write_volatile(ptr, core::ptr::read_volatile(ptr) | bit);
	}
}

fn clear_state(bit: u32) {
	unsafe {
		let ptr = 0x10002000usize as *mut u32;
		core::ptr::write_volatile(ptr, core::ptr::read_volatile(ptr) & !bit);
	}
}

pub fn show_system_setup_menu() {
	unsafe {
		system_setup_menu();
		lcd_clear_buf();
	}
}

pub struct DM42Screen;

impl Screen for DM42Screen {
	fn width(&self) -> i32 {
		WIDTH
	}

	fn height(&self) -> i32 {
		HEIGHT
	}

	fn clear(&mut self) {
		lcd_clear_buf();
	}

	fn refresh(&mut self) {
		lcd_refresh();
	}

	fn fill(&mut self, rect: Rect, color: Color) {
		let rect = rect.clipped_to_screen(self);
		let color = color.to_bw();
		lcd_fill_rect(
			rect.x as u32,
			rect.y as u32,
			rect.w as u32,
			rect.h as u32,
			if color { 1 } else { 0 },
		);
	}

	fn draw_bits(&mut self, x: i32, y: i32, bits: u32, width: u8, color: Color) {
		let color = color.to_bw();
		if color {
			bitblt24(x as u32, width as u32, y as u32, bits, BLT_OR, BLT_NONE);
		} else {
			bitblt24(x as u32, width as u32, y as u32, bits, BLT_ANDN, BLT_NONE);
		}
	}
}

pub struct DM42InputQueue;

impl InputQueue for DM42InputQueue {
	fn has_input(&self) -> bool {
		!key_empty()
	}

	fn pop_raw(&mut self) -> Option<KeyEvent> {
		if key_empty() {
			None
		} else {
			let key = key_pop();
			if key > 0 {
				let key = match key {
					1 => Key::Sigma,
					2 => Key::Recip,
					3 => Key::Sqrt,
					4 => Key::Log,
					5 => Key::Ln,
					6 => Key::Xeq,
					7 => Key::Sto,
					8 => Key::Rcl,
					9 => Key::RotateDown,
					10 => Key::Sin,
					11 => Key::Cos,
					12 => Key::Tan,
					13 => Key::Enter,
					14 => Key::Swap,
					15 => Key::Neg,
					16 => Key::E,
					17 => Key::Backspace,
					18 => Key::Up,
					19 => Key::Seven,
					20 => Key::Eight,
					21 => Key::Nine,
					22 => Key::Div,
					23 => Key::Down,
					24 => Key::Four,
					25 => Key::Five,
					26 => Key::Six,
					27 => Key::Mul,
					28 => Key::Shift,
					29 => Key::One,
					30 => Key::Two,
					31 => Key::Three,
					32 => Key::Sub,
					33 => Key::Exit,
					34 => Key::Zero,
					35 => Key::Dot,
					36 => Key::Run,
					37 => Key::Add,
					38 => Key::F1,
					39 => Key::F2,
					40 => Key::F3,
					41 => Key::F4,
					42 => Key::F5,
					43 => Key::F6,
					44 => Key::Screenshot,
					45 => Key::ShiftUp,
					46 => Key::ShiftDown,
					99 => Key::DoubleRelease,
					_ => return None,
				};
				Some(KeyEvent::Press(key))
			} else if key == 0 {
				Some(KeyEvent::Release)
			} else {
				None
			}
		}
	}

	fn wait_raw(&mut self) -> KeyEvent {
		if let Some(key) = self.pop_raw() {
			reset_auto_off();
			return key;
		}

		loop {
			if (state(STAT_PGM_END) && state(STAT_SUSPENDED))
				|| (!state(STAT_PGM_END) && key_empty())
			{
				clear_state(STAT_RUNNING);
				sys_sleep();
			}

			if state(STAT_PGM_END) || state(STAT_SUSPENDED) {
				if !state(STAT_SUSPENDED) {
					lcd_set_buf_cleared(0);
					draw_power_off_image(1);
					LCD_power_off(0);
					set_state(STAT_SUSPENDED);
					set_state(STAT_OFF);
				}
				continue;
			}

			set_state(STAT_RUNNING);
			clear_state(STAT_SUSPENDED);

			if state(STAT_OFF) {
				LCD_power_on();
				rtc_wakeup_delay();
				clear_state(STAT_OFF);
				if !lcd_get_buf_cleared() {
					lcd_forced_refresh();
				}
			}

			if let Some(key) = self.pop_raw() {
				reset_auto_off();
				return key;
			}
		}
	}

	fn suspend(&self) {
		set_state(STAT_PGM_END);
	}
}

#[no_mangle]
pub extern "C" fn program_main() {
	let screen = DM42Screen;
	let input_queue = DM42InputQueue;
	calc_main(screen, input_queue);
}
