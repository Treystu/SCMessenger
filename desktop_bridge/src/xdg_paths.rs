// XDG Base Directory specification path resolution.
//
// Implements https://specifications.freedesktop.org/basedir-spec/
// Falls back gracefully when XDG variables are unset:
//   XDG_DATA_HOME → ~/.local/share/scmessenger
//   XDG_CONFIG_HOME → ~/.config/scmessenger
//   XDG_CACHE_HOME → ~/.cache/scmessenger
//   XDG_RUNTIME_DIR → None (falls back to /tmp if needed)

use std::path::PathBuf;

/// Resolve the full set of XDG paths for SCMessenger.
///
/// On Linux/Unix, this respects the XDG Base Directory spec.
/// On other platforms, it falls back to `dirs::data_dir()`.
pub fn resolve_xdg_paths() -> crate::XdgPaths {
    #[cfg(target_os = "linux")]
    {
        let data_home = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .ok()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".local/share")
            });

        let config_home = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .ok()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".config")
            });

        let cache_home = std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .ok()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".cache")
            });

        let data_dir = data_home.join("scmessenger");
        let config_dir = config_home.join("scmessenger");
        let cache_dir = cache_home.join("scmessenger");
        let store_path = data_dir.join("store");

        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .ok()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.join("scmessenger"))
            .and_then(|p| {
                if p.exists() || std::fs::create_dir_all(&p).is_ok() {
                    Some(p.to_string_lossy().to_string())
                } else {
                    None
                }
            });

        crate::XdgPaths {
            data_dir: data_dir.to_string_lossy().to_string(),
            config_dir: config_dir.to_string_lossy().to_string(),
            cache_dir: cache_dir.to_string_lossy().to_string(),
            runtime_dir,
            store_path: store_path.to_string_lossy().to_string(),
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Non-Linux fallback: use dirs crate defaults
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("scmessenger");
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("scmessenger");
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("scmessenger");

        crate::XdgPaths {
            data_dir: data_dir.to_string_lossy().to_string(),
            config_dir: config_dir.to_string_lossy().to_string(),
            cache_dir: cache_dir.to_string_lossy().to_string(),
            runtime_dir: None,
            store_path: data_dir.join("store").to_string_lossy().to_string(),
        }
    }
}

/// Return the default sled store path.
/// On Linux: `$XDG_DATA_HOME/scmessenger/store` (via XdgPaths)
pub fn default_store_path() -> String {
    resolve_xdg_paths().store_path
}

/// Ensure all required XDG directories exist.
/// Returns Ok(()) on success, or an error string on failure.
pub fn ensure_directories(paths: &crate::XdgPaths) -> Result<(), String> {
    for dir in [&paths.data_dir, &paths.config_dir, &paths.cache_dir] {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create directory {dir}: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xdg_paths_returns_non_empty() {
        let paths = resolve_xdg_paths();
        assert!(!paths.data_dir.is_empty());
        assert!(!paths.config_dir.is_empty());
        assert!(!paths.cache_dir.is_empty());
        assert!(!paths.store_path.is_empty());
    }

    #[test]
    fn test_xdg_paths_contains_scmessenger() {
        let paths = resolve_xdg_paths();
        assert!(
            paths.data_dir.ends_with("scmessenger"),
            "data_dir: {}",
            paths.data_dir
        );
        assert!(
            paths.config_dir.ends_with("scmessenger"),
            "config_dir: {}",
            paths.config_dir
        );
    }

    #[test]
    fn test_store_path_under_data_dir() {
        let paths = resolve_xdg_paths();
        assert!(
            paths.store_path.ends_with("store"),
            "store_path: {}",
            paths.store_path
        );
    }
}
