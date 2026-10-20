# 08 — Using `bart-rs` as a library

This document is for someone who wants to build something on top of `bart-rs`
rather than use the command line: a GUI, a web service, a Discord bot, a
departure board on a wall.

The command line tool is a thin adapter over the library. Anything the CLI
can do, the library can do, and a fair amount that the CLI does not do is
available too.

## Adding it to a project

```toml
[dependencies]
bart-rs = { git = "https://github.com/gamersekefy/bart-rs", path = "crates/bart-rs" }
```

or, if you are working inside a checkout:

```toml
[dependencies]
bart-rs = { path = "../bart-rs" }
```

The crate's library target is named `bart_rs`; the binary is `bart-rs`. In
Rust code you refer to the library as `bart_rs`.

### Feature flags

```toml
[features]
default = ["cli", "download"]
cli      = ["dep:clap", "dep:colored"]
download = ["dep:reqwest", "dep:zip"]
```

A GUI wants the `download` feature, because it needs to fetch the feed and
the realtime data. It does **not** want `cli`: that pulls in `clap` and
`colored`, which are only there to serve the command line.

```toml
[dependencies]
bart-rs = { path = "../bart-rs", default-features = false, features = ["download"] }
```

With `default-features = false` you get `attributes`, `feed`, `schedule`,
`realtime`, `gtfsrt`, `paths`, `helper` and `error`, and nothing that only
the command line needs. The `realtime` module's *decoding* half is in the
core; only its *fetching* half needs `reqwest`, and that is behind the
`download` feature.

### One wrinkle: `reqwest`

`update::client()` returns a `reqwest::blocking::Client`, and the realtime
fetch functions take one. That type is part of the public API, so if you want
to hold a client in your own struct — or even just to name the type in a
function signature — you need `reqwest` as a direct dependency of your own
crate, with the same TLS backend:

```toml
[dependencies]
bart-rs = { path = "../bart-rs", default-features = false, features = ["download"] }
reqwest = { version = "0.12", default-features = false, features = ["blocking", "rustls-tls"] }
```

Cargo will unify the two, so you get one copy of `reqwest` and one TLS
backend. Use `rustls-tls`, not the default `native-tls`, unless you have a
reason not to: `native-tls` pulls OpenSSL in, and this crate is deliberately
built without it.

### Minimum supported Rust

`bart-rs` needs **Rust 1.88 or newer**. That is the floor the dependency
tree imposes — `time`, `zopfli` and the `icu_*` crates behind `idna` all
require it — and not the 1.85 floor of edition 2024, which the crate also
uses.

Because the crate uses **resolver 3**, Cargo honours that floor when it
resolves dependencies for *your* project too: `cargo update` will prefer
dependency versions that still build on 1.88.

## The shape of the API

The library has three layers, and you will use the top one for almost
everything.

### Layer 1 — the tables

`bart_rs::attributes` has one module per GTFS table, each with a typed
struct and a `parse_*_file` function:

```rust
use bart_rs::attributes::routes::Route;

let routes = Route::parse_routes_file(std::path::Path::new("routes.txt"))?;
for route in &routes {
    println!("{} {}", route.id, route.long_name.as_deref().unwrap_or(""));
}
```

You will rarely need this. It is there for when you want a table on its own,
without the rest of the feed.

### Layer 2 — the feed

`bart_rs::Feed` is a whole feed in memory, with the cross-file queries
built on top of it. This is the layer you want:

```rust
use bart_rs::Feed;

let feed = Feed::from_data_dir()?;          // the installed feed
let feed = Feed::open(std::path::Path::new("/some/dir"))?;  // any directory
```

`Feed` holds the 18 tables as `Vec`s and a set of indexes for the lookups
that would otherwise be O(n) scans. It is not `Sync`, and it is not cheap to
clone — treat it as a thing you own, hold onto, and query.

The queries are the interesting part:

| Method | What it answers |
| --- | --- |
| `find_stop(query)` | "balboa park" → the station, by name, id, or code |
| `find_route(query)` | "Red-S" → the route, by name, id, or code |
| `arrivals_at(stop, date, from)` | the trains arriving at a stop, from a time onwards |
| `departures_from(stop, date, from)` | the trains departing from a stop |
| `routes_serving(stop)` | the lines that serve a stop |
| `stations_on_route(route)` | the stations a route serves, in order |
| `fare(from, to)` | the fare between two places |
| `coverage()` | the window during which the feed's service actually runs |
| `service_ids_for(date)` | which services run on a date |

