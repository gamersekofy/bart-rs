# 01 — Architecture

## Crate shape

`bart-rs` is a single package containing **both** a library and a binary.

```
package "bart-rs"
├── lib target  -> src/lib.rs      crate name: bart_rs
└── bin target  -> src/main.rs     binary name: bart-rs
```

Cargo discovers both automatically from the conventional paths; no `[lib]` or
`[[bin]]` sections are required. The binary is a five-line adapter:

```rust
fn main() -> bart_rs::Result<()> {
    bart_rs::cli::run()
}
```

Everything of substance lives in the library. This is a deliberate choice, not
an accident of layout:

- Integration tests can exercise the real API instead of spawning a process and
  scraping stdout.
- The heavy lifting (parsing, indexing, service-day math, realtime decoding) is
  reusable by other Rust projects — a TUI, a web service, a Discord bot.
- The CLI is a thin adapter over a clean API, so it stays honest.

### Why not a separate crate?

A `bart-rs-core` + `bart-rs-cli` split was considered and rejected: it doubles
the ceremony (two packages, two `Cargo.toml`s, a path dependency) for a project
that is one person learning Rust. A single package with a `lib` and a `bin`
gives the same separation with none of the overhead. If a second consumer ever
needs a *different* dependency profile, the `cli` and `download` feature flags
below already allow forking the surface without forking the package.

## Feature flags

```toml
[features]
default = ["cli", "download"]
cli      = ["dep:clap", "dep:colored"]
download = ["dep:reqwest", "dep:zip"]
```

- **`cli`** — the `cli` and `commands` modules, i.e. everything argument-parsing
  and colour-related. A library consumer that only wants to parse a feed does
  not pull in `clap`.
- **`download`** — `update.rs`: the HTTP client and the zip extractor. A
  consumer that already has a feed on disk does not pull in `reqwest`, `zip`,
  or the `tokio` runtime that `reqwest`'s blocking API sits on.

The core (`attributes`, `feed`, `schedule`, `gtfsrt`, `realtime`, `paths`,
`helper`, `error`) depends on neither. `realtime` needs only `reqwest` for the
fetch — see the note in `03-data-sources.md`; if that turns out to be awkward
we can split the decode half from the fetch half and gate only the latter.

## Edition and minimum supported Rust

| | |
| --- | --- |
| Edition | 2024 |
| Resolver | 3 (implied by edition 2024) |
| `rust-version` | 1.88 |
| Development toolchain | rustc 1.99.1, from the Nix flake |

The MSRV is **1.88**, not the 1.85 floor of edition 2024. The dependency
tree imposes it: `time`, `time-core` and `zopfli` require 1.88.0, and the
`icu_*` crates behind `url` → `idna` → `icu_normalizer` require 1.88.
Declaring 1.85 would have been a promise the crate could not keep.

Edition 2024 also means **resolver 3**, which honours `rust-version` when
choosing dependency versions. Resolver 2, the edition 2021 default, ignores
it — which is exactly how a crate claiming to support 1.85 ended up with a
lockfile that needed 1.88. See `06-decisions.md` D25.

Two `unsafe` blocks exist, both for `std::env::set_var`, which became
`unsafe` in edition 2024 because setting an environment variable is not
thread safe on Unix. Both carry a SAFETY comment explaining why no other
thread can be touching the environment at that point. There is no other
`unsafe` in the crate.

## Module layout

```
src/
├── lib.rs                    crate root; module declarations + re-exports
├── main.rs                   binary entry point
│
├── error.rs                  `Error` enum (thiserror) + `Result` alias
├── helper.rs                 shared parsing utilities (CSV reading, GTFS time/date)
├── paths.rs                  on-disk cache locations
│
├── attributes/               ONE MODULE PER GTFS TABLE
│   ├── mod.rs
│   ├── agency.rs             agency.txt
│   ├── calendar.rs           calendar.txt
│   ├── calendar_attributes.rs  calendar_attributes.txt
│   ├── calendar_dates.rs     calendar_dates.txt
│   ├── directions.rs         directions.txt
│   ├── fare_attributes.rs    fare_attributes.txt
│   ├── fare_rider_categories.rs  fare_rider_categories.txt
│   ├── fare_rules.rs         fare_rules.txt
│   ├── feed_info.rs          feed_info.txt
│   ├── realtime_routes.rs    realtime_routes.txt
│   ├── rider_categories.rs   rider_categories.txt
│   ├── route_attributes.rs   route_attributes.txt
│   ├── routes.rs             routes.txt
│   ├── shapes.rs             shapes.txt
│   ├── stops.rs              stops.txt
│   ├── stop_times.rs         stop_times.txt
│   ├── transfers.rs          transfers.txt
│   └── trips.rs              trips.txt
│
├── feed.rs                   `Feed`: all 18 tables + indexes + query API
├── schedule.rs               service-day resolution (calendar + calendar_dates)
├── update.rs                 download, extract, atomic swap, staleness   [download]
├── gtfsrt.rs                 GTFS-Realtime protobuf message definitions
├── realtime.rs               fetching + BART-tuned decoding of realtime feeds
│
├── cli.rs                    clap command tree                          [cli]
└── commands/                 one module per CLI command                 [cli]
    ├── mod.rs
    ├── update.rs   status.rs   routes.rs   stops.rs   route.rs   stop.rs
    ├── arrivals.rs departures.rs   trip.rs   fare.rs   alerts.rs   table.rs
```

