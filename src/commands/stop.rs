//! `bart-rs stop` — details for one station.

use colored::Colorize;

use crate::cli::{Cli, StopArgs};
use crate::error::Result;

use super::print_table;

/// Show a station's details and the lines that serve it.
pub fn run(cli: &Cli, args: &StopArgs) -> Result<()> {
    let context = super::Context::load(cli.json)?;
    let feed = &context.feed;

    let stop = super::resolve_stop(feed, &args.stop)?;
    let station = feed.station_of(&stop.id).unwrap_or(stop);
    let platforms = feed.platforms_of(&station.id);
    let entrances = feed.entrances_of(&station.id);
    let routes = feed.routes_serving(&station.id);

    if cli.json {
        return super::print_json(&serde_json::json!({
            "stop": station,
            "platforms": platforms,
            "entrances": entrances,
            "routes": routes,
        }));
    }

    println!("{}", station.display_name().bold());
    println!();
    println!("  id          {}", station.id);
    if let Some(code) = &station.code {
        println!("  code        {code}");
    }
    if let Some(zone) = &station.zone_id {
        println!("  zone        {zone}");
    }
    if let (Some(lat), Some(lon)) = (station.lat, station.lon) {
        println!("  location    {lat}, {lon}");
    }
    if let Some(url) = &station.url {
        println!("  url         {url}");
    }

    if !platforms.is_empty() {
        println!();
        println!("  Platforms");
        let rows: Vec<Vec<String>> = platforms
            .iter()
            .map(|p| vec![p.id.clone(), p.platform_code.clone().unwrap_or_default()])
            .collect();
        print_table(&["ID", "PLATFORM"], &rows);
    }

    if !entrances.is_empty() {
        println!();
        println!("  Entrances");
        for entrance in &entrances {
            println!("    {}", entrance.display_name());
        }
    }

    println!();
    if routes.is_empty() {
        println!("  No lines serve this station.");
    } else {
        println!("  Lines serving this station");
        let names: Vec<String> = routes
            .iter()
            .map(|r| r.short_name.clone().unwrap_or_else(|| r.id.clone()))
            .collect();
        println!("    {}", names.join("  "));
    }

    Ok(())
}
