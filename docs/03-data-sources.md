# 03 — Data sources

Three upstream endpoints, one on-disk cache, and the rules for keeping the
cache fresh.

## The three URLs

### 1. Static schedule (GTFS)

```
https://www.bart.gov/dev/schedules/google_transit.zip
```

This is BART's **permalink** — it always serves the current feed. It is *not*
the same URL as the "schedule download" links on the GTFS developer page,
which point at dated files.

- Responds `200` with `Content-Type: application/zip`.
- Issues a `301` to a versioned URL, e.g.
  `https://www.bart.gov/sites/default/files/2026-08/google_transit_20260810-20270108_v02.zip`.
  The filename encodes the coverage window and a revision counter.
- Size is around 900 KB compressed; ~4.8 MB uncompressed across 18 files.
- No API key, no authentication, no rate-limit headers observed.
- Cloudflare sits in front of it, so set a descriptive `User-Agent`.

**Always follow the redirect and record the final URL.** The versioned filename
is the *only* reliable release identifier — see the two findings below.

### ⚠ `feed_info.txt`'s dates describe the previous release

`feed_info.txt` carries `feed_start_date` and `feed_end_date`, which the GTFS
spec defines as the window for which the dataset is complete and reliable.
BART does not maintain them. Measured on the release whose file is named
`google_transit_20260810-20270108_v02.zip`:

| Source | Window |
| --- | --- |
| `feed_info.txt` | `20260112` → `20260830` |
| `calendar.txt` (earliest `start_date`, latest `end_date`) | `20260810` → `20270110` |
| `calendar_dates.txt` (earliest and latest date) | `20260822` → `20270101` |

`feed_info.txt` is describing the release *before* this one. A tool that
trusts it will tell the user that a perfectly current feed expired 50 days
ago, and will not let them download the feed that would fix it.

**The coverage window therefore comes from the calendar**, not from
`feed_info.txt`: the earliest `start_date` and the latest `end_date` across
`calendar.txt`, widened by any single-date exception in `calendar_dates.txt`.
`feed_info.txt`'s dates are still shown in `bart-rs status`, labelled as the
stale claim they are, so the discrepancy is visible rather than hidden.

### ⚠ `feed_version` is a constant, and cannot detect a new release

`feed_info.txt`'s `feed_version` was `72` in both the January 2026 release
and the August 2026 release. It does not change between releases, so
comparing it can never tell you that a new feed has been published — the
check silently succeeds forever, and the tool never downloads a new feed.

The release is identified instead by the **versioned filename** the permalink
redirects to:

```
google_transit_20260810-20270108_v02.zip
           ^^^^^^^^ ^^^^^^^   ^^
           coverage window    revision counter
```

Both parts change on every release. A `HEAD` request to the permalink
follows the redirect and reports the final URL and its `Last-Modified`
header without transferring the body, so checking for a new release costs
almost nothing.

### 2. Realtime trip updates (GTFS-Realtime)

```
http://api.bart.gov/gtfsrt/tripupdate.aspx
```

- The `http://` URL issues a `301` to `https://api.bart.gov/gtfsrt/tripupdate.aspx`.
  Use the `https://` form directly in code; keep the `http://` form in the docs
  because it is the one BART publishes.
- **No API key is required.** A `?key=` query parameter is accepted and ignored.
- Responds `200` with `Content-Type: text/html` — **this is wrong**. The body is
  a GTFS-Realtime `FeedMessage` protobuf. Never branch on `Content-Type`; read
  the raw bytes and decode them as protobuf.
- Typical size ~38 KB.
- Roughly 74 of the trip ids present in a live sample resolve against the
  current static feed; the rest are trips the realtime feed knows about but the
  downloaded schedule does not (schedule changes mid-service-day). Treat an
  unresolvable `trip_id` as normal, not as an error.

### 3. Realtime service alerts (GTFS-Realtime)

```
http://api.bart.gov/gtfsrt/alerts.aspx
```