The `attributes/` module tree is inherited from the original code and is kept
deliberately: one file per GTFS table makes it obvious which tables are
handled, and it keeps each file small enough to read in one sitting.

## Data flow

```
                       ┌──────────────────────────────┐
   bart.gov ──────────▶│ update::download_latest()    │
   (zip)               │  reqwest blocking + rustls    │
                       └──────────────┬───────────────┘
                                      │ zip
                                      ▼
                       ┌──────────────────────────────┐
                       │ <data_dir>/archives/v72.zip   │
                       │ <data_dir>/feed/current/*.txt │  ← extracted, atomic
                       └──────────────┬───────────────┘
                                      │ csv + serde
                                      ▼
                       ┌──────────────────────────────┐
                       │ attributes::*::parse_*_file  │  one Vec<T> per table
                       └──────────────┬───────────────┘
                                      │
                                      ▼
                       ┌──────────────────────────────┐
                       │ Feed { stops, routes, trips,  │  owns the tables,
                       │       stop_times, ... }      │  builds the indexes
                       └──────────────┬───────────────┘
                                      │
             ┌────────────────────────┼────────────────────────┐
             ▼                        ▼                        ▼
      schedule.rs               feed.rs queries        commands/*.rs
      (which services run       (routes_serving,        (rendering,
       on a given date)          arrivals_at, fare)      coloured output)
```

`Feed` is constructed **once** and passed by reference. A `Feed` owns all 18
tables in memory (~30 MB parsed for the current BART feed — see
`02-data-model.md` for the per-table row counts). That is acceptable for a CLI;
it is not appropriate for a long-lived server, and `05-testing.md` records the
intended future change (a borrowed/lazy loader) if that ever matters.

## The `Feed` type

This is the centre of the library. It owns the parsed tables and the indexes
built over them.

```rust
pub struct Feed {
    // raw tables, one per GTFS file
    pub agency: Vec<Agency>,
    pub calendar: Vec<Calendar>,
    pub calendar_attributes: Vec<CalendarAttribute>,
    pub calendar_dates: Vec<CalendarDate>,
    pub directions: Vec<Direction>,
    pub fare_attributes: Vec<FareAttribute>,
    pub fare_rider_categories: Vec<FareRiderCategory>,
    pub fare_rules: Vec<FareRule>,
    pub feed_info: Vec<FeedInfo>,
    pub realtime_routes: Vec<RealtimeRoute>,
    pub rider_categories: Vec<RiderCategory>,
    pub route_attributes: Vec<RouteAttribute>,
    pub routes: Vec<Route>,
    pub shapes: Vec<Shape>,
    pub stops: Vec<Stop>,
    pub stop_times: Vec<StopTime>,
    pub transfers: Vec<Transfer>,
    pub trips: Vec<Trip>,

    // indexes (private)
    stops_by_id: HashMap<String, usize>,
    routes_by_id: HashMap<String, usize>,
    trips_by_id: HashMap<String, usize>,
    stop_times_by_trip: HashMap<String, Vec<usize>>,
    platforms_by_station: HashMap<String, Vec<usize>>,
    stop_times_by_stop: HashMap<String, Vec<usize>>,
    ...
}
```

Tables are `pub` so consumers can do their own joins. The indexes are private
and reached through methods.

### Public query API

