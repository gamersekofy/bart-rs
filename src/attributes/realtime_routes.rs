//! `realtime_routes.txt` — which routes carry live updates.
//!
//! This is a BART extension, not part of the core GTFS specification.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::{de_bool_01, read_table};

/// Whether a route is covered by the realtime feed.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RealtimeRoute {
    /// The route id.
    #[serde(rename = "route_id")]
    pub route_id: String,
    /// Whether realtime updates are published for this route.
    #[serde(rename = "realtime_enabled", default, deserialize_with = "de_bool_01")]
    pub realtime_enabled: bool,
    /// The route's name as shown in realtime messages.
    #[serde(rename = "realtime_routename", default)]
    pub realtime_routename: Option<String>,
    /// The route's code as shown in realtime messages.
    #[serde(rename = "realtime_routecode", default)]
    pub realtime_routecode: Option<String>,
}

impl RealtimeRoute {
    /// Read `realtime_routes.txt` from `path`.
    pub fn parse_realtime_routes_file(path: &Path) -> Result<Vec<RealtimeRoute>> {
        read_table(path)
    }
}
