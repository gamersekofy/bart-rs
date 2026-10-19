# 05 — Implementation plan

The build plan. Work top to bottom. Each task has a checkbox and an
**acceptance criterion** — a concrete, checkable statement of "done".

When you complete a task, tick its box here. That checkbox is the shared state
between agents; do not rely on commit messages alone.

**Status: complete.** Every phase below was implemented on `feat/rewrite`. The
notes under each phase record what actually turned out to be true, so that a
successor agent does not have to rediscover it.

**Legend:** `[ ]` todo · `[~]` in progress · `[x]` done

---

## Phase 0 — Documentation (the plan itself)

- [x] Create `docs/` with the nine documents listed in `docs/README.md`.
- [x] Record the verified upstream findings (URLs, redirects, no API key,
      mislabelled content type, the BART alert field deviation, the stop
      hierarchy, the `stops.txt` schema change) in `02-data-model.md` and
      `03-data-sources.md`.

**Acceptance:** a new agent can read `docs/` and answer "how do I add a new
GTFS table?", "where does the feed live?", and "how do I decode the realtime
alerts?" without reading any Rust source.

---

## Phase 1 — Foundation

- [x] `nix flake update` — bumps `flake.lock` from nixpkgs 2024-11 to
      2026-10, which moves the toolchain from rustc 1.82.0 to **1.99.1**.
      *This was mandatory:* rustc 1.82 predates the `edition2024` cargo
      feature and cannot compile current crates.io versions of `zeroize`,
      `cpufeatures`, `time-core`, and others.
- [x] Verify the toolchain: `nix develop --command rustc --version` → 1.99.1,
      and that `rustfmt` and `clippy` are on the path.
- [x] Add `rust-toolchain.toml` with `channel = "stable"` so non-Nix users get
      a current toolchain. The flake's `fromRustupToolchainFile` branch picks
      it up; verified working.
- [x] Rewrite `Cargo.toml`:
      - `[lib]` (auto-discovered from `src/lib.rs`) + `[[bin]` (auto-discovered
        from `src/main.rs`).
      - `[features] default = ["cli", "download"]`,
        `cli = ["dep:clap", "dep:colored"]`,
        `download = ["dep:reqwest", "dep:zip"]`.
      - Dependencies: `clap` (derive), `colored`, `csv`, `serde` (derive),
        `serde_json`, `chrono` (serde), `chrono-tz`, `reqwest`
        (blocking + rustls-tls), `zip`, `prost`, `dirs`, `thiserror`.
      - `[dev-dependencies] gtfs-structures = { version = "0.42",
        default-features = false }` — kept **as a test oracle** only; it is
        not used by the library.
      - Package metadata: `description`, `license = "MIT"`, `repository`,
        `homepage`, `keywords`, `categories`, `readme`.
- [x] Resolve and build.

**Acceptance:** `cargo build` succeeds with no warnings; `cargo clippy
--all-targets --all-features -- -D warnings` is clean; `nix develop --command
cargo --version` reports 1.99.1. — **met**

### What actually happened

- `gtfs-structures`'s **default feature is `read-url`**, which enables
  `reqwest` → `native-tls` → `openssl-sys`. Using it as a dev-dependency with
  `default-features = false` removes OpenSSL from the tree entirely, so
  `openssl` and `pkg-config` could be dropped from the dev shell.
- `reqwest` with `rustls-tls` uses the `ring` crypto provider, not
  `aws-lc-rs`, so no `cmake` or `perl` is needed in the shell.
- The FlakeHub `nixpkgs` URL was replaced with `github:NixOS/nixpkgs/nixos-unstable`,
  because a fresh clone should not depend on FlakeHub still being up. The
  lock file pins the exact commit, so the build is still reproducible.

---

## Phase 2 — Repository hygiene

- [x] `.gitignore`: stop ignoring `Cargo.lock` (this is a binary crate; the
      lock file must be committed). Keep ignoring `/target`, `.direnv`,
      `.envrc`, `/.idea/`. Add `/data`.
- [x] `git rm -r --cached .idea` — the IDE config was committed in the
      initial commit even though `.gitignore` already excluded it.
- [x] `git rm -r --cached data` and delete the stale feed. The committed feed
      is **feed version 69 from January 2024** whose coverage ended
      2024-08-11. The tool manages its own data; see
      `03-data-sources.md`.
