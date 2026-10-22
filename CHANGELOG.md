# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-10-21

The first release. This is a rewrite of the project from the ground up: the
original code parsed `stops.txt` by splitting lines on commas and leaked every
field, had a stub `trips` parser, and shipped a stale January-2024 feed in the
repository.

### Added

- A library, `bart_rs`, with a documented public API, and a thin command line
  tool on top of it.
- All 18 GTFS tables parsed with `csv` and `serde`, driven by the file's own
  header row rather than by column position.
- Twelve commands: `update`, `status`, `routes`, `route`, `stops`, `stop`,
  `arrivals`, `departures`, `trip`, `fare`, `alerts` and `table`.
- `--json` output for every command that has a meaningful machine-readable
  form.
- GTFS-Realtime trip updates and service alerts, merged onto the schedule.
- `update::ensure_current`, which runs the whole update lifecycle in one call
  and reports being offline as a value rather than as an error.
- Route colours, rendered as a swatch in each train's line colour.
- A Nix flake, providing the development shell and the build.
- A design record under `docs/`, and a guide to using the library in
  `docs/07-library-guide.md`.

### Fixed

- The feed's coverage window now comes from the calendar. BART leaves
  `feed_info.txt` describing the *previous* release, so a perfectly current
  feed was being reported as expired.
- A feed release is now identified by the versioned filename the permalink
  redirects to. `feed_version` is `72` in every release BART has published,
  so comparing it could never detect a new feed.
- Route colours are parsed by hand. `colored`'s own string-to-colour
  conversion falls back to *white* for anything that is not one of its
  sixteen named colours, so BART's hex colours came out white.
- Table columns are measured in visible characters rather than bytes, so a
  colour escape sequence no longer pushes a column out of line.
- The output pipe being closed early is no longer a panic. `bart-rs routes |
  head -1` exits quietly, as any Unix tool should.
- A delay of less than a minute is reported in seconds. The realtime feed
  measures in seconds, so a 33-second delay was shown as `+0 min late`.
- The platform column shows the platform number rather than the platform's
  GTFS id, and a train the realtime feed is not tracking says `scheduled`
  rather than claiming to be on time.

### Security

- `rustls-webpki` is bumped to 0.103.16, which rejects certification paths
  containing the eight compromised eMudhra intermediate CAs. Their private
  keys were generated with effectively no entropy in April 2025, so the keys
  are recoverable from the public certificates, and anyone able to obtain a
  certificate from one of them could impersonate any domain. Browsers and
  most certificate revocation mechanisms already refused them, but `rustls`
  performs no revocation checking, so the denylist has to be explicit in the
  verifier. The `download` feature is on by default, so this affects the
  `update` command and the realtime fetchers. **Downstream consumers need to
  re-resolve their lockfiles** to pick this up.

[Unreleased]: https://github.com/gamersekofy/bart-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/gamersekofy/bart-rs/releases/tag/v0.1.0
