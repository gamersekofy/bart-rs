//! Parsing each GTFS table from the committed fixtures.

use std::path::PathBuf;

use bart_rs::attributes::*;
use bart_rs::Feed;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gtfs")
}

fn fixture(name: &str) -> PathBuf {
    fixture_dir().join(format!("{name}.txt"))
}

#[test]
fn parses_agency() {
    let agencies = agency::Agency::parse_agency_file(&fixture("agency")).expect("parses");
    assert_eq!(agencies.len(), 1);
    assert_eq!(agencies[0].name, "Bay Area Rapid Transit");
    assert_eq!(agencies[0].timezone.as_deref(), Some("America/Los_Angeles"));
}

#[test]
fn parses_calendar_with_zero_one_booleans() {
    let calendar = calendar::Calendar::parse_calendar_file(&fixture("calendar")).expect("parses");
    assert_eq!(calendar.len(), 3);

    let weekday = &calendar[0];
    assert_eq!(weekday.service_id, "WEEKDAY");
    assert!(weekday.monday);
    assert!(!weekday.saturday);
    assert!(!weekday.sunday);
    assert_eq!(
        weekday.start_date,
        chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
    );
}

#[test]
fn parses_calendar_attributes() {
    let attributes = calendar_attributes::CalendarAttribute::parse_calendar_attributes_file(
        &fixture("calendar_attributes"),
    )
    .expect("parses");
    assert_eq!(attributes.len(), 3);
    assert_eq!(attributes[0].service_description, "Weekday");
}

#[test]
fn parses_calendar_dates_with_both_exception_kinds() {
    let dates = calendar_dates::CalendarDate::parse_calendar_dates_file(&fixture("calendar_dates"))
        .expect("parses");
    assert_eq!(dates.len(), 2);
    assert_eq!(
        dates[0].exception_type,
        calendar_dates::ExceptionType::Removed
    );
    assert_eq!(
        dates[1].exception_type,
        calendar_dates::ExceptionType::Added
    );
}

#[test]
fn parses_directions() {
    let directions =
        directions::Direction::parse_directions_file(&fixture("directions")).expect("parses");
    assert_eq!(directions.len(), 4);
    assert_eq!(directions[0].direction, "North");
    assert_eq!(directions[0].direction_id, 0);
}

#[test]
fn parses_fares_as_cents() {
    let fares =
        fare_attributes::FareAttribute::parse_fare_attributes_file(&fixture("fare_attributes"))
            .expect("parses");
    assert_eq!(fares.len(), 3);
    assert_eq!(fares[0].price_cents, 755);
    assert_eq!(fares[1].price_cents, 1220);
    assert_eq!(fares[0].price_display(), "$7.55");
}

#[test]
fn parses_fare_rider_categories() {
    let categories = fare_rider_categories::FareRiderCategory::parse_fare_rider_categories_file(
        &fixture("fare_rider_categories"),
    )
    .expect("parses");
    assert_eq!(categories.len(), 9);
    assert_eq!(categories[0].price_cents, 375);
}

#[test]
fn parses_fare_rules() {
    let rules =
        fare_rules::FareRule::parse_fare_rules_file(&fixture("fare_rules")).expect("parses");
    assert_eq!(rules.len(), 3);
    assert_eq!(rules[0].origin_id.as_deref(), Some("LAKE"));
    assert_eq!(rules[0].destination_id.as_deref(), Some("SFIA"));
    assert!(rules[0].route_id.is_none());
}

#[test]
fn parses_feed_info() {
    let info = feed_info::FeedInfo::parse_feed_info_file(&fixture("feed_info")).expect("parses");
    assert_eq!(info.len(), 1);
    assert_eq!(info[0].version.as_deref(), Some("99"));
    assert_eq!(
        info[0].end_date,
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31)
    );
}

#[test]
fn parses_realtime_routes() {
    let routes =
        realtime_routes::RealtimeRoute::parse_realtime_routes_file(&fixture("realtime_routes"))
            .expect("parses");
    assert_eq!(routes.len(), 3);
    assert!(routes[0].realtime_enabled);
    assert!(!routes[1].realtime_enabled);
}

