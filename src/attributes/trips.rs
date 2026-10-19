//! `trips.txt` — the individual runs of a route.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::read_table;

/// A single trip: one run of a route, on one service day.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Trip {
    /// The route this trip runs on.
    #[serde(rename = "route_id")]
    pub route_id: String,
    /// The service pattern that decides which days this trip runs.
    #[serde(rename = "service_id")]
    pub service_id: String,
    /// The trip id.
    #[serde(rename = "trip_id")]
    pub trip_id: String,
    /// The destination shown to riders, e.g. `OAK Airport / SF / Daly City`.
    #[serde(rename = "trip_headsign", default)]
    pub trip_headsign: Option<String>,
    /// The direction of travel: `0` or `1`.
    #[serde(rename = "direction_id", default)]
    pub direction_id: Option<u8>,
    /// The block this trip belongs to, used to group trips run by the same
    /// vehicle.
    #[serde(rename = "block_id", default)]
    pub block_id: Option<String>,
    /// The shape the trip follows.
    #[serde(rename = "shape_id", default)]
    pub shape_id: Option<String>,
    /// Free-form information about the trip's load.
    #[serde(rename = "trip_load_information", default)]
    pub trip_load_information: Option<String>,
    /// Whether the vehicle is wheelchair accessible: `0` no information,
    /// `1` accessible, `2` not accessible.
    #[serde(rename = "wheelchair_accessible", default)]
    pub wheelchair_accessible: Option<u8>,
    /// Whether bikes are allowed: `0` no information, `1` allowed,
    /// `2` not allowed.
    #[serde(rename = "bikes_allowed", default)]
    pub bikes_allowed: Option<u8>,
}

impl Trip {
    /// Read `trips.txt` from `path`.
    pub fn parse_trips_file(path: &Path) -> Result<Vec<Trip>> {
        read_table(path)
    }
}
