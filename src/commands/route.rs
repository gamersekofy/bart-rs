//! `bart-rs route` — details for one route.

use colored::Colorize;

use crate::cli::{Cli, RouteArgs};
use crate::error::{Error, Result};

/// Show a route's details and the stations it serves.
pub fn run(cli: &Cli, args: &RouteArgs) -> Result<()> {
    let context = super::Context::load(cli.json)?;
    let feed = &context.feed;

    let route = feed
        .find_route(&args.route)
        .ok_or_else(|| Error::msg(format!("no route matching \"{}\"", args.route)))?;

    let stations = feed.stations_on_route(&route.id);
    let trips = feed.trips_on_route(&route.id);

    if cli.json {
        return super::print_json(&serde_json::json!({
            "route": route,
            "stations": stations,
            "trip_count": trips.len(),
        }));
    }

    println!(
        "{}  {}",
        route.short_name.clone().unwrap_or_default().bold(),
        route.long_name.clone().unwrap_or_default()
    );
    println!();
    println!("  id             {}", route.id);
    if let Some(kind) = route.route_type {
        println!("  type           {kind:?}");
    }
    if let Some(color) = &route.color {
        println!("  color          {} {}", super::swatch(Some(color)), color);
    }
    if let Some(url) = &route.url {
        println!("  url            {url}");
    }
    if let Some(desc) = &route.desc {
        println!("  description    {desc}");
    }

    if let Some(attributes) = feed.route_attributes_of(&route.id) {
        println!(
            "  attributes     category {}, subcategory {}, running way {}",
            attributes.category.unwrap_or(0),
            attributes.subcategory.unwrap_or(0),
            attributes.running_way.unwrap_or(0)
        );
    }

    if feed.realtime_enabled(&route.id) {
        println!("  realtime       enabled");
    }

    // BART's routes are directional pairs, so each route has its own
    // direction.
    for direction in 0..=1u8 {
        if let Some(d) = feed.direction_of(&route.id, direction) {
            println!("  direction      {}", d.direction);
            break;
        }
    }

    println!("  trips          {}", trips.len());
    println!();

    if stations.is_empty() {
        println!("  No stations found for this route.");
        return Ok(());
    }

    println!("  Stations served");
    for (index, station) in stations.iter().enumerate() {
        println!(
            "    {:>2}  {:<34} {}",
            index + 1,
            station.display_name(),
            station.id
        );
    }

    Ok(())
}
