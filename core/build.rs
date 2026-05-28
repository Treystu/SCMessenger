use std::process;

fn main() {
    println!("cargo:rerun-if-changed=src/api.udl");

    if let Err(e) = uniffi::generate_scaffolding("src/api.udl") {
        eprintln!("error: UniFFI scaffolding failed for src/api.udl");
        eprintln!("  {e}");
        process::exit(1);
    }
}
