//! The documentation's links.
//!
//! The `docs/` directory is a design record with a lot of cross-references,
//! and a broken link in it is a small mistake that is easy to make and
//! annoying to find: the file exists, the heading is there, but the anchor
//! does not match.
//!
//! Two things are checked:
//!
//! - every relative link points at a file that exists, and
//! - every anchor points at a heading (or an explicit `<a id=...>`) that
//!   exists in the file it names.
//!
//! Anchors are checked against explicit `<a id="...">` markers rather than
//! against GitHub's generated heading slugs, because a slug that embeds a
//! row count — `fare_rules.txt` — 2500 rows, 5 columns` — breaks the
//! moment BART publishes a feed with a different number of rows.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The repository root.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every markdown file that ships with the crate.
fn markdown_files() -> Vec<PathBuf> {
    let root = root();
    let mut files = vec![root.join("README.md")];
    let docs = root.join("docs");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&docs)
        .expect("the docs directory exists")
        .map(|entry| entry.expect("a docs entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort();
    files.append(&mut entries);
    files
}

/// The link targets in a markdown file, with their link text.
fn links(text: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let bytes: Vec<char> = text.chars().collect();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != '[' {
            index += 1;
            continue;
        }

        // The link text, up to the matching `]`.
        let mut text_end = index + 1;
        while text_end < bytes.len() && bytes[text_end] != ']' {
            text_end += 1;
        }
        if text_end >= bytes.len() || bytes.get(text_end + 1) != Some(&'(') {
            index += 1;
            continue;
        }

        let link_text: String = bytes[index + 1..text_end].iter().collect();

        // The target, up to the matching `)`.
        let mut target_end = text_end + 2;
        while target_end < bytes.len() && bytes[target_end] != ')' {
            target_end += 1;
        }
        if target_end >= bytes.len() {
            index += 1;
            continue;
        }

        let target: String = bytes[text_end + 2..target_end].iter().collect();
        found.push((link_text, target));
        index = target_end + 1;
    }

    found
}

/// The anchors a markdown file defines, both explicit and heading slugs.
fn anchors(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();

    for line in text.lines() {
        let trimmed = line.trim();

        // An explicit anchor: `<a id="stops"></a>`.
        if let Some(rest) = trimmed.strip_prefix("<a id=\"") {
            if let Some(end) = rest.find('"') {
                found.insert(rest[..end].to_string());
            }
            continue;
        }

        // A heading, slugged the way GitHub slugs it: lower case, with
        // punctuation dropped and spaces turned into hyphens.
        if let Some(heading) = trimmed.strip_prefix('#') {
            let slug: String = heading
                .trim()
                .trim_end_matches('#')
                .trim()
                .to_lowercase()
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '-' || c == '_' {
                        c
                    } else if c.is_whitespace() {
                        '-'
                    } else {
                        '\u{0}'
                    }
                })
                .filter(|c| *c != '\u{0}')
                .collect();
            found.insert(slug);
        }
    }

    found
}

