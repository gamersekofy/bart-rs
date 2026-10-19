//! `calendar_attributes.txt` — human-readable names for service patterns.
//!
//! This is a BART extension, not part of the core GTFS specification.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::read_table;

/// A human-readable description of a service pattern.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CalendarAttribute {
    /// The service id this describes.
    #[serde(rename = "service_id")]
    pub service_id: String,
    /// The description, e.g. `Weekday` or `Saturday`.
    #[serde(rename = "service_description")]
    pub service_description: String,
}

impl CalendarAttribute {
    /// Read `calendar_attributes.txt` from `path`.
    pub fn parse_calendar_attributes_file(path: &Path) -> Result<Vec<CalendarAttribute>> {
        read_table(path)
    }
}
