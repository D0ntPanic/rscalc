#![feature(alloc_error_handler)]
#![cfg_attr(feature = "dm42", no_std)]
#![cfg_attr(feature = "dm42", no_main)]

extern crate alloc;
extern crate intel_dfp;

#[cfg(not(feature = "dm42"))]
extern crate gtk;
#[cfg(not(feature = "dm42"))]
extern crate glib;

#[cfg(feature = "dm42")]
mod dm42;

#[cfg(not(feature = "dm42"))]
mod simulated;

mod screen;
mod font;
mod input;

use screen::{Screen, Rect, Color};
use input::{InputQueue, InputMode, AlphaMode, InputEvent};

#[cfg(feature = "dm42")]
use alloc::vec::Vec;

fn draw_header<ScreenT: Screen>(screen: &mut ScreenT, mode: &InputMode) {
    screen.fill(Rect {
        x: 0,
        y: 0,
        w: screen.width(),
        h: font::SANS_16.height
    }, Color::StatusBarBackground);
    let x = 2;
    if mode.shift {
        font::SANS_16.draw(screen, x, 0, "Shift", Color::StatusBarText);
    }
    let x = x + font::SANS_16.width("Shift") + 8;
    match mode.alpha {
        AlphaMode::UpperAlpha => {
            font::SANS_16.draw(screen, x, 0, "[A]", Color::StatusBarText);
        }
        AlphaMode::LowerAlpha => {
            font::SANS_16.draw(screen, x, 0, "[a]", Color::StatusBarText);
        }
        _ => ()
    }
}

pub fn calc_main<ScreenT: Screen, InputT: InputQueue>(mut screen: ScreenT, mut input: InputT) {
    screen.clear();

    let mut value = intel_dfp::Decimal::from(0);
    let mut mode = InputMode {
        alpha: AlphaMode::Normal,
        shift: false
    };

    loop {
        draw_header(&mut screen, &mode);

        let string = value.to_str();
        let width = screen.width();
        let height = screen.height();
        screen.fill(Rect { x: 0, y: height - font::MONO_24.height, w: width, h: font::MONO_24.height },
            Color::ContentBackground);
        font::MONO_24.draw(&mut screen, 2, height - font::MONO_24.height,
            &string, Color::FloatText);
        screen.refresh();

        match input.wait(&mut mode) {
            InputEvent::Character(ch) => {
                match ch {
                    '0'..='9' => {
                        value *= 10.into();
                        value += ch.to_digit(10).unwrap().into();
                    }
                    _ => ()
                }
            }
            InputEvent::Run => {
                panic!("panic");
            }
            InputEvent::Sto => {
                // Test OOM
                let mut v = Vec::new();
                for i in 0u32..100000u32 {
                    v.push(i);
                }
            }
            InputEvent::Setup => {
                #[cfg(feature = "dm42")]
                dm42::show_system_setup_menu();
            }
            InputEvent::Off => {
                input.suspend();
            }
            _ => (),
        }
    }
}

#[cfg(not(feature = "dm42"))]
fn main() {
    simulated::App::run();
}
