#[cfg(feature = "gen-bindings")]
fn main() {
    use camino::Utf8Path;
    use std::fs;
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

    // Keep generated Swift bindings compatible with targets that default to
    // MainActor isolation (Swift 6 strict concurrency). UniFFI helper
    // converters must stay nonisolated so synchronous FFI call sites compile.
    let generated_file = out_dir.join("api.swift");
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
            (
                "FfiConverterTypeIronCoreError.lift",
                "try FfiConverterTypeIronCoreError.lift",
            ),
            (
                "try FfiConverterTypeIronCoreError.lift",
                "try rustCallWithError(FfiConverterTypeIronCoreError.lift",
            ),
            (
                "rustCallWithError(FfiConverterTypeIronCoreError.lift",
                "rustCallWithError { try FfiConverterTypeIronCoreError.lift",
            ),
            (
                "try rustCallWithError { try FfiConverterTypeIronCoreError.lift",
                "try rustCallWithError(FfiConverterTypeIronCoreError.lift",
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
