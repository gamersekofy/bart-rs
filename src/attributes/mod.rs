//! One module per GTFS table.
//!
//! Each submodule contains the typed struct for one file in BART's feed and a
//! `parse_*_file` function that reads it. Keeping one module per table makes
//! it obvious at a glance which of the 18 files are handled, and keeps each
//! file small enough to read in one sitting.
//!
//! The schemas are documented field by field in `docs/02-data-model.md`.

pub mod agency;
pub mod calendar;
pub mod calendar_attributes;
pub mod calendar_dates;
pub mod directions;
pub mod fare_attributes;
pub mod fare_rider_categories;
pub mod fare_rules;
pub mod feed_info;
pub mod realtime_routes;
pub mod rider_categories;
pub mod route_attributes;
pub mod routes;
pub mod shapes;
pub mod stop_times;
pub mod stops;
pub mod transfers;
pub mod trips;

/// The file name of every table in a BART feed, without the `.txt` extension.
///
/// Used by the `table` command and by the feed loader.
pub const TABLE_NAMES: [&str; 18] = [
    "agency",
    "calendar",
    "calendar_attributes",
    "calendar_dates",
    "directions",
    "fare_attributes",
    "fare_rider_categories",
    "fare_rules",
    "feed_info",
    "realtime_routes",
    "rider_categories",
    "route_attributes",
    "routes",
    "shapes",
    "stop_times",
    "stops",
    "transfers",
    "trips",
];
