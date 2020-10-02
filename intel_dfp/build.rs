use std::env;
use std::path::Path;

fn main() {
	let source_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
	let lib_path = Path::new(&source_dir).join("lib");

	// Unfortunately the Intel floating point library is not compatible with clang, so it cannot
	// be built with the Rust toolchain. We have to use pre-built libraries built by GCC/MSVC.
	let target = env::var("TARGET").unwrap();
	match target.as_str() {
		"thumbv7em-none-eabihf" => {
			let lib_path = lib_path.join("dm42");
			println!(
				"cargo:rustc-link-search=native={}",
				lib_path.to_str().unwrap()
			);
			println!("cargo:rustc-link-lib=static=bid");
		}
		"x86_64-unknown-linux-gnu" => {
			let lib_path = lib_path.join("linux").join("x86_64");
			println!(
				"cargo:rustc-link-search=native={}",
				lib_path.to_str().unwrap()
			);
			println!("cargo:rustc-link-lib=static=bid");
		}
		_ => panic!(
			"Intel floating point library not pre-compiled for target '{}'",
			target
		),
	}
}
