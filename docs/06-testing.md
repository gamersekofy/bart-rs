# 06 — Testing

## Goals

1. **Offline by default.** `cargo test` must pass with no network access. Most
   tests must not even need a downloaded feed.
2. **Catch the things that actually go wrong.** The bugs in the original code
   were all in the "parse the data correctly" layer: naive comma splitting,
   header rows treated as data, positional column access, a schema change
   breaking the parser. Those get the most tests.
3. **Validate against a reference implementation.** `gtfs-structures` is kept
   as a **dev-dependency** purely to cross-check our parser against a
   well-known one on the real feed. It is not used by the library.
4. **Test the schedule logic properly.** Service-day resolution is the part
   most likely to be subtly wrong and the hardest to notice by hand.

## Test layers

### Unit tests — `src/**/*.rs`, `#[cfg(test)] mod tests`

Live next to the code they test. Cover the pure functions:

| Target | Cases |
| --- | --- |
| `helper::de_gtfs_time` | `"05:52:00"` → 21120 · `"25:30:00"` → 91800 · `"24:00:00"` → 86400 · `""` → `None` · `"bogus"` → `None` |
| `helper::de_gtfs_date` | `"20260112"` → date · `""` → `None` · `"2026-01-12"` → `None` |
| `helper::de_bool_01` | `"0"` → false · `"1"` → true · `""` → false |
| `helper::de_cents` | `"7.55"` → 755 · `"0.00"` → 0 · `"10.10"` → 1010 |
| `schedule::current_service_date` | 02:00 Pacific → **previous** date · 04:00 → previous date · 04:01 → same date · 23:59 → same date |
| `schedule::fmt_gtfs_time` | 91800 → `"25:30:00"`; human form → `"01:30:00 (+1d)"` |
| `realtime` extraction | alert text falls back 11 → 6 → 10 → 5 |

### Integration tests — `tests/*.rs`

These use the public library API and the committed fixtures. There are 107
tests in total; they run offline and take well under a second.

| File | Covers |
| --- | --- |
| `tests/parse_tests.rs` | one test per GTFS table: parse the fixture, assert field values, assert the row count |
| `tests/schema_drift_tests.rs` | the old 10-column `stops.txt` and the current 11-column one both parse to the same `Stop` shape; the `plc_url` alias works |
| `tests/schedule_tests.rs` | `service_ids_for` across a weekday, a Saturday, a Sunday, a holiday exception (added), and a holiday exception (removed) |
| `tests/feed_tests.rs` | `routes_serving`, `stations_on_route`, `arrivals_at`, `departures_from`, `fare`, `station_of`, `find_stops` |
| `tests/coverage_tests.rs` | the coverage window comes from the calendar, not from `feed_info.txt`; a current feed is not reported as expired |
| `tests/release_tests.rs` | two releases with the same `feed_version` are still different releases; the release is recovered from `state.json` |
| `tests/colour_tests.rs` | a hex route colour becomes a true-colour escape, and never the white fallback; escape sequences do not count towards a column's width |
| `tests/cli_tests.rs` | spawn the built binary; assert on exit codes and key substrings in stdout; the swatch does not push a table out of alignment; a closed output pipe is not an error |
| `tests/live_feed.rs` | `#[ignore]`d; downloads the real feed and cross-checks against `gtfs-structures` |

### Regression tests for the upstream quirks

Three of the tests above exist because of a bug that shipped and was caught
in use. They are worth naming, because each one guards a piece of BART's
behaviour that looks like a bug in this code but is not:

- **`coverage_tests.rs`** guards the finding that `feed_info.txt`'s dates
  describe the previous release. Without it, `bart-rs status` reports a
  current feed as expired.
- **`release_tests.rs`** guards the finding that `feed_version` never
  changes. Without it, `bart-rs update` never downloads a new feed.
- **`colour_tests.rs`** guards the `colored` crate's silent white
  fallback for colour names it does not recognise.

Each of these was found by running the tool against the real feed and
reading the output, not by reading the code. The live-feed test cannot
catch any of them: they are all about *reporting*, not about the data.

### Live-feed test — `tests/live_feed.rs`

```rust
#[test]
#[ignore = "requires network access"]
fn the_live_feed_agrees_with_the_reference_implementation() {
    // 1. download the real feed into a temp dir
    // 2. parse it with our Feed::open
    // 3. parse it with gtfs_structures::Gtfs::new
    // 4. assert the counts agree
    //    routes, stops, trips, fares, shape points
    // 5. assert every platform and entrance resolves to a station
    // 6. assert every stop_time references a stop that exists
}
```

