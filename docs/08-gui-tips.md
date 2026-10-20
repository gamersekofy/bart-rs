# 08 — Building a GUI on top of `bart-rs`

`bart-rs` is a library first and a command line tool second. This document is
for the case where you want the first: a desktop application, a kiosk, a
departure board on a wall, a web service that renders arrivals.

It covers the one thing that will otherwise cost you an afternoon of
debugging — **the library is entirely blocking**, and the interesting numbers
are big enough to freeze a window.

## The problem

Two operations in this crate take long enough to be noticeable:

| Operation | Time |
| --- | --- |
| `Feed::from_data_dir()` | ~65 ms |
| `realtime::fetch_trip_updates()` | ~500 ms |
| `realtime::fetch_alerts()` | ~80 ms |
| `update::ensure_current()` | ~0 ms if current, ~1 s if downloading |

Measured against the live feed on 2026-10-19.

65 ms is survivable on a UI thread, if you do it once. 500 ms is not
survivable at all, and it happens on every poll. Neither is a network round
trip that fails and has to be waited out.

There is no async API, and adding one would have meant making every public
type in the crate `async` for the benefit of one caller. The design is
instead: **the library blocks, and you put a thread between it and your
event loop.**

## The shape of the solution

```
  UI thread                     worker thread
  ──────────                    ─────────────
  tick every 20s  ──────────▶   fetch the realtime feed
                                overlay it on the schedule
  redraw          ◀──────────   send back a Vec<LiveArrival>
```

The worker owns the `Feed` and the `reqwest` client. The UI thread owns
nothing but the last thing the worker sent it.

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use bart_rs::{realtime, schedule, update};

enum Tick {
    Arrivals(Vec<bart_rs::feed::LiveArrival>),
    Alert(String),
    Failed(String),
}

fn main() -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel::<Tick>();

    // The worker owns everything expensive.
    thread::spawn(move || -> anyhow::Result<()> {
        let client = update::client()?;
        let feed = bart_rs::Feed::from_data_dir()?;

        loop {
            let (date, from) = schedule::now_service_day();

            let live = match realtime::fetch_trip_updates(&client) {
                Ok(live) => live,
                // Being offline is not a crash. Show the last known data
                // with a "stale" marker instead of an error dialog.
                Err(error) => {
                    tx.send(Tick::Failed(error.to_string())).ok();
                    thread::sleep(Duration::from_secs(20));
                    continue;
                }
            };

            let scheduled = feed.arrivals_at("EMBR", date, from);
            let arrivals = realtime::overlay(&feed, &scheduled, &live, false);
            tx.send(Tick::Arrivals(arrivals)).ok();

            thread::sleep(Duration::from_secs(20));
        }
    });

    // The UI thread only ever receives.
    while let Ok(tick) = rx.recv() {
        match tick {
            Tick::Arrivals(arrivals) => {
                for arrival in arrivals.iter().take(10) {
                    println!("{}", schedule::fmt_human_time(arrival.time));
                }
            }
            Tick::Alert(text) => println!("{text}"),
            Tick::Failed(reason) => println!("showing stale data: {reason}"),
        }
    }

    Ok(())
}
```

Swap the `println!`s for your toolkit's redraw call and the shape is the
same.

## What to do on which thread

- **Load the feed once, on a background thread, at startup.** The schedule
  only changes when the feed is replaced, so there is no reason to re-parse
  it on a timer. Keep the `Feed` alive; do not re-open it per poll.
- **Poll the realtime feed on a timer, on a background thread.** 15 to 30
  seconds is sensible. BART's realtime feed refreshes about once a minute,
  so polling faster buys nothing.
- **Never block the UI thread on the network.** A failure to reach BART
  should show the last known data with a "stale" marker, not an error
  dialog. `update::ensure_current` returns `UpdateOutcome::Unreachable` for
  exactly this case, rather than failing.
- **Do the download on the background thread too.** `update::ensure_current`
  can take a second or more when it actually downloads something, and that
  is a very visible freeze.

## Sending data across the thread

`Feed` is not `Sync` and is not cheap to clone. The usual arrangement is to
own the `Feed` inside the worker and send *derived* data back:

| Type | `Clone`? | `Send`? |
| --- | --- | --- |
| `feed::Arrival` | yes | yes |
| `feed::LiveArrival` | yes | yes |
| `feed::Fare` | yes | yes |
| `attributes::routes::Route` | yes | yes |
| `Feed` | no | no |

So `Vec<LiveArrival>` crosses the thread boundary cleanly, and `Feed` never
does.

If your UI wants structured data rather than formatted text, the public
types derive `serde::Serialize`:

```rust
#[derive(serde::Serialize)]
struct Board {
    stop: String,
    arrivals: Vec<bart_rs::feed::Arrival>,
}

let json = serde_json::to_string(&board)?;
```

## Toolkit notes

**GTK (gtk-rs).** Use a `glib::MainContext` and a channel. Spawn the worker
thread with `glib::spawn_future_local` or a plain thread, and marshal results
onto the main context with `glib::idle_add` — GTK objects are not `Send`, so
the data has to cross as plain data, never as a widget.

**Qt (cxx-qt / qt-rs).** Run the worker on a `std::thread` and deliver
results with `QMetaObject::invokeMethod`, or use a Qt signal. The same rule
applies: send owned data, not widgets.

**TUI (ratatui).** The simplest case. A background thread plus a channel,
and the draw loop redraws on a timer or on receipt of a message.

**Web service.** A request handler is allowed to block, so you may not need
a worker at all — but a cache in front of the feed is still worth having, so
that a burst of requests does not become a burst of downloads.

## If you want a map

`bart-rs` parses `shapes.txt` and exposes the points, but it does not draw
anything. The shape data is a flat list of `(lat, lon, shape_id,
shape_pt_sequence)` tuples in `feed.shapes`, and the shape id is what links a
point to a trip's route. Rendering it is left entirely to you.

## Where to look next

| Document | What it answers |
| --- | --- |
| [07-library-guide.md](07-library-guide.md) | the whole public API: the layers, the worked example, the feature flags, the upstream gotchas |
| [01-architecture.md](01-architecture.md) | how the crate is laid out, and where the data comes from |
| [03-data-sources.md](03-data-sources.md) | the upstream URLs, the cache layout, and the staleness rules |
