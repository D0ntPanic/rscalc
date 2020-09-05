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
mod input;
mod number;
mod screen;
mod stack;

use input::{AlphaMode, InputEvent, InputMode, InputQueue};
use intel_dfp::Decimal;
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
                '0'..='9' | 'A'..='Z' | 'a'..='z' | '.' => {
                    stack.push_char(ch, &format);
                }
                _ => (),
            },
            InputEvent::E => {
                stack.exponent();
            }
            InputEvent::Enter => {
                stack.enter();
            }
            InputEvent::Backspace => {
                stack.backspace();
            }
            InputEvent::Neg => {
                stack.neg();
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
            InputEvent::Log => {
                let value = stack.top().log();
                stack.set_top(value);
            }
            InputEvent::TenX => {
                let value = stack.top().exp10();
                stack.set_top(value);
            }
            InputEvent::Ln => {
                let value = stack.top().log();
                stack.set_top(value);
            }
            InputEvent::EX => {
                let value = stack.top().exp();
                stack.set_top(value);
            }
            InputEvent::Percent => {
                if stack.len() >= 2 {
                    let one_hundred: Number = 100.into();
                    let value = stack.entry(1) * &(stack.entry(0) / &one_hundred);
                    stack.set_top(value);
                }
            }
            InputEvent::Pi => stack.input_num(Number::Decimal(Decimal::from_str(
                "3.141592653589793238462643383279503",
            ))),
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
            InputEvent::RotateDown => {
                if stack.len() >= 2 {
                    stack.rotate_down();
                }
            }
            InputEvent::Swap => {
                if stack.len() >= 2 {
                    stack.swap(0, 1);
                }
            }
            InputEvent::Base => {
                if format.integer_radix == 10 {
                    format.integer_radix = 16;
                    stack.end_edit();
                } else if format.integer_radix == 16 {
                    format.integer_radix = 10;
                    stack.end_edit();
                }
            }
            InputEvent::FunctionKey(func, _) => {
                if format.integer_radix == 16 {
                    stack.push_char(
                        char::from_u32('A' as u32 + func as u32 - 1).unwrap(),
                        &format,
                    );
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
