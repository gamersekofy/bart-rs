# 06 — Decisions

Architecture decision records. Each entry states the decision, the reasoning,
and the alternatives that were rejected. If you change one of these, add a new
entry rather than editing the old one — the history is the point.

Format: **Status** (Accepted / Superseded) · **Date** · **Context**.

---

## D1 — Parse GTFS by hand with `csv` + `serde`, not with `gtfs-structures`

**Status:** Accepted · 2026-10-19 · amended 2026-10-21 (see D28)

**Context.** `gtfs-structures` 0.42 was already a declared dependency of the
original project, but it was never used. It parses the standard GTFS files
correctly. It does *not* cover the five BART-only extension files
(`calendar_attributes`, `directions`, `fare_rider_categories`,
`realtime_routes`, `route_attributes`), and it owns the data model, which
makes it awkward to extend.

`rider_categories.txt` is a sixth file that `gtfs-structures` cannot read
from BART's feed, but for a different reason: it is a core GTFS file that
BART publishes with a field name the specification does not use. That is
D28. This entry originally counted `rider_categories` as a BART extension,
which conflated a file BART invented with a file BART publishes
non-conformantly, and was amended when D28 was written.

**Decision.** Hand-roll the parsing with the `csv` and `serde` crates, one
module per GTFS table, keeping the original project's `src/attributes/`
layout. Keep `gtfs-structures` as a **dev-dependency** only, as a
cross-validation oracle in the test suite, with `default-features = false`.

**Why.**

- The project's stated purpose is to practise Rust. This is the part of the
  problem where the practice is.
- The BART-only extension tables have no equivalent in `gtfs-structures`
  anyway, so a hand-rolled layer is needed regardless. Doing all 18 the same
  way is more consistent than doing 13 one way and 5 another.
- Header-driven deserialization is what makes the `stops.txt` schema change a
  non-event. A library that owns the schema makes that a library upgrade.
- The `gtfs-structures` data model is opinionated in ways that fight the
  requirements here (it merges `stop_times` into `Trip` and hides them, which
  makes "arrivals at a stop" awkward to express).

**Rejected.**

- *Use `gtfs-structures` for the 13 spec files.* Less code, but a split
  mental model and no path to the extension files.
- *Use `gtfs-structures` for everything and accept the missing extensions.*
  Would leave 5 of 18 tables unhandled, close to a third of the job.
- *Parse with `split(',')` as the original did.* Broken: breaks on quoted
  fields, treats the header as data, and is position-dependent.

**Cost.** About 1,100 lines of struct definitions and `parse_*` functions.
Mitigated by `csv`'s `serde` support making each table roughly 20 lines.

**Follow-up.** `gtfs-structures`'s default feature is `read-url`, which pulls
in `reqwest` → `native-tls` → `openssl-sys`. As a dev-dependency it is
declared with `default-features = false`, which drops all of that: the
project's only OpenSSL dependency was the test oracle, and the dev shell no
longer needs `openssl` or `pkg-config`.

---

## D2 — One crate with a library and a binary, not a separate library crate

**Status:** Accepted · 2026-10-19

**Context.** The project was a binary-only crate whose entire implementation
lived in `src/main.rs` and `src/attributes/`. The goal is for the heavy
lifting to be reusable by other Rust projects.

**Decision.** A single package with `src/lib.rs` (crate `bart_rs`) and
`src/main.rs` (binary `bart-rs`). The binary is a five-line adapter.

**Why.** Cargo supports this with no ceremony. The library is the product and
the CLI is one front end over it, so the two concerns separate without the
overhead of two packages. Integration tests get to exercise the real API
rather than scraping a process's stdout.

**Rejected.**

- *Two separate crates (`bart-rs-core` + `bart-rs-cli`).* Doubles the
  ceremony — two `Cargo.toml`s, a path dependency, two version bumps — for no
  benefit at this size.
- *Stay binary-only.* Makes the library unreusable and forces tests to spawn a
  process. The user explicitly raised the library option, so this was a
  deliberate answer.
- *Start a brand new repository.* The existing history is three commits; there
  is nothing worth preserving beyond it, and keeping it means the README
  checklist and the issue history survive. The user offered this as a
  fallback; it was not needed because the conversion is cheap.

**Note.** If a second consumer ever needs a *different* dependency profile
(for example, a WASM build that cannot use `reqwest`), the `cli` and
`download` feature flags already provide the seam, and only then is a split
warranted.

---

## D3 — Store the feed in the user cache directory, not in the repository

**Status:** Accepted · 2026-10-19

**Context.** The original project committed the extracted feed to `data/`. The
committed copy was **feed version 69 from January 2024**, whose coverage
window ended 2024-08-11 — two years stale, with
nothing to warn about it.

**Decision.** The feed lives in `$XDG_DATA_HOME/bart-rs` (default
`~/.local/share/bart-rs`), with the extracted files in `feed/current/`, the
raw download in `archives/<version>.zip`, and bookkeeping in `state.json`.
`BART_RS_DATA_DIR` overrides the location.

