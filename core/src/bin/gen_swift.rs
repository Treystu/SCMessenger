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
    let out_dir = manifest_path.join("target/generated-sources/uniffi/swift");

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&out_dir).expect("Failed to create output directory");

    // Use the new uniffi_bindgen 0.31 API
    let options = GenerateOptions {
        languages: vec![TargetLanguage::Swift],
        source: udl_file,
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
                "nonisolated(unsafe) static func lift(_ value: FfiType) throws -> SwiftType",
            ),
            (
                "static func lower(_ value: SwiftType) -> FfiType",
                "nonisolated(unsafe) static func lower(_ value: SwiftType) -> FfiType",
            ),
            (
                "static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> SwiftType",
                "nonisolated(unsafe) static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> SwiftType",
            ),
            (
                "static func write(_ value: SwiftType, into buf: inout [UInt8])",
                "nonisolated(unsafe) static func write(_ value: SwiftType, into buf: inout [UInt8])",
            ),
            (
                "public static func lift(_ value: FfiType) throws -> SwiftType",
                "public nonisolated(unsafe) static func lift(_ value: FfiType) throws -> SwiftType",
            ),
            (
                "public static func lower(_ value: SwiftType) -> FfiType",
                "public nonisolated(unsafe) static func lower(_ value: SwiftType) -> FfiType",
            ),
            (
                "public static func lift(_ buf: RustBuffer) throws -> SwiftType",
                "public nonisolated(unsafe) static func lift(_ buf: RustBuffer) throws -> SwiftType",
            ),
            (
                "public static func lower(_ value: SwiftType) -> RustBuffer",
                "public nonisolated(unsafe) static func lower(_ value: SwiftType) -> RustBuffer",
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
