//! The command line interface.
//!
//! The full command reference is in `docs/04-cli-spec.md`.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Command line tool for exploring BART's GTFS schedule and realtime data.
#[derive(Debug, Parser)]
#[command(
    name = "bart-rs",
    version,
    about = "Explore BART's GTFS schedule and realtime data",
    long_about = None,
)]
pub struct Cli {
    /// Disable coloured output.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Override the directory the feed is cached in.
    #[arg(long, global = true, value_name = "PATH")]
    pub data_dir: Option<PathBuf>,

    /// Emit JSON instead of formatted text.
    #[arg(long, global = true)]
    pub json: bool,

    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// The commands `bart-rs` understands.
#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Download and install the latest GTFS feed.
    Update(UpdateArgs),
    /// Show the installed feed's version, coverage and staleness.
    Status,
    /// List BART's routes.
    Routes,
    /// List or search stations.
    Stops(StopsArgs),
    /// Show a route's details and the stations it serves.
    Route(RouteArgs),
    /// Show a station's details and the lines that serve it.
    Stop(StopArgs),
    /// Show the trains arriving at a stop.
    Arrivals(ScheduleArgs),
    /// Show the trains departing from a stop.
    Departures(ScheduleArgs),
    /// Show a trip's full stop-time sequence.
    Trip(TripArgs),
    /// Look up the fare between two stations.
    Fare(FareArgs),
    /// Show BART's current service alerts.
    Alerts(AlertsArgs),
    /// Dump a raw GTFS table.
    Table(TableArgs),
}

/// Arguments for `bart-rs update`.
#[derive(Debug, Clone, Args)]
pub struct UpdateArgs {
    /// Only report whether a newer feed is available.
    #[arg(long)]
    pub check: bool,

    /// Download even if the remote version matches the installed one.
    #[arg(long)]
    pub force: bool,
}

/// Arguments for `bart-rs stops`.
#[derive(Debug, Clone, Args)]
pub struct StopsArgs {
    /// Case-insensitive substring to search for.
    pub query: Option<String>,

    /// Include platforms and entrances as well as stations.
    #[arg(long)]
    pub all: bool,

    /// Maximum number of results.
    #[arg(long, default_value_t = 50)]
    pub limit: usize,
}

/// Arguments for `bart-rs route`.
#[derive(Debug, Clone, Args)]
pub struct RouteArgs {
    /// The route id (`1`, `BB-A`) or short name (`Yellow-S`).
    pub route: String,
}

/// Arguments for `bart-rs stop`.
#[derive(Debug, Clone, Args)]
pub struct StopArgs {
    /// The station id (`LAKE`), a platform id (`A10-1`), or a name.
    pub stop: String,
}

/// Arguments for `bart-rs arrivals` and `bart-rs departures`.
#[derive(Debug, Clone, Args)]
pub struct ScheduleArgs {
    /// The station id, a platform id, or a name.
    pub stop: String,

    /// Show entries at or after this time, as `HH:MM`. Defaults to now.
    #[arg(long, value_name = "HH:MM")]
    pub at: Option<String>,

    /// The service date, as `YYYY-MM-DD`. Defaults to the current service day.
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub date: Option<String>,

    /// Maximum number of rows.
    #[arg(long, default_value_t = 10)]
    pub limit: usize,

    /// Overlay live trip updates from BART's realtime feed.
    #[arg(long)]
    pub realtime: bool,
}

/// Arguments for `bart-rs trip`.
#[derive(Debug, Clone, Args)]
pub struct TripArgs {
    /// The trip id.
    pub trip_id: String,
}

/// Arguments for `bart-rs fare`.
#[derive(Debug, Clone, Args)]
pub struct FareArgs {
    /// The origin station id, platform id, or name.
    #[arg(long)]
    pub from: String,

    /// The destination station id, platform id, or name.
    #[arg(long)]
    pub to: String,

    /// Only show the price for this rider category id.
    #[arg(long)]
    pub category: Option<String>,
}

/// Arguments for `bart-rs alerts`.
#[derive(Debug, Clone, Args)]
pub struct AlertsArgs {
    /// Only show alerts mentioning this route id.
    #[arg(long)]
    pub route: Option<String>,

    /// Only show alerts mentioning this stop id.
    #[arg(long)]
    pub stop: Option<String>,
}

/// Arguments for `bart-rs table`.
#[derive(Debug, Clone, Args)]
pub struct TableArgs {
    /// The table name, e.g. `stops` or `stop_times`.
    pub name: String,

    /// Maximum number of rows.
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}

impl Command {
    /// Whether this command can produce JSON output.
    pub fn supports_json(&self) -> bool {
        !matches!(self, Command::Update(_) | Command::Status)
    }

    /// A short name for the command, used in error messages.
    pub fn name(&self) -> &'static str {
        match self {
            Command::Update(_) => "update",
            Command::Status => "status",
            Command::Routes => "routes",
            Command::Stops(_) => "stops",
            Command::Route(_) => "route",
            Command::Stop(_) => "stop",
            Command::Arrivals(_) => "arrivals",
            Command::Departures(_) => "departures",
            Command::Trip(_) => "trip",
            Command::Fare(_) => "fare",
            Command::Alerts(_) => "alerts",
            Command::Table(_) => "table",
        }
    }
}

/// Parse the command line and run the requested command.
pub fn run() -> crate::Result<()> {
    let cli = Cli::parse();
    crate::commands::dispatch(cli)
}
