use crate::dm42::calc_main;
use crate::dm42::font;
use crate::dm42::input::{InputQueue, Key, KeyEvent};
use crate::dm42::screen::{RenderMode, Screen, ScreenLayoutRenderer};
use alloc::alloc::Layout;
use core::alloc::GlobalAlloc;
use rscalc_layout::layout::Rect;
use rscalc_math::format::Format;
use spin::Mutex;

struct Heap;

const INTERFACE_MAJOR_VERSION: u32 = 3;
const INTERFACE_MINOR_VERSION: u32 = 13;
const QSPI_DATA_SIZE: u32 = 1370864;
const QSPI_DATA_CRC: u32 = 0x000cfed6;
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
const STAT_CLK_WKUP_ENABLE: u32 = 1 << 10;
const STAT_CLK_WKUP_SECONDS: u32 = 1 << 11;
const STAT_CLK_WKUP_FLAG: u32 = 1 << 12;
const STAT_CLK24: u32 = 1 << 14;
const STAT_POWER_CHANGE: u32 = 1 << 15;

const MENU_RESET: i32 = 0;
const MI_MSC: u8 = 196;
const MI_SYSTEM_ENTER: u8 = 200;
const MI_SET_TIME: u8 = 202;
const MI_SET_DATE: u8 = 203;

const PROG_INFO_MAGIC: u32 = 0xd377c0de;
const RUN_DMCP_MAGIC: u32 = 0x3ce7ea37;

extern "C" {
	static _sidata: u8;
	static mut _sdata: u8;
	static _edata: u8;
	static mut _sbss: u8;
	static _ebss: u8;
}

#[repr(C)]
struct ProgramInfo {
	magic: u32,
	size: u32,
	entry: unsafe extern "C" fn() -> !,
	interface_major_version: u32,
	interface_minor_version: u32,
	qspi_data_size: u32,
	qspi_data_crc: u32,
	program_name: [u8; 16],
	program_version: [u8; 16],
	keymap_id: u32,
}

// Program header used by DM42 firmware, linker script will map this to the start
// of the program file.
#[no_mangle]
#[allow(non_snake_case)]
static prog_info: ProgramInfo = ProgramInfo {
	magic: PROG_INFO_MAGIC,
	size: 0, // Filled in after linking
	entry: program_entry,
	interface_major_version: INTERFACE_MAJOR_VERSION,
	interface_minor_version: INTERFACE_MINOR_VERSION,
	qspi_data_size: QSPI_DATA_SIZE,
	qspi_data_crc: QSPI_DATA_CRC,
	program_name: *b"rscalc\0\0\0\0\0\0\0\0\0\0",
	program_version: *include!(concat!(env!("OUT_DIR"), "/version.txt")),
	keymap_id: 0xffffffff,
};

#[repr(C)]
struct Menu {
	name: &'static u8,
	items: &'static u8,
	message: Option<&'static u8>,
	post_display: Option<extern "C" fn()>,
}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
	let mut screen = DM42Screen;
	let screen_rect = screen.screen_rect();
	screen.clear();
	font::SANS_24.draw(
		&mut screen,
		&screen_rect,
		2,
		2,
		"Internal error - Panic",
		true,
	);
	font::SANS_16.draw(
		&mut screen,
		&screen_rect,
		2,
		2 + font::SANS_24.height,
		"Press a key to restart...",
		true,
	);
	lcd_forced_refresh();
	wait_for_key_press();
	reset();
}

#[alloc_error_handler]
fn alloc_error_handler(_layout: Layout) -> ! {
	let mut screen = DM42Screen;
	let screen_rect = screen.screen_rect();
	screen.clear();
	font::SANS_24.draw(&mut screen, &screen_rect, 2, 2, "Out of memory", true);
	font::SANS_16.draw(
		&mut screen,
		&screen_rect,
		2,
		2 + font::SANS_24.height,
		"Unhandled memory allocation error.",
		true,
	);
	font::SANS_16.draw(
		&mut screen,
		&screen_rect,
		2,
		2 + font::SANS_24.height + font::SANS_16.height * 2,
		"Press a key to restart...",
		true,
	);
	lcd_forced_refresh();
	wait_for_key_press();
	reset();
}