**Why.** The stale committed feed is the single worst thing about the original
project: it looks authoritative, it is silently wrong, and it is in git
history forever. A tool that downloads its own data can know when that data is
stale and say so. Storing it outside the repo also keeps a ~5 MB blob out of
git and lets multiple feed versions coexist.

**Rejected.**

- *Keep `data/` in the repo but refresh it automatically.* Still stale for
  anyone who clones without running the tool, and still in git history.
- *Store in the system package directory.* Requires root, and the feed is
  user data, not system data.

---

## D4 — Use `reqwest` with `rustls-tls` and the blocking client

**Status:** Accepted · 2026-10-19

**Context.** The project needs to fetch a ~900 KB zip and two small protobuf
feeds. The original `flake.nix` already listed `openssl` and `pkg-config`,
suggesting `reqwest` with `native-tls` was anticipated.

**Decision.** `reqwest` with the `blocking` client and `rustls-tls`
(webpki-roots). No `tokio` runtime in our own code.

**Why.** `rustls-tls` is pure Rust, so the build needs no OpenSSL headers —
`cargo build` works on a bare machine, in CI, and in the Nix shell without
system dependencies. The `openssl` and `pkg-config` entries in `flake.nix`
become unnecessary. The blocking client means no `#[tokio::main]`, no async
command implementations, and no `tokio` in our dependency graph (though
`reqwest`'s blocking API does pull it in internally).

**Rejected.**

- *`native-tls`.* Requires OpenSSL dev headers at build time; a common source
  of "works on my machine" failures.
- *Async `reqwest` + `tokio`.* No benefit for three sequential HTTP calls.
- *`ureq`.* Lighter, but less common and less familiar; `reqwest` is what the
  original `flake.nix` was already provisioned for.

**Verified.** The full download-and-extract path was run end to end against
the live BART feed with this configuration and produced all 18 files.

---

## D5 — Decode GTFS-Realtime with hand-written `prost` structs

**Status:** Accepted · 2026-10-19

**Context.** The two realtime feeds are protobuf. The obvious crates are
`gtfs-rt` (barzamin) and `gtfs-realtime` (catenarytransit), both of which are
`prost`-based — and both of which have a `build.rs` that runs `prost-build`,
which requires the `protoc` binary at compile time.

**Decision.** Write the ~15 GTFS-Realtime message types by hand as
`#[derive(prost::Message)]` structs in `src/gtfsrt.rs`, depending only on
`prost` (which is pure Rust).

**Why.** No `protoc` dependency, so the crate builds anywhere `cargo build`
works — no system packages, no Nix shell required, no CI setup. It also means
the field numbers are visible and auditable in our own source, which turned out
to matter: BART's alert feed does not use the spec's field numbers for the
alert text, and writing the structs by hand made that discrepancy obvious
rather than hidden inside a third-party crate.

**Rejected.**

- *`gtfs-rt` crate.* Mature, but needs `protoc`; also pins `prost` 0.11.
- *`gtfs-realtime` crate.* Also needs `protoc`; newer and more minimal.
- *Vendor a pre-generated `.rs` file.* Same dependency problem, plus a large
  generated blob to maintain.

**Cost.** ~250 lines of struct definitions. Mitigated by the field numbers
being tabulated in `03-data-sources.md`.

**Risk accepted.** If BART's protobuf diverges further from the spec, our
hand-written structs may not match. The mitigation is the live-feed test:
decode the live feed and assert every entity renders. We keep all fields,
decode leniently, and apply the fallback chain described in
`03-data-sources.md`.

---

## D6 — Realtime data merges into the existing commands

**Status:** Accepted · 2026-10-19

**Context.** Live data could have been a separate command group, or merged
into the scheduled-data commands.

**Decision.** `bart-rs arrivals <stop> --realtime` overlays live trip updates
onto the scheduled arrivals. `bart-rs alerts` is a separate top-level command,
because there is no static equivalent of a service alert.

**Why.** A user asking "when is the next train" should not have to know
whether the answer is live or scheduled, and should not have to learn a second
command to get the better answer. Alerts have no static counterpart, so they
need their own command regardless.

**Rejected.**

- *A separate `bart-rs realtime ...` command group.* Two places to look for
  arrivals, and the user has to decide up front which one they want.
- *Always live, never scheduled.* The static feed works offline and is
  authoritative for anything further out than the realtime feed's horizon.

---

## D7 — Do not trust the `Content-Type` header on the realtime feeds

**Status:** Accepted · 2026-10-19

**Context.** Both realtime endpoints respond `Content-Type: text/html` while
serving a protobuf `FeedMessage`.

**Decision.** Read the response body as raw bytes and decode it as protobuf.
Never branch on `Content-Type`.

**Why.** The header is simply wrong. Any code that checks it (a generic
"is this HTML? bail out" guard, or a JSON-ish deserializer) will fail on a
perfectly valid feed. This is worth recording because it is the kind of thing
that looks like a bug in our code rather than in the server.

**Also noted.** The `http://` URLs issue a `301` to `https://`. Use the
`https://` form in code, and let `reqwest` follow redirects as a safety net.

---

## D8 — BART's alert feed does not follow the GTFS-Realtime field numbering

**Status:** Accepted · 2026-10-19

**Context.** Decoding the live alerts feed with `protoc --decode_raw` showed
BART placing the short title and the long alert text in `Alert` fields **10**
and **11**, where the spec defines `created_at` (a `uint64`) and `update` (a
`TimeRange`).

**Decision.** Decode every field by number, then extract the alert text
through an explicit fallback chain: field 11 → field 6 → field 10 → field 5.
Do not assume spec conformance.

**Why.** Being spec-strict would render every BART alert as an empty string.
Being BART-specific would break on a future spec-conformant feed. The
fallback chain handles both, and the live-feed test catches regressions in
either direction.

**Verification.** `protoc --decode_raw` output for a live alert is recorded in
`03-data-sources.md#-bart-alert-feed-is-not-spec-conformant-and-is-not-self-consistent`
so it can be compared later without re-deriving it.

---

## D9 — Hold BART's alert text fields as raw bytes, not as `TranslatedString`

**Status:** Accepted · 2026-10-19

**Context.** Implementing D8 exposed a second problem. Working the bytes out
by hand showed that BART is **not self-consistent about the shape of its own
text fields**:

- `Alert.header_text` (field 5) holds `0a 04 "BART"` — the `TranslatedString`'s
  `translation` entry is a **bare string**, not a `Translation` message.
- Fields 8, 10 and 11 hold `0a <len> <Translation>` — a properly nested
  `Translation { text, language }`.

Declaring the fields as `TranslatedString` made the *entire* feed fail to
decode, with `delimited length exceeded`: the bytes `BART` are not a valid
protobuf message, because `0x42` reads as a length-delimited field claiming 65
bytes that are not there.

**Decision.** Declare the alert's text fields as `prost(bytes = "vec")` — raw
length-delimited payloads that prost accepts without interpreting — and add a
small tolerant extractor, `gtfsrt::alert_text`, that takes the first
length-delimited field, tries to decode it as a `Translation`, and falls back
to treating the raw bytes as the text when that fails.

**Why.** Protobuf is wire-type driven, so once a field's expected type is
wrong the whole message fails. Holding the bytes decouples "what shape did
BART use" from "how do I read this field", and one small function then
handles every shape. It also means a spec-conformant feed — where the fields
are proper `TranslatedString`s — decodes correctly through the same path.

**Cost.** A ~40 line hand-rolled wire-format scanner (varint reading and
length-delimited field extraction) plus the extractor itself.

**Rejected.**

- *Declare the fields as `TranslatedString` and special-case only field 5.*
  Fragile: the next field BART gets wrong breaks the feed again, and it does
  not explain the general problem.
- *Parse the whole alert message by hand.* More code for no benefit; prost
  handles the rest of the feed correctly.
- *Give up on alerts.* They are one of the most useful things the realtime
  feed offers.

---

## D10 — Update the Nix toolchain; rustc 1.82 cannot build current crates

**Status:** Accepted · 2026-10-19

**Context.** `flake.lock` pinned nixpkgs from 2024-11-15, giving rustc
**1.82.0**. That toolchain predates the `edition2024` cargo feature and
**cannot compile current versions** of `zeroize`, `zeroize_derive`,
`cpufeatures`, `time-core`, and others — the resolver picks the newest
versions and the build fails with
`feature 'edition2024' is required`.

This blocks adding *any* new dependency, including `clap`.

**Decision.** Run `nix flake update`, moving nixpkgs to 2026-10-15 and
rust-overlay to 2026-10-19, which provides rustc **1.99.1** with `rustfmt`
and `clippy`. Add a `rust-toolchain.toml` with `channel = "stable"` for
non-Nix users.

**Why.** The flake is the project's declared development environment (there
is a `.envrc` with `use flake`), so keeping it current is the right fix. The
alternative — pinning a long tail of transitive crates to old versions — is
fiddly and would break again with the next `cargo add`.

