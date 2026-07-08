use std::process::Command;

fn main() {
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let build_time = chrono::Utc::now().to_rfc3339();

    println!("cargo:rustc-env=SCM_GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=SCM_BUILD_TIME={}", build_time);
}
