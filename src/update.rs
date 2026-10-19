//! Downloading, extracting and installing a GTFS feed.
//!
//! The feed is fetched from BART's permalink, which always serves the
//! current release, and installed into the user cache directory. The
//! installation is atomic: a partially extracted feed is never visible to
//! readers.
//!
//! See `docs/03-data-sources.md` for the on-disk layout and the update
//! lifecycle.

use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::attributes::calendar::Calendar;
use crate::attributes::calendar_dates::CalendarDate;
use crate::attributes::feed_info::FeedInfo;
use crate::error::{Error, Result};
use crate::helper;
use crate::paths;

/// The permalink to BART's current GTFS schedule feed.
///
/// The response is a `301` to a versioned file whose name encodes the
/// coverage window; the redirect is followed automatically.
pub const FEED_URL: &str = "https://www.bart.gov/dev/schedules/google_transit.zip";

/// The tables a BART feed is expected to contain.
pub const EXPECTED_TABLES: [&str; 18] = crate::attributes::TABLE_NAMES;

/// A downloaded feed, before it is installed.
pub struct DownloadedFeed {
    /// The raw archive bytes.
    pub bytes: Vec<u8>,
    /// The URL the download was ultimately served from.
    pub source_url: String,
    /// The feed's version, read out of the archive.
    pub version: Option<String>,
    /// The feed's metadata, read out of the archive.
    pub feed_info: Option<FeedInfo>,
    /// The window during which the feed's service actually runs.
    pub coverage: Option<(NaiveDate, NaiveDate)>,
    /// The `Last-Modified` header of the response, if the server sent one.
    pub last_modified: Option<String>,
}

/// A pointer to the release the server is currently offering.
///
/// BART's permalink redirects to a file whose name encodes the coverage
/// window and a release counter — `google_transit_20260810-20270108_v02.zip`
/// — and that name changes with every release.
///
/// It is the only reliable release identifier: `feed_info.txt`'s
/// `feed_version` is the same in every release BART has published, so
/// comparing it can never detect a new feed.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Release {
    /// The URL the release is served from, after redirects.
    pub url: String,
    /// The file name, e.g. `google_transit_20260810-20270108_v02.zip`.
    pub name: Option<String>,
    /// The `Last-Modified` header of the release, if the server sent one.
    pub last_modified: Option<String>,
}

impl Release {
    /// Whether this release is the same as `other`.
    ///
    /// The name is the primary signal. The modification time only breaks
    /// ties, and only when both sides have one.
    pub fn is_same(&self, other: &Release) -> bool {
        if self.name != other.name {
            return false;
        }
        match (&self.last_modified, &other.last_modified) {
            (Some(a), Some(b)) => a == b,
            _ => true,
        }
    }

    /// A short name for the release, for printing.
    pub fn label(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.url)
    }
}

/// The name of the release a URL points at.
pub fn release_name(url: &str) -> Option<String> {
    url.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .map(str::to_string)
}

/// The bookkeeping the tool keeps about the installed feed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateState {
    /// The version of the installed feed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feed_version: Option<String>,
    /// When the feed was downloaded, in RFC 3339.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downloaded_at: Option<String>,
    /// When the feed was last checked for updates, in RFC 3339.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_checked_at: Option<String>,
    /// The URL the feed was downloaded from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    /// The `Last-Modified` header of the downloaded release.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
}

impl UpdateState {
    /// The release the installed feed came from, as far as the state records.
    pub fn release(&self) -> Option<Release> {
        let url = self.source_url.clone()?;
        Some(Release {
            name: release_name(&url),
            url,
            last_modified: self.last_modified.clone(),
        })
    }
}

/// Something worth telling the user about the installed feed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Staleness {
    /// The feed's coverage window has ended.
    Expired {
        /// How many days ago it ended.
        days: i64,
    },
    /// The feed's coverage window ends soon.
    ExpiringSoon {
        /// How many days are left.
        days: i64,
    },
    /// The feed's coverage window has not started yet.
    NotYetValid,
    /// The feed has not been checked for updates in a while.
    CheckStale {
        /// How many days since the last check.
        days: i64,
    },
    /// There is no feed installed.
    Missing,
    /// The feed is installed but its coverage window could not be worked out.
    UnknownCoverage,
}

