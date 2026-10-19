//! Cross-checks against the live BART feed.
//!
//! These tests need network access, so they are `#[ignore]`d by default:
//!
//! ```text
//! cargo test -- --ignored
//! ```
//!
//! They download the real feed and check that our hand-rolled parser agrees
//! with `gtfs-structures`, an independent reference implementation. Run them
//! before and after changing anything in `src/attributes/` or `src/feed.rs`;
//! they are the fastest way to know whether a change broke reality.

use std::path::{Path, PathBuf};

use bart_rs::Feed;

/// A scratch directory, unique per call so that the tests, which run in
/// parallel, do not fight over the same one.
fn scratch_dir() -> PathBuf {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let dir = std::env::temp_dir().join(format!("bart-rs-live-{}-{unique}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("creates the scratch directory");
    dir
}

fn download(dir: &Path) -> Feed {
    let client = bart_rs::update::client().expect("builds a client");
    let download = bart_rs::update::download_latest(&client).expect("downloads the feed");

    let staging = dir.join("staging");
    bart_rs::update::extract(&download.bytes, &staging).expect("extracts the feed");
    bart_rs::update::install(&staging, &dir.join("feed")).expect("installs the feed");

    Feed::open(&dir.join("feed/current")).expect("parses the feed")
}

#[test]
#[ignore = "requires network access"]
fn the_live_feed_parses() {
    let dir = scratch_dir();
    let feed = download(&dir);

    assert!(!feed.routes.is_empty());
    assert!(!feed.stops.is_empty());
    assert!(!feed.trips.is_empty());
    assert!(!feed.stop_times.is_empty());

    // Every platform and entrance must resolve to a station.
    for stop in feed.stops.iter() {
        if stop.is_platform() || stop.is_entrance() {
            assert!(
                stop.parent_station.is_some(),
                "{} has no parent station",
                stop.id
            );
            assert!(
                feed.station_of(&stop.id).is_some(),
                "{} does not resolve to a station",
                stop.id
            );
        }
    }

    // Every stop time must reference a stop that exists.
    for time in &feed.stop_times {
        assert!(
            feed.stop(&time.stop_id).is_some(),
            "unknown stop {}",
            time.stop_id
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "requires network access"]
fn the_live_feed_agrees_with_the_reference_implementation() {
    let dir = scratch_dir();
    let feed = download(&dir);

    let reference = gtfs_structures::Gtfs::new(dir.join("feed/current").to_str().expect("a path"))
        .expect("the reference implementation parses the feed");

    assert_eq!(feed.routes.len(), reference.routes.len(), "route count");
    assert_eq!(feed.stops.len(), reference.stops.len(), "stop count");
    assert_eq!(feed.trips.len(), reference.trips.len(), "trip count");
    assert_eq!(
        feed.fare_attributes.len(),
        reference.fare_attributes.len(),
        "fare count"
    );

    // The reference groups shape points by shape id; we keep a flat list of
    // points, so compare the totals rather than the shape count.
    let reference_points: usize = reference.shapes.values().map(|points| points.len()).sum();
    assert_eq!(feed.shapes.len(), reference_points, "shape point count");
    assert_eq!(
        reference.shapes.len(),
        feed.shapes
            .iter()
            .map(|shape| shape.shape_id.as_str())
            .collect::<std::collections::HashSet<_>>()
            .len(),
        "distinct shape id count"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "requires network access"]
fn the_live_alert_feed_decodes() {
    let client = bart_rs::update::client().expect("builds a client");
    let alerts = bart_rs::realtime::fetch_alerts(&client).expect("fetches the alerts");

    for alert in &alerts {
        assert!(
            !alert.is_empty(),
            "alert {} produced no text at all",
            alert.id
        );
    }
}

#[test]
#[ignore = "requires network access"]
fn the_live_trip_update_feed_decodes() {
    let client = bart_rs::update::client().expect("builds a client");
    let trips = bart_rs::realtime::fetch_trip_updates(&client).expect("fetches the trip updates");

    for trip in &trips {
        assert!(!trip.trip_id.is_empty());
    }
}
