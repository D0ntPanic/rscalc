#[cfg(feature = "dm42")]
use crate::dm42::{rtc_read, rtc_updated};
#[cfg(not(feature = "dm42"))]
use chrono::{DateTime, Local};

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Weekday};

pub trait Now {
	/// Gets the current date and time in the local timezone.
	fn now() -> Self;

	/// Returns true if the clock may have been updated at the minute resolution
	/// since the last call to now(). The time may not actually be different, as
	/// this is only for use in optimization.
	fn clock_minute_updated() -> bool;
}

pub struct SimpleDateTimeFormat {
	date: bool,
	year: bool,
	time: bool,
	seconds: bool,
	centiseconds: bool,
	am_pm: bool,
}

/// Trait for a simple conversion of date and time to a string. This needs to be used
/// instead of the standard package implementation because we must work without a
/// C runtime and in nostd. There is an option for enabling this in the chrono crate
/// but it is unstable (an in fact not working in nostd when last tested).
pub trait SimpleDateTimeToString {
	fn to_str(&self, format: &SimpleDateTimeFormat) -> String;
}

impl Now for NaiveDateTime {
	#[cfg(feature = "dm42")]
	fn now() -> Self {
		rtc_read()
	}

	#[cfg(not(feature = "dm42"))]
	fn now() -> Self {
		let result: DateTime<Local> = Local::now();
		result.naive_local()
	}

	fn clock_minute_updated() -> bool {
		#[cfg(feature = "dm42")]
		return rtc_updated();

		#[cfg(not(feature = "dm42"))]
		return false;
	}
}

impl SimpleDateTimeToString for NaiveDate {
	fn to_str(&self, format: &SimpleDateTimeFormat) -> String {
		let mut result = match self.weekday() {
			Weekday::Mon => "Mon ",
			Weekday::Tue => "Tue ",
			Weekday::Wed => "Wed ",
			Weekday::Thu => "Thu ",
			Weekday::Fri => "Fri ",
			Weekday::Sat => "Sat ",
			Weekday::Sun => "Sun ",
		}
		.to_string();

		result += match self.month() {
			1 => "Jan ",
			2 => "Feb ",
			3 => "Mar ",
			4 => "Apr ",
			5 => "May ",
			6 => "Jun ",
			7 => "Jul ",
			8 => "Aug ",
			9 => "Sep ",
			10 => "Oct ",
			11 => "Nov ",
			12 => "Dec ",
			_ => unreachable!(),
		};

		if self.day() < 10 {
			result.push(char::from_u32('0' as u32 + self.day() as u32).unwrap());
		} else {
			result.push(char::from_u32('0' as u32 + (self.day() / 10) as u32).unwrap());
			result.push(char::from_u32('0' as u32 + (self.day() % 10) as u32).unwrap());
		}

		if format.year {
			result += ", ";

			let mut year_chars = Vec::new();
			let mut year = self.year();
			while year != 0 {
				year_chars.push(char::from_u32('0' as u32 + (year % 10) as u32).unwrap());
				year /= 10;
			}
			year_chars.reverse();
			let year_str: String = year_chars.iter().collect();
			result += year_str.as_str();
		}

		result
	}
}

impl SimpleDateTimeToString for NaiveTime {
	fn to_str(&self, format: &SimpleDateTimeFormat) -> String {
		let mut result = String::new();

		let hour = if format.am_pm {
			let twelve_hour = self.hour() % 12;
			if twelve_hour == 0 {
				12
			} else {
				twelve_hour
			}
		} else {
			self.hour()
		};

		if hour < 10 {
			result.push(char::from_u32('0' as u32 + hour as u32).unwrap());
		} else {
			result.push(char::from_u32('0' as u32 + (hour / 10) as u32).unwrap());
			result.push(char::from_u32('0' as u32 + (hour % 10) as u32).unwrap());
		}

		result.push(':');
		result.push(char::from_u32('0' as u32 + (self.minute() / 10) as u32).unwrap());
		result.push(char::from_u32('0' as u32 + (self.minute() % 10) as u32).unwrap());

		if format.seconds {
			result.push(':');
			result.push(char::from_u32('0' as u32 + (self.second() / 10) as u32).unwrap());
			result.push(char::from_u32('0' as u32 + (self.second() % 10) as u32).unwrap());
		}

		if format.centiseconds {
			result.push('.');
			result.push(
				char::from_u32('0' as u32 + (self.nanosecond() / 100000000 % 10) as u32).unwrap(),
			);
			result.push(
				char::from_u32('0' as u32 + (self.nanosecond() / 10000000 % 10) as u32).unwrap(),
			);
		}

		if format.am_pm {
			if self.hour() >= 12 {
				result.push_str(&" PM");
			} else {
				result.push_str(&" AM");
			}
		}

		result
	}
}

impl SimpleDateTimeToString for NaiveDateTime {
	fn to_str(&self, format: &SimpleDateTimeFormat) -> String {
		// Minimal implementation for DM42 embedded version
		let mut result = String::new();

		if format.date {
			result += &self.date().to_str(format);
		}

		if format.time {
			if format.date {
				result += ", ";
			}
			result += &self.time().to_str(format);
		}

		result
	}
}

impl SimpleDateTimeFormat {
	pub fn full() -> Self {
		SimpleDateTimeFormat {
			date: true,
			year: true,
			time: true,
			seconds: true,
			centiseconds: true,
			am_pm: true,
		}
	}

	pub fn date() -> Self {
		SimpleDateTimeFormat {
			date: true,
			year: true,
			time: false,
			seconds: false,
			centiseconds: false,
			am_pm: true,
		}
	}

	pub fn time() -> Self {
		SimpleDateTimeFormat {
			date: false,
			year: false,
			time: true,
			seconds: true,
			centiseconds: true,
			am_pm: true,
		}
	}

	pub fn status_bar() -> Self {
		SimpleDateTimeFormat {
			date: true,
			year: false,
			time: true,
			seconds: false,
			centiseconds: false,
			am_pm: true,
		}
	}
}
