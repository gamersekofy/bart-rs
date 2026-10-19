//! `routes.txt` — the commercial lines in the network.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::read_table;

/// The kind of route, using the GTFS enumeration.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum RouteType {
    /// A tram, light rail or underground rail line.
    #[serde(rename = "1")]
    Subway,
    /// A railway service.
    #[serde(rename = "2")]
    Rail,
    /// A bus service.
    #[serde(rename = "3")]
    Bus,
    /// A ferry service.
    #[serde(rename = "4")]
    Ferry,
    /// A cable tram.
    #[serde(rename = "5")]
    CableTram,
    /// An aerial lift.
    #[serde(rename = "6")]
    AerialLift,
    /// A funicular.
    #[serde(rename = "7")]
    Funicular,
    /// A coach service.
    #[serde(rename = "200")]
    Coach,
}

/// A commercial line.
///
/// BART's routes are directional pairs: `1` and `2` are the two directions of
/// the Yellow line, `3` and `4` the Orange line, and so on. Each has its own
/// stop order.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Route {
    /// The route id.
    #[serde(rename = "route_id")]
    pub id: String,
    /// The short name, e.g. `Yellow-S`.
    #[serde(rename = "route_short_name", default)]
    pub short_name: Option<String>,
    /// The long name, e.g. `Antioch to SF Int'l Airport SFO/Millbrae`.
    #[serde(rename = "route_long_name", default)]
    pub long_name: Option<String>,
    /// A free-form description.
    #[serde(rename = "route_desc", default)]
    pub desc: Option<String>,
    /// The kind of route.
    #[serde(rename = "route_type", default)]
    pub route_type: Option<RouteType>,
    /// A URL with more information about the route.
    #[serde(rename = "route_url", default)]
    pub url: Option<String>,
    /// The display colour, as six hex digits without a leading `#`.
    #[serde(rename = "route_color", default)]
    pub color: Option<String>,
    /// The text colour to use on top of `color`.
    #[serde(rename = "route_text_color", default)]
    pub text_color: Option<String>,
}

impl Route {
    /// Read `routes.txt` from `path`.
    pub fn parse_routes_file(path: &Path) -> Result<Vec<Route>> {
        read_table(path)
    }

    /// The name to show for this route: the short name if there is one,
    /// otherwise the long name, otherwise the id.
    pub fn display_name(&self) -> &str {
        self.short_name
            .as_deref()
            .or(self.long_name.as_deref())
            .unwrap_or(&self.id)
    }
}
