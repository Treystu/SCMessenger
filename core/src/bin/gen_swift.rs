#[cfg(feature = "gen-bindings")]
fn main() {
    use camino::Utf8Path;
    use uniffi::SwiftBindingGenerator;

    let udl_file = Utf8Path::new("src/api.udl");
    let out_dir = Utf8Path::new("target/generated-sources/uniffi/swift");

    uniffi_bindgen::generate_bindings(
        udl_file,
        None,
        SwiftBindingGenerator,
        Some(out_dir),
        None,
        None,
        false,
    )
    .unwrap();
}

#[cfg(not(feature = "gen-bindings"))]
fn main() {}
