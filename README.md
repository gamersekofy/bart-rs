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

To run it straight from the repository, without building it first:

```console
$ nix run . -- --help
```

Without Nix you need a Rust toolchain (1.88 or newer) and can just use
`cargo build --release`.

### Trying it without cloning

Once the repository is public, anyone with Nix can run it directly from
GitHub, with no clone and no install:

```console
$ nix run github:gamersekofy/bart-rs -- --help
```

`nix` fetches the flake, builds the binary in the store and runs it. Add
`--` before the arguments to pass them to `bart-rs` rather than to `nix`.

> This uses the repository's **default branch**, so it runs whatever is
> merged into it — not an unmerged feature branch.

## Getting started

The tool downloads the schedule feed itself; the repository does not carry a
copy of it.

```console
$ bart-rs update
Downloading https://www.bart.gov/dev/schedules/google_transit.zip …
  resolved to https://www.bart.gov/sites/default/files/2026-08/google_transit_20260810-20270108_v02.zip
  downloaded 892312 bytes
Feed version 72
  covers 2026-08-10 → 2027-01-10
  stored /home/you/.local/share/bart-rs/archives/72.zip
  extracted 18 tables
Installed to /home/you/.local/share/bart-rs/feed/current
Done.
```

`update` first asks the server which release it is offering, and only
downloads when that is not the release already installed. Running it when
the feed is current prints one line and stops.

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

The command line tool is a thin adapter over the library. Anything it can do,
the library can do — and quite a lot that it does not.

```toml
[dependencies]
bart-rs = { git = "https://github.com/gamersekefy/bart-rs" }
```

```rust
use bart_rs::Feed;

let feed = Feed::from_data_dir()?;

// Which lines serve Lake Merritt?
for route in feed.routes_serving("LAKE") {
    println!("  {}", route.display_name());
}

// What is the fare between two places?
if let Some(fare) = feed.fare("Lake Merritt", "San Francisco Airport") {
    println!("  {}", fare.base_price_display());
}
# Ok::<(), bart_rs::Error>(())
```

The library is feature-flagged so that a consumer only pays for what it uses:

- `cli` (default) — the command line front end, and its `clap` and
  `colored` dependencies.
- `download` (default) — the HTTP and archive stack used by `update` and by
  the realtime fetchers.

`default-features = false` gives you the parsing, the service-day arithmetic
and the GTFS-Realtime decoding with no `clap`, no `reqwest` and no `zip`. A
GUI wants `download` but not `cli`:

```toml
[dependencies]
bart-rs = { git = "https://github.com/gamersekefy/bart-rs", default-features = false, features = ["download"] }
```

### The realtime feed is in the library too

GTFS-Realtime is decoded by the `realtime` module and merged onto the
schedule, so a GUI can show live times without any extra work:

```rust
use bart_rs::realtime;

let client = bart_rs::update::client()?;
let live = realtime::fetch_trip_updates(&client)?;
let arrivals = realtime::overlay(&feed, &scheduled, &live, false);

for arrival in arrivals.iter().take(5) {
    println!("{} {}", arrival.time, arrival.delay);
}
```

`update::ensure_current()` will fetch a new schedule feed if there is one,
and reports being offline as a value rather than as an error, so an
application keeps working with the feed it already has.

### Building a GUI on this

Two things to know before you start:

- **Everything blocks.** Parsing the feed takes about 65 ms and a realtime
  fetch about 500 ms, so do both on a worker thread.
- **Four upstream quirks will bite you** — the coverage window is not in
  `feed_info.txt`, the service day starts at 04:00, a platform's id is not
  its number, and a zero delay does not always mean "on time". The library
  handles all four, and [`docs/08-library-guide.md`](docs/08-library-guide.md)
  explains how to use each of them correctly.

That guide is the place to start: it has a complete worked example, the
threading notes, and the gotchas.

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

Two more quirks in the static feed are worth knowing about, because both
would otherwise make the tool lie to you:

- `feed_info.txt`'s `feed_start_date` and `feed_end_date` describe the
  *previous* release, so the coverage window is taken from `calendar.txt`
  and `calendar_dates.txt` instead.
- `feed_info.txt`'s `feed_version` is the same in every release, so the
  release is identified by the versioned filename the permalink redirects
  to.

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