- Same redirect, same "no API key", same mislabelled `Content-Type`.
- Small payload (a few hundred bytes when few alerts are active).

### Feed content type summary

| Endpoint | Redirects to | API key | Actual body | `Content-Type` says |
| --- | --- | --- | --- | --- |
| `google_transit.zip` | versioned `.zip` on `bart.gov` | no | zip archive | `application/zip` |
| `tripupdate.aspx` | `https://api.bart.gov/...` | no | protobuf | `text/html` (wrong) |
| `alerts.aspx` | `https://api.bart.gov/...` | no | protobuf | `text/html` (wrong) |

---

## The GTFS-Realtime protobuf

We decode GTFS-Realtime with hand-written `#[derive(prost::Message)]` structs
(see [07-decisions.md](07-decisions.md) for why we did not use a crate). The
message definitions live in `src/gtfsrt.rs`.

### Message set to implement

```
FeedMessage
├── header: FeedHeader                  (field 1)
│   ├── gtfs_realtime_version: String   (field 1)
│   ├── incrementality: enum            (field 2)
│   └── timestamp: u64                  (field 3)
└── entity: Vec<FeedEntity>             (field 2)
    ├── id: String                      (field 1)
    ├── is_deleted: bool                (field 2)
    ├── trip_update: Option<TripUpdate> (field 3)
    ├── vehicle: Option<Vehicle>        (field 4)
    └── alert: Option<Alert>            (field 5)

TripUpdate
├── trip: Option<TripDescriptor>        (field 1)
├── stop_time_update: Vec<StopTimeUpdate> (field 2)
├── vehicle: Option<VehicleDescriptor>  (field 3)
├── timestamp: u64                      (field 4)
└── delay: i32                          (field 5)

StopTimeUpdate
├── stop_sequence: Option<u32>          (field 1)
├── arrival: Option<StopTimeEvent>      (field 2)
├── departure: Option<StopTimeEvent>    (field 3)
├── stop_id: Option<String>             (field 4)
└── schedule_relationship: enum         (field 5)

StopTimeEvent
├── delay: i32                          (field 1)
├── time: Option<u64>                   (field 2)   epoch seconds
└── uncertainty: i32                    (field 3)   seconds

TripDescriptor
├── trip_id: String                     (field 1)
├── route_id: Option<String>            (field 5)
└── direction_id: Option<u32>           (field 6)

Alert  (see the deviation note below)
├── active_period: Vec<TimeRange>       (field 1)
├── cause: enum                         (field 2)
├── effect: enum                        (field 3)
├── url: Option<TranslatedString>       (field 4)
├── header_text: Option<TranslatedString> (field 5)
├── description_text: Option<TranslatedString> (field 6)
├── tts_header_text: Option<TranslatedString>  (field 7)
├── tts_description_text: Option<TranslatedString> (field 8)
└── ... fields 10 and 11: see below

TranslatedString
└── translation: Vec<Translation>       (field 1)

Translation
├── text: String                        (field 1)
└── language: Option<String>            (field 2)
```

### ⚠ BART's alert feed is not spec-conformant, and is not self-consistent

Decoded with `protoc --decode_raw` against the live feed on 2026-10-19, a BART
alert entity looks like this:

```
2 {                                 # FeedEntity
  1: "BSA_560"                     # entity id
  5 {                              # field 5 = alert
    5 { 1 { 1: "BART" } }          # field 5  = header_text   -> "BART"
    8 { 1 { 1: "http://www.bart.gov/schedules/advisories"
            2 { 12: "en-US" } } }  # field 8  = tts_description_text -> URL
    10 { 1 { 1: "BART.gov Alert"
             2 { 12: "en-US" } } } # field 10 -> "BART.gov Alert"  (short text)
    11 { 1 { 1: "Expect 30-minute delays starting at 9pm each night ..."
             2 { 12: "en-US" } } } # field 11 -> long description
  }
}
```