Run with `cargo test -- --ignored`. Run it **before** changing anything in
`attributes/` or `feed.rs` — it is the fastest way to know whether a change
broke reality. It is `#[ignore]`d by default so that `cargo test` stays
offline and deterministic.

The reference crate is used like this:

```rust
let reference = gtfs_structures::Gtfs::new(dir.join("feed/current").to_str().unwrap())
    .expect("reference parse");
assert_eq!(feed.routes.len(), reference.routes.len());
assert_eq!(feed.stops.len(), reference.stops.len());
assert_eq!(feed.trips.len(), reference.trips.len());
```

Note that `gtfs_structures::Gtfs::new` **errors** on a feed with dangling
references, so a successful reference parse is itself a meaningful assertion
about our feed's integrity.

Two differences in modelling to be aware of when writing assertions:

- The reference groups shape points by shape id
  (`HashMap<ShapeId, Vec<Shape>>`); we keep a flat list of points. Compare
  the **total point count** and the **distinct shape id count**, not the
  shape count.
- The reference merges `stop_times` into its `Trip` type, so there is no
  direct equivalent of our `stop_times` table.

## Fixtures

`tests/fixtures/gtfs/` — a trimmed copy of every one of the 18 tables. Keep
each file small (5–20 rows) but make it *adversarial*: the fixtures must
contain the edge cases, not just the happy path.

The fixture feed is built around three stations — `LAKE` (Lake Merritt),
`EMBR` (Embarcadero) and `SFIA` (San Francisco International Airport) — with
three routes and five trips spread over three service patterns.

Required edge cases in the fixtures:

| Case | Where |
| --- | --- |
| A time past 24:00:00 (`25:00:00`) | `stop_times.txt` |
| An empty optional field | `trips.txt` (`block_id`), `stops.txt` (`stop_desc`) |
| A `calendar_dates` row with `exception_type = 1` | `calendar_dates.txt` |
| A `calendar_dates` row with `exception_type = 2` | `calendar_dates.txt` |
| A `calendar` row whose `end_date` excludes the test date | `calendar.txt` |
| A fare rule with an empty `origin_id` | `fare_rules.txt` |
| A fare rule where `origin_id == destination_id` | `fare_rules.txt` |
| A stop with `location_type = 1` and two platforms pointing at it | `stops.txt` |
| An entrance pointing at that station | `stops.txt` |
| A platform with no `parent_station` | `stops.txt` |
| A trip with an empty `direction_id` | `trips.txt` |
| A route with an empty `route_desc` and a `route_color` | `routes.txt` |
| A `directions.txt` row for a bus-bridge route (`BB-A`) | `directions.txt` |
| A quoted field containing a `/` | `trips.txt` (`trip_headsign`) |

Additionally, `tests/fixtures/gtfs-2024/stops.txt` holds the old 10-column
`stops.txt` from the feed that was committed in the original `data/`
directory. This is the regression test for the schema drift described in
`02-data-model.md`.

**Keep the exception dates off the ordinary test dates.** An early version of
the fixture used the same Saturday for both "a Saturday activates the Saturday
service" and "the Saturday service is removed on this date", so the two tests
contradicted each other. The exception dates are now a week earlier
(2026-10-10 and 2026-10-11) from the ordinary ones (2026-10-19, 2026-10-24,
2026-10-25).

**Anything that writes to a temporary directory needs a unique name per
test.** The tests run in parallel, so a name derived only from the process id
is not enough; use a counter as well.

## Running the tests

```bash
cargo test                     # offline, deterministic
cargo test -- --ignored        # live feed, needs network
cargo test --test schedule_tests -- --nocapture
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## What is deliberately not tested

- **Pixel-exact terminal output.** Assert on substrings and structure, not on
  rendered colour codes.
- **The live realtime feeds, in CI.** They change every few minutes and are not
  reproducible. The realtime decoder is tested against the committed
  `tests/fixtures/*.pb` protobuf files, which were captured from the live feed
  and are committed as binary fixtures.
- **Download retry/backoff behaviour.** The update path has no retry logic by
  design; a failed download is an error the user retries. *(If retry logic is
  added, it needs a test with an injected failure.)*

## CI

`.github/workflows/ci.yml` runs on every push and pull request:

```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt --check
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo test --all-features
```

The live-feed test is not run in CI. Run it locally before merging changes to
the parsing layer.
