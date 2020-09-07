#![feature(alloc_error_handler)]
#![feature(assoc_char_funcs)]
#![cfg_attr(feature = "dm42", no_std)]
#![cfg_attr(feature = "dm42", no_main)]

extern crate alloc;
extern crate intel_dfp;
extern crate num_bigint;

#[cfg(not(feature = "dm42"))]
extern crate chrono;
#[cfg(not(feature = "dm42"))]
extern crate glib;
#[cfg(not(feature = "dm42"))]
extern crate gtk;

#[cfg(feature = "dm42")]
#[macro_use]
extern crate lazy_static;
#[cfg(feature = "dm42")]
extern crate spin;

#[cfg(feature = "dm42")]
mod dm42;

#[cfg(not(feature = "dm42"))]
mod simulated;

mod edit;
mod font;
mod functions;
mod input;
mod layout;
mod number;
mod screen;
mod stack;
mod state;
mod time;
mod value;

use input::InputQueue;
use screen::Screen;
use state::{InputResult, State};

pub fn calc_main<ScreenT: Screen, InputT: InputQueue>(mut screen: ScreenT, mut input: InputT) {
    screen.clear();

    let mut state = State::new();
    state.render(&mut screen);

    loop {
        if let Some(input_event) = input.wait(&mut state.input_mode) {
            match state.handle_input(input_event) {
                InputResult::Normal => (),
                InputResult::Suspend => input.suspend(),
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
