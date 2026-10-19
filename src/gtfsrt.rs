//! GTFS-Realtime protobuf message definitions.
//!
//! These are hand-written [`prost::Message`] implementations rather than
//! generated code, so that the crate depends only on `prost` and needs no
//! `protoc` binary at build time.
//!
//! # A note on BART's alert feed
//!
//! BART's alerts do not use the field numbers the GTFS-Realtime spec defines
//! for the alert text. In the spec, `Alert` field 10 is `created_at` (a
//! `uint64`) and field 11 is `update` (a `TimeRange`). BART puts translated
//! strings there instead: field 10 holds a short title and field 11 holds the
//! full text of the alert.
//!
//! The two fields are therefore declared as [`TranslatedString`] here, which
//! is what BART actually sends. Protobuf is wire-type driven, so a feed that
//! really did use the spec's layout would fail to decode these two fields;
//! BART is the only agency this crate targets.
//!
//! The behaviour was confirmed by decoding the live feed with
//! `protoc --decode_raw`; see `docs/03-data-sources.md`.

use prost::Message;

/// The top-level GTFS-Realtime message.
#[derive(Clone, PartialEq, Message)]
pub struct FeedMessage {
    /// Feed metadata.
    #[prost(message, optional, tag = "1")]
    pub header: Option<FeedHeader>,
    /// The entities in the feed: trip updates, vehicle positions, alerts.
    #[prost(message, repeated, tag = "2")]
    pub entity: Vec<FeedEntity>,
}

/// Metadata about the feed itself.
#[derive(Clone, PartialEq, Message)]
pub struct FeedHeader {
    /// The version of the GTFS-Realtime spec the feed conforms to.
    #[prost(string, tag = "1")]
    pub gtfs_realtime_version: String,
    /// Whether the feed is a full dataset or an increment.
    #[prost(int32, tag = "2")]
    pub incrementality: i32,
    /// The time the feed was generated, in Unix epoch seconds.
    #[prost(uint64, tag = "3")]
    pub timestamp: u64,
}

/// One entity in a feed.
#[derive(Clone, PartialEq, Message)]
pub struct FeedEntity {
    /// The entity's id. For trip updates BART uses the trip id.
    #[prost(string, tag = "1")]
    pub id: String,
    /// Whether the entity was deleted in this increment.
    #[prost(bool, tag = "2")]
    pub is_deleted: bool,
    /// Live updates for a trip.
    #[prost(message, optional, tag = "3")]
    pub trip_update: Option<TripUpdate>,
    /// A service alert.
    #[prost(message, optional, tag = "5")]
    pub alert: Option<Alert>,
}

/// Live updates for one trip.
#[derive(Clone, PartialEq, Message)]
pub struct TripUpdate {
    /// Which trip the updates are about.
    #[prost(message, optional, tag = "1")]
    pub trip: Option<TripDescriptor>,
    /// The updated times, one per stop that changed.
    #[prost(message, repeated, tag = "2")]
    pub stop_time_update: Vec<StopTimeUpdate>,
    /// The vehicle running the trip.
    #[prost(message, optional, tag = "3")]
    pub vehicle: Option<VehicleDescriptor>,
    /// When the update was generated, in Unix epoch seconds.
    #[prost(uint64, tag = "4")]
    pub timestamp: u64,
    /// The trip-wide delay, in seconds.
    #[prost(int32, tag = "5")]
    pub delay: i32,
}

/// An updated time at one stop.
#[derive(Clone, PartialEq, Message)]
pub struct StopTimeUpdate {
    /// The position of the stop along the trip.
    #[prost(uint32, optional, tag = "1")]
    pub stop_sequence: Option<u32>,
    /// The updated arrival.
    #[prost(message, optional, tag = "2")]
    pub arrival: Option<StopTimeEvent>,
    /// The updated departure.
    #[prost(message, optional, tag = "3")]
    pub departure: Option<StopTimeEvent>,
    /// The platform the update is about.
    #[prost(string, optional, tag = "4")]
    pub stop_id: Option<String>,
    /// The relationship between the update and the schedule.
    #[prost(int32, tag = "5")]
    pub schedule_relationship: i32,
}

