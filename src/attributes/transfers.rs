//! `transfers.txt` — transfer rules between stops.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::read_table;

/// A rule about transferring between two stops.
///
/// `transfer_type` uses the GTFS enumeration: `0` a recommended transfer
/// point, `1` a timed transfer, `2` no transfer possible, `3` no transfer on
/// a particular route, `4` and `5` passengers only. BART uses `1` and `2`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transfer {
    /// The platform id the transfer starts at.
    #[serde(rename = "from_stop_id")]
    pub from_stop_id: String,
    /// The platform id the transfer ends at.
    #[serde(rename = "to_stop_id")]
    pub to_stop_id: String,
    /// The kind of transfer rule.
    #[serde(rename = "transfer_type", default)]
    pub transfer_type: Option<u8>,
    /// The minimum time needed for the transfer, in seconds.
    #[serde(rename = "min_transfer_time", default)]
    pub min_transfer_time: Option<u32>,
    /// The route the transfer starts on, if limited.
    #[serde(rename = "from_route_id", default)]
    pub from_route_id: Option<String>,
    /// The route the transfer ends on, if limited.
    #[serde(rename = "to_route_id", default)]
    pub to_route_id: Option<String>,
    /// The trip the transfer starts on, if limited.
    #[serde(rename = "from_trip_id", default)]
    pub from_trip_id: Option<String>,
    /// The trip the transfer ends on, if limited.
    #[serde(rename = "to_trip_id", default)]
    pub to_trip_id: Option<String>,
}

impl Transfer {
    /// Read `transfers.txt` from `path`.
    pub fn parse_transfers_file(path: &Path) -> Result<Vec<Transfer>> {
        read_table(path)
    }
}
