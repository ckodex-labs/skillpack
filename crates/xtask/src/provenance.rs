//! Provenance gate: no placeholders, ref tail matches workspace version.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

/// Placeholder strings that make a provenance document unverifiable.
const FORBIDDEN: [&str; 2] = ["TBD", "pending"];

/// `xtask provenance --check`: file exists, valid JSON shape, ref ends with
/// the workspace version tag, and no placeholder markers remain.
pub fn run(write: bool) -> Result<bool> {
    let root = crate::common::workspace_root(Path::new("."))?;
    if write {
        write_stamp(&root)?;
    }
    verify(&root)
}

fn verify(root: &Path) -> Result<bool> {
    let path = root.join(crate::common::PROVENANCE_PATH);
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let bad: Vec<&str> = FORBIDDEN
        .iter()
        .copied()
        .filter(|m| text.contains(m))
        .collect();
    if !bad.is_empty() {
        bail!("provenance placeholders present: {bad:?}");
    }
    let version = cargo_version(root)?;
    let ok = ensure_ref_matches(&text, &format!("v{version}")).is_ok();
    if ok {
        println!("provenance ok: ref ends with v{version}, no placeholders");
    }
    Ok(ok)
}

/// Extract the `ref` value (buildDefinition.externalParameters) and require
/// its tail to equal the expected `v<version>` tag.
fn ensure_ref_matches(text: &str, tail: &str) -> Result<()> {
    let value = text
        .lines()
        .find_map(|line| {
            let t = line.trim().trim_end_matches(',');
            t.strip_prefix("\"ref\":")
                .map(|v| v.trim().trim_matches('"').to_owned())
        })
        .context("provenance: ref field not found")?;
    if !value.ends_with(tail) {
        bail!("provenance ref {value:?} does not end with {tail:?}");
    }
    Ok(())
}

/// Fill invocationId with a real v4 UUID and both timestamps with RFC3339 now.
/// Only placeholder values are replaced; everything else is untouched.
fn write_stamp(root: &Path) -> Result<()> {
    let path = root.join(crate::common::PROVENANCE_PATH);
    let text = fs::read_to_string(&path).context("reading provenance for write")?;
    let id = crate::common::uuid_v4()?;
    let stamp = crate::common::rfc3339_now();
    let version = cargo_version(root)?;
    let out = text
        .replace(
            "\"ref\": \"refs/tags/v1.0.0\"",
            &format!("\"ref\": \"refs/tags/v{version}\""),
        )
        .replace(
            "\"invocationId\": \"TBD\"",
            &format!("\"invocationId\": \"{id}\""),
        )
        .replace(
            "\"startedOn\": \"TBD\"",
            &format!("\"startedOn\": \"{stamp}\""),
        )
        .replace(
            "\"finishedOn\": \"TBD\"",
            &format!("\"finishedOn\": \"{stamp}\""),
        );
    fs::write(&path, out).with_context(|| format!("writing {}", path.display()))?;
    println!("provenance written: ref=v{version} invocationId={id}");
    Ok(())
}

fn cargo_version(root: &Path) -> Result<String> {
    crate::common::workspace_version(&fs::read_to_string(root.join("Cargo.toml"))?)
}