#[test]
fn parses_rider_categories() {
    let categories =
        rider_categories::RiderCategory::parse_rider_categories_file(&fixture("rider_categories"))
            .expect("parses");
    assert_eq!(categories.len(), 3);
    assert_eq!(categories[0].description, "Youth Clipper");
}

#[test]
fn parses_route_attributes() {
    let attributes =
        route_attributes::RouteAttribute::parse_route_attributes_file(&fixture("route_attributes"))
            .expect("parses");
    assert_eq!(attributes.len(), 3);
    assert_eq!(attributes[0].category, Some(2));
}

#[test]
fn parses_routes() {
    let routes = routes::Route::parse_routes_file(&fixture("routes")).expect("parses");
    assert_eq!(routes.len(), 3);
    assert_eq!(routes[0].id, "1");
    assert_eq!(routes[0].short_name.as_deref(), Some("Yellow-S"));
    assert_eq!(routes[0].color.as_deref(), Some("FFFF33"));
    assert_eq!(routes[0].route_type, Some(routes::RouteType::Subway));
    assert_eq!(routes[0].display_name(), "Yellow-S");
}

#[test]
fn parses_shapes() {
    let shapes = shapes::Shape::parse_shapes_file(&fixture("shapes")).expect("parses");
    assert_eq!(shapes.len(), 4);
    assert_eq!(shapes[0].shape_id, "001A_shp");
    assert_eq!(shapes[0].sequence, 1);
}

#[test]
fn parses_stops_with_the_three_level_hierarchy() {
    let stops = stops::Stop::parse_stops_file(&fixture("stops")).expect("parses");
    assert_eq!(stops.len(), 10);

    let station = stops.iter().find(|s| s.id == "LAKE").expect("station");
    assert!(station.is_station());
    assert_eq!(station.platform_code, None);
    assert_eq!(
        station.url.as_deref(),
        Some("https://www.bart.gov/stations/lake")
    );

    let platform = stops.iter().find(|s| s.id == "A10-1").expect("platform");
    assert!(platform.is_platform());
    assert_eq!(platform.parent_station.as_deref(), Some("LAKE"));
    assert_eq!(platform.platform_code.as_deref(), Some("1"));

    let entrance = stops.iter().find(|s| s.id == "LAKE_1").expect("entrance");
    assert!(entrance.is_entrance());
    assert_eq!(entrance.parent_station.as_deref(), Some("LAKE"));
}

#[test]
fn parses_stop_times_including_times_past_midnight() {
    let times =
        stop_times::StopTime::parse_stop_times_file(&fixture("stop_times")).expect("parses");
    assert_eq!(times.len(), 10);

    let late = times
        .iter()
        .find(|t| t.trip_id == "1002")
        .expect("late trip");
    assert_eq!(late.arrival_time, Some(24 * 3600 + 30 * 60));
    assert_eq!(late.departure_time, Some(24 * 3600 + 31 * 60));
}

#[test]
fn parses_transfers() {
    let transfers =
        transfers::Transfer::parse_transfers_file(&fixture("transfers")).expect("parses");
    assert_eq!(transfers.len(), 1);
    assert_eq!(transfers[0].from_stop_id, "A10-1");
    assert_eq!(transfers[0].transfer_type, Some(2));
}

#[test]
fn parses_trips() {
    let trips = trips::Trip::parse_trips_file(&fixture("trips")).expect("parses");
    assert_eq!(trips.len(), 5);
    assert_eq!(trips[0].trip_id, "1001");
    assert_eq!(trips[0].service_id, "WEEKDAY");
    assert_eq!(trips[0].direction_id, Some(1));
    assert_eq!(trips[0].trip_headsign.as_deref(), Some("SF Int'l Airport"));
}

#[test]
fn every_table_is_present_in_the_fixture() {
    for name in bart_rs::attributes::TABLE_NAMES {
        assert!(fixture(name).exists(), "the fixture is missing {name}.txt");
    }
}

#[test]
fn the_whole_fixture_feed_parses() {
    let feed = Feed::open(&fixture_dir()).expect("the fixture feed parses");
    assert_eq!(feed.routes.len(), 3);
    assert_eq!(feed.trips.len(), 5);
    assert_eq!(feed.stations().len(), 3);
    assert_eq!(feed.platforms().len(), 5);
    assert_eq!(feed.entrances().len(), 2);
}