**How the toolchain is pinned, precisely.** `channel = "stable"` is not a
pin, and the two audiences get different guarantees:

- **Nix users** get a reproducible toolchain. `flake.lock` pins the
  rust-overlay revision, and rust-overlay's manifest maps `stable` to a
  specific version at that revision — 1.99.1 here. The flake reads the same
  `rust-toolchain.toml` through `fromRustupToolchainFile`, so the file is
  load-bearing for the dev shell and not merely a courtesy to non-Nix users.
- **rustup users** get whatever stable is current. The file tells rustup to
  install the stable channel, which floats.

That is deliberate rather than an oversight: a project that wants to track
current Rust should not have to bump a version string by hand, and the Nix
path — the one this project actually develops in — is reproducible either
way. The `rust-src` component is there because the dev shell sets
`RUST_SRC_PATH`, which rust-analyzer needs.

**Note for the user's shell.** After `nix flake update`, run
`direnv reload` in the project directory so the new toolchain is on the path.
A stale `.direnv` profile will keep pointing at the 1.82.0 store path.

---

## D11 — Keep the module-per-GTFS-file layout from the original project

**Status:** Accepted · 2026-10-19

**Context.** The original `src/attributes/` directory with one module per GTFS
file (`stops.rs`, `trips.rs`) was the one structurally sound idea in the
original code, even though the implementation inside it was broken.

