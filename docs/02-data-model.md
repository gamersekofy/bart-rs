# 02 — Data model

BART publishes 18 files in its GTFS feed. This document gives the exact schema
of each, the Rust type it maps to, and the BART-specific quirks that will bite
you if you do not know about them.

**All column lists and row counts were measured against feed version 72**
(`google_transit_20260810-20270108_v02.zip`, 2026-08-10) on 2026-10-19.

## General rules for every table

1. **Never index by column position.** Deserialize by header name. The
   `stops.txt` schema has already changed once (see [Stops](02-data-model.md#stops)).
2. **Missing columns are normal.** BART adds and removes columns between feed
   versions. Every non-essential field is `Option<T>` with `#[serde(default)]`.
3. **Renamed columns need `alias`.** Where a column was renamed, declare the
   current name with `rename` and the old name with `alias` so old feeds still
   parse. `stops.txt` is the only table that has needed this so far.
4. **Empty string means "not provided".** GTFS uses `""` for absent values,
   not a null. `Option<String>` with `csv` deserializes `""` as `Some("")`, so
   use a helper that maps empty strings to `None` (see `helper.rs`).
5. **Booleans and enums are `0`/`1` integers**, and are frequently empty.
6. **All 18 files parse as strict, uniform CSV.** No ragged rows, no embedded
   commas inside fields (verified). `csv::Reader` with default settings
   handles the quoting; the `csv` crate also strips surrounding quotes, so
   `trim_quotes` is only needed for hand-rolled parsing paths.

---

## The tables

### `agency.txt` — 1 row, 6 columns

`agency_id,agency_name,agency_url,agency_timezone,agency_lang,agency_phone`

BART has exactly one agency. `agency_timezone` is `America/Los_Angeles`; this is
the authoritative timezone for all schedule arithmetic.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Agency {
    #[serde(rename = "agency_id", default)]
    pub id: Option<String>,
    #[serde(rename = "agency_name")]
    pub name: String,
    #[serde(rename = "agency_url", default)]
    pub url: Option<String>,
    #[serde(rename = "agency_timezone", default)]
    pub timezone: Option<String>,
    #[serde(rename = "agency_lang", default)]
    pub lang: Option<String>,
    #[serde(rename = "agency_phone", default)]
    pub phone: Option<String>,
}
```

### `calendar.txt` — 11 rows, 10 columns

`service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date`

The recurring service patterns. `service_id` looks like
`2026_08_10-DX-MVS-Weekday-015`; the prefix is the feed date, not meaningful.
Day flags are `0`/`1`. Dates are `YYYYMMDD`.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Calendar {
    #[serde(rename = "service_id")]
    pub service_id: String,
    #[serde(rename = "monday")]   pub monday: bool,
    #[serde(rename = "tuesday")]  pub tuesday: bool,
    #[serde(rename = "wednesday")] pub wednesday: bool,
    #[serde(rename = "thursday")] pub thursday: bool,
    #[serde(rename = "friday")]   pub friday: bool,
    #[serde(rename = "saturday")] pub saturday: bool,
    #[serde(rename = "sunday")]   pub sunday: bool,
    #[serde(rename = "start_date")] pub start_date: NaiveDate,
    #[serde(rename = "end_date")]   pub end_date: NaiveDate,
}
```

`bool` deserialization from `0`/`1` is not built into `serde`; use a
`deserialize_with` helper `de_bool_01` (see `helper.rs`).

### `calendar_attributes.txt` — 11 rows, 2 columns

`service_id,service_description`

A **BART extension**, not part of core GTFS. Maps a service id to a human
label such as `Weekday`, `Saturday`, `Sunday`. Use this to label which
schedule pattern a result came from.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct CalendarAttribute {
    #[serde(rename = "service_id")]
    pub service_id: String,
    #[serde(rename = "service_description")]
    pub service_description: String,
}
```

### `calendar_dates.txt` — 20 rows, 3 columns

`service_id,date,exception_type`

Exceptions to `calendar.txt`. `exception_type` is `1` = service **added** on
that date, `2` = service **removed** on that date. Both occur in BART's feed
(e.g. a Sunday that falls on a holiday is added back as a weekday service).

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct CalendarDate {
    #[serde(rename = "service_id")]
    pub service_id: String,
    #[serde(rename = "date")]
    pub date: NaiveDate,
    #[serde(rename = "exception_type")]
    pub exception_type: ExceptionType,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum ExceptionType {
    #[serde(rename = "1")] Added,
    #[serde(rename = "2")] Removed,
}
```

### `directions.txt` — 14 rows, 3 columns

`route_id,direction_id,direction`

BART extension. Maps `(route_id, direction_id)` to a compass direction
(`North` / `South`). Note this file includes the two bus-bridge routes
(`BB-A`, `BB-B`), so it has 14 rows while `routes.txt` has 13.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Direction {
    #[serde(rename = "route_id")]
    pub route_id: String,
    #[serde(rename = "direction_id")]
    pub direction_id: u8,
    #[serde(rename = "direction")]
    pub direction: String,
}
```

### `fare_attributes.txt` — 2500 rows, 6 columns

`fare_id,price,currency_type,payment_method,transfers,transfer_duration`

One row per fare. `price` is a decimal string in USD (`"7.55"`). BART leaves
`transfers` and `transfer_duration` empty.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct FareAttribute {
    #[serde(rename = "fare_id")]
    pub fare_id: String,
    #[serde(rename = "price")]
    pub price: Decimal,      // see note
    #[serde(rename = "currency_type", default)]
    pub currency_type: Option<String>,
    #[serde(rename = "payment_method", default)]
    pub payment_method: Option<u8>,
    #[serde(rename = "transfers", default)]
    pub transfers: Option<String>,
    #[serde(rename = "transfer_duration", default)]
    pub transfer_duration: Option<u32>,
}
```

**Note on `Decimal`:** avoid a `rust_decimal` dependency. Parse the string into
an integer number of **cents** (`price_cents: i64`) — BART prices are exact to
two decimal places and this sidesteps float rounding in money output.

### `fare_rider_categories.txt` — 7500 rows, 3 columns

`fare_id,rider_category_id,price`

BART extension. Gives the discounted price of each fare for each rider
category. There are 3 rider categories, so each `fare_id` appears 3 times.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct FareRiderCategory {
    #[serde(rename = "fare_id")]
    pub fare_id: String,
    #[serde(rename = "rider_category_id")]
    pub rider_category_id: String,
    #[serde(rename = "price")]
    pub price_cents: i64,
}
```

<a id="fare-rules"></a>
### `fare_rules.txt` — 2500 rows, 5 columns

`fare_id,route_id,origin_id,destination_id,contains_id`

Which fare applies between which places. `route_id` and `contains_id` are empty
in BART's feed; `origin_id` and `destination_id` are **station** ids
(`SSAN`, `SBRN`, `SFIA`, `MLBR`, `LAKE`, …) — *not* platform ids. A
`origin_id == destination_id` row is the "anywhere to anywhere" base fare.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct FareRule {
    #[serde(rename = "fare_id")]
    pub fare_id: String,
    #[serde(rename = "route_id", default)]
    pub route_id: Option<String>,
    #[serde(rename = "origin_id", default)]
    pub origin_id: Option<String>,
    #[serde(rename = "destination_id", default)]
    pub destination_id: Option<String>,
    #[serde(rename = "contains_id", default)]
    pub contains_id: Option<String>,
}
```

**Lookup rule:** resolve the user's `--from`/`--to` to a *station* id first
(see [Stops](02-data-model.md#stops)), then look for an exact `(origin_id, destination_id)`
match, then the reverse, then fall back to the `origin_id == destination_id`
row.

### `feed_info.txt` — 1 row, 6 columns

`feed_publisher_name,feed_publisher_url,feed_lang,feed_start_date,feed_end_date,feed_version`

The feed's own metadata. `feed_start_date` / `feed_end_date` are `YYYYMMDD`;
`feed_version` is an integer string (`"72"`).

> **Two of these three fields cannot be trusted.**
>
> - `feed_start_date` and `feed_end_date` describe the *previous* release.
>   BART does not update them when it publishes a new feed, so they are not
>   the feed's coverage window. The window comes from `calendar.txt` and
>   `calendar_dates.txt` instead — see
>   [03-data-sources.md](03-data-sources.md#feed-info-dates).
> - `feed_version` is the same (`72`) in every release BART has published,
>   so it cannot be used to tell two releases apart. The release is
>   identified by the versioned filename the permalink redirects to.

The struct still parses all of them: they are worth showing, and the
discrepancy is worth making visible. They are just not used as the source
of truth for anything.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct FeedInfo {
    #[serde(rename = "feed_publisher_name")]
    pub publisher_name: String,
    #[serde(rename = "feed_publisher_url", default)]
    pub publisher_url: Option<String>,
    #[serde(rename = "feed_lang", default)]
    pub lang: Option<String>,
    #[serde(rename = "feed_start_date", default)]
    pub start_date: Option<NaiveDate>,
    #[serde(rename = "feed_end_date", default)]
    pub end_date: Option<NaiveDate>,
    #[serde(rename = "feed_version", default)]
    pub version: Option<String>,
}
```

### `realtime_routes.txt` — 12 rows, 4 columns

`route_id,realtime_enabled,realtime_routename,realtime_routecode`

BART extension. `realtime_enabled` is `0`/`1`; only some routes carry live
updates. The name/code columns are empty in the current feed.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct RealtimeRoute {
    #[serde(rename = "route_id")]
    pub route_id: String,
    #[serde(rename = "realtime_enabled", default)]
    pub realtime_enabled: bool,
    #[serde(rename = "realtime_routename", default)]
    pub realtime_routename: Option<String>,
    #[serde(rename = "realtime_routecode", default)]
    pub realtime_routecode: Option<String>,
}
```

### `rider_categories.txt` — 3 rows, 2 columns

`rider_category_id,rider_category_description`

BART extension. Current values: `5` = `Youth Clipper`, `2` =
`Senior/Disabled Clipper`, `16` = `Clipper START`.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct RiderCategory {
    #[serde(rename = "rider_category_id")]
    pub rider_category_id: String,
    #[serde(rename = "rider_category_description")]
    pub description: String,
}
```

### `route_attributes.txt` — 14 rows, 4 columns

`route_id,category,subcategory,running_way`

BART extension, entirely opaque integer codes. `category` is `2` for
heavy-rail subway routes and `3` for the connector/transfer lines. There is no
published key for these values; treat them as opaque.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct RouteAttribute {
    #[serde(rename = "route_id")]
    pub route_id: String,
    #[serde(rename = "category", default)]
    pub category: Option<u32>,
    #[serde(rename = "subcategory", default)]
    pub subcategory: Option<u32>,
    #[serde(rename = "running_way", default)]
    pub running_way: Option<u32>,
}
```

### `routes.txt` — 14 rows, 8 columns

`route_id,route_short_name,route_long_name,route_desc,route_type,route_url,route_color,route_text_color`

`route_type` is the GTFS enum (`1` = Subway, `3` = Bus). `route_color` /
`route_text_color` are six-digit hex strings **without** a leading `#`.

BART's 14 routes are directional pairs — `1`/`2` are Yellow S/N, `3`/`4`
Orange, `5`/`6` Green, `7`/`8` Red, `11`/`12` Blue, `19`/`20` Grey — plus two
bus bridges `BB-A` / `BB-B`.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Route {
    #[serde(rename = "route_id")]
    pub id: String,
    #[serde(rename = "route_short_name", default)]
    pub short_name: Option<String>,
    #[serde(rename = "route_long_name", default)]
    pub long_name: Option<String>,
    #[serde(rename = "route_desc", default)]
    pub desc: Option<String>,
    #[serde(rename = "route_type", default)]
    pub route_type: Option<RouteType>,
    #[serde(rename = "route_url", default)]
    pub url: Option<String>,
    #[serde(rename = "route_color", default)]
    pub color: Option<String>,
    #[serde(rename = "route_text_color", default)]
    pub text_color: Option<String>,
}
```

### `shapes.txt` — 27,893 rows, 5 columns

`shape_id,shape_pt_lat,shape_pt_lon,shape_pt_sequence,shape_dist_traveled`

Geographic polyline points, grouped by `shape_id` (28 distinct shapes). Points
must be ordered by `shape_pt_sequence`; `shape_dist_traveled` is in metres and
is optional in the spec.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Shape {
    #[serde(rename = "shape_id")]
    pub shape_id: String,
    #[serde(rename = "shape_pt_lat")]
    pub lat: f64,
    #[serde(rename = "shape_pt_lon")]
    pub lon: f64,
    #[serde(rename = "shape_pt_sequence")]
    pub sequence: u32,
    #[serde(rename = "shape_dist_traveled", default)]
    pub dist_traveled: Option<f64>,
}
```

<a id="stops"></a>
### `stops.txt` — 287 rows, **11** columns

`stop_id,stop_code,stop_name,stop_desc,stop_lat,stop_lon,zone_id,stop_url,location_type,parent_station,platform_code`

> **This table changed.** The feed committed in the old `data/` directory (feed
> version 69, January 2024) had **10** columns with `plc_url` instead of
> `stop_url` and no `platform_code`. A positional parser silently misreads the
> current feed. Use `alias`.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Stop {
    #[serde(rename = "stop_id")]
    pub id: String,
    #[serde(rename = "stop_code", default)]
    pub code: Option<String>,
    #[serde(rename = "stop_name", default)]
    pub name: Option<String>,
    #[serde(rename = "stop_desc", default)]
    pub desc: Option<String>,
    #[serde(rename = "stop_lat", default)]
    pub lat: Option<f64>,
    #[serde(rename = "stop_lon", default)]
    pub lon: Option<f64>,
    #[serde(rename = "zone_id", default)]
    pub zone_id: Option<String>,
    #[serde(rename = "stop_url", alias = "plc_url", default)]
    pub url: Option<String>,
    #[serde(rename = "location_type", default)]
    pub location_type: LocationType,
    #[serde(rename = "parent_station", default)]
    pub parent_station: Option<String>,
    #[serde(rename = "platform_code", default)]
    pub platform_code: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum LocationType {
    #[serde(rename = "0")] StopOrPlatform,
    #[serde(rename = "1")] Station,
    #[serde(rename = "2")] EntranceOrExit,
    #[serde(rename = "3")] GenericNode,
    #[serde(rename = "4")] BoardingArea,
}
```

#### The three-level stop hierarchy

This is the single most important structural fact about BART's data.

| Level | `location_type` | Example id | Example name | Count |
| --- | --- | --- | --- | --- |
| Station | `1` | `LAKE` | Lake Merritt | 50 |
| Platform | `0` | `A10-1` | Lake Merritt | 105 |
| Entrance/exit | `2` | `LAKE_1` | B1 NW 8th & Oak St … | 132 |

- A **platform**'s `parent_station` is the **station** id (`A10-1` → `LAKE`).
- An **entrance**'s `parent_station` is also the **station** id (`LAKE_1` → `LAKE`).
- Platform and entrance ids are *not* derived from the station id: the station
  is `LAKE` but its platforms are `A10-1`/`A10-2`. BART's legacy platform codes
  and its modern station codes coexist.
- Verified: all 105 platforms and all 132 entrances resolve to an existing
  station, and all 57,188 `stop_times` rows reference an existing stop.

`stop_times.txt` and `transfers.txt` reference **platform** ids. `fare_rules.txt`
references **station** ids. The realtime feed references **platform** ids. Any
command taking a user-supplied stop must accept either and normalise.

### `stop_times.txt` — 57,188 rows, 9 columns

`trip_id,arrival_time,departure_time,stop_id,stop_sequence,stop_headsign,pickup_type,drop_off_type,shape_distance_traveled`

The biggest table. `arrival_time` / `departure_time` are `HH:MM:SS` and **may
exceed 24 hours** (a trip that departs 00:30 on the next day is written as
`25:30:00`). Store these as **seconds since midnight** (`u32`), not
`chrono::NaiveTime`, which cannot represent values past 24:00.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct StopTime {
    #[serde(rename = "trip_id")]
    pub trip_id: String,
    #[serde(rename = "arrival_time", default, deserialize_with = "de_gtfs_time")]
    pub arrival_time: Option<u32>,
    #[serde(rename = "departure_time", default, deserialize_with = "de_gtfs_time")]
    pub departure_time: Option<u32>,
    #[serde(rename = "stop_id")]
    pub stop_id: String,
    #[serde(rename = "stop_sequence")]
    pub stop_sequence: u32,
    #[serde(rename = "stop_headsign", default)]
    pub stop_headsign: Option<String>,
    #[serde(rename = "pickup_type", default)]
    pub pickup_type: Option<u8>,
    #[serde(rename = "drop_off_type", default)]
    pub drop_off_type: Option<u8>,
    #[serde(rename = "shape_distance_traveled", default)]
    pub shape_distance_traveled: Option<f64>,
}
```

`shape_distance_traveled` is present on every row in BART's feed but is
optional in the spec; keep it `Option<f64>`.

### `transfers.txt` — 40 rows, 8 columns

`from_stop_id,to_stop_id,transfer_type,min_transfer_time,from_route_id,to_route_id,from_trip_id,to_trip_id`

`transfer_type` is GTFS's `0` (recommended transfer point), `1` (timed
transfer), `2` (no transfer), `3` (no transfer on a specific route), `4` /
`5` (passengers only). BART uses `1` and `2`. The `from_trip_id` /
`to_trip_id` columns are empty in BART's feed.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Transfer {
    #[serde(rename = "from_stop_id")]
    pub from_stop_id: String,
    #[serde(rename = "to_stop_id")]
    pub to_stop_id: String,
    #[serde(rename = "transfer_type", default)]
    pub transfer_type: Option<u8>,
    #[serde(rename = "min_transfer_time", default)]
    pub min_transfer_time: Option<u32>,
    #[serde(rename = "from_route_id", default)]
    pub from_route_id: Option<String>,
    #[serde(rename = "to_route_id", default)]
    pub to_route_id: Option<String>,
    #[serde(rename = "from_trip_id", default)]
    pub from_trip_id: Option<String>,
    #[serde(rename = "to_trip_id", default)]
    pub to_trip_id: Option<String>,
}
```

### `trips.txt` — 4,417 rows, 10 columns

`route_id,service_id,trip_id,trip_headsign,direction_id,block_id,shape_id,trip_load_information,wheelchair_accessible,bikes_allowed`

`trip_headsign` is the **destination** the train is heading toward, e.g.
`"OAK Airport / SF / Daly City"`. `direction_id` is `0` or `1`.
`wheelchair_accessible` and `bikes_allowed` are `0`/`1`/`2` (GTFS's three-state
availability enum).

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Trip {
    #[serde(rename = "route_id")]
    pub route_id: String,
    #[serde(rename = "service_id")]
    pub service_id: String,
    #[serde(rename = "trip_id")]
    pub trip_id: String,
    #[serde(rename = "trip_headsign", default)]
    pub trip_headsign: Option<String>,
    #[serde(rename = "direction_id", default)]
    pub direction_id: Option<u8>,
    #[serde(rename = "block_id", default)]
    pub block_id: Option<String>,
    #[serde(rename = "shape_id", default)]
    pub shape_id: Option<String>,
    #[serde(rename = "trip_load_information", default)]
    pub trip_load_information: Option<String>,
    #[serde(rename = "wheelchair_accessible", default)]
    pub wheelchair_accessible: Option<u8>,
    #[serde(rename = "bikes_allowed", default)]
    pub bikes_allowed: Option<u8>,
}
```

---

## Row counts at a glance

| Table | Rows | Notes |
| --- | ---: | --- |
| `agency.txt` | 1 | BART only |
| `calendar.txt` | 11 | weekly patterns |
| `calendar_attributes.txt` | 11 | BART extension, human labels |
| `calendar_dates.txt` | 20 | holiday exceptions |
| `directions.txt` | 14 | includes 2 bus-bridge routes |
| `fare_attributes.txt` | 2,500 | one per fare |
| `fare_rider_categories.txt` | 7,500 | 3 per fare |
| `fare_rules.txt` | 2,500 | origin × destination |
| `feed_info.txt` | 1 | |
| `realtime_routes.txt` | 12 | |
| `rider_categories.txt` | 3 | |
| `route_attributes.txt` | 14 | opaque codes |
| `routes.txt` | 14 | 12 subway directions + 2 bus bridges |
| `shapes.txt` | 27,893 | 28 shapes |
| `stops.txt` | 287 | 50 stations / 105 platforms / 132 entrances |
| `stop_times.txt` | 57,188 | the big one |
| `transfers.txt` | 40 | |
| `trips.txt` | 4,417 | |

## Cross-table relationships

```
agency          ── (1) ─────────────────────────────────────┐
                                                          │
calendar       ── service_id ──▶ calendar_attributes        │
     │                                                     │
     └── service_id ──▶ calendar_dates ──▶ (service on a date)
                           │
trips ── service_id ─────┘
  │
  ├── route_id ──▶ routes ──▶ route_attributes
  │                   │
  │                   └──▶ directions (route_id, direction_id)
  │
  ├── shape_id ──▶ shapes
  │
  └── trip_id ──▶ stop_times ──▶ stops (platform ids)
                                  │       │
transfers ── from/to_stop_id ──────┘       └──▶ stops (station id via parent)
                                                    ▲
fare_rules ── origin_id / destination_id ─────────────┘
fare_attributes ── fare_id ──▶ fare_rules
fare_rider_categories ── fare_id ──▶ fare_attributes
                              └──▶ rider_categories
```

## Types shared across modules

These small helpers belong in `helper.rs`, not in a table module, because more
than one table needs them:

- `de_gtfs_time` — `"HH:MM:SS"` (possibly `> 24:00:00`) → `Option<u32>` seconds.
- `de_gtfs_date` — `"YYYYMMDD"` → `Option<NaiveDate>`.
- `de_bool_01` — `"0"` / `"1"` → `bool`.
- `de_opt_string` — `""` → `None`, otherwise `Some(s)`.
- `de_cents` — `"7.55"` → `755` (`i64`).
- `trim_quotes` — retained from the original code; only needed for the legacy
  hand-rolled parsing path, not for `csv`-based parsing.
