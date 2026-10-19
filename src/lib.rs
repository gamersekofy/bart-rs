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
//!
//! # Building a GUI on top of this
//!
//! The whole life cycle is three steps: make sure the feed on disk is
//! current, load it, and overlay the realtime data.
//!
//! ```no_run
//! use bart_rs::realtime;
//! use bart_rs::schedule;
//! use bart_rs::update::{self, UpdateOutcome};
//!
//! let client = update::client()?;
//!
//! // A HEAD request when the feed is already current, so it costs nothing.
//! match update::ensure_current(&client)? {
//!     UpdateOutcome::UpToDate { .. } => {}
//!     UpdateOutcome::Updated { bytes, .. } => eprintln!("downloaded {bytes} bytes"),
//!     // Being offline is not a failure: the installed feed is still good.
//!     UpdateOutcome::Unreachable { reason } => eprintln!("offline: {reason}"),
//! }
//!
//! let feed = bart_rs::Feed::from_data_dir()?;
//! let stop = feed.find_stop("balboa park").expect("a station");
//!
//! let (date, from) = schedule::now_service_day();
//! let scheduled = feed.arrivals_at(&stop.id, date, from);
//!
//! let live = realtime::fetch_trip_updates(&client)?;
//! for arrival in realtime::overlay(&feed, &scheduled, &live, false).iter().take(5) {
//!     println!(
//!         "{} {}",
//!         schedule::fmt_human_time(arrival.time),
//!         arrival.route_short_name.as_deref().unwrap_or(""),
//!     );
//! }
//! # Ok::<(), bart_rs::Error>(())
//! ```
//!
//! **Everything here blocks.** Parsing the feed takes about 65 ms and a
//! realtime fetch about 500 ms, so do both on a worker thread, not on the UI
//! thread.
//!
//! Four things about the upstream data are not what the GTFS specification
//! suggests, and each of them has cost a bug here. They are the ones worth
//! knowing before you build on top of this:
//!
//! - The feed's coverage window is in the calendar, **not** in
//!   `feed_info.txt`, which describes the previous release. Use
//!   [`Feed::coverage`].
//! - BART's *service day* starts at 04:00, so "today" is not always the
//!   date a train belongs to. Use [`schedule::now_service_day`].
//! - A platform's id is not its number: `M80-1` is platform 1 of Balboa
//!   Park. Use [`feed::Arrival::platform_label`], which gives you the number
//!   the sign shows.
//! - The realtime feed only covers the trains it is tracking, so a train
//!   two hours out has no live update at all. A zero delay does **not** mean
//!   punctual; check [`realtime::LiveArrival::has_update`].
//!
//! The full guide, with a worked example and the threading notes, is in
//! `docs/08-library-guide.md`.

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
