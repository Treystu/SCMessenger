#[cfg(feature = "gen-bindings")]
fn main() {
    use camino::Utf8Path;
    use uniffi::KotlinBindingGenerator;

    let udl_file = Utf8Path::new("src/api.udl");
    let config_file = Utf8Path::new("uniffi.toml");
    let out_dir = Utf8Path::new("target/generated-sources/uniffi/kotlin");

    uniffi_bindgen::generate_bindings(
        udl_file,
        Some(config_file),
        KotlinBindingGenerator,
        Some(out_dir),
        None,
        None,
        false,
    )
    .unwrap();
}

#[cfg(not(feature = "gen-bindings"))]
fn main() {}
