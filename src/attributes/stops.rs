//! `stops.txt` — stations, platforms and entrances.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::read_table;

/// The kind of stop, using the GTFS enumeration.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
pub enum LocationType {
    /// A stop or platform. The most common kind.
    #[serde(rename = "0")]
    #[default]
    StopOrPlatform,
    /// A station: a physical structure that contains one or more platforms.
    #[serde(rename = "1")]
    Station,
    /// A station entrance or exit.
    #[serde(rename = "2")]
    EntranceOrExit,
    /// A location within a station that is not a platform, such as a
    /// ticketing hall.
    #[serde(rename = "3")]
    GenericNode,
    /// A specific boarding area on a platform.
    #[serde(rename = "4")]
    BoardingArea,
}

/// A station, platform or entrance.
///
/// BART's stops form a three-level hierarchy:
///
/// | Level            | `location_type` | Example id | Example name   |
/// |------------------|-----------------|------------|----------------|
/// | Station          | `1`             | `LAKE`     | Lake Merritt   |
/// | Platform         | `0`             | `A10-1`    | Lake Merritt   |
/// | Entrance / exit  | `2`             | `LAKE_1`   | B1 NW 8th &…   |
///
/// A platform's and an entrance's `parent_station` is the **station** id.
/// Note that a platform's own id is *not* derived from its station: the
/// station is `LAKE` but its platforms are `A10-1` and `A10-2`, because BART
/// uses legacy platform codes alongside modern station codes.
///
/// `stop_times.txt` and `transfers.txt` reference platform ids, while
/// `fare_rules.txt` references station ids.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Stop {
    /// The stop id. Unique within the feed.
    #[serde(rename = "stop_id")]
    pub id: String,
    /// A short alphanumeric code for the stop, usually printed on the sign.
    #[serde(rename = "stop_code", default)]
    pub code: Option<String>,
    /// The name of the stop, e.g. `Lake Merritt`.
    ///
    /// Platforms share their station's name, so this is not a unique
    /// identifier.
    #[serde(rename = "stop_name", default)]
    pub name: Option<String>,
    /// A free-form description of the location.
    #[serde(rename = "stop_desc", default)]
    pub desc: Option<String>,
    /// The latitude of the stop.
    #[serde(rename = "stop_lat", default)]
    pub lat: Option<f64>,
    /// The longitude of the stop.
    #[serde(rename = "stop_lon", default)]
    pub lon: Option<f64>,
    /// The fare zone the stop belongs to.
    #[serde(rename = "zone_id", default)]
    pub zone_id: Option<String>,
    /// A URL with more information about the stop.
    ///
    /// Older feeds called this column `plc_url`; both spellings are accepted.
    #[serde(rename = "stop_url", alias = "plc_url", default)]
    pub url: Option<String>,
    /// The kind of stop.
    #[serde(rename = "location_type", default)]
    pub location_type: LocationType,
    /// The station this stop belongs to, for platforms and entrances.
    #[serde(rename = "parent_station", default)]
    pub parent_station: Option<String>,
    /// The platform number, for platforms.
    ///
    /// This column was added in a later feed version than the one this project
    /// originally shipped with.
    #[serde(rename = "platform_code", default)]
    pub platform_code: Option<String>,
}

impl Stop {
    /// Read `stops.txt` from `path`.
    pub fn parse_stops_file(path: &Path) -> Result<Vec<Stop>> {
        read_table(path)
    }

    /// Whether this stop is a station.
    pub fn is_station(&self) -> bool {
        self.location_type == LocationType::Station
    }

    /// Whether this stop is a platform.
    pub fn is_platform(&self) -> bool {
        self.location_type == LocationType::StopOrPlatform
    }

    /// Whether this stop is an entrance or exit.
    pub fn is_entrance(&self) -> bool {
        self.location_type == LocationType::EntranceOrExit
    }

    /// The best name available for this stop.
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.id)
    }
}
