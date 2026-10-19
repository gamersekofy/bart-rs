# bart-rs documentation

This directory is the working design record for `bart-rs`. It exists so that a
successor agent (or a human returning to this after a year) can pick the project
up cold and know exactly what has been decided, what is left to do, and why.

The original repository README listed a feature checklist but no design
rationale, no data model, and no CLI contract. That was the main reason the
project stalled. These documents fix that.

## Read in this order

| Document | What it answers |
| --- | --- |
| [00-overview.md](00-overview.md) | What is `bart-rs`, what is it for, what is explicitly out of scope, and how do we know when it is done? |
| [01-architecture.md](01-architecture.md) | How is the crate laid out? What is the public API? How does data flow from disk to screen? |
| [02-data-model.md](02-data-model.md) | Every GTFS table BART publishes: exact columns, Rust types, and the BART-specific quirks that will bite you. |
| [03-data-sources.md](03-data-sources.md) | The three upstream URLs, the on-disk cache layout, and the feed update / staleness lifecycle. |
| [04-cli-spec.md](04-cli-spec.md) | Every command, flag, and output format the binary exposes. |
| [05-implementation.md](05-implementation.md) | The phased build plan with per-task acceptance criteria. **Start here to do work.** |
| [06-testing.md](06-testing.md) | Test strategy, fixture layout, and how to validate against the live feed. |
| [07-decisions.md](07-decisions.md) | Architecture decision records. The "why" behind the choices, including the rejected alternatives. |
| [08-library-guide.md](08-library-guide.md) | How to use `bart-rs` from another Rust project — a GUI, a web service, a bot. The worked example, the threading notes, and the upstream gotchas. |

## Conventions used in these documents

- **"Must"** — required for the task to be considered done.
- **"Should"** — strongly preferred; deviate only with a note in `07-decisions.md`.
- **"May"** — optional, your call.
- Verified facts are stated as fact. Anything inferred is marked *(inferred)*.
- Field numbers, column counts, and record counts were measured against the live
  BART feed on **2026-10-19** (feed version 72, `google_transit_20260810-20270108_v02.zip`).
  BART changes its feed between versions; re-verify anything load-bearing.

## Ground rules for agents working in this repo

1. **Read `05-implementation.md` before writing code.** It has the task ordering
   and the acceptance criteria for each task.
2. **Do not remove the `gtfs-structures` dev-dependency.** It is used as a
   cross-validation oracle in the test suite (see `06-testing.md`). It must
   keep `default-features = false`, or OpenSSL comes back into the tree.
3. **Do not trust the `Content-Type` header on the realtime feeds.** They are
   mislabelled `text/html` and serve protobuf (see `03-data-sources.md`).
4. **Never index GTFS rows by column position.** The `stops.txt` schema already
   changed once (10 → 11 columns). Use header-driven deserialization.
5. **Do not trust `feed_info.txt`.** Its dates describe the previous release,
   and its `feed_version` is the same in every release. Take the coverage
   window from the calendar, and identify a release by the versioned filename
   the permalink redirects to (see `03-data-sources.md`).
6. **When you finish a task, tick its checkbox in `05-implementation.md`** and
   note any deviation in `07-decisions.md`. The checkbox is the shared state.

## Where the work stands

The rewrite described in `05-implementation.md` is complete: every phase is
ticked, and the ten phases' worth of decisions are in `07-decisions.md`. The
three things the original project got wrong — the stale feed, the
non-conformant alert feed, and the two `feed_info.txt` traps — are all
written up.

What is *not* done, and is deliberately out of scope for now:

- **A journey planner** that finds connections between two stations. The
  per-stop arrival tables are the groundwork for it, but the search itself
  is a separate piece of work.
- **A map view** of the route shapes. The shape points are parsed and
  available through the library, but there is no renderer.
- **Publishing to crates.io.** The crate is written to be publishable, but
  publishing it is not a goal of this work.
