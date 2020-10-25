# rscalc - RPN Stack Calculator

This project is a modern multiplatform RPN calculator written in Rust.

## Major Features

* **Large RPN stack**. The RPN stack is not limited to a fixed number of registers.
* **Accurate decimal floating point**. Uses 128-bit decimal floating point with 34
  digits of decimal precision.
* **Exact rationals with large integers**. Integers and rational numbers are kept
  in an exact rational representation. Results can be displayed as exact fractions
  when possible, and the implementation never guesses the closest fraction.
* **Unit conversions**. Supports conversions between common units. Utilizes the
  rational representation for unit conversions so that converting between them
  leads to no loss of precision.
* **RPN stack undo**. Operations performed on the RPN stack can be undone with
  a deep undo buffer.
* **Works on dedicated calculator hardware**. Designed to run on the excellent
  DM42 from Swiss Micros. This project has been optimized for highly efficient
  LCD rendering and memory usage to create a very responsive interface. This
  project is not an emulation and iterative improvement of the HP42 design
  like the default firmware, but is rather a ground-up modern redesign
  targeting the DM42 hardware.

## Design Goals

This project was inspired by the DM42 hardware from Swiss Micros, but had the
following design goals in mind:

* The desire was to create a modern reimagining of an RPN calculator on the
  DM42 form factor.
* This project was not meant to be constrained by the design or interface of
  the HP42 calculator that the DM42 was inspired by. No effort is expended on
  being compatible in any way with the HP42 ecosystem. If you are looking for
  an improved HP42 that runs existing programs, the default DM42 firmware is
  a better fit.
* Keep the efficient workflow of RPN calculation, but optimize it where
  workflow improvements can be made.
* Utilize modern high resolution displays to improve the interface. The DM42
  hardware has a much higher resolution than legacy RPN calculators and can
  present information in a more natural and readable way. Desktops and
  modern mobile devices can display even more detailed information and
  provide rich user interaction, and this should be taken advantage of.
* This project is designed for users who like an RPN interface. There are
  many options available for algebraic notation, so it not a goal to provide
  a non-RPN interface.

## Building

This project requires the nightly Rust compiler as it has some dependencies
that require it (embedded development with the `alloc` crate is only possible
on nightly at this time, and the `gtk` crate has dependencies on the unstable
`const_fn` feature).

To build the desktop version, you can use `cargo` normally. Use `cargo build`
to build or `cargo run` to build and run.

To build the DM42 version, you must use the `Makefile`. Invoking `make` will
build both the DM42 build and the desktop build. To build only the DM42
build, use `make rscalc.pgm`. The DM42 build has only been tested on Linux
and requires the `thumbv7em-none-eabihf` Rust target for cross compilation.
This target can be added with `rustup target add thumbv7em-none-eabihf`.

## DM42 Calculation Limits

The DM42 hardware, being an embedded device designed for very low power
consumption, has limited RAM available. To work well in this environment, a
few limitations on calculation range are present on the DM42 build:

* Exact integers up to ±2^8192
* Exact rational numbers with a denominator up to 2^128
* Stack is limited to 1000 entries
* Vectors are limited to 1000 entries
* Matricies are limited to 1024 elements total (rows × columns)

Additional notes:

* Undo buffer may be limited in low memory situations. Undo entries are
  automatically freed when memory gets low.
