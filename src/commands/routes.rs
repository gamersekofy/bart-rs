//! `bart-rs routes` — list BART's routes.

use crate::cli::Cli;
use crate::error::Result;

use super::print_table;

/// List every route.
pub fn run(cli: &Cli) -> Result<()> {
    let context = super::Context::load(cli.json)?;
    let feed = &context.feed;

    if cli.json {
        return super::print_json(&feed.routes);
    }

    let mut rows: Vec<Vec<String>> = Vec::new();
    for route in &feed.routes {
        rows.push(vec![
            route.id.clone(),
            route.short_name.clone().unwrap_or_default(),
            route.long_name.clone().unwrap_or_default(),
            format!(
                "{:?}",
                route
                    .route_type
                    .unwrap_or(crate::attributes::routes::RouteType::Bus)
            ),
            super::swatch(route.color.as_deref()),
            route.color.clone().unwrap_or_default(),
        ]);
    }

    print_table(&["ID", "SHORT", "LONG", "TYPE", "", "COLOR"], &rows);
    Ok(())
}
