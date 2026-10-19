//! `shapes.txt` — the geographic shape of a route.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::read_table;

/// One point of a shape.
///
/// Points belong to a shape id and must be ordered by `sequence`; together
/// they trace the path the vehicle follows.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Shape {
    /// The shape id, referenced by `trips.shape_id`.
    #[serde(rename = "shape_id")]
    pub shape_id: String,
    /// The latitude of the point.
    #[serde(rename = "shape_pt_lat")]
    pub lat: f64,
    /// The longitude of the point.
    #[serde(rename = "shape_pt_lon")]
    pub lon: f64,
    /// The position of this point along the shape.
    #[serde(rename = "shape_pt_sequence")]
    pub sequence: u32,
    /// The distance travelled along the shape to reach this point, in metres.
    #[serde(rename = "shape_dist_traveled", default)]
    pub dist_traveled: Option<f64>,
}

impl Shape {
    /// Read `shapes.txt` from `path`.
    pub fn parse_shapes_file(path: &Path) -> Result<Vec<Shape>> {
        read_table(path)
    }
}
