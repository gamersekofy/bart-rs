//! `bart-rs status` — what feed is installed, and is it current?

use chrono::Utc;

use crate::cli::Cli;
use crate::error::Result;
use crate::update;

use super::plural;

/// Show the installed feed's metadata and any staleness warnings.
pub fn run(cli: &Cli) -> Result<()> {
    let context = super::Context::load(cli.json)?;
    let feed = &context.feed;

    let info = feed.feed_info();
    let state = update::read_state()?;

    println!("BART GTFS feed");
    println!("  version        {}", feed.version().unwrap_or("unknown"));

    if let Some(release) = state.release() {
        println!("  release        {}", release.label());
    }

    // The coverage window comes from the calendar, not from feed_info.txt:
    // BART publishes a new feed without updating feed_info's dates, so
    // those describe the previous release.
    let coverage = feed.coverage();
    if let Some((start, end)) = coverage {
        println!("  coverage       {start} → {end}");
    }

    if let Some(info) = info {
        if let Some(publisher) = feed.publisher() {
            println!("  publisher      {}", publisher.name);
        }
        if let Some(lang) = &info.lang {
            println!("  language       {lang}");
        }
    }

    // Point out the discrepancy rather than hiding it: the feed_info dates
    // are what a reader of the raw files would expect, and they are wrong.
    if let (Some((start, end)), Some(info)) = (coverage, info)
        && let (Some(claimed_start), Some(claimed_end)) = (info.start_date, info.end_date)
        && (claimed_start, claimed_end) != (start, end)
    {
        println!(
            "                 note: feed_info.txt says {claimed_start} → {claimed_end}; BART leaves it stale"
        );
    }

    if let Some(downloaded) = &state.downloaded_at {
        println!("  downloaded     {downloaded}");
    }
    if let Some(checked) = &state.last_checked_at {
        println!("  last checked   {checked}");
    }
    println!(
        "  location       {}",
        crate::paths::current_feed_dir()?.display()
    );

    println!();
    println!("  routes         {}", feed.routes.len());
    println!("  stations       {}", feed.stations().len());
    println!("  platforms      {}", feed.platforms().len());
    println!("  entrances      {}", feed.entrances().len());
    println!("  trips          {}", feed.trips.len());
    println!("  stop times     {}", feed.stop_times.len());
    println!("  fares          {}", feed.fare_attributes.len());

    let warnings = update::staleness(coverage, &state, Utc::now());
    for warning in &warnings {
        println!();
        println!("⚠ {}", warning.message());
    }

    if warnings.is_empty() {
        println!();
        println!(
            "The feed is current ({} of {} tables loaded).",
            crate::attributes::TABLE_NAMES.len(),
            crate::attributes::TABLE_NAMES.len()
        );
    }

    let _ = plural(0, "table", "tables");
    Ok(())
}
