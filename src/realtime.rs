//! GTFS-Realtime: BART's live trip updates and service alerts.
//!
//! # Endpoints
//!
//! - Trip updates: <https://api.bart.gov/gtfsrt/tripupdate.aspx>
//! - Service alerts: <https://api.bart.gov/gtfsrt/alerts.aspx>
//!
//! Neither needs an API key. Both redirect from `http://` to `https://`, and
//! both are served with a **wrong** `Content-Type` of `text/html` — the body
//! is a protobuf `FeedMessage`. This module therefore never inspects the
//! content type; it reads the body as bytes and decodes it as protobuf.
//!
//! See `docs/03-data-sources.md` for the full notes on both feeds.

use crate::error::{Error, Result};
use crate::feed::{Arrival, Feed};
use crate::gtfsrt::{Alert, FeedMessage, TripUpdate};
use prost::Message;
use serde::Serialize;

/// The live trip update feed.
pub const TRIP_UPDATE_URL: &str = "https://api.bart.gov/gtfsrt/tripupdate.aspx";

/// The service alert feed.
pub const ALERTS_URL: &str = "https://api.bart.gov/gtfsrt/alerts.aspx";

/// The `schedule_relationship` value meaning "this stop is skipped".
const RELATIONSHIP_SKIPPED: i32 = 1;

/// The `schedule_relationship` value meaning "no information".
const RELATIONSHIP_NO_DATA: i32 = 2;

/// A live time for one stop of one trip.
#[derive(Debug, Clone, Serialize)]
pub struct LiveStopTime {
    /// The platform the update is about.
    pub stop_id: Option<String>,
    /// The position of the stop along the trip.
    pub stop_sequence: Option<u32>,
    /// How late the arrival is, in seconds. Negative means early.
    pub arrival_delay: i32,
    /// How late the departure is, in seconds. Negative means early.
    pub departure_delay: i32,
    /// The expected arrival, in Unix epoch seconds.
    pub arrival_time: Option<u64>,
    /// The expected departure, in Unix epoch seconds.
    pub departure_time: Option<u64>,
    /// Whether the stop is skipped.
    pub skipped: bool,
}

/// A live update for one trip.
#[derive(Debug, Clone, Serialize)]
pub struct LiveTrip {
    /// The trip id.
    pub trip_id: String,
    /// The route, if the update names one.
    pub route_id: Option<String>,
    /// The direction of travel, if the update names one.
    pub direction_id: Option<u32>,
    /// The updated stop times.
    pub stops: Vec<LiveStopTime>,
}

/// A service alert.
#[derive(Debug, Clone, Serialize)]
pub struct ServiceAlert {
    /// The entity id, e.g. `BSA_560`.
    pub id: String,
    /// A short title, e.g. `BART.gov Alert`.
    pub title: Option<String>,
    /// The full text of the alert.
    pub description: Option<String>,
    /// A URL with more information.
    pub url: Option<String>,
    /// The cause of the alert, as a raw GTFS-Realtime enum value.
    pub cause: i32,
    /// The effect of the alert, as a raw GTFS-Realtime enum value.
    pub effect: i32,
    /// The ids of the routes the alert mentions.
    pub affected_routes: Vec<String>,
    /// The ids of the stops the alert mentions.
    pub affected_stops: Vec<String>,
}

impl ServiceAlert {
    /// Whether the alert has anything worth showing.
    pub fn is_empty(&self) -> bool {
        self.title.is_none() && self.description.is_none() && self.url.is_none()
    }
}

/// Decode a GTFS-Realtime payload.
pub fn decode_feed(bytes: &[u8]) -> Result<FeedMessage> {
    FeedMessage::decode(bytes).map_err(|e| Error::RealtimeDecode(e.to_string()))
}

/// Extract the trip updates from a decoded feed.
pub fn live_trips(message: &FeedMessage) -> Vec<LiveTrip> {
    message
        .entity
        .iter()
        .filter_map(|entity| {
            let update = entity.trip_update.as_ref()?;
            let trip = update.trip.as_ref();

            let trip_id = trip
                .map(|t| t.trip_id.clone())
                .filter(|id| !id.is_empty())
                .unwrap_or_else(|| entity.id.clone());

            Some(LiveTrip {
                trip_id,
                route_id: trip.and_then(|t| t.route_id.clone()),
                direction_id: trip.and_then(|t| t.direction_id),
                stops: update
                    .stop_time_update
                    .iter()
                    .map(|stu| LiveStopTime {
                        stop_id: stu.stop_id.clone(),
                        stop_sequence: stu.stop_sequence,
                        arrival_delay: stu.arrival.as_ref().map(|a| a.delay).unwrap_or(0),
                        departure_delay: stu.departure.as_ref().map(|d| d.delay).unwrap_or(0),
                        arrival_time: stu.arrival.as_ref().and_then(|a| a.time),
                        departure_time: stu.departure.as_ref().and_then(|d| d.time),
                        skipped: stu.schedule_relationship == RELATIONSHIP_SKIPPED
                            || stu.schedule_relationship == RELATIONSHIP_NO_DATA,
                    })
                    .collect(),
            })
        })
        .collect()
}