- [x] Commit `Cargo.lock` once the dependency set is final.
- [x] `src/attributes/stops.rs` and `src/attributes/trips.rs` rewritten
      rather than patched (Phase 3).

**Acceptance:** `git ls-files` shows no `.idea/` entries and no `data/` entries;
`Cargo.lock` is tracked; a fresh clone builds and passes tests without
network access. — **met**

---

## Phase 3 — Library core

### 3a. Scaffolding

- [x] `src/lib.rs` — module declarations, re-exports of `Error`/`Result`/`Feed`,
      `#![warn(missing_docs)]`.
- [x] `src/main.rs` — a thin adapter that prints the `Display` form of any
      error and exits `1`.
- [x] `src/error.rs` — the `Error` enum from `01-architecture.md`, using
      `thiserror`. The `Http` variant is `#[cfg(feature = "download")]`, since
      that is the only configuration with an HTTP client compiled in.
- [x] `src/helper.rs` — the shared deserializers listed at the bottom of
      `02-data-model.md`:
      - `de_gtfs_time` → `Option<u32>` seconds since midnight, accepting
        `HH:MM:SS` with `HH > 24`.
      - `de_gtfs_date` → `Option<NaiveDate>` from `YYYYMMDD`.
      - `de_bool_01` → `bool` from `"0"` / `"1"`.
      - `de_cents` → `i64` from `"7.55"`.
      - `trim_quotes` — kept from the original code.
      - `read_table<T: DeserializeOwned>(path) -> Result<Vec<T>>` — the shared
        `csv::Reader` wrapper, with `trim(Trim::All)` and `flexible(false)`.

**Acceptance:** `de_gtfs_time("25:30:00") == Some(91800)`;
`de_gtfs_time("05:52:00") == Some(21120)`; `de_cents("7.55") == 755`. — **met**

### 3b. The 18 table modules

- [x] `agency.rs` · `calendar.rs` · `calendar_attributes.rs` ·
      `calendar_dates.rs` · `directions.rs` · `fare_attributes.rs` ·
      `fare_rider_categories.rs` · `fare_rules.rs` · `feed_info.rs` ·
      `realtime_routes.rs` · `rider_categories.rs` · `route_attributes.rs`
- [x] `routes.rs` · `shapes.rs` · `stops.rs` · `stop_times.rs` ·
      `transfers.rs` · `trips.rs`
- [x] `attributes/mod.rs` — `pub mod` for each, plus `TABLE_NAMES`.

**Acceptance:** one test per table; `stops.rs` has a test proving the
`plc_url` → `stop_url` `alias` works against the *old* 10-column schema as
well as the current 11-column one. — **met**

### What actually happened

- `calendar.txt`'s day flags are `0`/`1` and are **not** valid serde
  booleans; they need `de_bool_01`. Same for `realtime_routes.txt`.
- `Option<String>` deserializes an empty CSV field to `None` on its own, so
  a separate "empty string means absent" helper was not needed.

---

## Phase 4 — `Feed` and `schedule`

- [x] `src/feed.rs` — the `Feed` struct and every method in
      `01-architecture.md#the-feed-type`.
- [x] `src/schedule.rs`:
      - `service_ids_for(date) -> BTreeSet<String>`: the calendar's weekly
        pattern, then the `calendar_dates` exceptions, with a removal winning
        over an addition.
      - `current_service_date(now_pacific) -> NaiveDate` — the `[00:00, 04:00)`
        rule.
      - `fmt_gtfs_time`, `fmt_human_time`, `fmt_short_time`.
- [x] `Arrival` / `Departure` result types. `Departure` is a type alias for
      `Arrival`: the two carry exactly the same information, and only the
      meaning of `time` differs.

**Acceptance:**
- `service_ids_for` on a known weekday returns the weekday service ids and
  excludes the Saturday/Sunday ones; a `calendar_dates` row with
  `exception_type = 2` removes a service that `calendar.txt` would otherwise
  include; a row with `exception_type = 1` adds one. — **met**
- `current_service_date` at 2026-10-19 02:00 Pacific returns 2026-10-18. —
  **met**
- `arrivals_at` for a station returns times in ascending order, all ≥ the
  requested time, and only for trips whose `service_id` is active on the
  service date. — **met**