impl Staleness {
    /// A human-readable description of the condition.
    pub fn message(&self) -> String {
        match self {
            Staleness::Expired { days } => format!(
                "the feed expired {days} day{} ago; run `bart-rs update`",
                if *days == 1 { "" } else { "s" }
            ),
            Staleness::ExpiringSoon { days } => {
                format!(
                    "the feed expires in {days} day{}",
                    if *days == 1 { "" } else { "s" }
                )
            }
            Staleness::NotYetValid => "the feed's coverage window has not started yet".to_string(),
            Staleness::CheckStale { days } => format!(
                "the feed has not been checked for {days} day{}; run `bart-rs update --check`",
                if *days == 1 { "" } else { "s" }
            ),
            Staleness::Missing => "no feed is installed; run `bart-rs update`".to_string(),
            Staleness::UnknownCoverage => {
                "the feed's coverage window could not be determined".to_string()
            }
        }
    }
}

/// Build an HTTP client with a sensible user agent and timeouts.
pub fn client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .user_agent(concat!("bart-rs/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| Error::Http {
            url: FEED_URL.to_owned(),
            source: e,
        })
}

/// Download the current feed without installing it.
pub fn download_latest(client: &reqwest::blocking::Client) -> Result<DownloadedFeed> {
    let response = client.get(FEED_URL).send().map_err(|e| Error::Http {
        url: FEED_URL.to_owned(),
        source: e,
    })?;

    let status = response.status();
    if !status.is_success() {
        return Err(Error::HttpStatus {
            url: FEED_URL.to_owned(),
            status: status.as_u16(),
        });
    }

    let source_url = response.url().to_string();
    let last_modified = response
        .headers()
        .get(reqwest::header::LAST_MODIFIED)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let bytes = response
        .bytes()
        .map_err(|e| Error::Http {
            url: FEED_URL.to_owned(),
            source: e,
        })?
        .to_vec();

    let (version, feed_info) = read_feed_info(&bytes)?;
    let coverage = coverage_from_archive(&bytes);

    Ok(DownloadedFeed {
        bytes,
        source_url,
        version,
        feed_info,
        coverage,
        last_modified,
    })
}

/// Ask the server which release it is offering, without downloading it.
///
/// A `HEAD` request follows the permalink's redirect to the versioned file
/// and reports the name and modification time of that file, which is enough
/// to tell whether the installed feed is still the current one.
pub fn probe_release(client: &reqwest::blocking::Client) -> Result<Release> {
    let response = client.head(FEED_URL).send().map_err(|e| Error::Http {
        url: FEED_URL.to_owned(),
        source: e,
    })?;

    let status = response.status();
    if !status.is_success() {
        return Err(Error::HttpStatus {
            url: FEED_URL.to_owned(),
            status: status.as_u16(),
        });
    }

    let url = response.url().to_string();
    let last_modified = response
        .headers()
        .get(reqwest::header::LAST_MODIFIED)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    Ok(Release {
        name: release_name(&url),
        url,
        last_modified,
    })
}

/// Read one of the GTFS tables out of a zipped feed.
fn read_archive_table<T: serde::de::DeserializeOwned>(bytes: &[u8], name: &str) -> Result<Vec<T>> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| Error::BadArchive(e.to_string()))?;

    let mut file = archive
        .by_name(name)
        .map_err(|_| Error::BadArchive(format!("the archive has no {name}")))?;

    let mut text = String::new();
    file.read_to_string(&mut text)
        .map_err(|e| Error::BadArchive(format!("could not read {name}: {e}")))?;

    helper::parse_table(&text, name)
}

