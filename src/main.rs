#![feature(alloc_error_handler)]
#![feature(assoc_char_funcs)]
#![feature(c_variadic)]
#![feature(slice_fill)]
#![cfg_attr(feature = "dm42", no_std)]
#![cfg_attr(feature = "dm42", no_main)]

#[cfg(feature = "dm42")]
extern crate alloc;

#[cfg(feature = "dm42")]
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "dm42")]
mod dm42;
#[cfg(feature = "simulated")]
mod dm42;

fn main() {
    #[cfg(feature = "dm42")]
    dm42::device::program_main();

    #[cfg(feature = "simulated")]
    dm42::simulated::App::run();
}