/// An updated arrival or departure time.
#[derive(Clone, PartialEq, Message)]
pub struct StopTimeEvent {
    /// The delay relative to the schedule, in seconds. May be negative.
    #[prost(int32, tag = "1")]
    pub delay: i32,
    /// The updated time, in Unix epoch seconds.
    #[prost(uint64, optional, tag = "2")]
    pub time: Option<u64>,
    /// How uncertain the updated time is, in seconds.
    #[prost(int32, tag = "3")]
    pub uncertainty: i32,
}

/// Identifies a trip.
#[derive(Clone, PartialEq, Message)]
pub struct TripDescriptor {
    /// The trip id.
    #[prost(string, tag = "1")]
    pub trip_id: String,
    /// The time the trip starts, in `HH:MM:SS`.
    #[prost(string, optional, tag = "2")]
    pub start_time: Option<String>,
    /// The date the trip starts, in `YYYYMMDD`.
    #[prost(string, optional, tag = "3")]
    pub start_date: Option<String>,
    /// The relationship between the trip and the schedule.
    #[prost(int32, tag = "4")]
    pub schedule_relationship: i32,
    /// The route the trip runs on.
    #[prost(string, optional, tag = "5")]
    pub route_id: Option<String>,
    /// The direction of travel.
    #[prost(uint32, optional, tag = "6")]
    pub direction_id: Option<u32>,
}

/// Identifies a vehicle.
#[derive(Clone, PartialEq, Message)]
pub struct VehicleDescriptor {
    /// The vehicle's id.
    #[prost(string, optional, tag = "1")]
    pub id: Option<String>,
    /// The vehicle's licence plate.
    #[prost(string, optional, tag = "2")]
    pub license_plate: Option<String>,
    /// The route the vehicle is running.
    #[prost(string, optional, tag = "5")]
    pub route_id: Option<String>,
    /// The direction of travel.
    #[prost(uint32, optional, tag = "6")]
    pub direction_id: Option<u32>,
}

/// A service alert.
///
/// The text fields are held as raw bytes rather than as `TranslatedString`
/// messages, because BART is not self-consistent about their shape: some of
/// them hold a bare string where the spec expects a nested `Translation`.
/// See [`alert_text`] for how they are interpreted.
#[derive(Clone, PartialEq, Message)]
pub struct Alert {
    /// The periods the alert is active for.
    #[prost(message, repeated, tag = "1")]
    pub active_period: Vec<TimeRange>,
    /// The cause of the alert.
    #[prost(int32, tag = "2")]
    pub cause: i32,
    /// The effect of the alert.
    #[prost(int32, tag = "3")]
    pub effect: i32,
    /// A URL with more information. (The spec's field 4.)
    #[prost(bytes = "vec", optional, tag = "4")]
    pub url: Option<Vec<u8>>,
    /// The alert's header. BART puts the publishing organisation here.
    #[prost(bytes = "vec", optional, tag = "5")]
    pub header_text: Option<Vec<u8>>,
    /// The alert's description, per the spec.
    #[prost(bytes = "vec", optional, tag = "6")]
    pub description_text: Option<Vec<u8>>,
    /// A text-to-speech rendering of the header.
    #[prost(bytes = "vec", optional, tag = "7")]
    pub tts_header_text: Option<Vec<u8>>,
    /// BART puts the advisories URL in this slot; the spec expects a
    /// text-to-speech rendering of the description.
    #[prost(bytes = "vec", optional, tag = "8")]
    pub tts_description_text: Option<Vec<u8>>,
    /// How serious the alert is.
    #[prost(int32, tag = "9")]
    pub severity_level: i32,
    /// BART's short title for the alert.
    ///
    /// The spec defines this field as `created_at`, a `uint64`.
    #[prost(bytes = "vec", optional, tag = "10")]
    pub short_text: Option<Vec<u8>>,
    /// BART's full text for the alert.
    ///
    /// The spec defines this field as `update`, a `TimeRange`.
    #[prost(bytes = "vec", optional, tag = "11")]
    pub long_text: Option<Vec<u8>>,
    /// The trips affected by the alert.
    #[prost(message, repeated, tag = "12")]
    pub affected_trip: Vec<AffectedTrip>,
    /// The routes affected by the alert.
    #[prost(message, repeated, tag = "13")]
    pub affected_route: Vec<AffectedRoute>,
    /// The stops affected by the alert.
    #[prost(message, repeated, tag = "14")]
    pub affected_stop: Vec<AffectedStop>,
}

