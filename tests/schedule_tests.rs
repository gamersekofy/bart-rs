//! Service-day resolution: which services run on which date.

use std::path::PathBuf;

use bart_rs::Feed;
use bart_rs::attributes::calendar_dates::ExceptionType;

fn fixture_feed() -> Feed {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gtfs");
    Feed::open(&dir).expect("the fixture feed parses")
}

fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).expect("a valid date")
}

#[test]
fn a_weekday_activates_the_weekday_service() {
    let feed = fixture_feed();
    // 2026-10-19 is a Monday.
    let services = feed.service_ids_for(date(2026, 10, 19));
    assert!(services.contains("WEEKDAY"));
    assert!(!services.contains("SATURDAY"));
    assert!(!services.contains("SUNDAY"));
}

#[test]
fn a_saturday_activates_the_saturday_service() {
    let feed = fixture_feed();
    // 2026-10-24 is a Saturday, and has no exception against it.
    let services = feed.service_ids_for(date(2026, 10, 24));
    assert!(services.contains("SATURDAY"));
    assert!(!services.contains("WEEKDAY"));
}

#[test]
fn a_sunday_activates_the_sunday_service() {
    let feed = fixture_feed();
    // 2026-10-25 is a Sunday, and has no exception against it.
    let services = feed.service_ids_for(date(2026, 10, 25));
    assert!(services.contains("SUNDAY"));
    assert!(!services.contains("WEEKDAY"));
}

#[test]
fn a_removed_exception_takes_a_service_away() {
    let feed = fixture_feed();
    // The fixture removes SATURDAY on 2026-10-10, which is a Saturday.
    let services = feed.service_ids_for(date(2026, 10, 10));
    assert!(
        !services.contains("SATURDAY"),
        "the calendar_dates removal should win over the weekly pattern"
    );
}

#[test]
fn an_added_exception_brings_a_service_in() {
    let feed = fixture_feed();
    // The fixture adds WEEKDAY on 2026-10-11, a Sunday.
    let services = feed.service_ids_for(date(2026, 10, 11));
    assert!(
        services.contains("WEEKDAY"),
        "the calendar_dates addition should bring the weekday service in"
    );
    assert!(services.contains("SUNDAY"));
}

#[test]
fn dates_outside_the_calendar_window_have_no_services() {
    let feed = fixture_feed();
    let services = feed.service_ids_for(date(2025, 6, 1));
    assert!(services.is_empty());
}

#[test]
fn the_exception_types_are_distinguished() {
    let feed = fixture_feed();
    let removed = feed
        .calendar_dates
        .iter()
        .find(|e| e.service_id == "SATURDAY")
        .expect("an exception");
    assert_eq!(removed.exception_type, ExceptionType::Removed);
}

#[test]
fn arrivals_only_include_trips_whose_service_is_active() {
    let feed = fixture_feed();
    let weekday = date(2026, 10, 19);
    let arrivals = feed.arrivals_at("A10-1", weekday, 0);

    // Trip 1001 (WEEKDAY) and trip 1002 (WEEKDAY) stop at A10-1; trip 1004
    // (SUNDAY) does not run on a Monday.
    let trip_ids: Vec<&str> = arrivals.iter().map(|a| a.trip_id.as_str()).collect();
    assert!(trip_ids.contains(&"1001"));
    assert!(trip_ids.contains(&"1002"));
    assert!(!trip_ids.contains(&"1004"));
}

#[test]
fn arrivals_are_sorted_and_filtered_by_time() {
    let feed = fixture_feed();
    let weekday = date(2026, 10, 19);
    let arrivals = feed.arrivals_at("A10-1", weekday, 0);

    // 05:00, then 06:00, then the 24:30 train, which belongs to the same
    // service day and is therefore not wrapped round to 00:30.
    assert_eq!(arrivals[0].time, 5 * 3600);
    assert_eq!(arrivals[1].time, 6 * 3600);
    assert_eq!(arrivals[2].time, 24 * 3600 + 30 * 60);
    assert!(arrivals.windows(2).all(|w| w[0].time <= w[1].time));

    // A later cut-off drops the earlier train.
    let later = feed.arrivals_at("A10-1", weekday, 5 * 3600 + 60);
    assert_eq!(later.len(), 2);
    assert!(later.iter().all(|a| a.time >= 5 * 3600 + 60));
}

#[test]
fn a_trip_after_midnight_still_belongs_to_its_service_day() {
    let feed = fixture_feed();
    let weekday = date(2026, 10, 19);
    let arrivals = feed.arrivals_at("A10-1", weekday, 0);

    let late = arrivals
        .iter()
        .find(|a| a.trip_id == "1002")
        .expect("the late trip");
    assert_eq!(late.time, 24 * 3600 + 30 * 60);
    assert!(late.time > 24 * 3600, "the time must not be wrapped");
}

#[test]
fn departures_use_the_departure_time() {
    let feed = fixture_feed();
    let weekday = date(2026, 10, 19);
    let departures = feed.departures_from("A10-1", weekday, 0);
    let first = &departures[0];
    assert_eq!(first.trip_id, "1001");
    assert_eq!(first.time, 5 * 3600);
}

#[test]
fn the_current_service_date_rolls_over_after_midnight() {
    use chrono::TimeZone;

    let pacific = chrono_tz::America::Los_Angeles;
    let at = |y, m, d, h, min| {
        pacific
            .with_ymd_and_hms(y, m, d, h, min, 0)
            .single()
            .expect("a valid local time")
    };

    assert_eq!(
        bart_rs::schedule::current_service_date(at(2026, 10, 19, 2, 0)),
        date(2026, 10, 18)
    );
    assert_eq!(
        bart_rs::schedule::current_service_date(at(2026, 10, 19, 4, 1)),
        date(2026, 10, 19)
    );
    assert_eq!(
        bart_rs::schedule::current_service_date(at(2026, 10, 19, 4, 0)),
        date(2026, 10, 19),
        "the service day rolls over at 04:00"
    );
    assert_eq!(
        bart_rs::schedule::current_service_date(at(2026, 10, 19, 3, 59)),
        date(2026, 10, 18)
    );
    assert_eq!(
        bart_rs::schedule::current_service_date(at(2026, 10, 19, 23, 59)),
        date(2026, 10, 19)
    );
}
