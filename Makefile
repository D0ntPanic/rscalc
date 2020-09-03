.PHONY: all target/thumbv7em-none-eabihf/release/calc

all: calc.pgm calc

target/thumbv7em-none-eabihf/release/calc:
	cargo xbuild --target thumbv7em-none-eabihf --release --no-default-features --features dm42

calc.pgm: target/thumbv7em-none-eabihf/release/calc
	arm-none-eabi-objcopy --remove-section .qspi -O binary target/thumbv7em-none-eabihf/release/calc calc.bin
	arm-none-eabi-objcopy --only-section .qspi -O binary target/thumbv7em-none-eabihf/release/calc calc_qspi.bin
	dmcp/check_qspi_crc
	dmcp/add_pgm_chsum calc.bin calc.pgm

calc:
	cargo build