**Decision.** Keep the layout and extend it to all 18 tables.

**Why.** It makes coverage obvious at a glance — the module list *is* the
checklist of handled tables — and it keeps each file small enough to read in
one sitting. A single large `gtfs.rs` would be harder to navigate and would
make the per-table unit tests less natural.

**Rejected.**

- *One module per GTFS file, but named after the concept rather than the
  file.* The file names are the GTFS spec's names; using them makes it trivial
  to look up the authoritative column list.
- *A single `gtfs.rs`.* 18 tables in one file is a 1,000-line file.

---

## D12 — Represent GTFS times as seconds since midnight, not `NaiveTime`

**Status:** Accepted · 2026-10-19

**Context.** GTFS permits times past 24:00:00. A trip departing 00:30 the
next morning is written `25:30:00` (or `24:30:00`). `chrono::NaiveTime`
cannot represent that, and wrapping it to `01:30:00` silently changes the
service day the trip belongs to.

**Decision.** Store all `arrival_time` / `departure_time` values as `u32`
seconds since midnight of the trip's **service day**, with `> 24:00:00` values
preserved. Convert to a human-readable form only at display time.

**Why.** A trip's service day is the unit that matters for "which services run
today", and a time of 91800 seconds unambiguously means "01:30 on the
service day plus one". Naive-time wrapping destroys exactly the information
needed to answer "is this train running tonight or tomorrow morning".

**Rejected.**

- *`NaiveTime` with wrapping.* Loses the service-day boundary.
- *A `DateTime` with an artificial date.* Same problem, more machinery.

---

## D13 — Parse money into integer cents

**Status:** Accepted · 2026-10-19

**Context.** `fare_attributes.txt` and `fare_rider_categories.txt` carry
decimal prices as strings (`"7.55"`).

**Decision.** Parse into `i64` cents (`755`). Do not add a decimal crate, and
do not use `f64`.

**Why.** Money must not be floating point. BART's prices have exactly two
decimal places, so a string-to-cents parse is exact and trivial. Avoids a
dependency for something this simple.

---

## D14 — The realtime and static feeds share one id namespace for stops

**Status:** Accepted · 2026-10-19

**Context.** It looked as though the realtime feed might use a different stop
id scheme than the static feed: the static feed names its station `LAKE` but
names the platform `A10-1`, which raised the worry that the realtime feed's
`A70-1` referred to a different place than the static feed's `A70-1`.

**Decision.** Treat them as the same namespace. The realtime feed's stop ids
are **platform** ids, exactly as `stop_times.txt` uses them. The confusion
comes from BART using *legacy platform codes* (`A10-1`) while *modern station
codes* (`LAKE`) coexist in the same file — the platform's `parent_station` is
the station, but the platform's own id is not derived from it.

**Verified.** Realtime trip ids in a live sample resolve against the
downloaded static `trips.txt` (74 matched), and realtime stop ids are of the
platform form. Every `stop_times.txt` row references a platform id that exists
in `stops.txt`.

**Consequence.** Joining live data to the schedule needs no id translation —
but resolving a *user-supplied* stop still must accept a station id, a
platform id, or a name, and normalise to the platform set.

---

## D15 — Point the flake at nixpkgs on GitHub, not at FlakeHub

**Status:** Accepted · 2026-10-19

**Context.** The flake's `nixpkgs` input was
`https://flakehub.com/f/NixOS/nixpkgs/0.1.*.tar.gz`. FlakeHub has announced
its shutdown, and a project meant to be shared online should not have a fresh
clone depend on a third-party service that is going away.

**Decision.** Use `github:NixOS/nixpkgs/nixos-unstable` and let `flake.lock`
pin the exact commit.

**Why.** GitHub is where the flake inputs live anyway — the `rust-overlay`
input was already a GitHub flake — so this makes the flake self-consistent
and removes a dependency on a service with a shutdown date. The lock file
still pins a specific commit, so builds stay reproducible; the branch is
only "unstable" in the sense that `nix flake update` will move it forward.

**Note.** This is the change that required the `nix flake update` in D10: the
input URL changed, so the lock had to be regenerated against the new source.

---

## D16 — Keep the dev shell's environment configuration in `flake.nix`

**Status:** Accepted · 2026-10-19