/// The window during which a downloaded feed's service actually runs.
fn coverage_from_archive(bytes: &[u8]) -> Option<(NaiveDate, NaiveDate)> {
    let calendar = read_archive_table::<Calendar>(bytes, "calendar.txt").ok()?;
    let calendar_dates = read_archive_table::<CalendarDate>(bytes, "calendar_dates.txt").ok()?;
    crate::attributes::calendar::coverage(&calendar, &calendar_dates)
}

/// Read `feed_info.txt` out of a zipped feed.
fn read_feed_info(bytes: &[u8]) -> Result<(Option<String>, Option<FeedInfo>)> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| Error::BadArchive(e.to_string()))?;

    let mut file = archive
        .by_name("feed_info.txt")
        .map_err(|_| Error::BadArchive("the archive has no feed_info.txt".to_string()))?;

    let mut text = String::new();
    file.read_to_string(&mut text)
        .map_err(|e| Error::BadArchive(format!("could not read feed_info.txt: {e}")))?;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());

    let mut records = reader.records();
    let record = records
        .next()
        .transpose()
        .map_err(|e| Error::BadArchive(format!("could not parse feed_info.txt: {e}")))?
        .ok_or_else(|| Error::BadArchive("feed_info.txt is empty".to_string()))?;

    let mut info = FeedInfo {
        publisher_name: record.get(0).unwrap_or_default().to_string(),
        publisher_url: record.get(1).filter(|s| !s.is_empty()).map(str::to_string),
        lang: record.get(2).filter(|s| !s.is_empty()).map(str::to_string),
        start_date: record
            .get(3)
            .filter(|s| !s.is_empty())
            .and_then(crate::helper::parse_gtfs_date),
        end_date: record
            .get(4)
            .filter(|s| !s.is_empty())
            .and_then(crate::helper::parse_gtfs_date),
        version: record.get(5).filter(|s| !s.is_empty()).map(str::to_string),
    };
    if info.publisher_name.is_empty() {
        info.publisher_name = "Unknown".to_string();
    }

    let version = info.version.clone();
    Ok((version, Some(info)))
}

/// Extract a zipped feed into `dest`.
///
/// The archive must contain all 18 tables; anything else is rejected rather
/// than installed.
pub fn extract(bytes: &[u8], dest: &Path) -> Result<()> {
    fs::create_dir_all(dest).map_err(|e| Error::io(dest, e))?;

    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| Error::BadArchive(e.to_string()))?;

    let mut written: Vec<String> = Vec::new();

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| Error::BadArchive(e.to_string()))?;

        if entry.is_dir() {
            continue;
        }

        let name = entry.name().to_string();
        // Refuse anything that could escape the destination directory.
        if name.contains("..") || name.starts_with('/') || name.contains('\\') {
            return Err(Error::BadArchive(format!(
                "the archive contains an unsafe path: {name}"
            )));
        }

        // Only the top-level .txt tables are wanted; anything else (README,
        // licence files, nested directories) is ignored.
        if !name.ends_with(".txt") || name.contains('/') {
            continue;
        }

        let target = dest.join(&name);
        let mut buffer = Vec::with_capacity(entry.size() as usize);
        entry
            .read_to_end(&mut buffer)
            .map_err(|e| Error::io(&target, e))?;

        let mut file = fs::File::create(&target).map_err(|e| Error::io(&target, e))?;
        file.write_all(&buffer).map_err(|e| Error::io(&target, e))?;

        written.push(name);
    }

    let missing: Vec<String> = EXPECTED_TABLES
        .iter()
        .map(|t| format!("{t}.txt"))
        .filter(|f| !written.iter().any(|w| w == f))
        .collect();

    if !missing.is_empty() {
        return Err(Error::BadArchive(format!(
            "the archive is missing {} of the expected tables: {}",
            missing.len(),
            missing.join(", ")
        )));
    }

    Ok(())
}