/// Extract the service alerts from a decoded feed.
///
/// BART does not use the spec's field numbers for the alert text, so each
/// piece of text is looked for in several places. See `docs/03-data-sources.md`
/// for how this was determined.
pub fn service_alerts(message: &FeedMessage) -> Vec<ServiceAlert> {
    message
        .entity
        .iter()
        .filter_map(|entity| {
            let alert = entity.alert.as_ref()?;
            Some(alert_from(alert, &entity.id))
        })
        .collect()
}

fn alert_from(alert: &Alert, id: &str) -> ServiceAlert {
    let text = |value: &Option<Vec<u8>>| {
        value
            .as_deref()
            .and_then(crate::gtfsrt::alert_text)
            .map(|(text, _)| text)
    };

    // The long text is the body of the alert; the short text is its title.
    let long_text = text(&alert.long_text);
    let short_text = text(&alert.short_text);
    let header_text = text(&alert.header_text);
    let spec_description = text(&alert.description_text);

    let title = short_text.clone().or_else(|| header_text.clone());
    let description = long_text
        .clone()
        .or_else(|| spec_description.clone())
        .or_else(|| short_text.clone());

    // BART puts the advisories URL in the `tts_description_text` slot.
    let url = text(&alert.tts_description_text).or_else(|| text(&alert.url));

    let affected_routes = alert
        .affected_route
        .iter()
        .filter_map(|a| a.route.as_ref().map(|r| r.route_id.clone()))
        .collect();
    let affected_stops = alert
        .affected_stop
        .iter()
        .filter_map(|a| a.stop.as_ref().map(|s| s.stop_id.clone()))
        .collect();

    ServiceAlert {
        id: id.to_owned(),
        title,
        description,
        url,
        cause: alert.cause,
        effect: alert.effect,
        affected_routes,
        affected_stops,
    }
}

/// A scheduled arrival adjusted by a live update.
#[derive(Debug, Clone, Serialize)]
pub struct LiveArrival {
    /// The expected time, in seconds since midnight of the service day.
    pub time: u32,
    /// The scheduled time, in seconds since midnight of the service day.
    pub scheduled_time: u32,
    /// How late the train is, in seconds. Negative means early.
    pub delay: i32,
    /// The trip.
    pub trip_id: String,
    /// The route the trip runs on.
    pub route_id: String,
    /// The route's short name.
    pub route_short_name: Option<String>,
    /// Where the train is headed.
    pub headsign: Option<String>,
    /// The platform.
    pub platform_id: Option<String>,
    /// Whether the stop is skipped.
    pub skipped: bool,
}

impl LiveArrival {
    /// The delay in whole minutes, rounded towards zero.
    pub fn delay_minutes(&self) -> i32 {
        self.delay / 60
    }
}

/// Overlay live trip updates onto a list of scheduled arrivals.
///
/// Arrivals with a matching live update get their time adjusted by the
/// reported delay. Live updates that match nothing in the schedule are
/// dropped, because without a schedule there is no route, headsign or
/// platform to show alongside them.
pub fn overlay(
    feed: &Feed,
    scheduled: &[Arrival],
    live: &[LiveTrip],
    departing: bool,
) -> Vec<LiveArrival> {
    let mut results: Vec<LiveArrival> = Vec::new();

    for arrival in scheduled {
        let Some(trip) = live.iter().find(|t| t.trip_id == arrival.trip_id) else {
            results.push(LiveArrival {
                time: arrival.time,
                scheduled_time: arrival.time,
                delay: 0,
                trip_id: arrival.trip_id.clone(),
                route_id: arrival.route_id.clone(),
                route_short_name: arrival.route_short_name.clone(),
                headsign: arrival.headsign.clone(),
                platform_id: arrival.platform_id.clone(),
                skipped: false,
            });
            continue;
        };

        // Prefer an update that names the platform; fall back to the trip's
        // only update when there is just one.
        let update = trip
            .stops
            .iter()
            .find(|s| s.stop_id.as_deref() == arrival.platform_id.as_deref())
            .or_else(|| {
                if trip.stops.len() == 1 {
                    trip.stops.first()
                } else {
                    None
                }
            });

        let Some(update) = update else {
            results.push(LiveArrival {
                time: arrival.time,
                scheduled_time: arrival.time,
                delay: 0,
                trip_id: arrival.trip_id.clone(),
                route_id: arrival.route_id.clone(),
                route_short_name: arrival.route_short_name.clone(),
                headsign: arrival.headsign.clone(),
                platform_id: arrival.platform_id.clone(),
                skipped: false,
            });
            continue;
        };

        let delay = if departing {
            update.departure_delay
        } else {
            update.arrival_delay
        };

        let time = (arrival.time as i64 + delay as i64).max(0) as u32;

        results.push(LiveArrival {
            time,
            scheduled_time: arrival.time,
            delay,
            trip_id: arrival.trip_id.clone(),
            route_id: trip
                .route_id
                .clone()
                .unwrap_or_else(|| arrival.route_id.clone()),
            route_short_name: feed
                .route(
                    &trip
                        .route_id
                        .clone()
                        .unwrap_or_else(|| arrival.route_id.clone()),
                )
                .and_then(|r| r.short_name.clone())
                .or_else(|| arrival.route_short_name.clone()),
            headsign: arrival.headsign.clone(),
            platform_id: arrival.platform_id.clone(),
            skipped: update.skipped,
        });
    }

    results.sort_by(|a, b| a.time.cmp(&b.time).then_with(|| a.trip_id.cmp(&b.trip_id)));
    results
}

