fn main() {
    println!("cargo:rerun-if-changed=src/api.udl");
    uniffi::generate_scaffolding("src/api.udl").unwrap();
}
