//! The `stops.txt` schema changed between feed versions.
//!
//! The feed this project originally shipped with had ten columns and called
//! the URL column `plc_url`; the current feed has eleven columns and calls it
//! `stop_url`. A parser that indexes by column position silently misreads
//! one of them, so both are checked here.

use std::path::PathBuf;

use bart_rs::attributes::stops::Stop;

fn old_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gtfs-2024/stops.txt")
}

fn new_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gtfs/stops.txt")
}

#[test]
fn the_old_ten_column_schema_still_parses() {
    let stops = Stop::parse_stops_file(&old_fixture()).expect("the old schema parses");
    assert_eq!(stops.len(), 4);

    let station = stops
        .iter()
        .find(|s| s.id == "place_LAKE")
        .expect("a station");
    assert!(station.is_station());
    assert_eq!(
        station.url.as_deref(),
        Some("https://www.bart.gov/stations/lake"),
        "the `plc_url` column should still populate `url`"
    );

    let platform = stops.iter().find(|s| s.id == "LAKE").expect("a platform");
    assert!(platform.is_platform());
    assert_eq!(platform.parent_station.as_deref(), Some("place_LAKE"));
}

#[test]
fn the_new_eleven_column_schema_parses() {
    let stops = Stop::parse_stops_file(&new_fixture()).expect("the new schema parses");

    let station = stops.iter().find(|s| s.id == "LAKE").expect("a station");
    assert!(station.is_station());
    assert_eq!(
        station.url.as_deref(),
        Some("https://www.bart.gov/stations/lake")
    );
}

#[test]
fn the_platform_code_column_is_optional() {
    let old = Stop::parse_stops_file(&old_fixture()).expect("parses");
    assert!(old.iter().all(|s| s.platform_code.is_none()));

    let new = Stop::parse_stops_file(&new_fixture()).expect("parses");
    let platform = new.iter().find(|s| s.id == "A10-1").expect("a platform");
    assert_eq!(platform.platform_code.as_deref(), Some("1"));
}

#[test]
fn the_two_schemas_describe_the_same_stop_differently() {
    let old = Stop::parse_stops_file(&old_fixture()).expect("parses");
    let new = Stop::parse_stops_file(&new_fixture()).expect("parses");

    let old_lake = old
        .iter()
        .find(|s| s.name.as_deref() == Some("Lake Merritt") && s.is_platform());
    let new_lake = new.iter().find(|s| s.id == "A10-1");

    assert!(old_lake.is_some());
    assert!(new_lake.is_some());
    assert_eq!(old_lake.unwrap().name, new_lake.unwrap().name);
}
