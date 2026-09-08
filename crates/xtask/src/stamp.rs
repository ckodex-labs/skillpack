//! Version-stamp gate: root manifest == release manifest == newest changelog.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::common::{changelog_head_version, workspace_version};

fn release_manifest_version(root: &std::path::Path) -> Result<String> {
    let text = fs::read_to_string(root.join("publish/release-manifest.json"))
        .context("read publish/release-manifest.json")?;
    let json: serde_json::Value =
        serde_json::from_str(&text).context("parse publish/release-manifest.json")?;
    json.get("version")
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .context("release-manifest.json: no version string")
}

fn manifest_version(root: &Path) -> Result<String> {
    let text = fs::read_to_string(root.join("Cargo.toml")).context("read root Cargo.toml")?;
    workspace_version(&text)
}

fn changelog_version(root: &Path) -> Result<String> {
    let text = fs::read_to_string(root.join("CHANGELOG.md")).context("read CHANGELOG.md")?;
    changelog_head_version(&text).context("CHANGELOG.md: no version heading")
}

/// `xtask stamp` (check): all three version sources must agree.
/// `xtask stamp --write`: align release manifest and changelog to Cargo.
pub fn run(write: bool) -> Result<bool> {
    let root = crate::common::workspace_root(std::path::Path::new("."))?;
    let cargo_v = manifest_version(&root)?;
    let manifest_v = release_manifest_version(&root)?;
    let changelog_v = changelog_version(&root)?;

    if cargo_v == manifest_v && cargo_v == changelog_v {
        println!("stamp ok: {cargo_v} consistent across cargo/manifest/changelog");
        return Ok(true);
    }
    if !write {
        eprintln!("stamp mismatch: cargo={cargo_v} manifest={manifest_v} changelog={changelog_v}");
        return Ok(false);
    }

    let manifest_path = root.join("publish/release-manifest.json");
    fs::write(
        &manifest_path,
        fs::read_to_string(&manifest_path)?.replace(
            &format!(r#""version": "{manifest_v}""#),
            &format!(r#""version": "{cargo_v}""#),
        ),
    )?;
    if changelog_v != cargo_v {
        let changelog_path = root.join("CHANGELOG.md");
        let updated = fs::read_to_string(&changelog_path)?.replacen(
            "## [Unreleased]",
            &format!("## [Unreleased]\n\n## [{cargo_v}]"),
            1,
        );
        fs::write(&changelog_path, updated)?;
    }
    println!("stamp written: {cargo_v}");
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::changelog_head_version;

    #[test]
    fn picks_newest_bracketed_heading() {
        let text = "# Changelog\n## [Unreleased]\n## [1.0.0-beta.2] - 2026-09-08\n## [1.0.0-beta.1] - 2026-06-05\n";
        assert_eq!(
            changelog_head_version(text).as_deref(),
            Some("1.0.0-beta.2")
        );
    }

    #[test]
    fn unreleased_only_is_none() {
        assert_eq!(changelog_head_version("## [Unreleased]\n"), None);
    }

    #[test]
    fn empty_file_is_none() {
        assert_eq!(changelog_head_version(""), None);
    }
}
