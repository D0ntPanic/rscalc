use crate::error::Result;
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Weekday};

#[cfg(feature = "std")]
use chrono::{DateTime, Local};

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
#[cfg(not(feature = "dm42"))]
use crate::error::Error;

pub trait Now: Sized {
	/// Gets the current date and time in the local timezone.
	fn now() -> Result<Self>;
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
	fn simple_format(&self, format: &SimpleDateTimeFormat) -> String;
}

#[cfg(feature = "dm42")]
#[repr(C)]
struct dt_t {
	year: u16,
	month: u8,
	day: u8,
}

#[cfg(feature = "dm42")]
#[repr(C)]
struct tm_t {
	hour: u8,
	min: u8,
	sec: u8,
	csec: u8,
	dow: u8,
}

impl Now for NaiveDateTime {
	#[cfg(feature = "dm42")]
	fn now() -> Result<Self> {
		unsafe {
			const LIBRARY_BASE: usize = 0x8000201;
			let func_ptr: usize = LIBRARY_BASE + 204;
			let func: extern "C" fn(time: *mut tm_t, date: *mut dt_t) =
				core::mem::transmute(func_ptr);
			let mut date = core::mem::MaybeUninit::<dt_t>::uninit();
			let mut time = core::mem::MaybeUninit::<tm_t>::uninit();
			func(time.as_mut_ptr(), date.as_mut_ptr());
			let date = date.assume_init();
			let time = time.assume_init();
			let date = NaiveDate::from_ymd(date.year as i32, date.month as u32, date.day as u32);
			let time = NaiveTime::from_hms_milli(
				time.hour as u32,
				time.min as u32,
				time.sec as u32,
				time.csec as u32 * 10,
			);
			Ok(NaiveDateTime::new(date, time))
		}
	}

	#[cfg(not(feature = "std"))]
	#[cfg(not(feature = "dm42"))]
	fn now() -> Result<Self> {
		Err(Error::ValueNotDefined)
	}

	#[cfg(feature = "std")]
	fn now() -> Result<Self> {
		let result: DateTime<Local> = Local::now();
		Ok(result.naive_local())
	}
}

impl SimpleDateTimeToString for NaiveDate {
	fn simple_format(&self, format: &SimpleDateTimeFormat) -> String {
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
			result.push(core::char::from_u32('0' as u32 + self.day() as u32).unwrap());
		} else {
			result.push(core::char::from_u32('0' as u32 + (self.day() / 10) as u32).unwrap());
			result.push(core::char::from_u32('0' as u32 + (self.day() % 10) as u32).unwrap());
		}

		if format.year {
			result += ", ";

			let mut year_chars = Vec::new();
			let mut year = self.year();
			while year != 0 {
				year_chars.push(core::char::from_u32('0' as u32 + (year % 10) as u32).unwrap());
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
	fn simple_format(&self, format: &SimpleDateTimeFormat) -> String {
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
			result.push(core::char::from_u32('0' as u32 + hour as u32).unwrap());
		} else {
			result.push(core::char::from_u32('0' as u32 + (hour / 10) as u32).unwrap());
			result.push(core::char::from_u32('0' as u32 + (hour % 10) as u32).unwrap());
		}

		result.push(':');
		result.push(core::char::from_u32('0' as u32 + (self.minute() / 10) as u32).unwrap());
		result.push(core::char::from_u32('0' as u32 + (self.minute() % 10) as u32).unwrap());

		if format.seconds {
			result.push(':');
			result.push(core::char::from_u32('0' as u32 + (self.second() / 10) as u32).unwrap());
			result.push(core::char::from_u32('0' as u32 + (self.second() % 10) as u32).unwrap());
		}

		if format.centiseconds {
			result.push('.');
			result.push(
				core::char::from_u32('0' as u32 + (self.nanosecond() / 100000000 % 10) as u32)
					.unwrap(),
			);
			result.push(
				core::char::from_u32('0' as u32 + (self.nanosecond() / 10000000 % 10) as u32)
					.unwrap(),
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
	fn simple_format(&self, format: &SimpleDateTimeFormat) -> String {
		// Minimal implementation for DM42 embedded version
		let mut result = String::new();

		if format.date {
			result += &self.date().simple_format(format);
		}

		if format.time {
			if format.date {
				result += ", ";
			}
			result += &self.time().simple_format(format);
		}

		result
	}
}

impl SimpleDateTimeFormat {
	pub fn full(time_24_hour: bool) -> Self {
		SimpleDateTimeFormat {
			date: true,
			year: true,
			time: true,
			seconds: true,
			centiseconds: true,
			am_pm: !time_24_hour,
		}
	}

	pub fn date(time_24_hour: bool) -> Self {
		SimpleDateTimeFormat {
			date: true,
			year: true,
			time: false,
			seconds: false,
			centiseconds: false,
			am_pm: !time_24_hour,
		}
	}

	pub fn time(time_24_hour: bool) -> Self {
		SimpleDateTimeFormat {
			date: false,
			year: false,
			time: true,
			seconds: true,
			centiseconds: true,
			am_pm: !time_24_hour,
		}
	}

	pub fn status_bar(time_24_hour: bool) -> Self {
		SimpleDateTimeFormat {
			date: true,
			year: false,
			time: true,
			seconds: false,
			centiseconds: false,
			am_pm: !time_24_hour,
		}
	}
}
