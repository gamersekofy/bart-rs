# 04 — CLI specification

The `bart-rs` binary. One command per question. Every command that reads the
feed works offline once `bart-rs update` has been run.

> Output samples below are illustrative. The samples that quote real BART data
> (routes, stops, fares) are accurate as of feed version 72; the schedule
> samples use round numbers. Regenerate them from real output as part of
> implementation and correct them here if they drift.

## Synopsis

```
bart-rs [GLOBAL OPTIONS] <COMMAND> [ARGS]
```

### Global options

| Flag | Effect |
| --- | --- |
| `--no-color` | Disable ANSI colour. Also honoured: the `NO_COLOR` environment variable (https://no-color.org). |
| `--data-dir <PATH>` | Override the feed cache location. Same as `BART_RS_DATA_DIR`. |
| `--json` | Emit machine-readable JSON instead of formatted text. **Available on:** `routes`, `stops`, `route`, `stop`, `arrivals`, `departures`, `trip`, `fare`, `alerts`, `table`. |
| `-h`, `--help` | Per-command help. |
| `-V`, `--version` | Version. |

`--json` is the same flag everywhere; it is not a per-subcommand flag. When a
command has no natural JSON representation (`update`, `status`), `--json` is
rejected with a clear error rather than silently ignored.

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success. Also used when the output pipe is closed early — `bart-rs routes \| head -1` exits quietly, as any Unix tool should, rather than panicking with "Broken pipe". |
| `1` | The command failed. The error is printed to stderr in its `Display` form. |
| `2` | The command line was wrong. `clap` prints the usage message. |

### Tables

Every command that prints a list prints a table with aligned columns.

Column widths are measured in **visible** characters. A colour swatch is one
character on screen but about twenty bytes of escape sequence, so measuring
the raw string would push every column after the swatch out of alignment.

The swatch column has no heading of its own; it is the empty column before
the route's short name.

---

## `bart-rs update`

Download the current BART GTFS feed and install it into the cache.

```
bart-rs update [--check] [--force]
```

| Flag | Effect |
| --- | --- |
| `--check` | Do not download. Report whether a newer feed is available and exit. |
| `--force` | Download and install even if the remote release matches the installed one. |

Output:

```
$ bart-rs update
The installed feed is already up to date (google_transit_20260810-20270108_v02.zip).
```

```
$ bart-rs update
A newer feed is available (google_transit_20260810-20270108_v02.zip); downloading it.
Downloading https://www.bart.gov/dev/schedules/google_transit.zip …
  resolved to https://www.bart.gov/sites/default/files/2026-08/google_transit_20260810-20270108_v02.zip
  downloaded 892312 bytes
Feed version 72
  covers 2026-08-10 → 2027-01-10
  stored /home/you/.local/share/bart-rs/archives/72.zip
  extracted 18 tables
Installed to /home/you/.local/share/bart-rs/feed/current
Done.
```

The `covers` line is the calendar's window, not `feed_info.txt`'s — see
`03-data-sources.md`.

```
$ bart-rs update --check
Checking https://www.bart.gov/dev/schedules/google_transit.zip
Installed release: google_transit_20260810-20270108_v02.zip
Feed version:      72
Offered release:   google_transit_20260810-20270108_v02.zip
The installed feed is up to date.
```

Exit codes: `0` success (or "already up to date"), `1` error.

---

## `bart-rs status`

Feed metadata and staleness.

```
bart-rs status
```

```
$ bart-rs status
BART GTFS feed
  version        72
  release        google_transit_20260810-20270108_v02.zip
  coverage       2026-08-10 → 2027-01-10
  publisher      Bay Area Rapid Transit
  language       en
                 note: feed_info.txt says 2026-01-12 → 2026-08-30; BART leaves it stale
  downloaded     2026-10-19T19:39:31.010512220+00:00
  last checked   2026-10-19T19:39:31.010521567+00:00
  location       /home/you/.local/share/bart-rs/feed/current

  routes         14
  stations       50
  platforms      105
  entrances      132
  trips          4417
  stop times     57188
  fares          2500

The feed is current (18 of 18 tables loaded).
```

Warning lines (at most one of each) appear when the conditions in
[03-data-sources.md](03-data-sources.md#staleness-rules) are met.

---

## `bart-rs routes`

List BART's routes.

```
bart-rs routes
```

```
$ bart-rs routes
ID    SHORT     LONG                                       TYPE       COLOR
1     Yellow-S  Antioch to SF Int'l Airport SFO/Millbrae   Subway  █  FFFF33
2     Yellow-N  Millbrae/SF Int'l SFO to Antioch           Subway  █  FFFF33
3     Orange-N  Berryessa/North San Jose to Richmond       Subway  █  FF9933
4     Orange-S  Richmond to Berryessa/North San Jose       Subway  █  FF9933
5     Green-S   Berryessa/North San Jose to Daly City      Subway  █  339933
6     Green-N   Daly City to Berryessa/North San Jose      Subway  █  339933
7     Red-S     Richmond to SF Int'l Airport SFO/Millbrae  Subway  █  FF0000
8     Red-N     Millbrae/SF Int'l SFO to Richmond          Subway  █  FF0000
11    Blue-S    Dublin/Pleasanton to Daly City             Subway  █  0099CC
12    Blue-N    Daly City to Dublin/Pleasanton             Subway  █  0099CC
19    Grey-N    Oakland Int'l Airport OAK to Coliseum      Subway  █  B0BEC7
20    Grey-S    Coliseum to Oakland Int'l Airport OAK      Subway  █  B0BEC7
BB-A  BridgeA   Bus Bridge                                 Bus     █  000000
BB-B  BridgeB   Bus Bridge                                 Bus     █  000000
```

The swatch column is rendered in the route's own colour when colour is
enabled. BART publishes the colour as a six-digit hex string, which is
emitted as a 24-bit colour escape (`\e[38;2;R;G;Bm`) on terminals that
advertise `COLORTERM=truecolor`, and as the nearest of the sixteen ANSI
colours otherwise.

---

## `bart-rs stops`

List or search stations.

```
bart-rs stops [QUERY] [--all] [--limit N]
```

| Argument / flag | Effect |
| --- | --- |
| `QUERY` | Case-insensitive substring match against station name, id, and stop code. |
| `--all` | Include platforms and entrances. Without it, only **stations** (`location_type = 1`) are listed. |
| `--limit N` | Cap the number of results. Default 50 when a `QUERY` is given, unlimited otherwise. |

```
$ bart-rs stops lake
CODE  NAME                 PLATFORMS  ZONE
LAKE  Lake Merritt                2  LAKE

$ bart-rs stops "millbrae"
CODE  NAME                                        PLATFORMS  ZONE
MLBR  Millbrae (Caltrain Transfer Platform)              1  MLBR
```

Matching is fuzzy in a useful way: a query matches a station if it is a
substring of the station's **name**, **id**, or **stop code**, case-insensitively.
`bart-rs stops EMBR` finds Embarcadero by code; `bart-rs stops embarcadero`
finds it by name.

---

## `bart-rs route <ROUTE>`

Details for one route.

```
bart-rs route <ROUTE>
```

`ROUTE` accepts a route id (`1`, `BB-A`) or a short name (`Yellow-S`).

```
$ bart-rs route 5
Green-S  Berryessa/North San Jose to Daly City

  type            Subway
  color           339933
  url             https://www.bart.gov/schedules/bylineresults?route=5
  direction       South
  attributes      category 2, subcategory 201, running way 1
  realtime        enabled

  Stations served
    1  Berryessa/North San Jose   BERY
    2  Milpitas                    MLPT
    3  Warm Springs/South San Jose  WARM
    4  Dublin/Pleasanton          DUBL
    5  West Dublin/Pleasanton     WDUB
    6  Castro Valley              CAST
    7  Dublin/Pleasanton          DUBL
    …
```

`Stations served` is derived by taking the longest stop-time sequence among the
route's trips (see the note in [02-data-model.md](02-data-model.md) about
directional pairs: routes 1 and 2 are the two directions of the same Yellow
line, and each has its own stop order).

---

## `bart-rs stop <STOP>`

Details for one station.

```
bart-rs stop <STOP>
```

`STOP` accepts a station id (`LAKE`), a platform id (`A10-1`), or a name
(`Lake Merritt`).

```
$ bart-rs stop LAKE
Lake Merritt

  id          LAKE
  code        902109
  zone        LAKE
  location    37.797322, -122.265247
  url         https://www.bart.gov/stations/lake

  Platforms
ID     PLATFORM
A10-1  1
A10-2  2

  Entrances
    B1 NW 8th & Oak St Jack London Sq Entrance / Exit
    B2 SW 8th St, Laney College Entrance / Exit
    A2 SE 9th & Oak St Entrance / Exit
    A1 NE Chinatown Entrance / Exit

  Lines serving this station
       Blue-S   Blue-N   Orange-N  Orange-S  Green-S  Green-N
```

The lines are listed with a swatch in the line's own colour, so the lines
serving a station can be picked out at a glance:

```
  Lines serving this station
  █   Blue-S    Dublin/Pleasanton to Daly City
  █   Blue-N    Daly City to Dublin/Pleasanton
  █   Orange-N  Berryessa/North San Jose to Richmond
  █   Orange-S  Richmond to Berryessa/North San Jose
  █   Green-S   Berryessa/North San Jose to Daly City
  █   Green-N   Daly City to Berryessa/North San Jose
```

---

## `bart-rs arrivals <STOP>`

Scheduled arrivals at a stop.

```
bart-rs arrivals <STOP> [--at HH:MM] [--date YYYY-MM-DD] [--limit N] [--realtime]
```

| Flag | Effect |
| --- | --- |
| `--at HH:MM` | Show arrivals at or after this time. Defaults to "now" in Pacific time. |
| `--date YYYY-MM-DD` | Service date. Defaults to the current Pacific service date (see [03-data-sources.md](03-data-sources.md#times-and-timezones)). |
| `--limit N` | Maximum rows. Default 10. |
| `--realtime` | Also fetch the GTFS-Realtime trip update feed and overlay live delays. |

```
$ bart-rs arrivals EMBR --at 17:30
TIME       ROUTE     DESTINATION                          PLATFORM
17:31  █   Yellow-S  San Francisco International Airport  M16-1
17:31  █   Yellow-N  SFO / SF / Pittsburg/Bay Point       M16-2
17:36  █   Blue-S    OAK Airport / SF / Daly City         M16-1
17:37  █   Green-N   SF / OAK Airport / Berryessa         M16-2
17:40  █   Yellow-S  San Francisco International Airport  M16-1
```

The swatch column holds the route's own colour, so the lines can be told
apart at a glance. It is empty in `--json` output, which carries the
`route_id` instead.

With `--realtime`:

```
$ bart-rs arrivals EMBR --realtime --limit 3
TIME       ROUTE     DESTINATION                           PLATFORM  STATUS
12:45  █   Blue-N    SF / OAK Airport / Dublin/Pleasanton  M16-2     on time
12:45  █   Green-S   OAK Airport / SF / Daly City          M16-1     +1 min late
12:47  █   Red-S     SF / SFO Airport / Millbrae           M16-1     on time
```

Realtime rows that cannot be matched to a scheduled trip are shown with the
time the realtime feed reports, marked `live`.

Delays are reported by the realtime feed in seconds. A delay of less than a
minute is shown in seconds (`+33 sec late`) rather than rounded away to
`+0 min late`.

---

## `bart-rs departures <STOP>`

Scheduled departures from a stop. Same flags as `arrivals`.

```
$ bart-rs departures EMBR --at 17:30
TIME       ROUTE     DESTINATION                           PLATFORM
17:32  █   Red-S     SF / SFO Airport / Millbrae            A10-1
17:36  █   Blue-S    OAK Airport / SF / Daly City          A10-1
```

---

## `bart-rs trip <TRIP_ID>`

The full stop-time sequence of one trip.

```
bart-rs trip <TRIP_ID>
```

```
$ bart-rs trip 1965366
Trip 1965366

  route         █ Green-S (5)
  service       2026_08_10-DX-MVS-Weekday-015
  headsign      Daly City
  direction     1
  wheelchair    1
  bikes         1
  shape         005B_shp

  SEQ  STOP   NAME                ARRIVE    DEPART
    0  A10-1  Lake Merritt        05:52:00  05:52:00
    1  A20-1  Fruitvale           05:59:00  05:59:00
    …
```

---

## `bart-rs fare --from <STOP> --to <STOP>`

Fare between two stations.

```
bart-rs fare --from <STOP> --to <STOP> [--category <ID>]
```

| Flag | Effect |
| --- | --- |
| `--category <ID>` | Show the price for a specific rider category (`5` = Youth Clipper, `2` = Senior/Disabled Clipper, `16` = Clipper START). Without it, all categories are shown. |

```
$ bart-rs fare --from LAKE --to SFIA
Lake Merritt → San Francisco International Airport

  Regular fare   $12.20
RIDER CATEGORY           ID  PRICE
Youth Clipper            5   $6.10
Senior/Disabled Clipper  2   $4.55
Clipper START            16  $6.10
```

`--from` and `--to` accept station ids, platform ids, or names, and are
normalised to **station** ids before lookup (see the `fare_rules` note in
[02-data-model.md](02-data-model.md#fare_rulestxt--2500-rows-5-columns)).

---

## `bart-rs alerts`

Current service alerts, from the GTFS-Realtime alerts feed. This is always
live; there is no static equivalent.

```
bart-rs alerts [--route <ROUTE_ID>] [--stop <STOP>]
```

```
$ bart-rs alerts
BART.gov Alert
  Expect 30-minute delays starting at 9pm each night on Sunday-Thursday for
  riders traveling between Milbrae, SFO, and Daly City stations due to planned
  train control modernization construction work.

  more info: http://www.bart.gov/schedules/advisories
```

---

## `bart-rs table <NAME>`

Dump a raw GTFS table. Primarily a debugging and inspection tool, and the
quickest way to confirm that a table parsed correctly.

```
bart-rs table <NAME> [--limit N]
```

`NAME` is the GTFS file name without the extension, and must be one of:

```
agency            calendar          calendar_attributes  calendar_dates
directions        fare_attributes   fare_rider_categories fare_rules
feed_info         realtime_routes   rider_categories      route_attributes
routes            shapes            stops                 stop_times
transfers         trips
```

```
$ bart-rs table stops --limit 3
stop_id  stop_code  stop_name  stop_desc  stop_lat    stop_lon   zone_id  stop_url  location_type  parent_station  platform_code
A10-1    902101     Lake Merritt            37.797296  -122.265287  LAKE              0              LAKE             1
A10-2    902102     Lake Merritt            37.797357  -122.265232  LAKE              0              LAKE             2
A20-1    902201     Fruitvale               37.774818  -122.224121  FTVL              0              FTVL             1
```

---

## Command → table dependency map

A command only needs the tables it touches. The `Feed` loads all 18, but this
table documents intent (and is a hint for future lazy-loading work).

| Command | Tables used |
| --- | --- |
| `update` | `feed_info` (post-extract) |
| `status` | `feed_info`, `agency`, `routes`, `stops`, `trips`, `stop_times` (counts) |
| `routes` | `routes` |
| `stops` | `stops` |
| `route` | `routes`, `route_attributes`, `directions`, `realtime_routes`, `trips`, `stop_times`, `stops` |
| `stop` | `stops`, `routes`, `trips`, `stop_times` |
| `arrivals` / `departures` | `stops`, `trips`, `stop_times`, `calendar`, `calendar_dates`, `routes`, `calendar_attributes` |
| `trip` | `trips`, `stop_times`, `stops`, `routes`, `calendar_attributes` |
| `fare` | `stops`, `fare_rules`, `fare_attributes`, `fare_rider_categories`, `rider_categories` |
| `alerts` | (realtime only) |
| `table` | the one requested |
