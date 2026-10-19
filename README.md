# bart-rs

A command line tool — and a library — for exploring **BART** (Bay Area Rapid
Transit) schedule and realtime data.

`bart-rs` downloads BART's GTFS schedule feed, parses all eighteen of its
tables, and answers questions about the network: when the next train is due,
which lines serve a station, what a trip between two stations costs, and what
service alerts BART is publishing right now.

It is also usable as a Rust library, so other projects can reuse the parsing,
the service-day arithmetic, and the GTFS-Realtime decoding.

## Features

- [x] Download the latest GTFS data from BART
- [x] Automatically extract the GTFS data
- [x] Remind the user to update the GTFS data
- [x] Handle automatic GTFS updates
- [x] Read and parse all 18 GTFS tables in a meaningful way
- [x] Show scheduled arrivals and departures for any station
- [x] Overlay live trip updates from BART's GTFS-Realtime feed
- [x] Show BART's current service alerts
- [x] Look up fares, including the per-rider-category prices
- [x] Work as a library, with a documented public API
- [ ] A journey planner that finds connections between two stations
- [ ] A map view of route shapes

## Installing

The project is a Nix flake. With [Nix](https://nixos.org) and flakes enabled:

```console
$ nix develop
$ cargo run -- --help
```

`nix develop` drops you into a shell with the right Rust toolchain, `clippy`,
`rustfmt`, `cargo-watch`, `rust-analyzer` and `cargo-deny` already on the
path. Nothing is installed outside the Nix store.

If you use [direnv](https://direnv.net), the included `.envrc` means the
environment loads automatically when you `cd` into the project:

```console
$ direnv allow
$ cargo run -- --help
```

To build a release binary:

```console
$ nix build          # ./result/bin/bart-rs
```

Without Nix you need a Rust toolchain (1.85 or newer) and can just use
`cargo build --release`.

## Getting started

The tool downloads the schedule feed itself; the repository does not carry a
copy of it.

```console
$ bart-rs update
Downloading https://www.bart.gov/dev/schedules/google_transit.zip …
  resolved to https://www.bart.gov/sites/default/files/2026-08/google_transit_20260810-20270108_v02.zip
  downloaded 892312 bytes
Feed version 72
  covers 2026-01-12 → 2026-08-30
  stored /home/you/.local/share/bart-rs/archives/72.zip
  extracted 18 tables
Installed to /home/you/.local/share/bart-rs/feed/current
Done.
```

The feed lives in `$XDG_DATA_HOME/bart-rs` (usually
`~/.local/share/bart-rs`). Set `BART_RS_DATA_DIR` to put it somewhere else.

## Commands

| Command | What it does |
| --- | --- |
| `bart-rs update [--check] [--force]` | Download and install the latest feed |
| `bart-rs status` | The installed feed's version, coverage and staleness |
| `bart-rs routes` | List BART's routes |
| `bart-rs stops [QUERY] [--all]` | List or search stations |
| `bart-rs route <ROUTE>` | A route's details and the stations it serves |
| `bart-rs stop <STOP>` | A station's platforms, entrances and lines |
| `bart-rs arrivals <STOP>` | Scheduled arrivals, optionally with live delays |
| `bart-rs departures <STOP>` | Scheduled departures |
| `bart-rs trip <TRIP_ID>` | A trip's full stop-time sequence |
| `bart-rs fare --from A --to B` | The fare between two stations |
| `bart-rs alerts` | BART's current service alerts |
| `bart-rs table <NAME>` | Dump one of the raw GTFS tables |

Every command takes `--help`. Add `--json` to any of the query commands to
get machine-readable output.

### Examples

```console
$ bart-rs arrivals EMBR --at 17:30
TIME   ROUTE     DESTINATION                          PLATFORM
17:31  Yellow-S  San Francisco International Airport  M16-1
17:31  Yellow-N  SFO / SF / Pittsburg/Bay Point       M16-2
17:36  Blue-S    OAK Airport / SF / Daly City         M16-1
17:37  Green-N   SF / OAK Airport / Berryessa         M16-2
17:40  Yellow-S  San Francisco International Airport  M16-1
```

```console
$ bart-rs arrivals EMBR --realtime
TIME   ROUTE     DESTINATION                          PLATFORM  STATUS
12:45  Blue-N    SF / OAK Airport / Dublin/Pleasanton  M16-2     on time
12:45  Green-S   OAK Airport / SF / Daly City          M16-1     +1 min late
12:47  Red-S     SF / SFO Airport / Millbrae           M16-1     on time
```

```console
$ bart-rs fare --from LAKE --to SFIA
Lake Merritt → San Francisco International Airport

  Regular fare   $12.20
RIDER CATEGORY           ID  PRICE
Youth Clipper            5   $6.10
Senior/Disabled Clipper  2   $4.55
Clipper START            16   $6.10
```

```console
$ bart-rs alerts
BART.gov Alert
  Expect 30-minute delays starting at 9pm each night on Sunday-Thursday for
  riders traveling between Milbrae, SFO, and Daly City stations due to
  planned train control modernization construction work.

  more info: http://www.bart.gov/schedules/advisories
```

Stops can be named in whichever way you have them: a station id (`LAKE`), a
platform id (`A10-1`), or a name (`Lake Merritt`).

## Using it as a library

```toml
[dependencies]
bart-rs = { git = "https://github.com/gamersekofy/bart-rs" }
```

```rust
use bart_rs::Feed;

let feed = Feed::from_data_dir()?;

for route in &feed.routes {
    println!("{} {}", route.id, route.display_name());
}

// Which lines serve Lake Merritt?
for route in feed.routes_serving("LAKE") {
    println!("  {}", route.display_name());
}
# Ok::<(), bart_rs::Error>(())
```

The library is feature-flagged so that a consumer only pays for what it uses:

- `cli` (default) — the command line front end, and its `clap` and
  `colored` dependencies.
- `download` (default) — the HTTP and archive stack used by `update` and by
  the realtime fetchers.

`default-features = false` gives you the parsing, the service-day arithmetic
and the GTFS-Realtime decoding with no `clap`, no `reqwest` and no `zip`.

## Data sources

| Feed | URL | Notes |
| --- | --- | --- |
| GTFS schedule | `https://www.bart.gov/dev/schedules/google_transit.zip` | A permalink that always serves the current release. No API key. |
| GTFS-Realtime trip updates | `https://api.bart.gov/gtfsrt/tripupdate.aspx` | No API key. Served as protobuf with a mislabelled `Content-Type`. |
| GTFS-Realtime alerts | `https://api.bart.gov/gtfsrt/alerts.aspx` | No API key. Same mislabelled content type. |

BART's alert feed does not follow the GTFS-Realtime specification's field
numbering, and is not even self-consistent about the shape of its text
fields. `bart-rs` handles this with a tolerant decoder; the details are in
[`docs/03-data-sources.md`](docs/03-data-sources.md).

## Documentation

[`docs/`](docs/README.md) holds the full design record: the architecture, the
data model, the CLI contract, the build plan, the testing strategy, and the
architecture decision log.

## Development

```console
$ nix develop
$ cargo test                # offline, against the committed fixtures
$ cargo test -- --ignored   # the live feed; needs network
$ cargo clippy --all-targets --all-features -- -D warnings
$ cargo fmt --check
```

## License

MIT.
