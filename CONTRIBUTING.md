# Contributing to `bart-rs`

Thank you for considering it. This document covers how to work on the
project, and the handful of things that have caused bugs before and will
again.

## Getting set up

The project is a Nix flake. With [Nix](https://nixos.org) and flakes enabled:

```console
$ nix develop
$ cargo test
```

`nix develop` gives you the right Rust toolchain plus `clippy`, `rustfmt`,
`cargo-watch`, `rust-analyzer` and `cargo-deny`. Nothing is installed outside
the Nix store. If you use [direnv](https://direnv.net), `direnv allow` loads
the same environment automatically.

Without Nix, install Rust 1.88 or newer and use `cargo` directly.

## The checks

```console
$ cargo fmt --check
$ cargo clippy --all-targets --all-features -- -D warnings
$ cargo test --all-features --locked
$ cargo doc --all-features --no-deps
```

All four run in CI on every push and pull request. `cargo test` is offline
and deterministic; it uses the committed fixtures in `tests/fixtures/`.

There is one test that is not offline. It downloads the real feed and
cross-checks the parser against a reference implementation:

```console
$ cargo test -- --ignored
```

It is worth running it before changing anything in `src/attributes/` or
`src/feed.rs`.

## What CI checks

| Job | What it does |
| --- | --- |
| `test` | formatting, lints, build, tests, documentation |
| `msrv` | builds and tests with exactly the Rust version in `rust-version` |
| `deny` | advisories, licences, duplicate versions, unexpected sources |
| `package` | builds and tests the crate as it would be published |
| `nix` | `nix flake check` and the dev shell |

The `msrv` job is what keeps `rust-version` honest. If a dependency raises
its floor, the job fails rather than letting the manifest quietly go stale.
If it fails because of a dependency, raise `rust-version` and update the
README and `docs/01-architecture.md` to match — the number is a promise to
whoever depends on the crate.

The `package` job builds the tarball that `cargo publish` would upload, and
then builds and tests it again from the extracted tarball. That is the only
way to be sure the published crate is self-contained.

## Things that will bite you

These are properties of BART's data, or of this crate's build, that have
each caused a bug at least once.

1. **Never index GTFS rows by column position.** The `stops.txt` schema has
   already changed once, from 10 columns to 11. Use header-driven
   deserialization.
2. **Do not trust `feed_info.txt`.** Its dates describe the *previous*
   release, and its `feed_version` is the same in every release. Take the
   coverage window from the calendar, and identify a release by the
   versioned filename the permalink redirects to.
3. **Do not trust the `Content-Type` header on the realtime feeds.** They
   are mislabelled `text/html` and serve protobuf.
4. **Do not remove the `gtfs-structures` dev-dependency.** It is used as a
   cross-validation oracle in the test suite. It must keep
   `default-features = false`, or OpenSSL comes back into the tree.
5. **Do not add a dependency without running `cargo deny check`.** A new
   licence or a new advisory fails the `deny` job.

The full details are in [`docs/`](docs/README.md).

## Conventions

- **Rust edition 2024, MSRV 1.88.** Do not use a language or library feature
  newer than the declared MSRV; the `msrv` job will catch it, but it is
  cheaper not to.
- **No `unsafe` outside the two `set_var` calls.** There are exactly two
  `unsafe` blocks in the tree, both `std::env::set_var`, both with a `SAFETY`
  comment.
- **Public items are documented.** The crate is built with `missing_docs`
  denied, so a new public item needs a doc comment.
- **Bugs get a regression test.** A test that fails without the fix.

## Documentation

The design record lives in [`docs/`](docs/README.md). It is numbered, and
the numbering is the reading order.

- **Links are checked.** `tests/doc_tests.rs` verifies that every relative
  link points at a file that exists and that every anchor points at a
  heading that exists. Add the `<a id="...">` marker when a heading's text
  would make a brittle slug — for example, one that embeds a row count.
- **The README's links are absolute.** crates.io renders the README itself
  and rewrites relative links against a base it derives, which is not always
  the repository root. Every link in the README is a full
  `https://github.com/gamersekofy/bart-rs/blob/master/...` URL. There is a
  test that enforces this.
- **Significant decisions get an ADR.** Append one to
  [`docs/06-decisions.md`](docs/06-decisions.md), with the context, the
  decision, the alternatives that were rejected, and the cost. It is the
  document that answers "why is it like this?".

## Releasing

Releases are cut by tagging and pushing:

```console
$ cargo publish --dry-run
$ git tag -a v0.1.0 -m "v0.1.0"
$ git push origin v0.1.0
```

The push starts
[the release workflow](https://github.com/gamersekofy/bart-rs/blob/master/.github/workflows/release.yml),
which re-runs the checks, verifies that the tag and `Cargo.toml` agree on the
version, publishes to crates.io using [trusted publishing](https://crates.io/settings/trusted-publishers)
— so there is no API token to store — and creates a GitHub release.

The version must not already exist on crates.io; a published version cannot
be replaced or removed. Bump it with `cargo set-version` and move the
changelog's `Unreleased` section to the new version.
