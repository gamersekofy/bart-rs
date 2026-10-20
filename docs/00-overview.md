# 00 — Overview

## What is bart-rs?

`bart-rs` is a Rust command line tool and library that gives you useful
information about the Bay Area Rapid Transit (BART) agency's operations,
derived from the agency's published GTFS schedule feed and its GTFS-Realtime
feeds.

Concretely it lets you ask questions like:

- *When is the next train leaving Embarcadero, and where is it going?*
- *Which lines serve Millbrae, and what is the full stop sequence for the Red line?*
- *What does a ticket from Rockridge to SFO cost, and what is it with a Clipper card?*
- *Is my local feed stale, and what version of the schedule is it?*
- *What service alerts is BART currently publishing?*

The library is the product; the binary is one front end over it. Anything the
CLI can do, the library can do, and a good deal that it does not is
available too — see [07-library-guide.md](07-library-guide.md).

## What it does for you

BART's feed is GTFS, and GTFS is a format that is easy to get subtly wrong.
The work of doing it correctly is not obvious, and most of it cannot be read
off the specification:

- **CSV quirks.** Quoted fields containing commas, optional fields that are
  empty rather than absent, header rows that are not data.
- **GTFS's greater-than-24-hour clock.** A train at `25:30:00` is tomorrow
  morning at 01:30, and belongs to the previous service day.
- **Service day boundaries.** BART's service day runs from 04:00 to 04:00,
  so "now" is not always today's date.
- **Calendar exceptions.** `calendar_dates.txt` adds and removes service on
  individual dates, on top of the weekly patterns in `calendar.txt`.
- **BART's three-level stop hierarchy.** Stations, platforms and entrances
  are all `stops.txt` rows, distinguished by `location_type`.
- **BART's non-conformant alert feed.** It does not follow the GTFS-Realtime
  field numbering and is not self-consistent about the shape of its own text
  fields.

`bart-rs` handles all of that, so that a consumer does not have to. The
findings are written up in
[02-data-model.md](02-data-model.md) and
[03-data-sources.md](03-data-sources.md), and the decisions they forced are
in [06-decisions.md](06-decisions.md).

## Design goals

1. **Parse the whole feed.** All 18 files BART publishes are read into typed
   Rust structs. No table is left unhandled.
2. **Answer real questions.** Cross-file queries, not just table dumps.
3. **Stay current.** The tool downloads the feed itself, knows when it is
   stale, and says so.
4. **Show live service state.** Scheduled times from the static feed, actual
   times from the realtime feed, and service alerts.
5. **Be reusable.** Other Rust projects should be able to depend on
   `bart-rs` without inheriting a command line tool they did not ask for —
   which is why the CLI is a feature, and can be turned off.

## Non-goals

- **Not a journey planner.** There is no routing engine, no transfer
  optimization, no walking directions. Arrivals and departures are
  per-stop.
- **Not a map renderer.** Shape data is parsed and exposed, but nothing
  draws it.
- **Not multi-agency.** Everything here is BART-specific. The code may be
  *inspired* by GTFS generally, but the quirks, extensions, and endpoints
  are BART's.
- **Not a real-time vehicle tracker.** The realtime feed contains expected
  arrival times and delays, not positions.

## What is in the box

- A library (`bart_rs`) and a thin binary (`bart-rs`) on top of it.
- All 18 tables parsed with `csv` and `serde`, header-driven.
- The feed downloaded, extracted and installed by the tool itself, into the
  user's cache directory, with an atomic swap.
- Twelve commands, all of them `--json`-capable where that makes sense.
- A test suite that runs offline against committed fixtures, plus an
  optional test against the live feed.
- A Nix flake that builds the binary, provides a dev shell, and defines the
  CI checks.

## Related reading

- [01-architecture.md](01-architecture.md) — how the code is organised.
- [02-data-model.md](02-data-model.md) — the 18 tables and their quirks.
- [04-cli-spec.md](04-cli-spec.md) — the command line contract.
- [06-decisions.md](06-decisions.md) — why it is built this way.
