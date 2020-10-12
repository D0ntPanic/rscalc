#[cfg(feature = "dm42")]
pub mod device;

#[cfg(not(feature = "dm42"))]
pub mod simulated;

mod catalog;
mod edit;
mod font;
mod functions;
mod input;
mod menu;
mod screen;
mod state;
mod unit;

use input::{InputQueue, KeyEvent};
use screen::Screen;
use state::{InputResult, State};

pub fn calc_main<ScreenT: Screen, InputT: InputQueue>(mut screen: ScreenT, mut input: InputT) {
	screen.clear();

	let mut state = State::new();
	state.render(&mut screen);

	loop {
		if let Some(input_event) = state.wait_for_input(&mut input) {
			match state.handle_input(input_event, &screen) {
				Ok(InputResult::Normal) => (),
				Ok(InputResult::Suspend) => input.suspend(),
				Err(error) => {
					state.show_error(error);
					state.render(&mut screen);

					for _ in 0..30 {
						#[cfg(feature = "dm42")]
						device::sys_delay(100);
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