Per the GTFS-Realtime spec, `Alert` field 10 is `created_at` (a `uint64`) and
field 11 is `update` (a `TimeRange`). BART puts **translated strings** there
instead: field 10 holds the short title and field 11 holds the full alert text.
(The `2 { 12: ... }` nesting is an artifact of `protoc --decode_raw` guessing
that the language string is a nested message; the real encoding is
`Translation { text = 1, language = 2 }`.)

There is a second, subtler problem. Working the bytes out by hand shows that
BART is **not consistent about the shape of its own text fields**:

- Field 5 (`header_text`) holds `0a 04 "BART"` — the `TranslatedString`'s
  `translation` entry is a **bare string**, not a `Translation` message.
- Fields 8, 10 and 11 hold `0a <len> <Translation>` — a properly nested
  `Translation { text, language }`.

Decoding field 5 as a `Translation` therefore fails outright, with
`delimited length exceeded`: the bytes `BART` are not a valid message, because
`0x42` reads as a length-delimited field claiming 65 bytes that are not
there.

**Consequence for the implementation:**

- The alert's text fields are held as **raw bytes** (`prost(bytes = "vec")`),
  not as `TranslatedString` messages, so that prost accepts whatever BART
  sends without interpreting it.
- A small tolerant extractor (`gtfsrt::alert_text`) then interprets them: it
  takes the first length-delimited field, tries to decode it as a
  `Translation`, and falls back to treating the raw bytes as the text if that
  fails. That handles both shapes.
- The alert text is then taken from a fallback chain: field 11 → field 6 →
  field 10 → field 5. The URL comes from field 8 → field 4.
- Decode *all* the fields regardless; a spec-conformant feed should still
  render correctly.
- **Acceptance test:** decode the live alerts feed and render a non-empty,
  human-readable line for every entity in it. A later agent should re-run this
  against the live feed, because BART could fix their generator.

### Trip updates *are* spec-conformant

A live sample decodes as expected:

```
2 {
  1: "1965550"                        # entity id == trip id
  3 {                                 # trip_update
    1 { 1: "1965550"  4: 0 }          # trip descriptor: trip_id, schedule_relationship
    2 {                              # stop_time_update
      2 { 1: 32  2: 1792437342  3: 30 }   # arrival:  delay=32s, time, uncertainty=30s
      3 { 1: 32  2: 1792437360  3: 30 }   # departure
      4: "A70-1"                     # stop_id (a platform id)
    }
    ...
  }
}
```

Note that BART only emits `stop_time_update` entries for a subset of the
trip's stops (in the sample, the tail of the run). Do not assume the realtime
feed contains every stop of every trip — fall back to the static schedule for
stops it does not mention.

Also note the `TripDescriptor` carries no `route_id`; the route must be joined
from the static feed via `trip_id`.

---

## On-disk layout

The feed is **not** stored in the repository. It lives in a per-user cache
directory, resolved by `paths.rs`:

```
$XDG_DATA_HOME/bart-rs/          (default: ~/.local/share/bart-rs)
├── feed/
│   └── current/                 the extracted 18 .txt files
│       ├── agency.txt
│       ├── calendar.txt
│       └── … (16 more)
├── archives/
│   └── 72.zip                   the raw download, named by feed_version
└── state.json                   bookkeeping written by `bart-rs update`
```

`state.json`:

```json
{
  "feed_version": "72",
  "downloaded_at": "2026-10-19T12:00:00Z",
  "last_checked_at": "2026-10-19T12:00:00Z",
  "source_url": "https://www.bart.gov/sites/default/files/2026-08/google_transit_20260810-20270108_v02.zip",
  "last_modified": "Tue, 04 Aug 2026 18:23:24 GMT"
}
```

`source_url` and `last_modified` together identify the release. `feed_version`
is recorded because it is worth showing, but it is never used to decide
whether anything needs downloading.

### Resolution order