### Layer 3 — realtime

`bart_rs::realtime` decodes GTFS-Realtime and merges it onto the schedule:

```rust
use bart_rs::realtime;

let client = bart_rs::update::client()?;
let live = realtime::fetch_trip_updates(&client)?;
let overlaid = realtime::overlay(&feed, &scheduled, &live, departing);
```

`overlay` takes the scheduled arrivals and adjusts the ones the realtime
feed knows about. The result is a `Vec<LiveArrival>`.

Service alerts are a separate feed:

```rust
let alerts = realtime::fetch_alerts(&client)?;
for alert in &alerts {
    if alert.is_empty() {
        continue;
    }
    println!("{}", alert.title.as_deref().unwrap_or("(no title)"));
    if let Some(description) = &alert.description {
        println!("{description}");
    }
}
```

`ServiceAlert` carries `title`, `description`, `url`, `cause`, `effect`, and
the `affected_routes` and `affected_stops` the alert mentions. BART's alert
feed does not follow the GTFS-Realtime field numbering, so `description` is
the *raw text* of the alert rather than a `TranslatedString` — see
`03-data-sources.md`.

## A complete worked example

Here is a program that downloads the feed if it needs updating, then prints
the next trains at a station. It is the whole life cycle, and it is about
twenty lines.

```rust
use bart_rs::realtime;
use bart_rs::schedule;
use bart_rs::update::{self, UpdateOutcome};

fn main() -> anyhow::Result<()> {
    // 1. Make sure the feed on disk is the current one. This is a HEAD
    //    request when the feed is already current, so it costs nothing.
    let client = update::client()?;
    match update::ensure_current(&client)? {
        UpdateOutcome::UpToDate { .. } => {}
        UpdateOutcome::Updated { bytes, .. } => {
            eprintln!("downloaded a new feed ({bytes} bytes)");
        }
        // Being offline is not a failure: the installed feed is still
        // perfectly good.
        UpdateOutcome::Unreachable { reason } => {
            eprintln!("could not reach BART: {reason}");
        }
    }

    // 2. Load it. Parsing takes about 65 ms for the whole feed.
    let feed = bart_rs::Feed::from_data_dir()?;

    // 3. Find the station the user asked for.
    let stop = feed
        .find_stop("balboa park")
        .ok_or_else(|| anyhow::anyhow!("no such station"))?;

    // 4. Work out which service day we are on. BART's service day runs from
    //    04:00 to 04:00 the next morning, so "now" is not always today's
    //    date.
    let (date, from) = schedule::now_service_day();

    // 5. Get the trains.
    let scheduled = feed.arrivals_at(&stop.id, date, from);

    // 6. Optionally, overlay the live data.
    let live = realtime::fetch_trip_updates(&client)?;
    let arrivals = realtime::overlay(&feed, &scheduled, &live, false);

    for arrival in arrivals.iter().take(10) {
        println!(
            "{} {} {}",
            schedule::fmt_human_time(arrival.time),
            arrival.route_short_name.as_deref().unwrap_or(""),
            arrival.headsign.as_deref().unwrap_or(""),
        );
    }

    Ok(())
}
```

## Things that will trip you up

These are the four behaviours that are not obvious from the GTFS
specification, and that have each cost a bug in this project. They are all
in `03-data-sources.md` in more detail.

### 1. The coverage window is not in `feed_info.txt`

`feed_info.txt` carries `feed_start_date` and `feed_end_date`, and BART
leaves them describing the **previous** release. A feed whose calendar runs
from 2026-08-10 to 2027-01-10 still says its feed runs until 2026-08-30.

Use `Feed::coverage()`, which reads the calendar instead:

```rust
if let Some((start, end)) = feed.coverage() {
    println!("the schedule runs from {start} to {end}");
}
```

### 2. The service day is not the calendar day

BART's service day starts at 04:00. A train that leaves at 02:30 belongs to
the *previous* day. `schedule::now_service_day()` returns the right
`(date, seconds_since_midnight)` pair; use it rather than computing it
yourself.