/// A time range.
#[derive(Clone, PartialEq, Message)]
pub struct TimeRange {
    /// The start of the range, in Unix epoch seconds.
    #[prost(uint64, optional, tag = "1")]
    pub start: Option<u64>,
    /// The end of the range, in Unix epoch seconds.
    #[prost(uint64, optional, tag = "2")]
    pub end: Option<u64>,
}

/// A string with one or more translations.
#[derive(Clone, PartialEq, Message)]
pub struct TranslatedString {
    /// The translations.
    #[prost(message, repeated, tag = "1")]
    pub translation: Vec<Translation>,
}

impl TranslatedString {
    /// The text of the first translation, if there is one.
    pub fn text(&self) -> Option<&str> {
        self.translation.first().map(|t| t.text.as_str())
    }
}

/// One translation of a string.
#[derive(Clone, PartialEq, Message)]
pub struct Translation {
    /// The translated text.
    #[prost(string, tag = "1")]
    pub text: String,
    /// The language of the text, e.g. `en-US`.
    #[prost(string, optional, tag = "2")]
    pub language: Option<String>,
}

/// A trip affected by an alert.
#[derive(Clone, PartialEq, Message)]
pub struct AffectedTrip {
    /// The affected trip.
    #[prost(message, optional, tag = "1")]
    pub trip: Option<TripDescriptor>,
}

/// A route affected by an alert.
#[derive(Clone, PartialEq, Message)]
pub struct AffectedRoute {
    /// The affected route.
    #[prost(message, optional, tag = "1")]
    pub route: Option<RouteDescriptor>,
}

/// Identifies a route.
#[derive(Clone, PartialEq, Message)]
pub struct RouteDescriptor {
    /// The route id.
    #[prost(string, tag = "1")]
    pub route_id: String,
    /// The agency operating the route.
    #[prost(string, optional, tag = "2")]
    pub agency_id: Option<String>,
    /// The route's short name.
    #[prost(string, optional, tag = "3")]
    pub route_short_name: Option<String>,
    /// The route's long name.
    #[prost(string, optional, tag = "4")]
    pub route_long_name: Option<String>,
    /// The kind of route.
    #[prost(int32, optional, tag = "5")]
    pub route_type: Option<i32>,
    /// The direction of travel.
    #[prost(uint32, optional, tag = "6")]
    pub direction_id: Option<u32>,
}

/// A stop affected by an alert.
#[derive(Clone, PartialEq, Message)]
pub struct AffectedStop {
    /// The affected stop.
    #[prost(message, optional, tag = "1")]
    pub stop: Option<StopDescriptor>,
}

/// Identifies a stop.
#[derive(Clone, PartialEq, Message)]
pub struct StopDescriptor {
    /// The stop id.
    #[prost(string, tag = "1")]
    pub stop_id: String,
    /// The platform code.
    #[prost(string, optional, tag = "2")]
    pub platform_code: Option<String>,
    /// The station code.
    #[prost(string, optional, tag = "3")]
    pub station_code: Option<String>,
}

