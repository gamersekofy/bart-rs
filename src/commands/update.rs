//! `bart-rs update` — download and install the latest feed.

use chrono::Utc;

use crate::cli::UpdateArgs;
use crate::error::Result;
use crate::update::{self, UpdateState};

/// Download the current feed and install it.
pub fn run(args: &UpdateArgs) -> Result<()> {
    let client = update::client()?;
    let mut state = update::read_state()?;

    // Ask the server which release it is offering. This is a HEAD request,
    // so it costs nothing even when the feed is 900 kB.
    //
    // If the server will not answer a HEAD request, fall back to fetching
    // the feed and comparing the releases that way.
    let remote = update::probe_release(&client).ok();

    let local = state.release();

    if args.check {
        println!("Checking {}", update::FEED_URL);
        state.last_checked_at = Some(Utc::now().to_rfc3339());
        update::write_state(&state)?;

        let local_label = local
            .as_ref()
            .map(update::Release::label)
            .unwrap_or("(none)");
        println!("Installed release: {local_label}");
        println!(
            "Feed version:      {}",
            state.feed_version.as_deref().unwrap_or("(none)")
        );

        match &remote {
            Some(remote) => {
                println!("Offered release:   {}", remote.label());
                let up_to_date = local.as_ref().is_some_and(|local| local.is_same(remote));
                if up_to_date {
                    println!("The installed feed is up to date.");
                } else {
                    println!("A newer feed is available. Run `bart-rs update`.");
                }
            }
            None => {
                println!("Could not determine which release the server is offering.");
            }
        }
        return Ok(());
    }

    if !args.force
        && let (Some(local), Some(remote)) = (&local, &remote)
    {
        if local.is_same(remote) {
            println!(
                "The installed feed is already up to date ({}).",
                remote.label()
            );
            state.last_checked_at = Some(Utc::now().to_rfc3339());
            update::write_state(&state)?;
            return Ok(());
        }
        println!(
            "A newer feed is available ({}); downloading it.",
            remote.label()
        );
    }

    println!("Downloading {} …", update::FEED_URL);
    let download = update::download_latest(&client)?;
    println!("  resolved to {}", download.source_url);
    println!("  downloaded {} bytes", download.bytes.len());

    let version = download.version.clone();
    if let Some(version) = &version {
        println!("Feed version {version}");
    }

    if let Some((start, end)) = download.coverage {
        println!("  covers {start} → {end}");
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
        last_modified: download.last_modified.clone(),
    };
    update::write_state(&new_state)?;

    println!("Done.");
    Ok(())
}
