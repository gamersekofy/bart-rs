//! `bart-rs trip` — a trip's full stop-time sequence.

use crate::cli::{Cli, TripArgs};
use crate::error::{Error, Result};
use crate::schedule;

use super::print_table;

/// Show a trip's stop times in order.
pub fn run(cli: &Cli, args: &TripArgs) -> Result<()> {
    let context = super::Context::load(cli.json)?;
    let feed = &context.feed;

    let trip = feed
        .trip(&args.trip_id)
        .ok_or_else(|| Error::msg(format!("no trip with id \"{}\"", args.trip_id)))?;

    let times = feed.stop_times_for_trip(&trip.trip_id);
    let route = feed.route(&trip.route_id);

    if cli.json {
        return super::print_json(&serde_json::json!({
            "trip": trip,
            "route": route,
            "stop_times": times,
        }));
    }

    println!("Trip {}", trip.trip_id);
    println!();
    println!(
        "  route         {} {} ({})",
        super::swatch(route.and_then(|r| r.color.as_deref())),
        route.map(|r| r.display_name()).unwrap_or("?"),
        trip.route_id
    );
    println!("  service       {}", trip.service_id);
    if let Some(description) = feed.service_description(&trip.service_id) {
        println!("                {description}");
    }
    if let Some(headsign) = &trip.trip_headsign {
        println!("  headsign      {headsign}");
    }
    if let Some(direction) = trip.direction_id {
        println!(
            "  direction     {direction} ({})",
            feed.direction_of(&trip.route_id, direction)
                .map(|d| d.direction.as_str())
                .unwrap_or("?")
        );
    }
    if let Some(shape) = &trip.shape_id {
        println!("  shape         {shape}");
    }
    if let Some(value) = trip.wheelchair_accessible {
        println!("  wheelchair    {}", availability(value));
    }
    if let Some(value) = trip.bikes_allowed {
        println!("  bikes         {}", availability(value));
    }
    println!();

    let rows: Vec<Vec<String>> = times
        .iter()
        .map(|time| {
            let name = feed
                .stop(&time.stop_id)
                .map(|s| s.display_name().to_string())
                .unwrap_or_else(|| time.stop_id.clone());
            vec![
                time.stop_sequence.to_string(),
                time.stop_id.clone(),
                name,
                time.arrival_time
                    .map(schedule::fmt_gtfs_time)
                    .unwrap_or_default(),
                time.departure_time
                    .map(schedule::fmt_gtfs_time)
                    .unwrap_or_default(),
            ]
        })
        .collect();

    print_table(&["SEQ", "STOP", "NAME", "ARRIVE", "DEPART"], &rows);

    Ok(())
}

/// Render GTFS's three-state availability encoding.
fn availability(value: u8) -> &'static str {
    match value {
        1 => "yes",
        2 => "no",
        _ => "no information",
    }
}
