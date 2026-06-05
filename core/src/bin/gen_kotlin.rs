#[cfg(feature = "gen-bindings")]
fn main() {
    use camino::Utf8PathBuf;
    use std::fs;
    use uniffi_bindgen::bindings::{generate, GenerateOptions, TargetLanguage};

    // Use CARGO_MANIFEST_DIR to resolve paths correctly regardless of working directory
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let manifest_path = Utf8PathBuf::from(manifest_dir);

    let _udl_file = manifest_path.join("src/api.udl");
    let config_file = manifest_path.join("uniffi.toml");
    let out_dir = manifest_path.join("target/generated-sources/uniffi/kotlin");

    // Create output directory
    std::fs::create_dir_all(&out_dir).expect("Failed to create output directory");

    // Locate the scmessenger-mobile cdylib for library-mode binding generation.
    // Library mode scans proc-macro metadata from the compiled cdylib and merges
    // it with the UDL declarations, producing complete bindings for proc-macro-
    // exported interfaces (IronCore, MeshService, ContactManager, etc.).
    //
    // Search order:
    //   1. SCMESSENGER_CDYLIB_PATH (explicit override, set by Gradle to bypass
    //      cargo incremental target invalidation races)
    //   2. CARGO_TARGET_DIR (if set) — both debug and release variants
    //   3. Hardcoded relative paths covering default target dir and the
    //      Windows MSVC triple (used by gradle's host-mode binding gen on Win)
    //   4. Panic with actionable message
    let mut candidate_paths: Vec<Utf8PathBuf> = Vec::new();

    if let Ok(override_path) = std::env::var("SCMESSENGER_CDYLIB_PATH") {
        candidate_paths.push(Utf8PathBuf::from(override_path));
    }

    let host_triple_dir = format!(
        "../target/{}/debug",
        std::env::var("HOST").unwrap_or_else(|_| "x86_64-pc-windows-msvc".into())
    );
    let host_triple_dir_release = format!(
        "../target/{}/release",
        std::env::var("HOST").unwrap_or_else(|_| "x86_64-pc-windows-msvc".into())
    );

    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        let target_dir = Utf8PathBuf::from(target_dir);
        candidate_paths.push(target_dir.join("debug/libscmessenger_mobile.so"));
        candidate_paths.push(target_dir.join("release/libscmessenger_mobile.so"));
        candidate_paths.push(target_dir.join("debug/scmessenger_mobile.dll"));
        candidate_paths.push(target_dir.join("release/scmessenger_mobile.dll"));
        candidate_paths.push(target_dir.join("debug/libscmessenger_mobile.dylib"));
        candidate_paths.push(target_dir.join("release/libscmessenger_mobile.dylib"));
        // Android per-ABI build outputs (the cdylib compiled for android targets
        // lives under <CARGO_TARGET_DIR>/<triple>/.../<name>.so).
        for triple in &[
            "aarch64-linux-android",
            "armv7-linux-androideabi",
            "x86_64-linux-android",
            "i686-linux-android",
        ] {
            candidate_paths
                .push(target_dir.join(format!("{triple}/debug/libscmessenger_mobile.so")));
            candidate_paths
                .push(target_dir.join(format!("{triple}/release/libscmessenger_mobile.so")));
        }
    }

    candidate_paths.extend(
        [
            host_triple_dir.as_str(),
            host_triple_dir_release.as_str(),
            "../target/x86_64-pc-windows-msvc/debug/scmessenger_mobile.dll",
            "../target/x86_64-pc-windows-msvc/release/scmessenger_mobile.dll",
            "../target/debug/scmessenger_mobile.dll",
            "../target/release/scmessenger_mobile.dll",
            "../target/debug/libscmessenger_mobile.so",
            "../target/release/libscmessenger_mobile.so",
            "../target/debug/libscmessenger_mobile.dylib",
            "../target/release/libscmessenger_mobile.dylib",
        ]
        .iter()
        .map(|p| manifest_path.join(p)),
    );

    let library_file = candidate_paths
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| {
            panic!(
                "scmessenger_mobile cdylib not found. \
                 Please run: cargo build -p scmessenger-mobile"
            )
        });

    println!("Generating Kotlin bindings from library: {}", library_file);

    // Use the new uniffi_bindgen 0.31 API with library mode
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
