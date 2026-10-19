//! `stop_times.txt` — when a trip stops where.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::{de_gtfs_time, read_table};

/// One stop of one trip.
///
/// Times are stored as seconds since midnight of the trip's **service day**,
/// which may exceed 24 hours: a train leaving at 00:30 the next morning is
/// written `24:30:00` in the feed and kept that way here, because the service
/// day a trip belongs to is exactly what wrapping would destroy.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StopTime {
    /// The trip this stop belongs to.
    #[serde(rename = "trip_id")]
    pub trip_id: String,
    /// The arrival time, in seconds since midnight of the service day.
    #[serde(rename = "arrival_time", default, deserialize_with = "de_gtfs_time")]
    pub arrival_time: Option<u32>,
    /// The departure time, in seconds since midnight of the service day.
    #[serde(rename = "departure_time", default, deserialize_with = "de_gtfs_time")]
    pub departure_time: Option<u32>,
    /// The platform id this stop refers to.
    #[serde(rename = "stop_id")]
    pub stop_id: String,
    /// The position of this stop along the trip.
    #[serde(rename = "stop_sequence")]
    pub stop_sequence: u32,
    /// The headsign shown to riders at this stop.
    #[serde(rename = "stop_headsign", default)]
    pub stop_headsign: Option<String>,
    /// When a rider may board here: `0` scheduled, `1` no pickup,
    /// `2` phone agency, `3` coordinate with driver.
    #[serde(rename = "pickup_type", default)]
    pub pickup_type: Option<u8>,
    /// When a rider may alight here, using the same encoding as
    /// `pickup_type`.
    #[serde(rename = "drop_off_type", default)]
    pub drop_off_type: Option<u8>,
    /// The distance travelled along the trip's shape to reach this stop.
    #[serde(rename = "shape_distance_traveled", default)]
    pub shape_distance_traveled: Option<f64>,
}

impl StopTime {
    /// Read `stop_times.txt` from `path`.
    pub fn parse_stop_times_file(path: &Path) -> Result<Vec<StopTime>> {
        read_table(path)
    }

    /// The time to use for arrivals, falling back to the departure time.
    pub fn arrival_or_departure(&self) -> Option<u32> {
        self.arrival_time.or(self.departure_time)
    }
}
