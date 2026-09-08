// xtask helpers: workspace discovery, version extraction, changelog parsing,
// output tails, time and UUID v4. Dependency-light by design.

use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, bail, ensure};

/// Workspace root marker used while walking up from cwd.
const WORKSPACE_MARKER: &str = "[workspace]";

/// Find the workspace root by walking up from `start` until a manifest
/// containing `[workspace]` is found.
pub fn workspace_root(start: &Path) -> anyhow::Result<PathBuf> {
    let mut dir = start
        .canonicalize()
        .with_context(|| format!("cannot canonicalize {}", start.display()))?;
    loop {
        let manifest = dir.join("Cargo.toml");
        let has_marker = fs::read_to_string(&manifest)
            .map(|t| t.contains(WORKSPACE_MARKER))
            .unwrap_or(false);
        if has_marker {
            return Ok(dir);
        }
        ensure!(dir.pop(), "no workspace manifest above {}", start.display());
    }
}

/// Extract `workspace.package.version` from the root manifest text.
/// Pure function; tests cover parse and missing-field paths.
pub fn workspace_version(manifest: &str) -> anyhow::Result<String> {
    let mut in_table = false;
    for line in manifest.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_table = t == "[workspace.package]";
            continue;
        }
        if !in_table {
            continue;
        }
        if let Some(rest) = t.strip_prefix("version")
            && let Some(value) = rest.trim_start().strip_prefix('=')
        {
            let quoted = value.split('"').nth(1).unwrap_or_default();
            ensure!(!quoted.is_empty(), "empty workspace.package.version");
            return Ok(quoted.to_string());
        }
    }
    bail!("workspace.package.version not found in root manifest");
}

/// Newest `## [x.y.z]` heading in a CHANGELOG, ignoring `[Unreleased]`.
/// Pure function; tests cover normal, empty and Unreleased-only inputs.
pub fn changelog_head_version(text: &str) -> Option<String> {
    text.lines()
        .filter_map(|l| l.strip_prefix("## ["))
        .filter_map(|rest| rest.split(']').next())
        .filter(|v| !v.eq_ignore_ascii_case("unreleased"))
        .map(|v| v.trim().to_owned())
        .next()
}

/// Keep the last `keep` lines of `text` for stage output tails.
pub fn tail_lines(text: &str, keep: usize) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(keep);
    lines[start..].iter().map(|s| (*s).to_string()).collect()
}

/// RFC 4122 v4 UUID from OS entropy.
pub fn uuid_v4() -> anyhow::Result<String> {
    let mut raw = [0u8; 16];
    fs::File::open("/dev/urandom")
        .with_context(|| "open /dev/urandom".to_string())?
        .read_exact(&mut raw)
        .with_context(|| "read /dev/urandom".to_string())?;
    raw[6] = (raw[6] & 0x0f) | 0x40;
    raw[8] = (raw[8] & 0x3f) | 0x80;
    let hex: String = raw.iter().map(|b| format!("{b:02x}")).collect();
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    ))
}

/// Current UTC moment as an RFC 3339 timestamp (second resolution).
pub fn rfc3339_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let (y, mo, d) = civil_from_days(i64::try_from(secs / 86_400).unwrap_or_default());
    let rem = secs % 86_400;
    format!(
        "{y:04}-{mo:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Days since 1970-01-01 to (year, month, day), proleptic Gregorian.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (
        year,
        u32::try_from(month).unwrap_or_default(),
        u32::try_from(day).unwrap_or_default(),
    )
}

/// Provenance file path, relative to the workspace root.
pub const PROVENANCE_PATH: &str = "proof/provenance/slsa-provenance.json";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_from_workspace_package() {
        let manifest =
            "[package]\nname = \"x\"\n\n[workspace.package]\nversion = \"1.0.0-beta.2\"\n";
        assert_eq!(
            workspace_version(manifest).ok().as_deref(),
            Some("1.0.0-beta.2")
        );
    }

    #[test]
    fn version_missing_is_error() {
        let manifest = "[package]\nname = \"x\"\n";
        assert!(workspace_version(manifest).is_err());
    }

    #[test]
    fn head_version_prefers_newest() {
        let text = "# C\n## [Unreleased]\n## [1.2.3] - 2020-01-01\n## [1.1.0] - 2019-01-01\n";
        assert_eq!(changelog_head_version(text).as_deref(), Some("1.2.3"));
    }

    #[test]
    fn head_version_empty_is_none() {
        assert_eq!(changelog_head_version(""), None);
    }

    #[test]
    fn head_version_unreleased_only_is_none() {
        assert_eq!(changelog_head_version("## [Unreleased]\n"), None);
    }
}