**Context.** The project's environment is defined by `flake.nix` plus a
`.envrc` that does `use flake`, so `direnv` loads it automatically. Anything
that has to be true for the build — the toolchain, `RUST_SRC_PATH` for
rust-analyzer, `RUST_BACKTRACE` — belongs in the flake, where it is
reproducible for anyone who clones the project.

**Decision.** All permanent environment configuration lives in `flake.nix`'s
`env` block. `RUST_SRC_PATH` was already there; `RUST_BACKTRACE = "1"` was
added. The CLI's own `--data-dir` flag sets `BART_RS_DATA_DIR` **in-process
only**, for the lifetime of that command, and never touches the user's shell.

**Why.** A shell that has been mutated by hand is a shell that differs from
what the project says it is. Putting the configuration in the flake means
`nix develop`, `direnv allow` and CI all see exactly the same environment.

---

## D17 — Identify a feed release by the URL it is served from, not by `feed_version`

**Status:** Accepted · 2026-10-19

**Context.** The update check compared the installed feed's `feed_version`
against the remote feed's. In practice `feed_version` is `72` in every
release BART has published — the January 2026 feed and the August 2026 feed
both say `72` — so the comparison always succeeded, the tool always said
"The installed feed is already version 72", and it never once downloaded a
new feed. The check was not just imprecise, it was permanently satisfied.

**Decision.** Identify a release by the **versioned filename** the permalink
redirects to — `google_transit_20260810-20270108_v02.zip` — which encodes
both the coverage window and a revision counter, and changes on every
release. The final URL and the response's `Last-Modified` header are stored
in `state.json` and compared. `feed_version` is still parsed and displayed,
but never used to decide anything.

**Why.** A release identifier has to change when the release changes. The
filename does; the version number does not. The `Last-Modified` header is
kept as a secondary signal, used only to break ties between two releases
with the same name.

**Cost.** A `HEAD` request instead of a `GET` for the check, so that
checking for a new release does not transfer 900 KB. The server has to
answer `HEAD`; if it does not, the code falls back to downloading the feed
and comparing there.

**Rejected.**

- *Trust `feed_version` and hope BART starts bumping it.* The bug is silent
  and permanent; there is no version of "hope" that fixes it.
- *Compare the feed's contents.* Requires downloading the whole feed to
  answer a question that a `HEAD` answers.
- *Compare only the `Last-Modified` header.* A CDN may re-stamp it without
  the content changing, and it says nothing about which release is on offer.

---

## D18 — Take the feed's coverage window from the calendar, not from `feed_info.txt`

**Status:** Accepted · 2026-10-19

**Context.** `feed_info.txt` carries `feed_start_date` and `feed_end_date`,
which the GTFS spec defines as the window for which the dataset is complete
and reliable. BART does not maintain them: the release whose file is named
`google_transit_20260810-20270108_v02.zip` declares `20260112` → `20260830`,
which is the *previous* release's window. Its calendar covers
`20260810` → `20270110`.

**Decision.** Derive the coverage window from `calendar.txt` — the earliest
`start_date` and the latest `end_date` — widened by any single-date
exception in `calendar_dates.txt`. Use it for the staleness check. Show
`feed_info.txt`'s dates in `bart-rs status` too, labelled as the stale claim
they are, so the discrepancy is visible rather than hidden.

**Why.** The tool told the user that a perfectly current feed had expired
50 days ago, and then refused to download a newer one — a dead end with no
way out. The guarantee BART actually offers is the calendar's, so that is
what the tool should believe.

**Cost.** One extra pass over `calendar.txt` and `calendar_dates.txt`, and
a slightly more honest `status` output.

---

## D19 — Parse hex colours by hand rather than through `colored`'s string conversion

**Status:** Accepted · 2026-10-19

**Context.** BART publishes route colours as six-digit hex strings such as
`FF0000`. They were passed straight to the `colored` crate as
`.color("FF0000")`. `colored`'s `From<&str> for Color` is implemented as
`src.parse().unwrap_or(Color::White)`, and its `FromStr` only understands
the sixteen *named* colours — so `FF0000` did not parse, the fallback
applied, and every route's swatch was rendered **white**. The hex string
was printed correctly next to it, which is what made the bug look like a
terminal problem rather than a code problem.

**Decision.** Parse the hex into its three components and build a
`Color::TrueColor` explicitly. `colored` then emits a 24-bit colour escape
on terminals that advertise `COLORTERM=truecolor`, and the nearest of the
sixteen ANSI colours otherwise.

**Why.** The failure mode was silent and total: the code did the wrong
thing for every colour, and the output looked plausible. Parsing the hex
ourselves is four lines and cannot fail silently.

**Rejected.**

- *Use `colored`'s named colours and map hex to the nearest name by hand.*
  Loses the exact colour, which is the whole point of the swatch.
- *Emit the escape sequence directly, bypassing `colored`.* Would have to
  reimplement `--no-color` and `NO_COLOR` handling, and the nearest-ANSI
  fallback, for no gain.

---

## D20 — Measure table columns in visible characters, not bytes

**Status:** Accepted · 2026-10-19