#[test]
fn every_relative_link_points_at_a_file_that_exists() {
    let root = root();
    let mut problems = Vec::new();

    for file in markdown_files() {
        let text = std::fs::read_to_string(&file).expect("a readable markdown file");
        let directory = file.parent().expect("a file with a parent");

        for (_, target) in links(&text) {
            // Skip links that leave the repository, and links to an anchor
            // within the same file.
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
                || target.starts_with('#')
            {
                continue;
            }

            let path = target.split('#').next().unwrap_or(&target);
            if path.is_empty() {
                continue;
            }

            let resolved = directory.join(path);
            if !resolved.exists() {
                problems.push(format!(
                    "{} links to {target}, which does not exist",
                    file.strip_prefix(&root).unwrap_or(&file).display()
                ));
            }
        }
    }

    assert!(
        problems.is_empty(),
        "the documentation has links to files that do not exist:\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn every_anchor_points_at_a_heading_that_exists() {
    let root = root();
    let mut problems = Vec::new();

    for file in markdown_files() {
        let text = std::fs::read_to_string(&file).expect("a readable markdown file");
        let directory = file.parent().expect("a file with a parent");

        for (link_text, target) in links(&text) {
            let Some((path, anchor)) = target.split_once('#') else {
                continue;
            };

            // A bare anchor refers to the file it is in.
            let target_file = if path.is_empty() {
                file.clone()
            } else {
                directory.join(path)
            };

            let Ok(target_text) = std::fs::read_to_string(&target_file) else {
                // The file itself is checked by the other test.
                continue;
            };

            if !anchors(&target_text).contains(anchor) {
                problems.push(format!(
                    "{} links to {target}, but there is no such anchor in {}",
                    file.strip_prefix(&root).unwrap_or(&file).display(),
                    target_file
                        .strip_prefix(&root)
                        .unwrap_or(&target_file)
                        .display()
                ));
            }

            let _ = link_text;
        }
    }

    assert!(
        problems.is_empty(),
        "the documentation has links to anchors that do not exist:\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn the_readme_links_to_files_that_are_in_the_repository() {
    // The README is the one file crates.io renders, and it is rendered by
    // crates.io's own markdown renderer, which rewrites relative links
    // against a base it derives itself. Every link in it is therefore
    // absolute, and points into this repository.
    let readme = root().join("README.md");
    let text = std::fs::read_to_string(&readme).expect("a readable README");
    let prefix = "https://github.com/gamersekofy/bart-rs/blob/master/";

    for (_, target) in links(&text) {
        if !target.starts_with("http://") && !target.starts_with("https://") {
            assert!(
                target.starts_with('#'),
                "the README has a relative link, {target:?}, which crates.io \
                 rewrites against the wrong base; use an absolute URL"
            );
            continue;
        }

        if let Some(path) = target.strip_prefix(prefix) {
            assert!(
                root().join(path).exists(),
                "the README links to {path}, which is not in the repository"
            );
        }
    }
}

#[test]
fn the_changelog_names_the_current_version() {
    // A crate whose changelog has drifted from its manifest is a crate
    // whose release notes are a lie.
    let manifest = std::fs::read_to_string(root().join("Cargo.toml")).expect("a readable manifest");
    let version = manifest
        .lines()
        .find_map(|line| line.strip_prefix("version = \""))
        .and_then(|rest| rest.split('"').next())
        .expect("the manifest declares a version");

    let changelog =
        std::fs::read_to_string(root().join("CHANGELOG.md")).expect("a readable changelog");
    assert!(
        changelog.contains(&format!("## [{version}]")),
        "the changelog does not mention version {version}"
    );
}

#[test]
fn the_licence_is_declared_and_present() {
    let manifest = std::fs::read_to_string(root().join("Cargo.toml")).expect("a readable manifest");
    assert!(
        manifest.contains("license = "),
        "the manifest does not declare a licence"
    );
    assert!(
        root().join("LICENSE").exists(),
        "the manifest declares a licence but there is no LICENSE file"
    );
}

#[test]
fn the_toolchain_and_the_manifest_agree_about_the_minimum_rust() {
    // The MSRV in Cargo.toml is a promise. This checks that the promise is
    // at least the floor of the edition, which is the one thing that can
    // be checked without building the whole dependency tree.
    let manifest = std::fs::read_to_string(root().join("Cargo.toml")).expect("a readable manifest");

    let edition = manifest
        .lines()
        .find_map(|line| line.strip_prefix("edition = \""))
        .and_then(|rest| rest.split('"').next())
        .expect("the manifest declares an edition");

    let msrv = manifest
        .lines()
        .find_map(|line| line.strip_prefix("rust-version = \""))
        .and_then(|rest| rest.split('"').next())
        .expect("the manifest declares a rust-version");

    // Edition 2024 needs Rust 1.85; edition 2021 needs 1.56.
    let floor = match edition {
        "2024" => "1.85",
        "2021" => "1.56",
        other => panic!("unknown edition {other}"),
    };

    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .take(2)
            .map(|part| part.parse().expect("a version number"))
            .collect()
    };

    assert!(
        parse(msrv) >= parse(floor),
        "rust-version {msrv} is below the floor of edition {edition}, which is {floor}"
    );
}

/// The toolchain file has to exist for the Nix dev shell to know what to
/// build. This is here because the file is easy to delete by accident.
#[test]
fn the_toolchain_file_exists() {
    assert!(
        root().join("rust-toolchain.toml").exists(),
        "rust-toolchain.toml is missing; the flake reads it to pick the toolchain"
    );
    assert!(
        Path::new(&root().join("flake.nix")).exists(),
        "flake.nix is missing"
    );
}