```rust
let (date, from) = schedule::now_service_day();
```

`schedule::service_ids_for(&feed, date)` gives you the set of service
patterns that run on that date, honouring both `calendar.txt` and
`calendar_dates.txt`.

### 3. Platform ids are not platform numbers

BART's platform ids are built from a legacy station code and the platform
number: `M80-1` is platform 1 of Balboa Park, `W40-3` is platform 3 of
Millbrae. The number a passenger sees on the sign is the `platform_code`
column of `stops.txt`.

Both are available, and `Arrival::platform_label()` picks the right one:

```rust
arrival.platform_id;    // Some("M80-1") — the GTFS stop id
arrival.platform_code;  // Some("1")    — what the sign says
arrival.platform_label(); // Some("1")  — what to show
```

### 4. "On time" and "no information" are different

The realtime feed only covers the trains it is currently tracking. A train
two hours out has no live update at all. Its delay is zero, but zero does
not mean punctual — it means nobody has said.

`LiveArrival::has_update` tells the two apart:

```rust
if arrival.has_update {
    // The realtime feed had something to say about this train.
    if arrival.delay > 0 { /* late */ } else { /* on time */ }
} else {
    // Nothing is known. Do not claim it is on time.
}
```

The CLI shows `scheduled` in that case, and `on time` only when the feed
actually said so.

## Building a GUI: the blocking problem

**The library is entirely blocking.** `Feed::from_data_dir()` parses 57,000
rows of CSV synchronously, and `fetch_trip_updates()` does a network request
synchronously. On a GUI thread this will freeze the window.

The numbers, measured against the live feed on 2026-10-19:

| Operation | Time |
| --- | --- |
| `Feed::from_data_dir()` | ~65 ms |
| `realtime::fetch_trip_updates()` | ~500 ms |
| `realtime::fetch_alerts()` | ~80 ms |
| `update::ensure_current()` | ~0 ms if current, ~1 s if downloading |

65 ms is survivable on a UI thread but not pleasant; 500 ms is not. Do this:

- **Load the feed once at startup, on a background thread**, and keep it.
  It does not change unless the user updates it.
- **Poll the realtime feed on a timer, on a background thread.** 15 to 30
  seconds is a sensible interval; the feed itself refreshes every minute or
  so.
- **Never block the UI thread on the network.** A failure to reach BART
  should show the last known data with a "stale" marker, not an error
  dialog.

In a GTK or Qt application, that means a worker thread with a channel back
to the main loop. In a TUI, it means a background thread and a redraw
trigger. In a web service, it means a request handler that is allowed to
block — or a cache in front of it.

Because `Feed` is not `Sync`, the usual shape is to own the `Feed` inside
the worker thread and send *derived* data (the `Vec<Arrival>`, or a
serialized form of it) back to the UI. `Arrival`, `LiveArrival`, `Fare` and
`Route` are all `Clone` and `Send`, so that works well.

## Serialization

The public data types derive `serde::Serialize`, so you can hand them
straight to a JSON or MessagePack serializer:

```rust
#[derive(serde::Serialize)]
struct Board {
    stop: String,
    arrivals: Vec<bart_rs::feed::Arrival>,
}

let board = Board {
    stop: stop.name.clone().unwrap_or_default(),
    arrivals: feed.arrivals_at(&stop.id, date, from),
};

let json = serde_json::to_string(&board)?;
```

If you are serving this over HTTP, this is the whole story. Note that
`Arrival::time` is **seconds since midnight**, not a timestamp — the service
day may not be today, so there is no date to attach it to without also
carrying the service date.

`Feed` itself is not `Serialize`: it holds indexes, and those are an
implementation detail. Use the query methods to get data out of it.

## Where to look next

| Document | What it answers |
| --- | --- |
| [01-architecture.md](01-architecture.md) | how the crate is laid out, and what the feature flags do |
| [02-data-model.md](02-data-model.md) | every table, every column, every BART-specific quirk |
| [03-data-sources.md](03-data-sources.md) | the URLs, the cache layout, the update lifecycle |
| [07-decisions.md](07-decisions.md) | why it is built this way, including the alternatives that were rejected |

And the API documentation itself, which is built from the source and checked
in CI with `missing_docs` warnings denied:

```console
$ cargo doc --all-features --open
```