**Context.** `print_table` sized its columns with `str::chars().count()`.
That counts the escape sequences in a coloured cell as though they were
printed, so a cell holding a colour swatch — one character on screen, about
twenty bytes of escape sequence — made its column twenty columns too wide
and pushed everything after it out of line. The `bart-rs routes` table was
visibly ragged for this reason: the `COLOR` heading sat eleven columns to
the right of the colour it labelled.

**Decision.** Size columns with a `visible_width` function that skips ANSI
escape sequences, and pad cells to that width rather than relying on
`format!`'s width specifier, which also counts the escapes.

**Why.** The table is the main way this tool reports things, and a ragged
table is the sort of thing that makes a reader distrust the numbers next to
it. The bug predated the colour work and was invisible until a swatch was
put in a table.

**Cost.** About thirty lines: a small ANSI scanner and a padding helper.

**Rejected.**

- *Do not put coloured cells in tables.* That would mean giving up the
  swatch everywhere, which is the point of it.
- *Put the swatch in its own trailing column.* The escape sequences would
  still be counted, just less visibly.
- *Use a crate that handles this.* There is one that would do; hand-writing
  thirty lines keeps the dependency list short and the behaviour obvious.

---

## D21 — Give the swatch its own column, rather than colouring the text

**Status:** Accepted · 2026-10-19

**Context.** `bart-rs arrivals` and `departures` list the line serving each
train. There are two ways to colour-code them: colour the text, or put a
swatch next to it.

**Decision.** A swatch in its own column, immediately before the route's
short name. The same treatment is applied to `bart-rs stop`'s list of the
lines serving a station, and to the route line of `bart-rs trip`.

**Why.** A coloured *background* on the text is unreadable when the colour
is light — BART's yellow is `FFFF33`, which on a white terminal is
invisible, and on a dark one is blinding. A solid block of the colour is
readable either way, and it is the same shape BART uses in its own maps and
timetables, so it reads as "this is the colour of the line" rather than as
decoration. It also survives `--no-color`: the swatch becomes a blank
column and the table still lines up.

**Cost.** One extra column in three commands.

---

## D22 — Show the platform number, not the platform id

**Status:** Accepted · 2026-10-19

**Context.** `stop_times.txt` refers to platforms by their `stop_id`, and
BART's platform ids are built from a legacy station code and the platform
number: `M80-1` is platform 1 of Balboa Park, `W40-3` is platform 3 of
Millbrae. The number a passenger actually sees, painted on the sign, is the
`platform_code` column of `stops.txt`.

The arrivals table was showing the id.

**Decision.** Show the `platform_code`. The library exposes both — the id as
`Arrival::platform_id` and the number as `Arrival::platform_code` — and
`Arrival::platform_label()` picks the number, falling back to the id for the
handful of platforms with no code.

**Why.** `M80-1` is an internal identifier that happens to be visible. It
looks like a platform number with a prefix, and reading it requires knowing
BART's station code scheme. `1` is what the sign says and what the
passenger is looking for.

**Cost.** One extra field on `Arrival` and `LiveArrival`, and a lookup in
`stops.txt` when building an arrival.

**Rejected.**

- *Show both, in two columns.* The id is not information a passenger wants;
  it is an identifier for looking up the platform, which the library
  already provides.
- *Derive the number from the id by splitting on `-`.* It happens to work for
  most platforms, but `H10` — the Caltrain transfer platform at Millbrae —
  does not follow the pattern, and its code is `3`. The feed says so
  explicitly; read it.

---

## D23 — Distinguish "on time" from "nothing is known"

**Status:** Accepted · 2026-10-19

**Context.** The realtime feed only covers the trains it is currently
tracking. A train two hours out has no live update at all. The overlay
gave every unmatched arrival a delay of zero, and the CLI rendered a zero
delay as `on time`.

Measured against the live feed: at 15:00 the realtime feed had updates for
10 of the next 10 trains at Embarcadero; at 17:00 it had updates for 0 of
the next 10.

**Decision.** `LiveArrival` gains a `has_update` flag. The CLI shows
`on time` only when the realtime feed actually reported on that train, and
`scheduled` when it did not. Without `--realtime`, every row is
`scheduled`, because the static feed carries no delay information at all.

**Why.** `on time` is a claim about the world. A train with no live data has
not been reported punctual; it has not been reported on at all. A departure
board that says "on time" about a train it knows nothing about is worse
than one that says nothing, because the reader trusts it.

**Cost.** One field on `LiveArrival`, and a third arm in the status
rendering.

---

## D24 — Give the library a one-call update, and a guide for building on it

**Status:** Accepted · 2026-10-19

**Context.** The update lifecycle — probe, compare, download, extract,
install, write state — lived in the `bart-rs update` command. A library
consumer wanting a current feed had to copy that sequence out of the
command and into their own program.

**Decision.** Two additions:

1. `update::ensure_current(client)` runs the whole lifecycle and returns an
   `UpdateOutcome`: `UpToDate`, `Updated`, or `Unreachable`. Being unable
   to reach the server is a *value*, not an error, because an application
   showing a schedule should keep working with the feed it already has.
2. `docs/07-library-guide.md`, a guide to using the crate from another Rust
   project, with a worked example, the feature-flag and `reqwest`
   requirements, the measured timings, and the four upstream gotchas.

**Why.** The library exists to be used by other programs. An update
lifecycle that only exists in the command line is a lifecycle that other
programs have to reimplement, and reimplement it badly.

**Cost.** About a hundred lines in `update.rs`, and a document.

---

## D25 — Edition 2024, and an MSRV that matches the dependency tree

**Status:** Accepted · 2026-10-19

**Context.** The crate was on edition 2021 and declared
`rust-version = "1.85"`. Both numbers were wrong in the same direction:
conservative to the point of being untrue.

**Decision.**

1. Move to **edition 2024**, which implies **resolver 3**.
2. Declare `rust-version = "1.88"` — the floor the locked dependency tree
   actually imposes, not the floor of the edition.
3. Keep `resolver = "3"` written out explicitly, so that the reason is on
   the manifest rather than in a reader's head.
4. Accept two `unsafe { std::env::set_var(...) }` blocks, which edition 2024
   requires.

**Why resolver 3 is the important half.** Resolver 3 honours `rust-version`
when it chooses dependency versions. Resolver 2 — the edition 2021
default — ignores it, so the resolver cheerfully picks the newest version of
every crate regardless of whether it will compile. That is exactly how a
crate claiming to support Rust 1.85 ended up with a lockfile that needs
1.88.

**Why 1.88 and not 1.85.** The floor comes from the dependencies:

| Crate | Needs | Reached through |
| --- | --- | --- |
| `time 0.3.55`, `time-core 0.1.9` | 1.88.0 | `zip` → `cookie` |
| `zopfli 0.8.4` | 1.88.0 | `zip` |
| `icu_properties`, `icu_normalizer`, `icu_provider`, `icu_collections`, `icu_locale_core`, `icu_normalizer_data`, `icu_properties_data` | 1.88 | `reqwest` → `url` → `idna` → `idna_adapter` → `icu_normalizer` |

Declaring 1.85 would have been a promise the crate could not keep: someone
on 1.85 would get `package 'time v0.3.55' cannot be built because it
requires rustc 1.88 or newer` from a transitive dependency, rather than a
clear error from this crate's own manifest.

With resolver 3 the promise is now enforced in both directions. `cargo
update` picks the highest version that still builds on 1.88 — which is how
`webpki-roots` was resolved to 1.0.10 rather than to a version needing a
newer compiler.

**Why edition 2024 anyway.** Edition 2021 has been in maintenance since
February 2025. The only thing holding the crate on it was
`std::env::set_var`, which became `unsafe` in edition 2024 because setting
an environment variable is not thread safe on Unix. There are exactly two
call sites, and both are sound:

- `src/commands/mod.rs` — the `--data-dir` flag, handled by `dispatch` on
  the main thread during start-up, before any other thread exists.
- `tests/colour_tests.rs` — sets `COLORTERM` so that `colored` picks true
  colour. Sound because the two checks in that file were merged into a
  single test: the file's other check reads `COLORTERM` through `swatch`,
  and with two tests running in parallel threads that was a write racing a
  read. One test means one worker thread, and separate `tests/*.rs` files
  are separate processes.

Edition 2024 also buys let chains, which the code now uses in six places
where it previously nested an `if let` inside an `if`.

**Cost.** Two `unsafe` blocks, both with a SAFETY comment, and the loss of
one test *count* — the two colour checks are now one test, because they
were already coupled by process-global state.

---

## D26 — Move to `zip` 9, and audit the dependency tree

**Status:** Accepted · 2026-10-19

**Context.** `cargo deny check` — which had never been run against this
project, because there was no `deny.toml` — found three advisories against
the `zip` crate: RUSTSEC-2026-0367, -0368 and -0369. All three are in the
same family: the functions that write an archive's entries out to disk
(`ZipArchive::extract` and `ZipStreamReader::extract`) did not confine
symlink targets to the destination directory, and did not preserve file
modes safely.

**Decision.**

1. Move the direct dependency from `zip` 2 to `zip` 9, which has the fix.
2. Keep the three advisories in `deny.toml`'s ignore list, with the reason
   written down, because the vulnerable functions are not reachable.
3. Add `deny.toml` and wire `cargo deny` into CI.

**Why the advisories are ignored rather than fixed.** The vulnerable
functions are `ZipArchive::extract` and `ZipStreamReader::extract`, and
neither is called anywhere in the tree:

- `src/update.rs` opens the archive with `ZipArchive::new`, reads each entry
  with `by_index`, and writes the bytes out itself, one plain file at a
  time, after rejecting any path that could escape the destination
  directory.
- `gtfs-structures`, the dev-dependency used as a cross-validation oracle,
  also reads entries with `by_index` and deserialises them in memory.

