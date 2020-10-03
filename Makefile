.PHONY: all target/thumbv7em-none-eabihf/release/rscalc

all: rscalc.pgm rscalc

target/thumbv7em-none-eabihf/release/rscalc:
	cargo build --target thumbv7em-none-eabihf -Z build-std=core,alloc --release --no-default-features --features dm42

rscalc.pgm: target/thumbv7em-none-eabihf/release/rscalc
	arm-none-eabi-objcopy --remove-section .qspi -O binary target/thumbv7em-none-eabihf/release/rscalc rscalc.bin
	arm-none-eabi-objcopy --only-section .qspi -O binary target/thumbv7em-none-eabihf/release/rscalc rscalc_qspi.bin
	dmcp/check_qspi_crc
	dmcp/add_pgm_chsum rscalc.bin rscalc.pgm

rscalc:
	cargo build
