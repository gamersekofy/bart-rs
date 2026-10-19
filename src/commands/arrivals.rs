//! `bart-rs arrivals` and `bart-rs departures`.

use chrono::NaiveDate;

use crate::cli::{Cli, ScheduleArgs};
use crate::error::{Error, Result};
use crate::feed::{Arrival, Feed};
use crate::schedule;

use super::print_table;

/// Show the trains arriving at, or departing from, a stop.
pub fn run(cli: &Cli, args: &ScheduleArgs, departing: bool) -> Result<()> {
    let context = super::Context::load(cli.json)?;
    let feed = &context.feed;

    let stop = super::resolve_stop(feed, &args.stop)?;

    let (date, from) = resolve_time(&args.date, &args.at)?;

    let scheduled: Vec<Arrival> = if departing {
        feed.departures_from(&stop.id, date, from)
    } else {
        feed.arrivals_at(&stop.id, date, from)
    };

    let mut rows_source: Vec<(Arrival, Option<i32>, bool)> = scheduled
        .iter()
        .take(args.limit)
        .map(|a| (a.clone(), None, false))
        .collect();

    if args.realtime {
        let client = crate::update::client()?;
        let live = crate::realtime::fetch_trip_updates(&client)?;
        let overlaid = crate::realtime::overlay(feed, &scheduled, &live, departing);

        rows_source = overlaid
            .iter()
            .take(args.limit)
            .map(|a| {
                (
                    Arrival {
                        time: a.time,
                        trip_id: a.trip_id.clone(),
                        route_id: a.route_id.clone(),
                        route_short_name: a.route_short_name.clone(),
                        headsign: a.headsign.clone(),
                        platform_id: a.platform_id.clone(),
                        service_id: String::new(),
                        service_description: None,
                    },
                    Some(a.delay),
                    a.skipped,
                )
            })
            .collect();
    }

    if cli.json {
        let payload: Vec<serde_json::Value> = rows_source
            .iter()
            .map(|(arrival, delay, skipped)| {
                serde_json::json!({
                    "time": schedule::fmt_gtfs_time(arrival.time),
                    "trip_id": arrival.trip_id,
                    "route_id": arrival.route_id,
                    "route_short_name": arrival.route_short_name,
                    "headsign": arrival.headsign,
                    "platform_id": arrival.platform_id,
                    "delay_seconds": delay,
                    "skipped": skipped,
                })
            })
            .collect();
        return super::print_json(&payload);
    }

    if rows_source.is_empty() {
        println!(
            "No {} at {} on {date}.",
            if departing { "departures" } else { "arrivals" },
            stop.display_name()
        );
        return Ok(());
    }

    let rows: Vec<Vec<String>> = rows_source
        .iter()
        .map(|(arrival, delay, skipped)| {
            let status = match (*delay, *skipped) {
                (Some(_), true) => "skipped".to_string(),
                (Some(d), false) if d > 0 => format!("+{} min late", d / 60),
                (Some(d), false) if d < 0 => format!("{} min early", -d / 60),
                (Some(_), false) => "on time".to_string(),
                (None, _) => String::new(),
            };
            vec![
                schedule::fmt_short_time(arrival.time),
                super::swatch(
                    feed.route(&arrival.route_id)
                        .and_then(|route| route.color.as_deref()),
                ),
                arrival.route_short_name.clone().unwrap_or_default(),
                arrival.headsign.clone().unwrap_or_default(),
                arrival.platform_id.clone().unwrap_or_default(),
                status,
            ]
        })
        .collect();

    print_table(
        &["TIME", "", "ROUTE", "DESTINATION", "PLATFORM", "STATUS"],
        &rows,
    );

    Ok(())
}

/// Work out the service date and the time of day to start from.
fn resolve_time(date: &Option<String>, at: &Option<String>) -> Result<(NaiveDate, u32)> {
    let (default_date, default_time) = schedule::now_service_day();

    let date = match date {
        Some(text) => NaiveDate::parse_from_str(text, "%Y-%m-%d")
            .map_err(|e| Error::msg(format!("could not parse the date \"{text}\": {e}")))?,
        None => default_date,
    };

    let from = match at {
        Some(text) => parse_clock(text)?,
        None => default_time,
    };

    Ok((date, from))
}

/// Parse a `HH:MM` clock time into seconds since midnight.
fn parse_clock(text: &str) -> Result<u32> {
    let (hours, minutes) = text.split_once(':').ok_or_else(|| {
        Error::msg(format!(
            "could not parse the time \"{text}\": expected HH:MM"
        ))
    })?;

    let hours: u32 = hours.trim().parse().map_err(|_| {
        Error::msg(format!(
            "could not parse the time \"{text}\": expected HH:MM"
        ))
    })?;
    let minutes: u32 = minutes.trim().parse().map_err(|_| {
        Error::msg(format!(
            "could not parse the time \"{text}\": expected HH:MM"
        ))
    })?;

    if hours > 24 || minutes > 59 {
        return Err(Error::msg(format!(
            "could not parse the time \"{text}\": expected HH:MM"
        )));
    }

    Ok(hours * 3600 + minutes * 60)
}

/// The names of the lines serving a stop, for use in messages.
pub fn line_names(feed: &Feed, stop_id: &str) -> Vec<String> {
    feed.routes_serving(stop_id)
        .into_iter()
        .map(|r| r.short_name.clone().unwrap_or_else(|| r.id.clone()))
        .collect()
}
