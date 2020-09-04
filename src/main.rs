#![feature(alloc_error_handler)]
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

mod font;
mod input;
mod number;
mod screen;
mod stack;

use input::{AlphaMode, InputEvent, InputMode, InputQueue};
use number::{Number, NumberFormat};
use screen::{Color, Rect, Screen};
use stack::Stack;

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
        _ => (),
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
        shift: false,
    };
    let mut format = NumberFormat::new();

    loop {
        screen.clear();
        draw_header(&mut screen, &mode);

        let stack_area = Rect {
            x: 0,
            y: header_size(),
            w: screen.width(),
            h: screen.height() - header_size(),
        };

        stack.render(&mut screen, &format, stack_area);
        screen.refresh();

        match input.wait(&mut mode) {
            InputEvent::Character(ch) => match ch {
                '0'..='9' => {
                    let top = stack.top_mut();
                    *top *= 10.into();
                    *top += ch.to_digit(10).unwrap().into();
                }
                _ => (),
            },
            InputEvent::Enter => {
                stack.push(0.into());
            }
            InputEvent::Backspace => {
                stack.pop();
            }
            InputEvent::Add => {
                if stack.len() >= 2 {
                    let value = stack.entry(1) + stack.entry(0);
                    stack.replace_entries(2, value);
                }
            }
            InputEvent::Sub => {
                if stack.len() >= 2 {
                    let value = stack.entry(1) - stack.entry(0);
                    stack.replace_entries(2, value);
                }
            }
            InputEvent::Mul => {
                if stack.len() >= 2 {
                    let value = stack.entry(1) * stack.entry(0);
                    stack.replace_entries(2, value);
                }
            }
            InputEvent::Div => {
                if stack.len() >= 2 {
                    let value = stack.entry(1) / stack.entry(0);
                    stack.replace_entries(2, value);
                }
            }
            InputEvent::Recip => {
                let one: Number = 1.into();
                let value = &one / stack.top();
                stack.set_top(value);
            }
            InputEvent::Pow => {
                if stack.len() >= 2 {
                    let value = stack.entry(1).pow(stack.entry(0));
                    stack.replace_entries(2, value);
                }
            }
            InputEvent::Sqrt => {
                let value = stack.top().sqrt();
                stack.set_top(value);
            }
            InputEvent::Square => {
                let value = stack.top() * stack.top();
                stack.set_top(value);
            }
            InputEvent::Sin => {
                let value = stack.top().sin();
                stack.set_top(value);
            }
            InputEvent::Cos => {
                let value = stack.top().cos();
                stack.set_top(value);
            }
            InputEvent::Tan => {
                let value = stack.top().tan();
                stack.set_top(value);
            }
            InputEvent::Asin => {
                let value = stack.top().asin();
                stack.set_top(value);
            }
            InputEvent::Acos => {
                let value = stack.top().acos();
                stack.set_top(value);
            }
            InputEvent::Atan => {
                let value = stack.top().atan();
                stack.set_top(value);
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
