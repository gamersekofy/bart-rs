# 09 — Releasing

How to publish `bart-rs` to crates.io, and how the release is wired up.

## What has to be true first

- The name `bart-rs` is unclaimed on crates.io. It was checked on
  2026-10-19; the API returns `crate 'bart-rs' does not exist`.
- `Cargo.toml` declares everything crates.io needs: `description`,
  `license`, `repository`, `homepage`, `documentation`, `readme`,
  `keywords` and `categories`.
- There is a `LICENSE` file, matching the declared `license = "MIT"`.
- There is a `CHANGELOG.md`, and it names the version being published.
- `cargo package` succeeds, and the packaged crate builds and tests on its
  own.
- `cargo deny check` passes: no unignored advisories, and every licence in
  the tree is allowed.

CI checks the last three on every push, so a release cannot be cut from a
tree that would not survive publication.

## Before the first release

crates.io needs to know who is allowed to publish this crate. The
recommended way is **trusted publishing**, which uses the job's OIDC
identity instead of a long-lived API token, so there is no credential to
store or leak.

Go to <https://crates.io/settings/trusted-publishers> and add:

| Field | Value |
| --- | --- |
| Owner | `gamersekofy` |
| Repository | `bart-rs` |
| Workflow | `release.yml` |
| Environment | leave empty |

The workflow name is the file name: `.github/workflows/release.yml`.

If you would rather use an API token, create one at
<https://crates.io/settings/tokens>, add it as a repository secret named
`CARGO_REGISTRY_TOKEN`, and replace the `Get a token from crates.io` step
in `.github/workflows/release.yml` with a plain `cargo publish` that reads
`CARGO_REGISTRY_TOKEN` from the environment. The workflow says as much in a
comment.

## Cutting a release

```console
$ cargo publish --dry-run      # checks everything without uploading
$ git tag -a v0.1.0 -m "v0.1.0"
$ git push origin v0.1.0
```

Pushing the tag is what starts the release. The workflow in
`.github/workflows/release.yml` then:

1. checks the formatting, the lints, the tests and the documentation;
2. checks that the tag and `Cargo.toml` agree on the version, and refuses
   to publish if they do not;
3. gets a short-lived token from crates.io;
4. publishes the crate;
5. creates a GitHub release with the generated notes.

To publish by hand instead, with the token in your environment:

```console
$ cargo publish
```

## After publishing

Wait a few minutes, then check:

- <https://crates.io/crates/bart-rs> — the page renders the README, and
  every link in it works.
- <https://docs.rs/bart-rs> — the API documentation builds. docs.rs builds
  against the declared MSRV, so if the crate does not build on 1.88 the
  build will fail here, which is a good reason to have checked the MSRV
  beforehand.
- <https://docs.rs/crate/bart-rs/latest> — the source is browsable.

Then bump the version for the next release:

```console
$ cargo set-version 0.1.1
$ git commit -am "chore: bump to 0.1.1"
```

The changelog's `Unreleased` section becomes that release's section.

## Things that will bite you

**The README's links are absolute on purpose.** crates.io renders the
README itself and rewrites relative links against a base it derives, which
is not always the repository root. A relative link to `docs/foo.md` can end
up pointing at a path that does not exist. Every link in the README is
therefore an absolute `https://github.com/gamersekofy/bart-rs/blob/master/...`
URL, which resolves identically on GitHub and on crates.io. A test in
`tests/doc_tests.rs` enforces it.

**`docs/` ships with the crate.** The design record travels with the
library, which is the point of writing it down. It costs about 180 kB in
the tarball, which is not worth optimising.

**The version must not already exist.** crates.io will not let a version
be republished, not even to delete it. Bump the version rather than trying
to reuse one.

**`Cargo.lock` is committed.** For a library that is unusual — cargo
ignores it when the library is used as a dependency — but this crate also
ships a binary, and the lockfile is what makes `nix build` reproducible.
The `package` job in CI builds and tests the packaged tarball, which is the
only way to be sure the package is self-contained.

## What the CI does

`.github/workflows/ci.yml`, on every push and pull request:

| Job | What it checks |
| --- | --- |
| `test` | formatting, lints, build, tests, documentation |
| `msrv` | builds and tests with exactly the declared minimum Rust |
| `deny` | advisories, licences, duplicate versions, sources |
| `package` | `cargo package`, then builds and tests the packaged crate |
| `nix` | `nix flake check` and the dev shell |

The `msrv` job is the one that keeps `rust-version` honest: if a
dependency raises its floor, the job fails and the manifest has to be
updated rather than quietly going stale.
