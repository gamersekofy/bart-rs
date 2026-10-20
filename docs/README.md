# `bart-rs` documentation

This directory is the design record for `bart-rs`. It covers what the crate
does, how it is put together, what it does with BART's non-standard feed, and
why the interesting decisions were made the way they were.

It is written for anyone using the crate — from the command line or as a
library — and for anyone reading the source who wants to know why it is
like this rather than how to change it.

## The documents

| Document | What it answers |
| --- | --- |
| [00-overview.md](00-overview.md) | What `bart-rs` is, what it is for, and what is explicitly out of scope. |
| [01-architecture.md](01-architecture.md) | How is the crate laid out? What is the public API? How does data flow from disk to screen? |
| [02-data-model.md](02-data-model.md) | Every GTFS table BART publishes: exact columns, Rust types, and the BART-specific quirks that will bite you. |
| [03-data-sources.md](03-data-sources.md) | The upstream URLs, the on-disk cache layout, and the feed update and staleness lifecycle. |
| [04-cli-spec.md](04-cli-spec.md) | Every command, flag, and output format the binary exposes. |
| [05-testing.md](05-testing.md) | Test strategy, fixture layout, and how to validate a change against the live feed. |
| [06-decisions.md](06-decisions.md) | Architecture decision records: the "why" behind the choices, including the alternatives that were rejected. |
| [07-library-guide.md](07-library-guide.md) | How to use `bart-rs` from another Rust project — the worked example, the feature flags, and the upstream gotchas. |
| [08-gui-tips.md](08-gui-tips.md) | Putting a GUI on top of the library, and the threading problem you have to solve. |

## Conventions

- Verified facts are stated as fact. Anything inferred is marked *(inferred)*.
- Field numbers, column counts, and record counts were measured against the
  live BART feed on **2026-10-19** (feed version 72,
  `google_transit_20260810-20270108_v02.zip`). BART changes its feed between
  versions; re-verify anything load-bearing.
- Significant decisions are recorded in
  [06-decisions.md](06-decisions.md) as architecture decision records. If you
  change one, add a new entry rather than editing the old one — the history
  is the point.

## Things to know before you touch the code

These are properties of the upstream data, or of this crate's build, that
have each caused a bug at least once.

1. **Never index GTFS rows by column position.** The `stops.txt` schema has
   already changed once, from 10 columns to 11. Use header-driven
   deserialization.
2. **Do not trust `feed_info.txt`.** Its dates describe the *previous*
   release, and its `feed_version` is the same in every release. Take the
   coverage window from the calendar, and identify a release by the
   versioned filename the permalink redirects to. See
   [03-data-sources.md](03-data-sources.md).
3. **Do not trust the `Content-Type` header on the realtime feeds.** They
   are mislabelled `text/html` and serve protobuf. See
   [03-data-sources.md](03-data-sources.md).
4. **Do not remove the `gtfs-structures` dev-dependency.** It is used as a
   cross-validation oracle in the test suite (see
   [05-testing.md](05-testing.md)). It must keep `default-features = false`,
   or OpenSSL comes back into the tree.
5. **Do not change the MSRV without checking it.** `rust-version` is a
   promise to consumers, and it is enforced by a CI job that builds the
   crate with exactly that toolchain. See
   [06-decisions.md](06-decisions.md) D25 and D27.

## Out of scope

Some things this crate deliberately does not do. They are listed here so
that nobody has to guess:

- **A journey planner.** There is no routing engine, no transfer
  optimization, no walking directions. Arrivals and departures are per-stop.
- **A map renderer.** Shape data is parsed and exposed, but nothing draws
  it.
- **A real-time vehicle tracker.** The realtime feed contains expected
  arrival times and delays, not positions.
- **Multi-agency support.** Everything here is BART-specific. The code may
  be *inspired* by GTFS generally, but the quirks, extensions, and endpoints
  are BART's.