- `stations_on_route` returns the stops of a single trip's sequence, in
  `stop_sequence` order. — **met**

---

## Phase 5 — `update` and the cache

- [x] `src/paths.rs` — `data_dir()` honouring `BART_RS_DATA_DIR`, then
      `$XDG_DATA_HOME/bart-rs`, then `~/.local/share/bart-rs`; plus
      `feed_dir()`, `current_feed_dir()`, `archives_dir()`, `state_path()`.
- [x] `src/update.rs`:
      - `download_latest(client)` — fetch, follow redirects, capture the final
        URL, read `feed_info.txt` out of the zip in memory to learn the
        version.
      - `extract(archive, dest)` — write the 18 files into a staging
        directory, rejecting unsafe paths and missing tables.
      - `install(staging, feed_dir)` — the atomic swap from
        `03-data-sources.md#update-lifecycle`, with rollback.
      - `check_latest(client)`, `read_state()`, `write_state()`,
        `staleness(...)`.

**Acceptance:**
- `bart-rs update` on a clean machine produces 18 files under
  `<data_dir>/feed/current/` and a `state.json`. — **met**
- Running `bart-rs update` twice is idempotent and the second run reports
  "already up to date" without a second download. — **met**
- A corrupt/truncated zip produces a clear error and **leaves the previous
  `feed/current` intact**. — **met** (the staging directory is only swapped in
  after extraction and validation succeed)
- `bart-rs status` on a feed whose `feed_end_date` is in the past prints the
  expiry warning. — **met**

---

## Phase 6 — GTFS-Realtime

- [x] `src/gtfsrt.rs` — hand-written `#[derive(prost::Message)]` structs for
      the message set listed in `03-data-sources.md#message-set-to-implement`.
- [x] `src/realtime.rs`:
      - `fetch_trip_updates(client)`, `fetch_alerts(client)`.
      - `alert_text(bytes)` — the tolerant extractor that handles both a bare
        string and a nested `Translation`.
      - `overlay(feed, scheduled, live, departing)` — adjusts the scheduled
        times by the reported delay.
- [x] A realtime `trip_id` that is not in the static feed is not an error.

**Acceptance:**
- The live alerts feed decodes and every entity yields a non-empty
  human-readable line. — **met**
- The live trip update feed decodes, and its entities join to trips in the
  downloaded static feed. — **met**
- An entity whose `trip_id` is absent from the static feed is reported
  gracefully. — **met**
- A truncated or non-protobuf body produces a clear error message. — **met**

### What actually happened

- BART's alert feed is **not self-consistent** about the shape of its text
  fields. `header_text` holds a bare string where a `Translation` message is
  expected, while the other fields hold proper nested messages. Declaring
  the fields as `TranslatedString` made the whole feed fail to decode with
  `delimited length exceeded`. They are now held as raw bytes
  (`prost(bytes = "vec")`) and interpreted by `gtfsrt::alert_text`. See
  `03-data-sources.md` for the byte-level analysis.

---

## Phase 7 — CLI

- [x] `src/cli.rs` — the clap tree matching `04-cli-spec.md` exactly:
  `update`, `status`, `routes`, `stops`, `route`, `stop`, `arrivals`,
  `departures`, `trip`, `fare`, `alerts`, `table`.
  - Global options: `--no-color`, `--data-dir`, `--json`.
  - `--json` is rejected on `update` and `status`.
- [x] `src/commands/` — one module per command, each returning `Result<()>`.
- [x] Colour handling: `NO_COLOR` and `--no-color` are honoured through
  `colored::control::set_override`.

**Acceptance:** every command in the spec has `--help` output; `bart-rs
--help` lists all twelve; `bart-rs routes --json` emits valid JSON;
`bart-rs stop LAKE` resolves by id, by platform id, and by name. — **met**

### What actually happened

- `Result`'s `Termination` impl prints the **Debug** form of an error, which
  is not what a user wants to read. `main` now prints the `Display` form
  itself and exits `1`.
- A missing feed is reported against the feed *directory*
  ("no GTFS feed found at …/feed/current"), and a missing *table* is a
  separate error ("the feed at … is missing transfers.txt").

---

## Phase 8 — Tests

- [x] `tests/fixtures/gtfs/` — a trimmed copy of every one of the 18 tables.
- [x] `tests/fixtures/gtfs-2024/stops.txt` — the old 10-column `stops.txt`,
      the regression test for the schema drift.
