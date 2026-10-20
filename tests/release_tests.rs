//! Release detection.
//!
//! BART's `feed_info.txt` says the same `feed_version` in every release it
//! has published — 72 for both the January and the August 2026 feeds — so
//! comparing versions can never detect a new feed. The release is
//! identified by the file the permalink redirects to, whose name encodes
//! the coverage window and a release counter.

use bart_rs::update::{Release, UpdateState, release_name};

/// The URL of the release that was current in January 2026.
const OLD_URL: &str =
    "https://www.bart.gov/sites/default/files/2026-01/google_transit_20260112-20260830_v01.zip";

/// The URL of the release that superseded it in August 2026.
const NEW_URL: &str =
    "https://www.bart.gov/sites/default/files/2026-08/google_transit_20260810-20270108_v02.zip";

fn release(url: &str) -> Release {
    Release {
        url: url.to_string(),
        name: release_name(url),
        last_modified: None,
    }
}

#[test]
fn the_release_name_is_the_last_path_segment() {
    assert_eq!(
        release_name(NEW_URL).as_deref(),
        Some("google_transit_20260810-20270108_v02.zip")
    );
    assert_eq!(release_name("https://example.com/").as_deref(), None);
    assert_eq!(release_name("").as_deref(), None);
}

#[test]
fn two_releases_with_the_same_feed_version_are_still_different() {
    // Both of these feeds say `feed_version` 72. The version is useless for
    // telling them apart; the file name is not.
    let old = release(OLD_URL);
    let new = release(NEW_URL);

    assert_ne!(old.name, new.name);
    assert!(!old.is_same(&new));
    assert!(!new.is_same(&old));
}

#[test]
fn the_same_file_name_is_the_same_release() {
    let old = release(OLD_URL);
    let old_again = release(OLD_URL);

    assert!(old.is_same(&old_again));
}

#[test]
fn a_changed_modification_time_breaks_a_tie() {
    let mut old = release(OLD_URL);
    old.last_modified = Some("Tue, 04 Aug 2026 18:23:24 GMT".to_string());

    let mut same_file = release(OLD_URL);
    same_file.last_modified = Some("Wed, 05 Aug 2026 18:23:24 GMT".to_string());

    assert!(!old.is_same(&same_file));
}

#[test]
fn a_missing_modification_time_does_not_break_a_tie() {
    let mut old = release(OLD_URL);
    old.last_modified = Some("Tue, 04 Aug 2026 18:23:24 GMT".to_string());

    let without = release(OLD_URL);

    assert!(old.is_same(&without));
}

#[test]
fn the_state_reconstructs_the_release_from_the_source_url() {
    let state = UpdateState {
        feed_version: Some("72".to_string()),
        source_url: Some(NEW_URL.to_string()),
        ..UpdateState::default()
    };

    let restored = state.release().expect("the state knows the release");
    assert_eq!(
        restored.name.as_deref(),
        Some("google_transit_20260810-20270108_v02.zip")
    );
    assert!(restored.is_same(&release(NEW_URL)));
}

#[test]
fn a_state_with_no_source_url_knows_no_release() {
    let state = UpdateState {
        feed_version: Some("72".to_string()),
        ..UpdateState::default()
    };
    assert!(state.release().is_none());
}
