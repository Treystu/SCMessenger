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
    let out_dir = manifest_path.join("target/generated-sources/uniffi/swift");

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&out_dir).expect("Failed to create output directory");

    // Locate the scmessenger-mobile cdylib for library-mode binding generation.
    // Library mode scans proc-macro metadata from the compiled cdylib and merges
    // it with the UDL declarations, producing complete bindings for proc-macro-
    // exported interfaces (IronCore, MeshService, ContactManager, etc.).
    //
    // Search order:
    //   1. SCMESSENGER_CDYLIB_PATH (explicit override, set by the build system
    //      to bypass cargo incremental target invalidation races)
    //   2. CARGO_TARGET_DIR (if set) — both debug and release variants, all OSes
    //   3. Hardcoded relative paths covering default target dir
    //   4. Panic with actionable message
    let mut candidate_paths: Vec<Utf8PathBuf> = Vec::new();

    if let Ok(override_path) = std::env::var("SCMESSENGER_CDYLIB_PATH") {
        candidate_paths.push(Utf8PathBuf::from(override_path));
    }

    let host_triple_dir = format!(
        "../target/{}/debug",
        std::env::var("HOST").unwrap_or_else(|_| "x86_64-apple-darwin".into())
    );
    let host_triple_dir_release = format!(
        "../target/{}/release",
        std::env::var("HOST").unwrap_or_else(|_| "x86_64-apple-darwin".into())
    );

    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        let target_dir = Utf8PathBuf::from(target_dir);
        candidate_paths.push(target_dir.join("debug/libscmessenger_core.so"));
        candidate_paths.push(target_dir.join("release/libscmessenger_core.so"));
        candidate_paths.push(target_dir.join("debug/scmessenger_core.dll"));
        candidate_paths.push(target_dir.join("release/scmessenger_core.dll"));
        candidate_paths.push(target_dir.join("debug/libscmessenger_core.dylib"));
        candidate_paths.push(target_dir.join("release/libscmessenger_core.dylib"));
        for triple in &[
            "aarch64-linux-android",
            "armv7-linux-androideabi",
            "x86_64-linux-android",
            "i686-linux-android",
            "aarch64-apple-ios",
            "aarch64-apple-ios-sim",
        ] {
            candidate_paths.push(target_dir.join(format!("{triple}/debug/libscmessenger_core.so")));
            candidate_paths
                .push(target_dir.join(format!("{triple}/release/libscmessenger_core.so")));
        }
    }

    candidate_paths.extend(
        [
            host_triple_dir.as_str(),
            host_triple_dir_release.as_str(),
            "../target/debug/libscmessenger_core.so",
            "../target/release/libscmessenger_core.so",
            "../target/debug/libscmessenger_core.dylib",
            "../target/release/libscmessenger_core.dylib",
        ]
        .iter()
        .map(|p| manifest_path.join(p)),
    );

    let library_file = candidate_paths
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| {
            panic!(
                "scmessenger_core cdylib not found. \
                 Please run: cargo build -p scmessenger-core"
            )
        });

    println!("Generating Swift bindings from library: {}", library_file);

    // Use the new uniffi_bindgen 0.31 API with library mode
    let options = GenerateOptions {
        languages: vec![TargetLanguage::Swift],
        source: library_file,
        out_dir: out_dir.clone(),
        config_override: config_file.exists().then_some(config_file),
        format: false,
        crate_filter: None,
        metadata_no_deps: false,
    };

    generate(options)
        .expect("Failed to generate Swift bindings. Check core/uniffi.toml and core/src/api.udl");

    // Keep generated Swift bindings compatible with targets that default to
    // MainActor isolation (Swift 6 strict concurrency). UniFFI helper
    // converters must stay nonisolated so synchronous FFI call sites compile.
    // UniFFI names the generated Swift file after the module_name set in
    // uniffi.toml.  Try the configured name first, then fall back to the
    // legacy "api.swift" name for compatibility.
    let generated_file = {
        let module_file = out_dir.join("SCMessengerCore.swift");
        let legacy_file = out_dir.join("api.swift");
        if module_file.exists() {
            module_file
        } else {
            legacy_file
        }
    };
    if generated_file.exists() {
        let mut content = fs::read_to_string(generated_file.as_std_path())
            .expect("Failed to read generated Swift bindings");

        let replacements = [
            (
                "static func lift(_ value: FfiType) throws -> SwiftType",
                "nonisolated static func lift(_ value: FfiType) throws -> SwiftType",
            ),
            (
                "static func lower(_ value: SwiftType) -> FfiType",
                "nonisolated static func lower(_ value: SwiftType) -> FfiType",
            ),
            (
                "static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> SwiftType",
                "nonisolated static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> SwiftType",
            ),
            (
                "static func write(_ value: SwiftType, into buf: inout [UInt8])",
                "nonisolated static func write(_ value: SwiftType, into buf: inout [UInt8])",
            ),
            (
                "public static func lift(_ value: FfiType) throws -> SwiftType",
                "public nonisolated static func lift(_ value: FfiType) throws -> SwiftType",
            ),
            (
                "public static func lower(_ value: SwiftType) -> FfiType",
                "public nonisolated static func lower(_ value: SwiftType) -> FfiType",
            ),
            (
                "public static func lift(_ buf: RustBuffer) throws -> SwiftType",
                "public nonisolated static func lift(_ buf: RustBuffer) throws -> SwiftType",
            ),
            (
                "public static func lower(_ value: SwiftType) -> RustBuffer",
                "public nonisolated static func lower(_ value: SwiftType) -> RustBuffer",
            ),
        ];

        for (from, to) in replacements {
            if content.contains(from) && !content.contains(to) {
                content = content.replacen(from, to, 1);
            }
        }

        fs::write(generated_file.as_std_path(), content)
            .expect("Failed to update generated Swift bindings for actor-isolation compatibility");
    }
}

#[cfg(not(feature = "gen-bindings"))]
fn main() {}
