#![feature(alloc_error_handler)]
#![feature(assoc_char_funcs)]
#![cfg_attr(feature = "dm42", no_std)]
#![cfg_attr(feature = "dm42", no_main)]

extern crate alloc;
extern crate intel_dfp;
extern crate linked_list_allocator;
extern crate num_bigint;
extern crate num_integer;
extern crate spin;

#[macro_use]
extern crate lazy_static;

#[cfg(not(feature = "dm42"))]
extern crate chrono;
#[cfg(not(feature = "dm42"))]
extern crate glib;
#[cfg(not(feature = "dm42"))]
extern crate gtk;

#[cfg(feature = "dm42")]
mod dm42;

#[cfg(not(feature = "dm42"))]
mod simulated;

mod edit;
mod error;
mod font;
mod functions;
mod input;
mod layout;
mod menu;
mod number;
mod screen;
mod stack;
mod state;
mod storage;
mod time;
mod undo;
mod unit;
mod value;

use input::{InputQueue, KeyEvent};
use screen::Screen;
use state::{InputResult, State};

pub fn calc_main<ScreenT: Screen, InputT: InputQueue>(mut screen: ScreenT, mut input: InputT) {
    screen.clear();

    let mut state = State::new();
    state.render(&mut screen);

    loop {
        if let Some(input_event) = input.wait(&mut state.input_mode) {
            match state.handle_input(input_event, &screen) {
                Ok(InputResult::Normal) => (),
                Ok(InputResult::Suspend) => input.suspend(),
                Err(error) => {
                    state.show_error(error);
                    state.render(&mut screen);

                    for _ in 0..30 {
                        #[cfg(feature = "dm42")]
                        dm42::sys_delay(100);
                        #[cfg(not(feature = "dm42"))]
                        std::thread::sleep(std::time::Duration::from_millis(100));

                        if let Some(KeyEvent::Press(_)) = input.pop_raw() {
                            break;
                        }
                    }

                    state.hide_error();
                }
            }
            state.render(&mut screen);
        } else {
            state.update_header(&mut screen);
        }
    }
}

#[cfg(not(feature = "dm42"))]
fn main() {
    simulated::App::run();
}
