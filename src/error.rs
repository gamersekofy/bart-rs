//! Error types.

use std::path::{Path, PathBuf};

/// The error type used throughout the crate.
///
/// The `Http` variant only exists when the `download` feature is enabled,
/// since that is the only configuration in which an HTTP client is compiled
/// in.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A file could not be read or written.
    #[error("failed to read {path}")]
    Io {
        /// The path that could not be read.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// A GTFS file could not be parsed.
    #[error("failed to parse {path}")]
    Csv {
        /// The path that could not be parsed.
        path: PathBuf,
        /// The underlying CSV error.
        #[source]
        source: csv::Error,
    },

    /// No feed has been downloaded yet.
    #[error("no GTFS feed found at {0} (run `bart-rs update`)")]
    FeedNotFound(PathBuf),

    /// The feed is there, but one of its tables is not.
    #[error("the feed at {path} is missing {table}")]
    MissingTable {
        /// The directory the feed was read from.
        path: PathBuf,
        /// The table that is missing.
        table: String,
    },

    /// A network request failed.
    #[cfg(feature = "download")]
    #[error("network error while fetching {url}")]
    Http {
        /// The URL that was requested.
        url: String,
        /// The underlying HTTP error.
        #[source]
        source: reqwest::Error,
    },

    /// A server returned a non-success status.
    #[error("{url} returned status {status}")]
    HttpStatus {
        /// The URL that was requested.
        url: String,
        /// The status code that was returned.
        status: u16,
    },

    /// The downloaded archive was not a usable GTFS feed.
    #[error("the downloaded archive is not a valid GTFS feed: {0}")]
    BadArchive(String),

    /// A GTFS-Realtime payload could not be decoded.
    #[error("failed to decode the GTFS-Realtime payload: {0}")]
    RealtimeDecode(String),

    /// The on-disk state file could not be read or written.
    #[error("failed to read the feed state file {path}")]
    State {
        /// The path of the state file.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Anything else, with a message meant for the user.
    #[error("{0}")]
    Message(String),
}

impl Error {
    /// Build a [`Error::Message`] from anything string-like.
    pub fn msg(message: impl Into<String>) -> Self {
        Error::Message(message.into())
    }

    /// Build an [`Error::Io`] that remembers which path failed.
    pub fn io(path: impl AsRef<Path>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }

    /// Build an [`Error::Csv`] that remembers which path failed to parse.
    pub fn csv(path: impl AsRef<Path>, source: csv::Error) -> Error {
        Error::Csv {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }
}

/// A `Result` alias for this crate.
pub type Result<T> = std::result::Result<T, Error>;
