#![cfg_attr(feature = "dm42", no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[macro_use]
extern crate lazy_static;

pub mod complex;
pub mod constant;
pub mod context;
pub mod error;
pub mod format;
pub mod functions;
pub mod matrix;
pub mod number;
pub mod stack;
pub mod storage;
pub mod time;
pub mod unit;
pub mod value;
pub mod vector;

mod undo;
