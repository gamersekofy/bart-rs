//! The platform an arrival is at, and the difference between "on time" and
//! "nothing is known".
//!
//! Two things here were wrong at first and are worth keeping fixed:
//!
//! - BART's platform *ids* are built from a legacy station code and the
//!   platform number — `M80-1` for platform 1 of Balboa Park — which is not
//!   something to show a user. The number to show is the `platform_code`
//!   column of `stops.txt`.
//! - The realtime feed only covers the trains it is currently tracking. A
//!   scheduled train two hours out has no live update at all, and reporting
//!   that as "on time" is a claim the feed has not made.

use std::path::PathBuf;

use bart_rs::Feed;
use bart_rs::realtime::{LiveTrip, overlay};
use chrono::NaiveDate;

fn fixture_feed() -> Feed {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gtfs");
    Feed::open(&dir).expect("the fixture feed parses")
}

/// The fixture's Lake Merritt platforms are `A10-1` and `A10-2`, with
/// platform codes `1` and `2`.
#[test]
fn an_arrival_carries_the_platform_number_as_well_as_the_id() {
    let feed = fixture_feed();
    let date = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let arrivals = feed.arrivals_at("LAKE", date, 0);

    assert!(
        !arrivals.is_empty(),
        "the fixture has arrivals at Lake Merritt"
    );

    let arrival = &arrivals[0];
    assert_eq!(arrival.platform_id.as_deref(), Some("A10-1"));
    assert_eq!(arrival.platform_code.as_deref(), Some("1"));
}

#[test]
fn the_platform_label_is_the_number_not_the_id() {
    let feed = fixture_feed();
    let date = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let arrivals = feed.arrivals_at("LAKE", date, 0);

    assert_eq!(arrivals[0].platform_label(), Some("1"));
    assert_ne!(
        arrivals[0].platform_label(),
        arrivals[0].platform_id.as_deref()
    );
}

#[test]
fn the_platform_label_falls_back_to_the_id_when_there_is_no_code() {
    let mut arrival = fixture_feed()
        .arrivals_at("LAKE", NaiveDate::from_ymd_opt(2026, 8, 10).unwrap(), 0)
        .remove(0);

    arrival.platform_code = None;
    assert_eq!(arrival.platform_label(), Some("A10-1"));
}

/// A trip update that says nothing about the delay.
fn live_trip(trip_id: &str) -> LiveTrip {
    LiveTrip {
        trip_id: trip_id.to_string(),
        route_id: Some("5".to_string()),
        direction_id: Some(0),
        stops: vec![bart_rs::realtime::LiveStopTime {
            stop_id: Some("A10-1".to_string()),
            stop_sequence: Some(1),
            arrival_delay: 0,
            departure_delay: 0,
            arrival_time: Some(1_756_000_000),
            departure_time: Some(1_756_000_000),
            skipped: false,
        }],
    }
}

#[test]
fn a_train_with_a_live_update_is_marked_as_having_one() {
    let feed = fixture_feed();
    let date = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let scheduled = feed.arrivals_at("LAKE", date, 0);

    let trip_id = scheduled[0].trip_id.clone();
    let live = vec![live_trip(&trip_id)];

    let overlaid = overlay(&feed, &scheduled, &live, false);
    assert!(overlaid[0].has_update);
    assert_eq!(overlaid[0].delay, 0);
}

#[test]
fn a_train_the_realtime_feed_is_not_tracking_is_not_reported_as_on_time() {
    let feed = fixture_feed();
    let date = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let scheduled = feed.arrivals_at("LAKE", date, 0);

    // No live updates at all: the realtime feed knows nothing about any of
    // these trains, so none of them may claim to be on time.
    let overlaid = overlay(&feed, &scheduled, &[], false);

    assert!(!overlaid.is_empty());
    for arrival in &overlaid {
        assert!(
            !arrival.has_update,
            "trip {} should not be marked as having a live update",
            arrival.trip_id
        );
        assert_eq!(arrival.delay, 0);
    }
}

#[test]
fn the_overlay_keeps_the_platform_number() {
    let feed = fixture_feed();
    let date = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let scheduled = feed.arrivals_at("LAKE", date, 0);

    let overlaid = overlay(&feed, &scheduled, &[], false);
    assert_eq!(overlaid[0].platform_code.as_deref(), Some("1"));
    assert_eq!(overlaid[0].platform_label(), Some("1"));
}
