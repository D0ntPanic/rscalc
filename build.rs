use std::path::Path;

fn main() {
    if std::env::var_os("CARGO_FEATURE_DM42").is_some()
        && std::env::var_os("CARGO_CFG_TARGET_OS").unwrap().to_str() == Some("none")
    {
        // Use crate version for the DM42 program version field
        let version = std::env::var_os("CARGO_PKG_VERSION").unwrap();
        let mut version = version.to_str().unwrap().to_string();
        let padding = 16 - version.len();
        for _ in 0..padding {
            version.push_str("\\0");
        }

        let out_dir = std::env::var("OUT_DIR").unwrap();
        let version_path = Path::new(&out_dir).join("version.txt");
        std::fs::write(&version_path, &format!("b\"{}\"", version)).unwrap();
    }
}
