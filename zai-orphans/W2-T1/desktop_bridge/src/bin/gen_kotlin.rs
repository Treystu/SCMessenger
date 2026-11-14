#[cfg(feature = "gen-bindings")]
fn main() {
    use camino::Utf8PathBuf;
    use uniffi_bindgen::bindings::{generate, GenerateOptions, TargetLanguage};

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let manifest_path = Utf8PathBuf::from(manifest_dir);

    let config_file = manifest_path.join("uniffi.toml");
    let out_dir = manifest_path.join("target/generated-sources/uniffi/kotlin");

    std::fs::create_dir_all(&out_dir).expect("Failed to create output directory");

    let mut candidate_paths: Vec<Utf8PathBuf> = Vec::new();

    if let Ok(override_path) = std::env::var("SCMESSENGER_DESKTOP_CDYLIB_PATH") {
        candidate_paths.push(Utf8PathBuf::from(override_path));
    }

    let host_triple = std::env::var("HOST").unwrap_or_else(|_| "x86_64-pc-windows-msvc".into());
    let host_triple_dir = format!("../target/{}/debug", host_triple);
    let host_triple_dir_release = format!("../target/{}/release", host_triple);

    candidate_paths.extend(
        [
            host_triple_dir.as_str(),
            host_triple_dir_release.as_str(),
            "../target/debug",
            "../target/release",
        ]
        .iter()
        .map(|p| {
            let path = manifest_path.join(p);
            if cfg!(target_os = "windows") {
                path.join("scmessenger_desktop_bridge.dll")
            } else if cfg!(target_os = "macos") {
                path.join("libscmessenger_desktop_bridge.dylib")
            } else {
                path.join("libscmessenger_desktop_bridge.so")
            }
        }),
    );

    let library_file = candidate_paths
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| {
            panic!(
                "scmessenger_desktop_bridge cdylib not found. \
                 Please run: cargo build -p scmessenger-desktop-bridge"
            )
        });

    println!("Generating Kotlin bindings from library: {}", library_file);

    let options = GenerateOptions {
        languages: vec![TargetLanguage::Kotlin],
        source: library_file,
        out_dir: out_dir.clone(),
        config_override: config_file.exists().then_some(config_file),
        format: false,
        crate_filter: None,
        metadata_no_deps: false,
    };

    generate(options)
        .expect("Failed to generate Kotlin bindings. Check desktop_bridge/uniffi.toml");
}

#[cfg(not(feature = "gen-bindings"))]
fn main() {}