- [x] `tests/parse_tests.rs` — one test per table.
- [x] `tests/schema_drift_tests.rs` — the old and new `stops.txt` schemas.
- [x] `tests/schedule_tests.rs` — the service-day rules.
- [x] `tests/feed_tests.rs` — the cross-file queries.
- [x] `tests/cli_tests.rs` — spawns the binary and asserts on its output.
- [x] `tests/live_feed.rs` — `#[ignore]`d; downloads the real feed and
      cross-checks against `gtfs-structures`.

**Acceptance:** `cargo test` passes offline with no network access (118
tests); `cargo test -- --ignored` passes when the network is available. —
**met**

### What actually happened

- The tests run in parallel, so anything that writes to a temporary directory
  needs a **unique** name per test. Using the process id alone was not
  enough, and caused flaky failures.
- The reference implementation models shapes as `HashMap<ShapeId, Vec<Shape>>`
  while we keep a flat list of shape *points*, so the cross-check compares
  the total point count and the distinct shape id count, not the shape count.

---

## Phase 9 — Documentation and CI

- [x] Rewrite `README.md`: what the tool is, how to install it with Nix, the
      command list, the data sources, and the feature checklist.
- [x] Add `.github/workflows/ci.yml`: `cargo fmt --check`, `cargo clippy
      --all-targets --all-features -- -D warnings`, `cargo build --locked`,
      `cargo test --locked`, `cargo doc`, and a `nix flake check` job.
- [x] Update `docs/` to match what was actually built.

**Acceptance:** CI is green; the README's checklist matches reality. — **met**

---

## Phase 10 — Fixes found by using the tool

These were not in the original plan. They were found by running the finished
tool against the live feed and reading its output, which is the thing the
plan is supposed to enable.

- [x] **The coverage window comes from the calendar, not from
      `feed_info.txt`.** BART leaves `feed_info.txt`'s dates describing the
      previous release, so `status` reported a current feed as expired 50
      days ago, and `update` would not download a newer one. (D18)
- [x] **A release is identified by the versioned filename, not by
      `feed_version`.** `feed_version` is `72` in every release, so the
      update check was permanently satisfied and never downloaded a new
      feed. The check is now a `HEAD` request, so it is also free. (D17)
- [x] **Hex route colours are parsed by hand.** `colored` silently falls
      back to white for any colour name it does not recognise, so every
      route's swatch was white. (D19)

**Acceptance:** each fix has a regression test that fails without it; 118
tests pass; the live output was re-checked by hand. — **met**

---

## Appendix — verified facts this plan depends on

Measured on 2026-10-19 against feed version 72.

| Fact | Value |
| --- | --- |
| Static feed URL | `https://www.bart.gov/dev/schedules/google_transit.zip` |
| Resolves to | `https://www.bart.gov/sites/default/files/2026-08/google_transit_20260810-20270108_v02.zip` |
| Archive size | 892,312 bytes, 18 files |
| Realtime URLs | `http://api.bart.gov/gtfsrt/tripupdate.aspx`, `.../alerts.aspx` |
| Realtime API key | **not required** |
| Realtime content type | `text/html` (wrong; body is protobuf) |
| Trip update size | ~38 KB; alerts ~333 B when idle |
| Feed tables | 18 |
| Rows | routes 14 · stops 287 (50 stations / 105 platforms / 132 entrances) · trips 4417 · stop_times 57188 · shapes 27893 (28 shapes) · fares 2500 |
| Stops schema | 11 columns now; 10 columns in the old committed feed (`plc_url` vs `stop_url`, `platform_code` added) |
| Stop hierarchy | station → platform, and station → entrance; all resolve |
| Fare rules | keyed on **station** ids, `contains_id` always empty |
| BART alert fields | text lives in Alert fields **10/11**, not the spec's 6/8; and the shapes of the text fields are inconsistent |
| `feed_info.txt` dates | describe the **previous** release; the calendar's window is `20260810` → `20270110` |
| `feed_version` | `72` in every release; cannot identify a release |
| Release identifier | the versioned filename, e.g. `google_transit_20260810-20270108_v02.zip` |
| Rust toolchain | rustc 1.99.1 after `nix flake update` (was 1.82.0) |