/// Install a staged feed as the current one.
///
/// The swap is atomic from a reader's point of view: the staging directory is
/// renamed into place, and the previous feed is removed afterwards. If the
/// rename fails, the previous feed is put back.
pub fn install(staging: &Path, feed_dir: &Path) -> Result<()> {
    fs::create_dir_all(feed_dir).map_err(|e| Error::io(feed_dir, e))?;

    let current = feed_dir.join("current");
    let previous = feed_dir.join(".previous");

    if previous.exists() {
        let _ = fs::remove_dir_all(&previous);
    }

    let had_current = current.exists();
    if had_current {
        fs::rename(&current, &previous).map_err(|e| Error::io(&current, e))?;
    }

    if let Err(e) = fs::rename(staging, &current) {
        // Put the old feed back before giving up.
        if had_current {
            let _ = fs::rename(&previous, &current);
        }
        return Err(Error::io(&current, e));
    }

    if had_current {
        let _ = fs::remove_dir_all(&previous);
    }

    Ok(())
}

/// Store the downloaded archive.
pub fn store_archive(bytes: &[u8], version: Option<&str>) -> Result<PathBuf> {
    let dir = paths::archives_dir()?;
    fs::create_dir_all(&dir).map_err(|e| Error::io(&dir, e))?;

    let name = match version {
        Some(v) => format!("{v}.zip"),
        None => "unknown.zip".to_string(),
    };
    let path = dir.join(name);
    fs::write(&path, bytes).map_err(|e| Error::io(&path, e))?;
    Ok(path)
}

/// Read the bookkeeping file, returning the default if it does not exist.
pub fn read_state() -> Result<UpdateState> {
    let path = paths::state_path()?;
    if !path.exists() {
        return Ok(UpdateState::default());
    }

    let text = fs::read_to_string(&path).map_err(|e| Error::State {
        path: path.clone(),
        source: e,
    })?;

    serde_json::from_str(&text).map_err(|e| Error::State {
        path: path.clone(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()),
    })
}

/// Write the bookkeeping file.
pub fn write_state(state: &UpdateState) -> Result<()> {
    let path = paths::state_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }

    let text = serde_json::to_string_pretty(state)
        .map_err(|e| Error::msg(format!("could not serialise the feed state: {e}")))?;
    fs::write(&path, text).map_err(|e| Error::State {
        path: path.clone(),
        source: e,
    })
}

/// Check the release the server is offering, without downloading the feed.
///
/// The whole feed is not fetched: a `HEAD` request is enough to learn which
/// release is current.
pub fn check_latest(client: &reqwest::blocking::Client) -> Result<Release> {
    probe_release(client)
}

/// Work out what is worth telling the user about the installed feed.
///
/// `coverage` is the window during which the feed's service actually runs.
/// It is taken from the calendar rather than from `feed_info.txt`, because
/// BART leaves `feed_info.txt` describing the previous release.
pub fn staleness(
    coverage: Option<(NaiveDate, NaiveDate)>,
    state: &UpdateState,
    now: DateTime<Utc>,
) -> Vec<Staleness> {
    let mut warnings = Vec::new();

    let Some((start, end)) = coverage else {
        warnings.push(if state.feed_version.is_some() {
            Staleness::UnknownCoverage
        } else {
            Staleness::Missing
        });
        return warnings;
    };

    let today = now.date_naive();

    if end < today {
        warnings.push(Staleness::Expired {
            days: (today - end).num_days(),
        });
    } else if (end - today).num_days() <= 7 {
        warnings.push(Staleness::ExpiringSoon {
            days: (end - today).num_days(),
        });
    }

    if start > today {
        warnings.push(Staleness::NotYetValid);
    }

    if let Some(checked) = state
        .last_checked_at
        .as_deref()
        .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
    {
        let days = (now - checked.with_timezone(&Utc)).num_days();
        if days >= 7 {
            warnings.push(Staleness::CheckStale { days });
        }
    }

    warnings
}

/// Whether a date falls inside a feed's coverage window.
pub fn is_covered(info: &FeedInfo, date: NaiveDate) -> bool {
    info.start_date.is_none_or(|start| date >= start) && info.end_date.is_none_or(|end| date <= end)
}

/// The age of a timestamp, in days.
pub fn age_in_days(now: DateTime<Utc>, then: DateTime<Utc>) -> i64 {
    Duration::num_days(&(now - then))
}
