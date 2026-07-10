// core/build.rs
use std::process;

fn main() {
    println!("cargo:rerun-if-changed=src/api.udl");

    // Get git commit hash
    let git_hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map(|output| {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                "unknown".to_string()
            }
        })
        .unwrap_or_else(|_| "unknown".to_string());

    // Get git branch name
    let git_branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map(|output| {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                "unknown".to_string()
            }
        })
        .unwrap_or_else(|_| "unknown".to_string());

    // Get build timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Format the stamp as "hash:branch:timestamp"
    let stamp = format!("{}:{}:{}", git_hash, git_branch, timestamp);

    println!("cargo:rustc-env=SCM_BUILD_STAMP={}", stamp);

    if let Err(e) = uniffi::generate_scaffolding("src/api.udl") {
        eprintln!("error: UniFFI scaffolding failed for src/api.udl");
        eprintln!("  {e}");
        process::exit(1);
    }
}