#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr0() -> ! {
	panic!("unhandled C++ exception");
}

#[no_mangle]
pub extern "C" fn raise(_sig: i32) -> ! {
	panic!("exception raised in external library");
}

// TODO: not needed anymore?
//#[no_mangle]
// This is missing from the linked C library but it can be provided by Rust
//pub extern "C" fn __aeabi_d2f(value: f64) -> f32 {
//	value as f32
//}

#[no_mangle]
pub unsafe extern "C" fn __errno() -> *mut i32 {
	// DM42 is single threaded, don't need a per-thread errno
	static mut ERROR: i32 = 0;
	&mut ERROR
}

#[no_mangle]
pub unsafe extern "C" fn sprintf(dest: *mut u8, fmt: *const u8, mut args: ...) -> usize {
	// Simple sprintf implementation that only supports basic integers. This is only called
	// from the floating point library with these format specifiers.
	let mut input = fmt;
	let mut output = dest;
	while *input != 0 {
		// Get next character in format string
		let ch = *input;
		input = input.offset(1);

		match ch {
			b'%' => {
				// Format specifier, grab type
				let ch = *input;
				input = input.offset(1);

				match ch {
					b'd' => {
						// Signed 32 bit integer as decimal
						let num: i32 = args.arg();
						let mut format = Format::new();
						format.thousands = false;
						let string = format.format_bigint(&num.into());
						core::slice::from_raw_parts_mut(output, string.len())
							.copy_from_slice(string.as_bytes());
						output = output.offset(string.len() as isize);
					}
					b'u' => {
						// Unsigned 32 bit integer as decimal
						let num: u32 = args.arg();
						let mut format = Format::new();
						format.thousands = false;
						let string = format.format_bigint(&num.into());
						core::slice::from_raw_parts_mut(output, string.len())
							.copy_from_slice(string.as_bytes());
						output = output.offset(string.len() as isize);
					}
					_ => {
						// Output unhandled character
						*output = *output;
						output = output.offset(1);
					}
				}
			}
			_ => {
				// Output character unmodified
				*output = *output;
				output = output.offset(1);
			}
		}
	}

	// Compute length and null terminate
	let len = output as usize - dest as usize;
	*output = 0;
	len
}

#[global_allocator]
static ALLOCATOR: Heap = Heap;

lazy_static! {
	static ref CLOCK_CHANGED: Mutex<bool> = Mutex::new(true);
}

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

fn lcd_refresh_dma() {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 644;
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

fn lcd_fill_lines(y: u32, value: u8, count: u32) {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 80;
		let func: extern "C" fn(y: u32, value: u8, count: u32) = core::mem::transmute(func_ptr);
		func(y, value, count);
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

pub fn rtc_updated() -> bool {
	*CLOCK_CHANGED.lock()
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

pub fn set_reset_magic(magic: u32) {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 272;
		let func: extern "C" fn(u32) = core::mem::transmute(func_ptr);
		func(magic)
	}
}

pub fn usb_powered() -> bool {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 288;
		let func: extern "C" fn() -> i32 = core::mem::transmute(func_ptr);
		func() != 0
	}
}

unsafe fn handle_menu(menu: &'static Menu, action: i32, line: i32) -> i32 {
	let func_ptr: usize = LIBRARY_BASE + 344;
	let func: extern "C" fn(&'static Menu, i32, i32) -> i32 = core::mem::transmute(func_ptr);
	func(menu, action, line)
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

pub fn sys_delay(ms_delay: u32) {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 516;
		let func: extern "C" fn(u32) = core::mem::transmute(func_ptr);
		func(ms_delay);
	}
}

fn sys_sleep() {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 536;
		let func: extern "C" fn() = core::mem::transmute(func_ptr);
		func();
	}
}

pub fn sys_free_mem() -> usize {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 540;
		let func: extern "C" fn() -> u32 = core::mem::transmute(func_ptr);
		func() as usize
	}
}

