//! `fare_attributes.txt` — the price of each fare.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::helper::{de_cents, read_table};

/// One fare and what it costs.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FareAttribute {
    /// The fare id, referenced by `fare_rules.fare_id`.
    #[serde(rename = "fare_id")]
    pub fare_id: String,
    /// The price, in cents. `755` is $7.55.
    #[serde(rename = "price", deserialize_with = "de_cents")]
    pub price_cents: i64,
    /// The currency, e.g. `USD`.
    #[serde(rename = "currency_type", default)]
    pub currency_type: Option<String>,
    /// When the fare must be paid: `0` before boarding, `1` on board.
    #[serde(rename = "payment_method", default)]
    pub payment_method: Option<u8>,
    /// The number of transfers included in the fare, if limited.
    #[serde(rename = "transfers", default)]
    pub transfers: Option<String>,
    /// The length of time in seconds a transfer remains valid.
    #[serde(rename = "transfer_duration", default)]
    pub transfer_duration: Option<u32>,
}

impl FareAttribute {
    /// Read `fare_attributes.txt` from `path`.
    pub fn parse_fare_attributes_file(path: &Path) -> Result<Vec<FareAttribute>> {
        read_table(path)
    }

    /// The price formatted as a currency string, e.g. `$7.55`.
    pub fn price_display(&self) -> String {
        format!(
            "${}.{:02}",
            self.price_cents / 100,
            (self.price_cents % 100).abs()
        )
    }
}
