#![feature(alloc_error_handler)]
#![cfg_attr(feature = "dm42", no_std)]
#![cfg_attr(feature = "dm42", no_main)]

extern crate alloc;
extern crate intel_dfp;
extern crate num_bigint;

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
mod number;
mod stack;

use screen::{Screen, Rect, Color};
use input::{InputQueue, InputMode, AlphaMode, InputEvent};
use number::Number;
use stack::Stack;

#[cfg(feature = "dm42")]
use alloc::vec::Vec;

fn draw_header<ScreenT: Screen>(screen: &mut ScreenT, mode: &InputMode) {
    screen.fill(Rect {
        x: 0,
        y: 0,
        w: screen.width(),
        h: font::SANS_16.height
    }, Color::StatusBarBackground);

    screen.fill(Rect {
        x: 0,
        y: font::SANS_16.height,
        w: screen.width(),
        h: 1
    }, Color::ContentBackground);

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

fn header_size() -> i32 {
    font::SANS_16.height + 1
}

pub fn calc_main<ScreenT: Screen, InputT: InputQueue>(mut screen: ScreenT, mut input: InputT) {
    screen.clear();

    let mut stack = Stack::new();
    let mut mode = InputMode {
        alpha: AlphaMode::Normal,
        shift: false
    };

    loop {
        screen.clear();
        draw_header(&mut screen, &mode);

        let stack_area = Rect {
            x: 0,
            y: header_size(),
            w: screen.width(),
            h: screen.height() - header_size()
        };

        stack.render(&mut screen, stack_area);
        screen.refresh();

        match input.wait(&mut mode) {
            InputEvent::Character(ch) => {
                match ch {
                    '0'..='9' => {
                        let top = stack.top_mut();
                        *top *= 10.into();
                        *top += ch.to_digit(10).unwrap().into();
                    }
                    _ => ()
                }
            }
            InputEvent::Enter => {
                stack.push(0.into());
            }
            InputEvent::Backspace => {
                stack.pop();
            }
            InputEvent::Add => {
                if stack.len() >= 2 {
                    let x = stack.pop();
                    let y = stack.top();
                    let value = y + &x;
                    stack.set_top(value);
                }
            }
            InputEvent::Sub => {
                if stack.len() >= 2 {
                    let x = stack.pop();
                    let y = stack.top();
                    let value = y - &x;
                    stack.set_top(value);
                }
            }
            InputEvent::Mul => {
                if stack.len() >= 2 {
                    let x = stack.pop();
                    let y = stack.top();
                    let value = y * &x;
                    stack.set_top(value);
                }
            }
            InputEvent::Run => {
                panic!("panic");
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