pub fn sys_reset() -> ! {
	unsafe {
		let func_ptr: usize = LIBRARY_BASE + 544;
		let func: extern "C" fn() -> ! = core::mem::transmute(func_ptr);
		func()
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
		set_state(STAT_CLK_WKUP_SECONDS);

		static NAME: &[u8] = b"System Settings\0";
		static ITEMS: &[u8] = &[MI_SET_DATE, MI_SET_TIME, MI_MSC, MI_SYSTEM_ENTER, 0];
		static SYSTEM_MENU: Menu = Menu {
			name: &NAME[0],
			items: &ITEMS[0],
			message: None,
			post_display: None,
		};
		handle_menu(&SYSTEM_MENU, MENU_RESET, 0);

		clear_state(STAT_CLK_WKUP_SECONDS);
		lcd_clear_buf();
	}
}

pub fn time_24_hour() -> bool {
	state(STAT_CLK24)
}

pub fn set_time_24_hour(value: bool) {
	if value {
		set_state(STAT_CLK24);
	} else {
		clear_state(STAT_CLK24);
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
		lcd_refresh_dma();
	}

	fn fill(&mut self, rect: &Rect, color: bool) {
		let rect = rect.clipped_to(&Rect {
			x: 0,
			y: 0,
			w: WIDTH,
			h: HEIGHT,
		});
		if rect.w == 0 || rect.h == 0 {
			return;
		}

		if rect.x == 0 && rect.w == WIDTH {
			lcd_fill_lines(rect.y as u32, if color { 0 } else { 0xff }, rect.h as u32);
		} else {
			lcd_fill_rect(
				rect.x as u32,
				rect.y as u32,
				rect.w as u32,
				rect.h as u32,
				if color { 1 } else { 0 },
			);
		}
	}

	fn draw_bits(&mut self, x: i32, y: i32, bits: u32, width: u8, color: bool) {
		if color {
			bitblt24(x as u32, width as u32, y as u32, bits, BLT_OR, BLT_NONE);
		} else {
			bitblt24(x as u32, width as u32, y as u32, bits, BLT_ANDN, BLT_NONE);
		}
	}

	fn renderer(&mut self, render_mode: RenderMode) -> ScreenLayoutRenderer {
		ScreenLayoutRenderer::new(self, render_mode)
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

	fn wait_raw(&mut self) -> Option<KeyEvent> {
		if let Some(key) = self.pop_raw() {
			reset_auto_off();
			return Some(key);
		}

		clear_state(STAT_CLK_WKUP_SECONDS);
		clear_state(STAT_CLK_WKUP_FLAG);
		set_state(STAT_CLK_WKUP_ENABLE);

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
					clear_state(STAT_CLK_WKUP_ENABLE);
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
				set_state(STAT_CLK_WKUP_ENABLE);
				*CLOCK_CHANGED.lock() = true;
				if !lcd_get_buf_cleared() {
					lcd_forced_refresh();
				}
				return None;
			}

			let mut changes = false;
			if state(STAT_CLK_WKUP_FLAG) {
				clear_state(STAT_CLK_WKUP_FLAG);
				*CLOCK_CHANGED.lock() = true;
				changes = true;
			}

			if state(STAT_POWER_CHANGE) {
				clear_state(STAT_POWER_CHANGE);
				changes = true;
			}

			if let Some(key) = self.pop_raw() {
				reset_auto_off();
				return Some(key);
			} else if changes {
				return None;
			}
		}
	}

	fn suspend(&self) {
		set_state(STAT_PGM_END);
	}
}

extern "C" fn program_entry() -> ! {
	unsafe {
		// Copy data section initial contents from flash to RAM
		let ram_data_len = &_edata as *const u8 as usize - &_sdata as *const u8 as usize;
		let ram_data = core::slice::from_raw_parts_mut(&mut _sdata as *mut u8, ram_data_len);
		let flash_data = core::slice::from_raw_parts(&_sidata, ram_data_len);
		ram_data.copy_from_slice(flash_data);

		// Zero fill BSS section
		let bss_len = &_ebss as *const u8 as usize - &_sbss as *const u8 as usize;
		let bss = core::slice::from_raw_parts_mut(&mut _sbss as *mut u8, bss_len);
		bss.fill(0);
	}

	crate::main();
	reset();
}

pub fn program_main() {
	let screen = DM42Screen;
	let input_queue = DM42InputQueue;
	calc_main(screen, input_queue);
}

fn reset() -> ! {
	// Reset if main exits
	set_reset_magic(RUN_DMCP_MAGIC);
	sys_reset();
}