/// Read a protobuf varint, returning the value and the offset just after it.
fn read_varint(bytes: &[u8], mut pos: usize) -> Option<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift = 0;
    loop {
        let byte = *bytes.get(pos)?;
        pos += 1;
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift > 63 {
            return None;
        }
    }
    Some((result, pos))
}

/// Return the payload of the first length-delimited field in `bytes`.
fn first_length_delimited(bytes: &[u8]) -> Option<&[u8]> {
    let mut pos = 0usize;
    while pos < bytes.len() {
        let (key, next) = read_varint(bytes, pos)?;
        pos = next;

        match key & 0x07 {
            0 => {
                let (_, next) = read_varint(bytes, pos)?;
                pos = next;
            }
            1 => pos += 8,
            5 => pos += 4,
            2 => {
                let (len, next) = read_varint(bytes, pos)?;
                pos = next;
                let end = pos.checked_add(len as usize)?;
                if end > bytes.len() {
                    return None;
                }
                return Some(&bytes[pos..end]);
            }
            _ => return None,
        }
    }
    None
}

/// Interpret one of BART's alert text fields.
///
/// The field's bytes are the content of a `TranslatedString`: a repeated
/// `translation` field whose entries are `Translation` messages. BART is not
/// self-consistent about this — some fields hold a bare string where a
/// `Translation` is expected — so both shapes are accepted:
///
/// - a well-formed `Translation` yields its text and language;
/// - anything else is taken to be the text itself.
pub fn alert_text(bytes: &[u8]) -> Option<(String, Option<String>)> {
    if bytes.is_empty() {
        return None;
    }

    let payload = first_length_delimited(bytes).unwrap_or(bytes);

    match Translation::decode(payload) {
        Ok(translation) if !translation.text.is_empty() => {
            Some((translation.text, translation.language))
        }
        _ => String::from_utf8(payload.to_vec())
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| (s, None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_an_empty_feed() {
        let bytes = FeedMessage {
            header: Some(FeedHeader {
                gtfs_realtime_version: "1.0".into(),
                incrementality: 0,
                timestamp: 1_700_000_000,
            }),
            entity: Vec::new(),
        }
        .encode_to_vec();

        let decoded = FeedMessage::decode(bytes.as_slice()).expect("decodes");
        assert_eq!(decoded.header.unwrap().gtfs_realtime_version, "1.0");
        assert!(decoded.entity.is_empty());
    }

    #[test]
    fn decodes_barts_non_standard_alert_fields() {
        // BART puts translated strings in Alert fields 10 and 11, where the
        // spec expects a uint64 and a TimeRange.
        let alert = Alert {
            short_text: Some(b"\x0a\x05hello".to_vec()),
            long_text: Some(b"\x0a\x05world".to_vec()),
            ..Default::default()
        };

        let bytes = alert.encode_to_vec();
        let decoded = Alert::decode(bytes.as_slice()).expect("decodes");
        assert_eq!(
            alert_text(decoded.short_text.as_deref().unwrap()),
            Some(("hello".to_string(), None))
        );
        assert_eq!(
            alert_text(decoded.long_text.as_deref().unwrap()),
            Some(("world".to_string(), None))
        );
    }

    #[test]
    fn reads_a_nested_translation() {
        let translation = Translation {
            text: "http://example.test/advisories".to_string(),
            language: Some("en-US".to_string()),
        };
        let mut bytes = vec![0x0a];
        bytes.push(translation.encoded_len() as u8);
        bytes.extend_from_slice(&translation.encode_to_vec());

        let (text, language) = alert_text(&bytes).expect("text");
        assert_eq!(text, "http://example.test/advisories");
        assert_eq!(language.as_deref(), Some("en-US"));
    }

    #[test]
    fn reads_a_bare_string() {
        // BART's `header_text` holds a bare string, not a Translation.
        let (text, language) = alert_text(b"BART").expect("text");
        assert_eq!(text, "BART");
        assert_eq!(language, None);
    }
}
