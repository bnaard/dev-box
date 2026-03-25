//! XDG Base Directory paths for aibox global state.
//!
//! Follows the XDG Base Directory Specification:
//!   - Config:  $XDG_CONFIG_HOME/aibox  (default: ~/.config/aibox)
//!   - Cache:   $XDG_CACHE_HOME/aibox   (default: ~/.cache/aibox)
//!   - Data:    $XDG_DATA_HOME/aibox    (default: ~/.local/share/aibox)
//!
//! We always use XDG-style paths (~/.config, ~/.cache, ~/.local/share)
//! even on macOS, for cross-platform consistency and to match the install
//! script which places addon definitions at ~/.config/aibox/addons.

use std::path::PathBuf;

const APP_NAME: &str = "aibox";

/// Global config directory: $XDG_CONFIG_HOME/aibox or ~/.config/aibox
///
/// We always use XDG-style paths (even on macOS) for cross-platform consistency
/// and to match the install script which places addons at ~/.config/aibox/addons.
pub fn config_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        Some(PathBuf::from(xdg).join(APP_NAME))
    } else {
        dirs::home_dir().map(|d| d.join(".config").join(APP_NAME))
    }
}

/// Global cache directory: $XDG_CACHE_HOME/aibox or ~/.cache/aibox
pub fn cache_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        Some(PathBuf::from(xdg).join(APP_NAME))
    } else {
        dirs::home_dir().map(|d| d.join(".cache").join(APP_NAME))
    }
}

/// Global data directory: $XDG_DATA_HOME/aibox or ~/.local/share/aibox
pub fn data_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        Some(PathBuf::from(xdg).join(APP_NAME))
    } else {
        dirs::home_dir().map(|d| d.join(".local").join("share").join(APP_NAME))
    }
}

/// All global directories that aibox may have created.
/// Used by `uninstall --purge` to clean up.
pub fn all_global_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(d) = config_dir() {
        dirs.push(d);
    }
    if let Some(d) = cache_dir() {
        dirs.push(d);
    }
    if let Some(d) = data_dir() {
        dirs.push(d);
    }
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_ends_with_aibox() {
        if let Some(d) = config_dir() {
            assert!(
                d.ends_with("aibox"),
                "config dir should end with 'aibox': {:?}",
                d
            );
        }
    }

    #[test]
    fn cache_dir_ends_with_aibox() {
        if let Some(d) = cache_dir() {
            assert!(
                d.ends_with("aibox"),
                "cache dir should end with 'aibox': {:?}",
                d
            );
        }
    }

    #[test]
    fn xdg_env_override_works() {
        // SAFETY: test runs single-threaded via serial_test or cargo test default
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", "/tmp/test-xdg-config");
        }
        let d = config_dir().unwrap();
        assert_eq!(d, PathBuf::from("/tmp/test-xdg-config/aibox"));
        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }
}
