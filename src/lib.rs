//! `bart-rs` — BART schedule and realtime data, for the command line and for
//! other Rust projects.
//!
//! The crate ships a library (this module) and a thin binary that wraps it.
//!
//! # What is in here
//!
//! - [`attributes`] — one module per GTFS table, each with a typed struct and
//!   a `parse_*_file` function.
//! - [`feed`] — [`Feed`], the in-memory representation of a whole GTFS feed
//!   plus the cross-file queries built on top of it.
//! - [`schedule`] — service-day resolution: which services run on a given date.
//! - [`update`] — downloading, extracting and installing a feed. *(requires the
//!   `download` feature)*
//! - [`realtime`] — GTFS-Realtime trip updates and service alerts.
//! - [`gtfsrt`] — the GTFS-Realtime protobuf message definitions.
//! - [`paths`] — where the feed is stored on disk.
//!
//! # Example
//!
//! ```no_run
//! use bart_rs::Feed;
//!
//! let feed = Feed::from_data_dir()?;
//! for route in &feed.routes {
//!     println!("{} {}", route.id, route.long_name.as_deref().unwrap_or(""));
//! }
//! # Ok::<(), bart_rs::Error>(())
//! ```

#![warn(missing_docs)]

pub mod attributes;
pub mod error;
pub mod feed;
pub mod gtfsrt;
pub mod helper;
pub mod paths;
pub mod realtime;
pub mod schedule;

#[cfg(feature = "download")]
pub mod update;

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod commands;

pub use crate::error::{Error, Result};
pub use feed::Feed;
pub use paths::data_dir;
