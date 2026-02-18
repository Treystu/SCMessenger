#[cfg(feature = "gen-bindings")]
fn main() {
    use camino::Utf8Path;
    use uniffi::SwiftBindingGenerator;

    // Use CARGO_MANIFEST_DIR to resolve paths correctly regardless of working directory
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let manifest_path = Utf8Path::new(&manifest_dir);

    let udl_file = manifest_path.join("src/api.udl");
    let config_file = manifest_path.join("uniffi.toml");
    let out_dir = manifest_path.join("target/generated-sources/uniffi/swift");

    // Pass config file to avoid "missing field `package`" error (issue #42)
    let config = config_file.exists().then_some(config_file.as_path());

    uniffi_bindgen::generate_bindings(
        udl_file.as_path(),
        config,
        SwiftBindingGenerator,
        Some(out_dir.as_path()),
        None,
        None,
        false,
    )
    .expect("Failed to generate Swift bindings. Check core/uniffi.toml and core/src/api.udl");
}

#[cfg(not(feature = "gen-bindings"))]
fn main() {}
