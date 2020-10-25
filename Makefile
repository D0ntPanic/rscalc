.PHONY: all target/thumbv7em-none-eabihf/release/rscalc

all: rscalc.pgm rscalc

target/thumbv7em-none-eabihf/release/rscalc:
	cargo build --target thumbv7em-none-eabihf -Z build-std=core,alloc --release --no-default-features --features dm42

rscalc.pgm: target/thumbv7em-none-eabihf/release/rscalc
	cd dmcp && cargo run --release ../target/thumbv7em-none-eabihf/release/rscalc ../rscalc.pgm

rscalc:
	cargo build
