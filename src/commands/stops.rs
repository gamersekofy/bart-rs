//! `bart-rs stops` — list or search stations.

use crate::cli::{Cli, StopsArgs};
use crate::error::Result;

use super::print_table;

/// List or search stops.
pub fn run(cli: &Cli, args: &StopsArgs) -> Result<()> {
    let context = super::Context::load(cli.json)?;
    let feed = &context.feed;

    let matches: Vec<_> = match &args.query {
        Some(query) => feed.find_stops(query),
        None => feed.stops.iter().collect(),
    };

    let filtered: Vec<_> = matches
        .into_iter()
        .filter(|stop| args.all || stop.is_station())
        .take(args.limit)
        .collect();

    if cli.json {
        return super::print_json(&filtered);
    }

    if filtered.is_empty() {
        if let Some(query) = &args.query {
            println!("No stops matching \"{query}\".");
        } else {
            println!("No stops found.");
        }
        return Ok(());
    }

    let rows: Vec<Vec<String>> = filtered
        .iter()
        .map(|stop| {
            let platforms = feed.platforms_of(&stop.id).len();
            vec![
                stop.code.clone().unwrap_or_default(),
                stop.display_name().to_string(),
                platforms.to_string(),
                stop.zone_id.clone().unwrap_or_default(),
                stop.id.clone(),
            ]
        })
        .collect();

    print_table(&["CODE", "NAME", "PLATFORMS", "ZONE", "ID"], &rows);
    Ok(())
}
