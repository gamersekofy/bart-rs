//! End-to-end tests that run the `bart-rs` binary.

use std::path::{Path, PathBuf};
use std::process::Command;

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_bart-rs")
}

/// Build a data directory containing the fixture feed.
///
/// The name is unique per call so that the tests, which run in parallel,
/// do not fight over the same directory.
fn fixture_data_dir() -> PathBuf {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let root =
        std::env::temp_dir().join(format!("bart-rs-cli-test-{}-{unique}", std::process::id()));
    let feed = root.join("feed/current");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&feed).expect("creates the feed directory");

    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gtfs");
    for entry in std::fs::read_dir(&source).expect("reads the fixture directory") {
        let entry = entry.expect("a fixture entry");
        std::fs::copy(entry.path(), feed.join(entry.file_name())).expect("copies a fixture");
    }

    root
}

fn run(data_dir: &Path, args: &[&str]) -> (String, String, bool) {
    let output = Command::new(binary())
        .args(args)
        .env("BART_RS_DATA_DIR", data_dir)
        .env("NO_COLOR", "1")
        .output()
        .expect("the binary runs");

    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.success(),
    )
}

#[test]
fn help_lists_every_command() {
    let output = Command::new(binary())
        .arg("--help")
        .output()
        .expect("the binary runs");
    let stdout = String::from_utf8_lossy(&output.stdout);

    for command in [
        "update",
        "status",
        "routes",
        "stops",
        "route",
        "stop",
        "arrivals",
        "departures",
        "trip",
        "fare",
        "alerts",
        "table",
    ] {
        assert!(stdout.contains(command), "--help should mention {command}");
    }
}

#[test]
fn status_reports_the_feed() {
    let data_dir = fixture_data_dir();
    let (stdout, _, ok) = run(&data_dir, &["status"]);
    assert!(ok, "status should succeed: {stdout}");
    assert!(stdout.contains("99"), "the feed version should be shown");
    assert!(
        stdout.contains("2026-01-01"),
        "the coverage start should be shown"
    );
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[test]
fn routes_are_listed() {
    let data_dir = fixture_data_dir();
    let (stdout, _, ok) = run(&data_dir, &["routes"]);
    assert!(ok);
    assert!(stdout.contains("Yellow-S"));
    assert!(stdout.contains("Green-S"));
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[test]
fn stops_are_searched() {
    let data_dir = fixture_data_dir();
    let (stdout, _, ok) = run(&data_dir, &["stops", "lake"]);
    assert!(ok);
    assert!(stdout.contains("Lake Merritt"));
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[test]
fn a_route_lists_its_stations() {
    let data_dir = fixture_data_dir();
    let (stdout, _, ok) = run(&data_dir, &["route", "1"]);
    assert!(ok);
    assert!(stdout.contains("Yellow-S"));
    assert!(stdout.contains("Lake Merritt"));
    assert!(stdout.contains("Embarcadero"));
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[test]
fn a_stop_lists_its_platforms_and_lines() {
    let data_dir = fixture_data_dir();
    let (stdout, _, ok) = run(&data_dir, &["stop", "LAKE"]);
    assert!(ok);
    assert!(stdout.contains("A10-1"));
    assert!(stdout.contains("Yellow-S"));
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[test]
fn arrivals_are_shown_for_a_time() {
    let data_dir = fixture_data_dir();
    let (stdout, _, ok) = run(
        &data_dir,
        &["arrivals", "A10-1", "--date", "2026-10-19", "--at", "05:00"],
    );
    assert!(ok, "{stdout}");
    assert!(stdout.contains("05:00"), "{stdout}");
    assert!(stdout.contains("Yellow-S"), "{stdout}");
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[test]
fn departures_are_shown_for_a_time() {
    let data_dir = fixture_data_dir();
    let (stdout, _, ok) = run(
        &data_dir,
        &[
            "departures",
            "A10-1",
            "--date",
            "2026-10-19",
            "--at",
            "05:00",
        ],
    );
    assert!(ok, "{stdout}");
    assert!(stdout.contains("05:00"), "{stdout}");
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[test]
fn a_trip_lists_its_stop_times() {
    let data_dir = fixture_data_dir();
    let (stdout, _, ok) = run(&data_dir, &["trip", "1001"]);
    assert!(ok, "{stdout}");
    assert!(stdout.contains("A10-1"));
    assert!(stdout.contains("05:00:00"));
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[test]
fn a_fare_is_looked_up() {
    let data_dir = fixture_data_dir();
    let (stdout, _, ok) = run(&data_dir, &["fare", "--from", "LAKE", "--to", "SFIA"]);
    assert!(ok, "{stdout}");
    assert!(stdout.contains("$7.55"), "{stdout}");
    assert!(stdout.contains("Youth Clipper"), "{stdout}");
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[test]
fn a_raw_table_is_dumped() {
    let data_dir = fixture_data_dir();
    let (stdout, _, ok) = run(&data_dir, &["table", "routes"]);
    assert!(ok, "{stdout}");
    assert!(stdout.contains("route_short_name"));
    assert!(stdout.contains("Yellow-S"));
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[test]
fn json_output_is_valid_json() {
    let data_dir = fixture_data_dir();
    let (stdout, _, ok) = run(&data_dir, &["--json", "routes"]);
    assert!(ok, "{stdout}");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(parsed.as_array().unwrap().len(), 3);
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[test]
fn json_is_refused_where_it_does_not_apply() {
    let data_dir = fixture_data_dir();
    let (_, stderr, ok) = run(&data_dir, &["--json", "status"]);
    assert!(!ok, "status should refuse --json");
    assert!(stderr.contains("does not support --json"), "{stderr}");
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[test]
fn an_unknown_stop_is_reported_clearly() {
    let data_dir = fixture_data_dir();
    let (_, stderr, ok) = run(&data_dir, &["stop", "nowhere"]);
    assert!(!ok);
    assert!(stderr.contains("no stop matching"), "{stderr}");
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[test]
fn an_unknown_table_is_reported_clearly() {
    let data_dir = fixture_data_dir();
    let (_, stderr, ok) = run(&data_dir, &["table", "nope"]);
    assert!(!ok);
    assert!(stderr.contains("unknown table"), "{stderr}");
    let _ = std::fs::remove_dir_all(&data_dir);
}

#[test]
fn a_missing_feed_is_reported_clearly() {
    let empty = std::env::temp_dir().join(format!("bart-rs-empty-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&empty);
    std::fs::create_dir_all(&empty).expect("creates the directory");

    let (_, stderr, ok) = run(&empty, &["routes"]);
    assert!(!ok);
    assert!(stderr.contains("no GTFS feed found"), "{stderr}");
    let _ = std::fs::remove_dir_all(&empty);
}