The advisories are still reported because `gtfs-structures` requires `zip`
8, and Cargo cannot resolve two semver-incompatible major versions of one
crate to a single version. That is a dev-dependency: it is not part of the
tree that reaches anyone who depends on `bart-rs`.

If this ever needs to change, the options are to drop `gtfs-structures` and
write the cross-validation against the fixture feed directly, or to wait for
`gtfs-structures` to move to `zip` 9.

**Why `cargo deny` at all.** A crate that is about to be published will be
depended on by other people, and the advisories in its tree become their
problem. `cargo audit` and `cargo deny` runs are common in consumer CI, and
a crate that trips them is a crate people are told not to use. Checking the
tree in this project's own CI means a new advisory is found here, where it
can be fixed, rather than in someone else's build.

The licence allow-list in `deny.toml` is the set of licences the tree
actually uses, all of them permissive. A dependency with a different licence
fails the check and forces a decision.

---

## D27 — Check the MSRV, rather than asserting it

**Status:** Accepted · 2026-10-19

**Context.** `rust-version` is a promise: a crate that declares 1.88 says
that 1.88 can build it. The value can be checked against the declared
`rust-version` of every dependency, but that is a proxy — a crate that
declares 1.70 may still use a 1.85 language feature.

**Decision.** Three things:

1. A CI job builds and tests the crate with exactly the declared MSRV, so
   a dependency that raises its floor breaks the build rather than quietly
   making the promise false.
2. The MSRV was verified by hand against real toolchains, not inferred: with
   Rust 1.88.0 the crate builds and all the tests pass; with Rust 1.87.0
   `cargo` refuses to build it, naming `bart-rs` first in the error.
3. The declared value is the *real* floor, not a round number below it. It
   was 1.85 because that is the floor of edition 2024, which was a number
   about the edition rather than about this crate.

**Why.** The error a consumer gets matters. On Rust 1.87 the message is

```
error: rustc 1.87.0 is not supported by the following packages:
  bart-rs@0.1.0 requires rustc 1.88
```

which says exactly what is wrong and what to do. Without the declared
version being right, the same consumer would have got an error from `time`,
a transitive dependency, about a crate they have never heard of.

**Cost.** One CI job, and the discipline of re-checking the floor whenever
the dependency tree changes.

---

## D28 — Follow BART's field names, not the specification's

**Status:** Accepted · 2026-10-21

**Context.** `rider_categories.txt` is an optional file in the core GTFS
specification. The specification gives it three fields: `rider_category_id`,
`rider_category_name`, both required, and `is_default_fare_category`, also
required. BART publishes the file with two columns — `rider_category_id` and
`rider_category_description` — and no `is_default_fare_category` at all.

D1 originally listed `rider_categories` among BART's extension files, on the
grounds that the specification's field name is not the one BART uses. That
conflated a file BART invented with a file BART publishes non-conformantly,
and it had a practical consequence that went unnoticed: `gtfs-structures`
follows the specification, so it rejects BART's file outright with `missing
field 'rider_category_name'`. The live-feed test that compares our parser
against the oracle therefore failed every time it was run, and the failure
read like upstream drift rather than the designed divergence it was. D1 has
been amended accordingly.

**Decision.** Parse the field BART actually serves. `RiderCategory` keeps
`#[serde(rename = "rider_category_description")]`, and the crate does not
attempt to parse the specification's name.

The oracle test compares against a copy of the feed with that one file left
out, and checks the file's header before it does so, so that a change at
BART is reported as a change at BART rather than as an unexplained parse
failure. Any other file the oracle cannot parse still fails the test.

**Why.**

- The crate exists to read BART's feed. A feed that does not conform to the
  specification is still the feed, and a parser that insists on the
  specification's field names parses none of it.
- Accepting both names would be more forgiving, but it would hide a
  divergence an operator should know about, and it would not help the
  oracle comparison, which is about counts, not about our field names.
- Excluding one file from the oracle is not the same as ignoring it. The
  header is still checked on every run, so a change at BART is caught.

**Rejected.**

- *Rename the field to `rider_category_name` to match the specification.*
  Breaks parsing of the feed we actually download: every rider category
  would fail to deserialise, and `fare_rider_categories.txt` would have
  nothing to refer to.
- *Make the field optional and accept either name.* Hides the divergence,
  and leaves the crate unable to say which name the feed actually used.
- *Drop `rider_categories.txt` from the crate.* It is the only place the
  rider categories are named, and the only thing that distinguishes
  `Youth Clipper` from `Senior/Disabled Clipper` in the fare output.
- *Give up on the oracle.* The oracle is what catches a change in the feed
  that we do not notice. It is worth more than one file.

**Cost.** The oracle test has to know about the divergence, and the crate
exposes a field name that does not appear in the specification. Both are
documented at the point where they bite: the module doc in
`src/attributes/rider_categories.rs`, and `docs/05-testing.md`.
