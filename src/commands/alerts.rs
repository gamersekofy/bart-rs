//! `bart-rs alerts` — BART's current service alerts.

use colored::Colorize;

use crate::cli::{AlertsArgs, Cli};
use crate::error::Result;

/// Show the service alerts BART is currently publishing.
pub fn run(cli: &Cli, args: &AlertsArgs) -> Result<()> {
    let client = crate::update::client()?;
    let alerts = crate::realtime::fetch_alerts(&client)?;

    let filtered: Vec<_> = alerts
        .into_iter()
        .filter(|alert| {
            args.route
                .as_ref()
                .is_none_or(|route| alert.affected_routes.iter().any(|r| r == route))
                && args
                    .stop
                    .as_ref()
                    .is_none_or(|stop| alert.affected_stops.iter().any(|s| s == stop))
        })
        .filter(|alert| !alert.is_empty())
        .collect();

    if cli.json {
        return super::print_json(&filtered);
    }

    if filtered.is_empty() {
        println!("No service alerts.");
        return Ok(());
    }

    for alert in &filtered {
        println!(
            "{}",
            alert
                .title
                .as_deref()
                .unwrap_or("BART service alert")
                .bold()
        );
        if let Some(description) = &alert.description {
            println!("  {description}");
        }
        if let Some(url) = &alert.url {
            println!("  more info: {url}");
        }
        if !alert.affected_routes.is_empty() {
            println!("  routes: {}", alert.affected_routes.join(", "));
        }
        if !alert.affected_stops.is_empty() {
            println!("  stops:  {}", alert.affected_stops.join(", "));
        }
        println!();
    }

    Ok(())
}