/// Fetch and decode the live trip update feed.
#[cfg(feature = "download")]
pub fn fetch_trip_updates(client: &reqwest::blocking::Client) -> Result<Vec<LiveTrip>> {
    let bytes = fetch_bytes(client, TRIP_UPDATE_URL)?;
    let message = decode_feed(&bytes)?;
    Ok(live_trips(&message))
}

/// Fetch and decode the service alert feed.
#[cfg(feature = "download")]
pub fn fetch_alerts(client: &reqwest::blocking::Client) -> Result<Vec<ServiceAlert>> {
    let bytes = fetch_bytes(client, ALERTS_URL)?;
    let message = decode_feed(&bytes)?;
    Ok(service_alerts(&message))
}

#[cfg(feature = "download")]
fn fetch_bytes(client: &reqwest::blocking::Client, url: &str) -> Result<Vec<u8>> {
    let response = client.get(url).send().map_err(|e| Error::Http {
        url: url.to_owned(),
        source: e,
    })?;

    let status = response.status();
    if !status.is_success() {
        return Err(Error::HttpStatus {
            url: url.to_owned(),
            status: status.as_u16(),
        });
    }

    response
        .bytes()
        .map(|b| b.to_vec())
        .map_err(|e| Error::Http {
            url: url.to_owned(),
            source: e,
        })
}

/// Pull the raw `TripUpdate` messages out of a decoded feed.
pub fn raw_trip_updates(message: &FeedMessage) -> Vec<&TripUpdate> {
    message
        .entity
        .iter()
        .filter_map(|e| e.trip_update.as_ref())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gtfsrt::{FeedEntity, FeedHeader, Translation};

    /// Build the bytes of a `TranslatedString` holding one translation.
    fn translated(text: &str) -> Vec<u8> {
        let translation = Translation {
            text: text.to_owned(),
            language: Some("en-US".to_owned()),
        };
        let mut bytes = vec![0x0a];
        bytes.push(translation.encoded_len() as u8);
        bytes.extend_from_slice(&translation.encode_to_vec());
        bytes
    }

    #[test]
    fn alert_text_falls_back_across_barts_field_numbers() {
        let alert = Alert {
            header_text: Some(translated("BART")),
            tts_description_text: Some(translated("http://example.test/advisories")),
            short_text: Some(translated("BART.gov Alert")),
            long_text: Some(translated("Expect 30-minute delays.")),
            ..Default::default()
        };

        let decoded = alert_from(&alert, "BSA_1");
        assert_eq!(decoded.title.as_deref(), Some("BART.gov Alert"));
        assert_eq!(
            decoded.description.as_deref(),
            Some("Expect 30-minute delays.")
        );
        assert_eq!(
            decoded.url.as_deref(),
            Some("http://example.test/advisories")
        );
    }

    #[test]
    fn alert_falls_back_to_the_spec_fields() {
        let alert = Alert {
            description_text: Some(translated("Spec description")),
            header_text: Some(translated("Spec header")),
            ..Default::default()
        };

        let decoded = alert_from(&alert, "BSA_2");
        assert_eq!(decoded.title.as_deref(), Some("Spec header"));
        assert_eq!(decoded.description.as_deref(), Some("Spec description"));
    }

    #[test]
    fn extracts_alerts_from_a_feed() {
        let message = FeedMessage {
            header: Some(FeedHeader {
                gtfs_realtime_version: "1.0".into(),
                incrementality: 0,
                timestamp: 0,
            }),
            entity: vec![FeedEntity {
                id: "BSA_560".into(),
                is_deleted: false,
                trip_update: None,
                alert: Some(Alert {
                    long_text: Some(translated("Delays.")),
                    ..Default::default()
                }),
            }],
        };

        let alerts = service_alerts(&message);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].id, "BSA_560");
        assert_eq!(alerts[0].description.as_deref(), Some("Delays."));
    }
}
