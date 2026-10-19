//! `bart-rs fare` — the fare between two stations.

use crate::cli::{Cli, FareArgs};
use crate::error::{Error, Result};
use colored::Colorize;

use super::print_table;

/// Look up the fare between two places.
pub fn run(cli: &Cli, args: &FareArgs) -> Result<()> {
    let context = super::Context::load(cli.json)?;
    let feed = &context.feed;

    let from = feed
        .find_stop(&args.from)
        .ok_or_else(|| Error::msg(format!("no stop matching \"{}\"", args.from)))?;
    let to = feed
        .find_stop(&args.to)
        .ok_or_else(|| Error::msg(format!("no stop matching \"{}\"", args.to)))?;

    let fare = feed.fare(&from.id, &to.id).ok_or_else(|| {
        Error::msg(format!(
            "no fare found between {} and {}",
            from.display_name(),
            to.display_name()
        ))
    })?;

    if cli.json {
        return super::print_json(&fare);
    }

    println!(
        "{} → {}",
        from.display_name().bold(),
        to.display_name().bold()
    );
    println!();
    println!("  Regular fare   {}", fare.base_price_display());

    let rows: Vec<Vec<String>> = fare
        .prices
        .iter()
        .filter(|p| args.category.as_ref().is_none_or(|c| &p.category_id == c))
        .map(|p| {
            vec![
                p.description.clone(),
                p.category_id.clone(),
                format!(
                    "${}.{:02}",
                    p.price_cents / 100,
                    (p.price_cents % 100).abs()
                ),
            ]
        })
        .collect();

    if rows.is_empty() {
        if let Some(category) = &args.category {
            println!("  No price for rider category {category}.");
        }
    } else {
        print_table(&["RIDER CATEGORY", "ID", "PRICE"], &rows);
    }

    Ok(())
}