1. `$BART_RS_DATA_DIR` — explicit override, used by the test suite and by
   anyone who wants the feed somewhere else.
2. `$XDG_DATA_HOME/bart-rs` — the XDG data directory.
3. `~/.local/share/bart-rs` — the XDG default.

### Why the user cache directory

The original project committed a copy of the feed to `data/`. That is wrong for
several reasons: it went stale silently (the committed copy was two years out
of date and nothing warned about it), it put a ~5 MB binary-ish blob in git
history, and it made it impossible to keep two feed versions around. The
tool manages its own data; the repository carries no feed.

---

## Update lifecycle

`bart-rs update` performs, in order:

1. **Probe** the permalink with a `HEAD` request, following redirects, to
   learn which release is on offer: its versioned filename and its
   `Last-Modified` header. If the server will not answer a `HEAD`, fall
   through to step 2 and decide there instead.
2. **Compare** the offered release against the one recorded in `state.json`.
   If they match and `--force` was not given, stop: the installed feed is
   already current. (Never compare `feed_version` — it never changes.)
3. **Fetch** the permalink with `reqwest` (blocking client, `rustls-tls`).
   Follow redirects. Capture the final URL and its `Last-Modified` header.
4. **Validate** that the payload is a zip archive and contains the 18 expected
   file names. Fail loudly if BART changed the file set.
5. **Store** the archive at `archives/<feed_version>.zip`. `feed_version` is
   read from the archive's own `feed_info.txt` — so the archive has to be
   opened before it can be named. (Read `feed_info.txt` out of the zip into
   memory, then write the archive.)
6. **Extract** to a temporary directory `feed/.staging-<pid>/`.
7. **Swap**: rename `feed/.staging-<pid>` → `feed/current`, replacing the
   previous `feed/current`. On Linux `rename` over an existing directory fails
   if the target is non-empty, so the sequence is:
   - `rename(feed/current, feed/.old-<pid>)` (if it exists)
   - `rename(feed/.staging-<pid>, feed/current)`
   - `remove_dir_all(feed/.old-<pid>)`
   If step 4 fails, restore `.old-<pid>` back to `feed/current`. A partially
   extracted feed is never visible to readers.
8. **Record** `state.json`.

`bart-rs update --check` performs step 1 only, then reports whether the
offered release differs from the installed one. It does not write anything
except the `last_checked_at` timestamp.

### Staleness rules

`bart-rs status` reports:

| Condition | Message |
| --- | --- |
| coverage window ended | **expired** — "the feed expired N days ago" |
| coverage window ends within 7 days | **expiring soon** |
| coverage window has not started | **not yet valid** |
| `last_checked_at` older than 7 days | **stale check** — "run `bart-rs update --check`" |
| no local feed at all | **missing** — "run `bart-rs update`" |
| a feed is installed but its window cannot be worked out | **unknown coverage** |

The expiry check uses the **calendar's** window, not `feed_info.txt`'s — see
the finding above. The guarantee BART actually offers is the calendar's, and
`feed_info.txt` does not describe the release it published.

### Times and timezones

All schedule arithmetic happens in `America/Los_Angeles` (BART's
`agency_timezone`), regardless of where the tool runs. `chrono-tz` is used for
this. A trip that runs past midnight is represented as seconds past midnight
*of its service day*, so `25:30:00` is a legitimate value and must not be
wrapped to `01:30:00` until it is being displayed for a human.

When a user passes `--at 01:30`, that is 01:30 **Pacific** on the chosen
service date. If the chosen service date is the previous calendar day (which is
what you want for the 00:00–04:00 window), the tool must not silently use
"today's" calendar service set.

**Rule:** the "current service date" is the Pacific date, except during
`[00:00, 04:00)` Pacific, when it is the *previous* date. The service day
rolls over at 04:00. Trips departing between midnight and 04:00 belong to the
previous service day. This is a detail that will be wrong if implemented
naively, so it gets its own unit test (see [06-testing.md](06-testing.md)).
