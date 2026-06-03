/// Desktop bridge for SCMessenger on Linux.
///
/// Provides XDG Base Directory path resolution and desktop integration helpers.
/// Non-Linux targets compile stubs that return empty/default values.

/// Returns the desktop bridge version string.
pub fn desktop_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Returns the XDG data directory for SCMessenger.
///
/// On Linux: respects `XDG_DATA_HOME`, defaulting to `$HOME/.local/share/scmessenger`.
/// On other targets: returns the current directory as a fallback.
pub fn xdg_data_dir() -> std::path::PathBuf {
    #[cfg(target_os = "linux")]
    {
        if let Some(dir) = dirs::data_dir() {
            return dir.join("scmessenger");
        }
    }
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}

/// Returns the XDG config directory for SCMessenger.
///
/// On Linux: respects `XDG_CONFIG_HOME`, defaulting to `$HOME/.config/scmessenger`.
/// On other targets: returns the current directory as a fallback.
pub fn xdg_config_dir() -> std::path::PathBuf {
    #[cfg(target_os = "linux")]
    {
        if let Some(dir) = dirs::config_dir() {
            return dir.join("scmessenger");
        }
    }
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_version() {
        let v = desktop_version();
        assert!(!v.is_empty(), "version should not be empty");
    }

    #[test]
    fn test_xdg_data_dir_returns_path() {
        let path = xdg_data_dir();
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_xdg_config_dir_returns_path() {
        let path = xdg_config_dir();
        assert!(!path.as_os_str().is_empty());
    }
}
