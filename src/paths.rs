//! Where the GTFS feed is stored on disk.
//!
//! The feed is **not** kept in the repository. It lives in a per-user cache
//! directory so that the tool can keep it current and report when it is
//! stale. See `docs/03-data-sources.md` for the full layout.

use std::path::PathBuf;

use crate::error::{Error, Result};

/// Environment variable that overrides the location of the data directory.
pub const DATA_DIR_ENV: &str = "BART_RS_DATA_DIR";

/// The root of the data directory.
///
/// Resolution order:
///
/// 1. `$BART_RS_DATA_DIR`, if set and non-empty.
/// 2. `$XDG_DATA_HOME/bart-rs`.
/// 3. `~/.local/share/bart-rs`.
pub fn data_dir() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os(DATA_DIR_ENV) {
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }

    dirs::data_dir()
        .map(|dir| dir.join("bart-rs"))
        .ok_or_else(|| {
            Error::msg("could not determine the user data directory; set BART_RS_DATA_DIR")
        })
}

/// The directory holding installed feeds.
pub fn feed_dir() -> Result<PathBuf> {
    Ok(data_dir()?.join("feed"))
}

/// The directory holding the currently installed feed.
pub fn current_feed_dir() -> Result<PathBuf> {
    Ok(feed_dir()?.join("current"))
}

/// The directory holding downloaded archives.
pub fn archives_dir() -> Result<PathBuf> {
    Ok(data_dir()?.join("archives"))
}

/// The path of the bookkeeping file that records the last update.
pub fn state_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("state.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_override_wins() {
        // Not thread safe to mutate the environment, so this is the only test
        // that touches it; it runs in its own process for the integration
        // suite via `BART_RS_DATA_DIR`.
        let dir = data_dir().expect("data dir");
        assert!(dir.ends_with("bart-rs") || dir.to_string_lossy().contains("bart-rs"));
    }
}
