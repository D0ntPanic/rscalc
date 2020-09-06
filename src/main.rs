#![feature(alloc_error_handler)]
#![feature(assoc_char_funcs)]
#![cfg_attr(feature = "dm42", no_std)]
#![cfg_attr(feature = "dm42", no_main)]

extern crate alloc;
extern crate intel_dfp;
extern crate num_bigint;

#[cfg(not(feature = "dm42"))]
extern crate glib;
#[cfg(not(feature = "dm42"))]
extern crate gtk;

#[cfg(feature = "dm42")]
mod dm42;

#[cfg(not(feature = "dm42"))]
mod simulated;

mod edit;
mod font;
mod functions;
mod input;
mod number;
mod screen;
mod stack;
mod state;

use input::{AlphaMode, InputMode, InputQueue};
use screen::{Color, Rect, Screen};
use state::{InputResult, State};

fn draw_header<ScreenT: Screen>(screen: &mut ScreenT, mode: &InputMode) {
    screen.fill(
        Rect {
            x: 0,
            y: 0,
            w: screen.width(),
            h: font::SANS_16.height,
        },
        Color::StatusBarBackground,
    );

    screen.fill(
        Rect {
            x: 0,
            y: font::SANS_16.height,
            w: screen.width(),
            h: 1,
        },
        Color::ContentBackground,
    );

    let x = 2;
    if mode.shift {
        font::SANS_16.draw(screen, x, 0, "⬏", Color::StatusBarText);
    }
    let x = x + font::SANS_16.width("⬏") + 4;
    match mode.alpha {
        AlphaMode::UpperAlpha => {
            font::SANS_16.draw(screen, x, 0, "[A]", Color::StatusBarText);
        }
        AlphaMode::LowerAlpha => {
            font::SANS_16.draw(screen, x, 0, "[a]", Color::StatusBarText);
        }
        _ => (),
    }
}

fn header_size() -> i32 {
    font::SANS_16.height + 1
}

pub fn calc_main<ScreenT: Screen, InputT: InputQueue>(mut screen: ScreenT, mut input: InputT) {
    screen.clear();

    let mut state = State::new();

    loop {
        screen.clear();
        draw_header(&mut screen, &state.input_mode);

        state.function_keys.update(&state.format);
        state.function_keys.render(&mut screen, &state);

        let stack_area = Rect {
            x: 0,
            y: header_size(),
            w: screen.width(),
            h: screen.height() - header_size() - state.function_keys.height(),
        };

        state.stack.render(&mut screen, &state.format, stack_area);
        screen.refresh();

        let input_event = input.wait(&mut state.input_mode);
        match state.handle_input(input_event) {
            InputResult::Normal => (),
            InputResult::Suspend => input.suspend(),
        }
    }
}

#[cfg(not(feature = "dm42"))]
fn main() {
    simulated::App::run();
}
