fn main() {
    if std::env::var_os("CARGO_FEATURE_DM42").is_some() {
        cc::Build::new()
            .file("dmcp/startup_pgm.s")
            .file("dmcp/sys/pgm_syscalls.c")
            .warnings(false)
            .compile("startup_pgm");
        println!("cargo:rerun-if-changed=dmcp/startup_pgm.s");
        println!("cargo:rerun-if-changed=dmcp/sys/pgm_syscalls.c");
        println!("cargo:rerun-if-changed=dmcp/main.h");
        println!("cargo:rerun-if-changed=dmcp/dmcp.h");
        println!("cargo:rerun-if-changed=dmcp/ff_ifc.h");
        println!("cargo:rerun-if-changed=dmcp/lft_ifc.h");
        println!("cargo:rerun-if-changed=dmcp/qspi_crc.h");
    }
}
