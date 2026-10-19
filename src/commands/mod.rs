//! The implementations behind each command.
//!
//! Every command module exposes a `run` function that takes the parsed
//! arguments and the shared context, and does its own printing. No command
//! reads the environment or parses arguments itself.

pub mod alerts;
pub mod arrivals;
pub mod fare;
pub mod route;
pub mod routes;
pub mod status;
pub mod stop;
pub mod stops;
pub mod table;
pub mod trip;
pub mod update;

use colored::{Color, Colorize};

use crate::cli::Cli;
use crate::error::{Error, Result};
use crate::feed::Feed;

/// The context every command needs.
pub struct Context {
    /// The installed feed.
    pub feed: Feed,
    /// Whether to emit JSON rather than formatted text.
    pub json: bool,
}

impl Context {
    /// Load the installed feed.
    pub fn load(json: bool) -> Result<Context> {
        Ok(Context {
            feed: Feed::from_data_dir()?,
            json,
        })
    }
}

/// Route a parsed command line to the right implementation.
pub fn dispatch(cli: Cli) -> Result<()> {
    if cli.no_color || std::env::var_os("NO_COLOR").is_some() {
        colored::control::set_override(false);
    }

    if let Some(dir) = &cli.data_dir {
        // Process-local only: this never touches the user's shell.
        std::env::set_var(crate::paths::DATA_DIR_ENV, dir);
    }

    if cli.json && !cli.command.supports_json() {
        return Err(Error::msg(format!(
            "`bart-rs {}` does not support --json",
            cli.command.name()
        )));
    }

    match cli.command.clone() {
        crate::cli::Command::Update(args) => update::run(&args),
        crate::cli::Command::Status => status::run(&cli),
        crate::cli::Command::Routes => routes::run(&cli),
        crate::cli::Command::Stops(args) => stops::run(&cli, &args),
        crate::cli::Command::Route(args) => route::run(&cli, &args),
        crate::cli::Command::Stop(args) => stop::run(&cli, &args),
        crate::cli::Command::Arrivals(args) => arrivals::run(&cli, &args, false),
        crate::cli::Command::Departures(args) => arrivals::run(&cli, &args, true),
        crate::cli::Command::Trip(args) => trip::run(&cli, &args),
        crate::cli::Command::Fare(args) => fare::run(&cli, &args),
        crate::cli::Command::Alerts(args) => alerts::run(&cli, &args),
        crate::cli::Command::Table(args) => table::run(&cli, &args),
    }
}

/// Print a table with aligned columns.
pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.chars().count());
            }
        }
    }

    let header: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!("{h:<width$}", width = widths[i]))
        .collect();
    println!("{}", header.join("  ").trim_end().bold());

    for row in rows {
        let line: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let width = widths.get(i).copied().unwrap_or(0);
                format!("{cell:<width$}")
            })
            .collect();
        println!("{}", line.join("  ").trim_end());
    }
}

/// Print a value as JSON.
pub fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| Error::msg(format!("could not serialise the output: {e}")))?;
    println!("{text}");
    Ok(())
}

/// Resolve a user-supplied stop, or fail with a helpful message.
pub fn resolve_stop<'a>(feed: &'a Feed, query: &str) -> Result<&'a crate::attributes::stops::Stop> {
    feed.find_stop(query).ok_or_else(|| {
        Error::msg(format!(
            "no stop matching \"{query}\" (try `bart-rs stops {query}` to search)"
        ))
    })
}

/// Render a route's colour as an ANSI-coloured block.
///
/// The colour is a hex string such as `FF0000`, which has to be turned into
/// a true-colour value by hand: `colored`'s own `&str` conversion only
/// understands the sixteen named colours and silently falls back to white
/// for anything else.
pub fn swatch(color: Option<&str>) -> String {
    match color.and_then(parse_hex_colour) {
        Some((r, g, b)) => "\u{2588}"
            .to_string()
            .color(Color::TrueColor { r, g, b })
            .to_string(),
        None => " ".to_string(),
    }
}

/// Parse a six-digit hex colour into its red, green and blue components.
fn parse_hex_colour(hex: &str) -> Option<(u8, u8, u8)> {
    if hex.len() != 6 {
        return None;
    }
    let component = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    Some((component(0..2)?, component(2..4)?, component(4..6)?))
}

/// Format a count with a pluralised noun.
pub fn plural(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{count} {singular}")
    } else {
        format!("{count} {plural}")
    }
}