```rust
impl Feed {
    // --- construction ---
    pub fn open(dir: &Path) -> Result<Feed>;          // parse a directory of .txt files
    pub fn from_data_dir() -> Result<Feed>;            // open the default cache dir

    // --- metadata ---
    pub fn feed_info(&self) -> Option<&FeedInfo>;
    pub fn publisher(&self) -> Option<&Agency>;

    // --- lookups ---
    pub fn route(&self, id: &str) -> Option<&Route>;
    pub fn trip(&self, id: &str) -> Option<&Trip>;
    pub fn stop(&self, id: &str) -> Option<&Stop>;

    // --- stop hierarchy (see 02-data-model.md) ---
    pub fn stations(&self) -> Vec<&Stop>;               // location_type == 1
    pub fn platforms(&self) -> Vec<&Stop>;              // location_type == 0
    pub fn entrances(&self) -> Vec<&Stop>;              // location_type == 2
    pub fn station_of(&self, stop_id: &str) -> Option<&Stop>;
    pub fn platforms_of(&self, station_id: &str) -> Vec<&Stop>;
    pub fn find_stops(&self, query: &str) -> Vec<&Stop>;   // case-insensitive substring

    // --- cross-file queries ---
    pub fn routes_serving(&self, stop_id: &str) -> Vec<&Route>;
    pub fn stations_on_route(&self, route_id: &str) -> Vec<&Stop>;
    pub fn trips_on_route(&self, route_id: &str) -> Vec<&Trip>;
    pub fn stop_times_for_trip(&self, trip_id: &str) -> Vec<&StopTime>;
    pub fn direction_of(&self, route_id: &str, direction_id: u8) -> Option<&Direction>;
    pub fn fare(&self, from: &str, to: &str) -> Option<Fare>;

    // --- schedule ---
    pub fn service_ids_for(&self, date: NaiveDate) -> BTreeSet<String>;
    pub fn arrivals_at(&self, stop_id: &str, date: NaiveDate, from_secs: u32) -> Vec<Arrival>;
    pub fn departures_from(&self, stop_id: &str, date: NaiveDate, from_secs: u32) -> Vec<Departure>;
}
```

The stop-taking methods accept **either** a station id (`LAKE`) or a platform id
(`A10-1`) and normalise to the platform set internally, so the user can type
whichever they have.

## Error model

A single error enum, no `Box<dyn Error>` anywhere:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to read GTFS file {path}")]
    Io { path: PathBuf, source: std::io::Error },

    #[error("failed to parse {path}: {source}")]
    Csv { path: PathBuf, source: csv::Error },

    #[error("no GTFS feed found at {0} (run `bart-rs update`)")]
    FeedNotFound(PathBuf),

    #[error("the feed at {path} is missing {table}")]
    MissingTable {
        path: PathBuf,
        table: String,
    },

    #[error("network error while fetching {url}")]
    Http { url: String, source: reqwest::Error },

    #[error("download of {url} failed with status {status}")]
    HttpStatus { url: String, status: u16 },

    #[error("the downloaded archive is not a valid GTFS zip: {0}")]
    BadArchive(String),

    #[error("failed to decode GTFS-Realtime payload: {0}")]
    RealtimeDecode(String),

    #[error("{0}")]
    Message(String),      // user-facing "no such stop" style errors
}
```

`Error` implements `std::error::Error`; `Result<T> = std::result::Result<T, Error>`
is re-exported at the crate root. The binary prints the `Display` form to
stderr and exits `1`. `Error::Message` exists so command code can produce a
clean message like `no station matching "foo"` without inventing a new variant
for every user mistake.

## Output style

Human-facing output uses `colored` (already a dependency in the original code)
for emphasis: route colours from `routes.txt` are rendered as a coloured
swatch, and column headers are dimmed. The `table` command prints aligned
columns with `text`/manual padding — no extra table crate.

Where a command has structured output, `--json` emits `serde_json` output of
the same underlying data structures. This is a "should" for the query commands
(`routes`, `stops`, `arrivals`, `fare`, `alerts`) and is explicitly *not* a goal
for `status` or `update`.

## Concurrency

None. Everything is single-threaded and synchronous. `reqwest`'s blocking
client is used so that `update` does not need an async runtime or `#[tokio::main]`.
The feed is parsed once and queried many times in memory.

## Threading and mutability

`Feed` is immutable after construction. All query methods take `&self`. The
indexes are built in `Feed::open` and never mutated, so `Feed` is `Send + Sync`
and can be shared across threads by a consumer.
