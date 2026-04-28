#[cfg(feature = "gen-bindings")]
fn main() {
    use camino::Utf8PathBuf;
    use std::fs;
    use uniffi_bindgen::bindings::{generate, GenerateOptions, TargetLanguage};

    // Use CARGO_MANIFEST_DIR to resolve paths correctly regardless of working directory
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let manifest_path = Utf8PathBuf::from(manifest_dir);

    let udl_file = manifest_path.join("src/api.udl");
    let config_file = manifest_path.join("uniffi.toml");
    let out_dir = manifest_path.join("target/generated-sources/uniffi/kotlin");

    // Create output directory
    std::fs::create_dir_all(&out_dir).expect("Failed to create output directory");

    // Use the new uniffi_bindgen 0.31 API
    let options = GenerateOptions {
        languages: vec![TargetLanguage::Kotlin],
        source: udl_file,
        out_dir: out_dir.clone(),
        config_override: config_file.exists().then_some(config_file),
        format: false,
        crate_filter: None,
        metadata_no_deps: false,
    };

    generate(options)
        .expect("Failed to generate Kotlin bindings. Check core/uniffi.toml and core/src/api.udl");

    // Keep Android lint stable for generated cleaner code paths without relying
    // on Gradle post-generation mutation.
    let generated_file = out_dir.join("uniffi/api/api.kt");
    if generated_file.exists() {
        let suppress = "@file:android.annotation.SuppressLint(\"NewApi\")";
        let content = fs::read_to_string(generated_file.as_std_path())
            .expect("Failed to read generated Kotlin bindings");
        if !content.starts_with(suppress) {
            fs::write(
                generated_file.as_std_path(),
                format!("{suppress}\n{content}"),
            )
            .expect("Failed to update generated Kotlin bindings with lint suppression");
        }
    }
}

#[cfg(not(feature = "gen-bindings"))]
fn main() {}
