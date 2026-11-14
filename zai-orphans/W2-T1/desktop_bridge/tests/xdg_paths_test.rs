//! Integration tests for XDG Base Directory path resolution
//!
//! Tests that `scmessenger_desktop_bridge::xdg_data_dir()` returns valid,
//! absolute paths on Linux and graceful fallbacks on other platforms.

use std::path::PathBuf;

/// Verify that `xdg_data_dir` returns an absolute path.
#[test]
fn test_xdg_data_dir() {
    let dir: PathBuf = scmessenger_desktop_bridge::xdg_data_dir();
    assert!(
        dir.is_absolute(),
        "xdg_data_dir() returned relative path: {dir:?}"
    );
}

/// Verify that `xdg_data_dir` path ends with "scmessenger".
#[test]
#[cfg(target_os = "linux")]
fn test_xdg_data_dir_contains_scmessenger() {
    let dir = scmessenger_desktop_bridge::xdg_data_dir();
    let dir_str = dir.to_string_lossy();
    assert!(
        dir_str.ends_with("scmessenger"),
        "xdg_data_dir() should end with 'scmessenger', got: {dir_str}"
    );
}

/// Verify that `xdg_config_dir` returns an absolute path.
#[test]
fn test_xdg_config_dir() {
    let dir: PathBuf = scmessenger_desktop_bridge::xdg_config_dir();
    assert!(
        dir.is_absolute(),
        "xdg_config_dir() returned relative path: {dir:?}"
    );
}

/// Verify that `xdg_config_dir` path ends with "scmessenger".
#[test]
#[cfg(target_os = "linux")]
fn test_xdg_config_dir_contains_scmessenger() {
    let dir = scmessenger_desktop_bridge::xdg_config_dir();
    let dir_str = dir.to_string_lossy();
    assert!(
        dir_str.ends_with("scmessenger"),
        "xdg_config_dir() should end with 'scmessenger', got: {dir_str}"
    );
}

/// Verify that overriding XDG_DATA_HOME is respected.
#[test]
#[cfg(target_os = "linux")]
fn test_xdg_data_home_env_override() {
    // Save original value
    let original = std::env::var("XDG_DATA_HOME").ok();

    // Set override
    std::env::set_var("XDG_DATA_HOME", "/tmp/test_xdg_override");

    let dir = scmessenger_desktop_bridge::xdg_data_dir();
    let dir_str = dir.to_string_lossy().to_string();
    assert!(
        dir_str.starts_with("/tmp/test_xdg_override"),
        "xdg_data_dir() should respect XDG_DATA_HOME override, got: {dir_str}"
    );

    // Restore original value
    match original {
        Some(val) => std::env::set_var("XDG_DATA_HOME", val),
        None => std::env::remove_var("XDG_DATA_HOME"),
    }
}

/// Verify that `desktop_version` returns a non-empty string.
#[test]
fn test_desktop_version_non_empty() {
    let version: String = scmessenger_desktop_bridge::desktop_version();
    assert!(!version.is_empty(), "desktop_version() should not be empty");
}
