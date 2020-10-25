use clap::{App, Arg};
use crc::crc32;
use goblin::elf::section_header::SHT_NOBITS;
use goblin::elf::Elf;
use sha1::{Digest, Sha1};
use std::convert::TryInto;
use std::path::Path;

const PROG_INFO_MAGIC: u32 = 0xd377c0de;

fn main() {
	// Parse command line
	let matches = App::new("elf2pgm")
		.version("0.1")
		.about("Converts an ELF to a DM42 PGM file")
		.arg(
			Arg::with_name("INPUT")
				.help("Input ELF filename")
				.required(true),
		)
		.arg(
			Arg::with_name("OUTPUT")
				.help("Output PGM filename")
				.required(true),
		)
		.get_matches();

	let input = matches.value_of("INPUT").expect("input file required");
	let output = matches.value_of("OUTPUT").expect("input file required");

	// Read input ELF and parse it
	let input_path = Path::new(input);
	let input_data = std::fs::read(input_path).expect("failed to read input file");
	let elf = Elf::parse(&input_data).expect("invalid input ELF");

	// We will split the ELF into the QSPI section (which contains floating point constants for use
	// by the program) and the PGM section (which is what is actually loaded).
	let mut qspi_data = Vec::new();
	let mut pgm_data = Vec::new();

	// Go through the section headers and collect the raw data
	for section in elf.section_headers {
		// Do not look at NOBITS sections (BSS)
		if section.sh_type == SHT_NOBITS {
			continue;
		}

		// Check for empty sections or sections that aren't in memory
		if section.vm_range().start == 0 {
			continue;
		}
		if section.file_range().start == section.file_range().end {
			continue;
		}

		// Grab section name
		let section_name = elf
			.shdr_strtab
			.get(section.sh_name)
			.expect("invalid section name offset")
			.expect("invalid section name");

		// Add file data to the correct vector
		if section_name == ".qspi" {
			qspi_data.extend_from_slice(
				&input_data[section.file_range().start..section.file_range().end],
			);
		} else {
			pgm_data.extend_from_slice(
				&input_data[section.file_range().start..section.file_range().end],
			);
		}
	}

	// Check header magic for DM42 PGM
	if u32::from_le_bytes(pgm_data[0..4].try_into().expect("invalid PGM header")) != PROG_INFO_MAGIC
	{
		panic!("PGM header magic does not match");
	}

	// Validate QSPI contents against the header data
	let expected_qspi_size =
		u32::from_le_bytes(pgm_data[20..24].try_into().expect("invalid PGM header"));
	let expected_qspi_crc =
		u32::from_le_bytes(pgm_data[24..28].try_into().expect("invalid PGM header"));

	if qspi_data.len() != expected_qspi_size as usize {
		panic!(format!(
			"QSPI data does not match: length {} != expected length {}",
			qspi_data.len(),
			expected_qspi_size
		));
	}
	if crc32::checksum_ieee(&qspi_data) != expected_qspi_crc {
		panic!(format!(
			"QPSI data does not match: CRC {:#x} != expected CRC {:#x}",
			crc32::checksum_ieee(&qspi_data),
			expected_qspi_crc
		));
	}

	// Place correct program size into the header
	let size_bytes = (pgm_data.len() as u32).to_le_bytes();
	pgm_data[4..8].copy_from_slice(&size_bytes);

	// Compute program checksum and add it to the end of the file
	let mut hash = Sha1::new();
	hash.update(&pgm_data);
	let digest = hash.finalize();
	pgm_data.extend_from_slice(&digest);

	// Write output PGM file
	let output_path = Path::new(output);
	std::fs::write(output_path, &pgm_data).expect("failed to write output file");

	println!("PGM file of {} bytes written.", pgm_data.len());
}
