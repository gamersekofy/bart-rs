//! The feed's coverage window.
//!
//! BART publishes a new feed without updating `feed_info.txt`'s dates, so
//! those describe the *previous* release. The real window has to come from
//! the calendar. This is the regression test for a bug where `bart-rs status`
//! reported a perfectly current feed as expired.

use std::path::PathBuf;

use bart_rs::attributes::calendar::coverage;
use bart_rs::feed::Feed;
use bart_rs::update::{staleness, Staleness, UpdateState};
use chrono::{DateTime, NaiveDate};

/// A `feed_info.txt` that describes an earlier window, the way BART's does.
const STALE_FEED_INFO: &str =
    "feed_publisher_name,feed_publisher_url,feed_lang,feed_start_date,feed_end_date,feed_version\n\
Bay Area Rapid Transit,http://www.bart.gov,en,20250101,20251231,72\n";

/// Removes a scratch directory when it goes out of scope.
struct Cleanup(PathBuf);

impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Build a feed directory from the fixture, with the given `feed_info.txt`.
fn feed_with_feed_info(feed_info: &str) -> (Cleanup, Feed) {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let root =
        std::env::temp_dir().join(format!("bart-rs-coverage-{}-{unique}", std::process::id()));
    let feed_dir = root.join("current");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&feed_dir).expect("creates the feed directory");

    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gtfs");
    for entry in std::fs::read_dir(&source).expect("reads the fixture directory") {
        let entry = entry.expect("a fixture entry");
        std::fs::copy(entry.path(), feed_dir.join(entry.file_name())).expect("copies a fixture");
    }

    std::fs::write(feed_dir.join("feed_info.txt"), feed_info).expect("writes the feed info");

    let feed = Feed::open(&feed_dir).expect("the fixture feed parses");
    (Cleanup(root), feed)
}

#[test]
fn coverage_comes_from_the_calendar_not_from_feed_info() {
    let (_cleanup, feed) = feed_with_feed_info(STALE_FEED_INFO);

    // The calendar says 2026-01-01 to 2026-12-31.
    let (start, end) = feed.coverage().expect("the feed has a coverage window");
    assert_eq!(start, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
    assert_eq!(end, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());

    // feed_info.txt claims something else, and is ignored.
    let claimed = feed.feed_info().expect("the feed has metadata");
    assert_eq!(
        claimed.start_date,
        Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
    );
    assert_eq!(
        claimed.end_date,
        Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap())
    );
}

#[test]
fn a_current_feed_is_not_reported_as_expired() {
    let (_cleanup, feed) = feed_with_feed_info(STALE_FEED_INFO);

    let state = UpdateState {
        feed_version: Some("72".to_string()),
        ..UpdateState::default()
    };

    // A date in the middle of the calendar's window.
    let now = DateTime::from_timestamp(1_767_225_600, 0).expect("a valid timestamp");
    let warnings = staleness(feed.coverage(), &state, now);

    assert!(
        !warnings.contains(&Staleness::Expired { days: 1 }),
        "the feed was reported as expired: {warnings:?}"
    );
    assert!(
        !warnings.contains(&Staleness::ExpiringSoon { days: 1 }),
        "the feed was reported as expiring: {warnings:?}"
    );
}

#[test]
fn a_feed_past_its_window_is_still_reported_as_expired() {
    let (_cleanup, feed) = feed_with_feed_info(STALE_FEED_INFO);

    let state = UpdateState {
        feed_version: Some("72".to_string()),
        ..UpdateState::default()
    };

    // Well after the calendar's window has closed.
    let now = DateTime::from_timestamp(1_800_000_000, 0).expect("a valid timestamp");
    let warnings = staleness(feed.coverage(), &state, now);

    assert!(
        warnings
            .iter()
            .any(|w| matches!(w, Staleness::Expired { .. })),
        "the expired feed was not reported: {warnings:?}"
    );
}

#[test]
fn a_feed_with_no_calendar_has_no_coverage_window() {
    assert!(coverage(&[], &[]).is_none());
}

#[test]
fn a_single_date_exception_widens_the_window() {
    let (_cleanup, feed) = feed_with_feed_info(STALE_FEED_INFO);

    // The fixture's calendar_dates.txt reaches past the calendar's end.
    let (start, end) = feed.coverage().expect("the feed has a coverage window");
    assert!(start <= NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
    assert!(end >= NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
}
