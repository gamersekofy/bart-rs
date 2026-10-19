//! `agency.txt` — the agency operating the network.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::read_table;

/// A transit agency.
///
/// BART publishes exactly one of these.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Agency {
    /// The agency's id, if it has one.
    #[serde(rename = "agency_id", default)]
    pub id: Option<String>,
    /// The agency's name, e.g. `Bay Area Rapid Transit`.
    #[serde(rename = "agency_name")]
    pub name: String,
    /// The agency's URL.
    #[serde(rename = "agency_url", default)]
    pub url: Option<String>,
    /// The timezone the agency operates in, e.g. `America/Los_Angeles`.
    ///
    /// This is the authoritative timezone for every schedule calculation the
    /// crate performs.
    #[serde(rename = "agency_timezone", default)]
    pub timezone: Option<String>,
    /// The agency's primary language, e.g. `en`.
    #[serde(rename = "agency_lang", default)]
    pub lang: Option<String>,
    /// A voice telephone number for the agency.
    #[serde(rename = "agency_phone", default)]
    pub phone: Option<String>,
}

impl Agency {
    /// Read `agency.txt` from `path`.
    pub fn parse_agency_file(path: &Path) -> Result<Vec<Agency>> {
        read_table(path)
    }
}
