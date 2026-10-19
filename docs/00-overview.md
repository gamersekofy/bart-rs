# 00 — Overview

## What is bart-rs?

`bart-rs` is a Rust command line tool (and library) that gives you useful
information about the Bay Area Rapid Transit (BART) agency's operations,
derived from the agency's published GTFS schedule feed and GTFS-Realtime feeds.

Concretely it lets you ask questions like:

- *When is the next train leaving Embarcadero, and where is it going?*
- *Which lines serve Millbrae, and what is the full stop sequence for the Red line?*
- *What does a ticket from Rockridge to SFO cost, and what is it with a Clipper card?*
- *Is my local feed stale, and what version of the schedule is it?*
- *What service alerts is BART currently publishing?*

## Why it exists

The original README said it plainly: *"Because I want to practice Rust lol."*
That is still true, and it is a legitimate design constraint rather than an
excuse. It is the reason the project hand-rolls GTFS parsing with `csv` + `serde`
instead of leaning on an existing crate, and the reason the realtime protobuf
schemas are written by hand rather than generated.

The practical consequence is that the library does the *boring, fiddly* work
correctly so that a consumer does not have to: CSV quirks, GTFS's
greater-than-24-hour clock, `calendar_dates` exceptions, BART's three-level
stop hierarchy, and BART's non-conformant alert feed.

## Goals

1. **Parse the whole feed.** All 18 files BART publishes are read into typed
   Rust structs. No table is left unhandled.
2. **Answer real questions.** Cross-file queries, not just table dumps.
3. **Stay current.** The tool downloads the feed itself, knows when it is stale,
   and says so.
4. **Show live service state.** Scheduled times from the static feed, actual
   times from the realtime feed, and service alerts.
5. **Be reusable.** The library is the product; the binary is one front end
   over it. Other Rust projects should be able to depend on `bart-rs`.

## Non-goals

- **Not a journey planner.** There is no routing engine, no transfer
  optimization, no walking directions. Arrivals/departures are per-stop.
- **Not a map renderer.** Shape data is parsed and exposed, but nothing draws
  it.
- **Not multi-agency.** Everything is BART-specific. The code may be *inspired*
  by GTFS generally, but the quirks, extensions, and endpoints are BART's.
- **Not a real-time vehicle tracker.** The realtime feed does not include
  positions; it includes expected arrival times and delays.
- **Not published to crates.io.** The crate is written to be publishable
  (clean docs, semantic versioning, no stray globals) but publishing is not a
  goal of this work. The name `bart-rs` happens to be unclaimed.

## Success criteria

The project is "done" when:

- [ ] `bart-rs update` downloads and extracts the current feed with no manual steps.
- [ ] `bart-rs status` reports the feed version, its coverage window, and warns
      when the feed has expired or has not been checked recently.
- [ ] All 18 GTFS tables are parsed into typed structs and can be dumped via
      `bart-rs table <name>`.
- [ ] `bart-rs arrivals <stop>` returns correct scheduled arrivals for a given
      stop and time, honouring `calendar.txt` and `calendar_dates.txt`.
- [ ] `bart-rs arrivals <stop> --realtime` overlays live trip updates with
      delay information.
- [ ] `bart-rs alerts` renders BART's service alerts legibly.
- [ ] `bart-rs fare --from A --to B` returns the fare and its rider-category
      variants.
- [ ] `cargo test` passes offline against committed fixtures.
- [ ] `cargo clippy --all-targets -- -D warnings` is clean.
- [ ] The public library API is documented and has no `missing_docs` warnings.

## Current state (as of 2026-10-19)

The repository has three commits, the last from January 2025. The code parses
`stops.txt` by splitting lines on `,` and leaking every field with
`Box::leak`; the `trips` parser is a stub that prints `"hi"`. The `gtfs-structures`
dependency is declared but never used. The committed `data/` directory holds a
**stale January-2024 feed** whose coverage window ended on 2024-08-11.

`git log` for the full history:

```
05e1796 Added better error handling code for when a file isn't found and added the
         colored crate to make output look better. Also removed standalone stops.txt
         and added whole data directory
2fdde56 Refactored code: moved the trim_quotes() function to a file named helper.rs
7e1624d Initial commit
```

The rewrite on `feat/rewrite` replaces the source tree wholesale while keeping
the module-per-GTFS-file idea from `src/attributes/`, which was the one good
structural decision in the original code.

## Related reading

- [01-architecture.md](01-architecture.md) — how the code is organised.
- [02-data-model.md](02-data-model.md) — the 18 tables and their quirks.
- [05-implementation.md](05-implementation.md) — the build plan.
- [07-decisions.md](07-decisions.md) — why it is built this way.
