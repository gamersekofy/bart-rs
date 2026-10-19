//! The cross-file queries on [`Feed`].

use std::path::PathBuf;

use bart_rs::Feed;

fn fixture_feed() -> Feed {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gtfs");
    Feed::open(&dir).expect("the fixture feed parses")
}

#[test]
fn looks_up_routes_by_id_and_short_name() {
    let feed = fixture_feed();
    assert_eq!(
        feed.route("5").unwrap().short_name.as_deref(),
        Some("Green-S")
    );
    assert_eq!(
        feed.find_route("Yellow-S").unwrap().id,
        "5".replace("5", "1")
    );
    assert!(feed.find_route("nope").is_none());
}

#[test]
fn resolves_stops_by_id_code_and_name() {
    let feed = fixture_feed();
    assert_eq!(feed.find_stop("LAKE").unwrap().id, "LAKE");
    assert_eq!(feed.find_stop("902101").unwrap().id, "A10-1");
    assert_eq!(feed.find_stop("Embarcadero").unwrap().id, "EMBR");
}

#[test]
fn finds_stops_by_substring() {
    let feed = fixture_feed();
    let found = feed.find_stops("lake");
    assert!(found.iter().any(|s| s.id == "LAKE"));
    assert!(found.iter().any(|s| s.id == "A10-1"));
}

#[test]
fn resolves_the_station_of_a_platform_and_an_entrance() {
    let feed = fixture_feed();
    assert_eq!(feed.station_of("A10-1").unwrap().id, "LAKE");
    assert_eq!(feed.station_of("LAKE_1").unwrap().id, "LAKE");
    assert_eq!(feed.station_of("LAKE").unwrap().id, "LAKE");
}

#[test]
fn lists_the_platforms_and_entrances_of_a_station() {
    let feed = fixture_feed();
    let platforms = feed.platforms_of("LAKE");
    assert_eq!(platforms.len(), 2);
    assert!(platforms.iter().any(|p| p.id == "A10-1"));

    let entrances = feed.entrances_of("LAKE");
    assert_eq!(entrances.len(), 1);
    assert_eq!(entrances[0].id, "LAKE_1");
}

#[test]
fn a_station_query_expands_to_its_platforms() {
    let feed = fixture_feed();
    let ids = feed.platform_ids_for("LAKE");
    assert_eq!(ids, vec!["A10-1".to_string(), "A10-2".to_string()]);

    let ids = feed.platform_ids_for("A10-1");
    assert_eq!(ids, vec!["A10-1".to_string()]);

    // An entrance resolves to the platforms of its station.
    let ids = feed.platform_ids_for("EMBR_1");
    assert_eq!(ids, vec!["M16-1".to_string(), "M16-2".to_string()]);
}

#[test]
fn finds_the_routes_serving_a_station() {
    let feed = fixture_feed();
    let mut ids: Vec<&str> = feed
        .routes_serving("LAKE")
        .into_iter()
        .map(|r| r.id.as_str())
        .collect();
    ids.sort_unstable();
    assert_eq!(ids, vec!["1", "7"]);
}

#[test]
fn lists_the_stations_a_route_serves_in_order() {
    let feed = fixture_feed();
    let stations = feed.stations_on_route("1");
    let ids: Vec<&str> = stations.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["LAKE", "EMBR", "SFIA"]);
}

#[test]
fn returns_a_trips_stop_times_in_order() {
    let feed = fixture_feed();
    let times = feed.stop_times_for_trip("1001");
    let ids: Vec<&str> = times.iter().map(|t| t.stop_id.as_str()).collect();
    assert_eq!(ids, vec!["A10-1", "M16-1", "Y10-1"]);
    assert_eq!(times[0].stop_sequence, 0);
}

#[test]
fn looks_up_a_routes_direction() {
    let feed = fixture_feed();
    assert_eq!(feed.direction_of("1", 0).unwrap().direction, "North");
    assert_eq!(feed.direction_of("1", 1).unwrap().direction, "South");
    assert!(feed.direction_of("5", 0).is_none());
}

#[test]
fn looks_up_a_fare_between_two_stations() {
    let feed = fixture_feed();
    let fare = feed.fare("LAKE", "SFIA").expect("a fare");
    assert_eq!(fare.fare_id, "1");
    assert_eq!(fare.base_price_cents, 755);
    assert_eq!(fare.base_price_display(), "$7.55");
    assert_eq!(fare.prices.len(), 3);
}

#[test]
fn a_fare_lookup_accepts_platforms_and_names() {
    let feed = fixture_feed();
    // A platform id resolves to its station, so the same fare is found.
    let fare = feed.fare("A10-1", "Y10-1").expect("a fare");
    assert_eq!(fare.fare_id, "1");
}

#[test]
fn the_service_description_is_available() {
    let feed = fixture_feed();
    assert_eq!(feed.service_description("WEEKDAY"), Some("Weekday"));
    assert_eq!(feed.service_description("NOPE"), None);
}

#[test]
fn realtime_routes_are_flagged() {
    let feed = fixture_feed();
    assert!(feed.realtime_enabled("1"));
    assert!(!feed.realtime_enabled("5"));
}

#[test]
fn feed_metadata_is_exposed() {
    let feed = fixture_feed();
    assert_eq!(feed.version(), Some("99"));
    assert_eq!(feed.publisher().unwrap().name, "Bay Area Rapid Transit");
}
