//! `bart-rs update` — download and install the latest feed.

use chrono::Utc;

use crate::cli::UpdateArgs;
use crate::error::Result;
use crate::update::{self, UpdateState};

/// Download the current feed and install it.
pub fn run(args: &UpdateArgs) -> Result<()> {
    let client = update::client()?;
    let mut state = update::read_state()?;

    if args.check {
        println!("Checking {}", update::FEED_URL);
        let remote = update::check_latest(&client)?;

        state.last_checked_at = Some(Utc::now().to_rfc3339());
        update::write_state(&state)?;

        let local = state.feed_version.as_deref().unwrap_or("(none)");
        match remote {
            Some(remote) => {
                println!("Local feed version:  {local}");
                println!("Remote feed version: {remote}");
                if remote != local {
                    println!("A newer feed is available. Run `bart-rs update`.");
                } else {
                    println!("The installed feed is up to date.");
                }
            }
            None => {
                println!("Local feed version:  {local}");
                println!("Could not determine the remote feed version.");
            }
        }
        return Ok(());
    }

    if !args.force {
        if let Some(remote) = update::check_latest(&client)? {
            let local = state.feed_version.as_deref().unwrap_or("");
            if remote == local {
                println!("The installed feed is already version {remote}.");
                state.last_checked_at = Some(Utc::now().to_rfc3339());
                update::write_state(&state)?;
                return Ok(());
            }
        }
    }

    println!("Downloading {} …", update::FEED_URL);
    let download = update::download_latest(&client)?;
    println!("  resolved to {}", download.source_url);
    println!("  downloaded {} bytes", download.bytes.len());

    let version = download.version.clone();
    if let Some(version) = &version {
        println!("Feed version {version}");
    }

    if let Some(info) = &download.feed_info {
        if let (Some(start), Some(end)) = (info.start_date, info.end_date) {
            println!("  covers {start} → {end}");
        }
    }

    let archive = update::store_archive(&download.bytes, version.as_deref())?;
    println!("  stored {}", archive.display());

    let feed_dir = crate::paths::feed_dir()?;
    let staging = feed_dir.join(format!(".staging-{}", std::process::id()));
    if staging.exists() {
        let _ = std::fs::remove_dir_all(&staging);
    }

    update::extract(&download.bytes, &staging)?;
    println!(
        "  extracted {} tables",
        crate::attributes::TABLE_NAMES.len()
    );

    update::install(&staging, &feed_dir)?;
    println!(
        "Installed to {}",
        crate::paths::current_feed_dir()?.display()
    );

    let new_state = UpdateState {
        feed_version: version,
        downloaded_at: Some(Utc::now().to_rfc3339()),
        last_checked_at: Some(Utc::now().to_rfc3339()),
        source_url: Some(download.source_url),
    };
    update::write_state(&new_state)?;

    println!("Done.");
    Ok(())
}
