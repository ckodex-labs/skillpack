//! Shared skill-path validation for gRPC, HTTP, and MCP entry points.
//!
//! One validator, applied everywhere a user-supplied `skill_path` / `skill_ref`
//! reaches the filesystem reader or a canonical-root join. transport layers keep
//! only error mapping.

use std::path::{Path, PathBuf};

/// Error returned when a user-supplied skill path is rejected.
#[derive(Debug, thiserror::Error)]
pub enum SkillPathError {
    /// Path could not be canonicalized (typo, missing directory; also catches
    /// traversal through `..` that lands outside any allowed root).
    #[error("failed to canonicalize path '{path}': {source}")]
    Canonicalize {
        path: String,
        #[source]
        source: std::io::Error,
    },
    /// Canonical path escapes every allowed root.
    #[error("path '{path}' escapes the allowed root")]
    Escape { path: String },
}

/// One allowed filesystem root for skill paths.
#[derive(Debug, Clone)]
pub struct AllowedRoot(PathBuf);

impl AllowedRoot {
    /// Capture a root and canonicalize it (/".$  .. inside the root string is
    /// normalized away here, not per-request).
    pub fn new(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        Ok(Self(path.into().canonicalize()?))
    }

    /// Root as an absolute path.
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// `true` if `candidate` is `allowed_root` itself or sits inside it.
fn contained_by(candidate: &Path, allowed_root: &Path) -> bool {
    candidate.starts_with(allowed_root)
}

/// Validate a user-supplied skill path against allowed roots.
///
/// Rejects: paths that fail canonicalization (also every `..`-based escape,
/// because the resolved target must exist under a root), symlink targets
/// outside every root (canonicalization resolves symlinks before the
/// containment check), and (before touching the filesystem) any `..`
/// component at all — traversal should fail loudly, not resolve somewhere
/// safe by luck.
///
/// Resolution: an absolute path is canonicalized as-is. A relative path is
/// resolved root-relative — each allowed root is tried (`root.join(raw)`),
/// first existing match wins. For the single-CWD-root transport servers this
/// is exactly the previous CWD-relative behavior, because the sole root IS
/// the canonicalized CWD.
pub fn validate_skill_path(raw: &str, roots: &[AllowedRoot]) -> Result<PathBuf, SkillPathError> {
    if roots.is_empty() {
        return Err(SkillPathError::Escape {
            path: raw.to_string(),
        });
    }

    let contains_parent_component = raw.split('/').any(|c| c == ".." || c.starts_with("..\\"))
        || raw.split('\\').any(|c| c == "..");
    if contains_parent_component {
        return Err(SkillPathError::Escape {
            path: raw.to_string(),
        });
    }

    // Relative: resolve against each root, first canonicalizable match wins.
    // Absolute: single canonicalize. Both then run the containment check.
    let mut last_canonicalize_err: Option<SkillPathError> = None;
    let candidates: Vec<PathBuf> = if Path::new(raw).is_absolute() {
        vec![PathBuf::from(raw)]
    } else {
        roots.iter().map(|r| r.as_path().join(raw)).collect()
    };

    let mut resolved: Option<PathBuf> = None;
    for candidate in candidates {
        match candidate.canonicalize() {
            Ok(canonical) => {
                resolved = Some(canonical);
                break;
            }
            Err(e) => {
                last_canonicalize_err = Some(SkillPathError::Canonicalize {
                    path: raw.to_string(),
                    source: e,
                });
            }
        }
    }

    let canonical = resolved.ok_or_else(|| {
        last_canonicalize_err.unwrap_or_else(|| SkillPathError::Escape {
            path: raw.to_string(),
        })
    })?;

    let in_some_root = roots.iter().any(|r| contained_by(&canonical, r.as_path()));
    if !in_some_root {
        return Err(SkillPathError::Escape {
            path: canonical.display().to_string(),
        });
    }

    Ok(canonical)
}

/// Validate a single path component (skill_ref / candidate_name / target_name)
/// for safe joining under a trusted root.
///
/// Rejects empty strings, `.`/`..`, any separator (both `/`, `\\`, and NUL),
/// plus `\\`-embedded traversal — the same `..` policy as `validate_skill_path`
/// but without filesystem access, for cases where the target need not exist
/// (create / promote flows).
pub fn validate_skill_component(raw: &str) -> Result<&str, SkillPathError> {
    if raw.is_empty()
        || raw == "."
        || raw == ".."
        || raw.contains('/')
        || raw.contains('\\')
        || raw.contains('\0')
    {
        return Err(SkillPathError::Escape {
            path: raw.to_string(),
        });
    }
    Ok(raw)
}

/// Validate against the single default root (current working directory).
pub fn validate_skill_path_cwd(raw: &str) -> Result<PathBuf, SkillPathError> {
    let canonicalize = |e: std::io::Error| SkillPathError::Canonicalize {
        path: raw.to_string(),
        source: e,
    };
    let root =
        AllowedRoot::new(std::env::current_dir().map_err(canonicalize)?).map_err(canonicalize)?;
    validate_skill_path(raw, &[root])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cwd_root() -> AllowedRoot {
        AllowedRoot::new(std::env::current_dir().unwrap()).unwrap()
    }

    #[test]
    fn rejects_parent_component_before_fs() {
        let err = validate_skill_path("skills/../../etc", &[cwd_root()]).unwrap_err();
        assert!(matches!(err, SkillPathError::Escape { .. }));
    }

    #[test]
    fn rejects_windows_backslash_traversal() {
        let err = validate_skill_path("skills\\..\\..\\etc", &[cwd_root()]).unwrap_err();
        assert!(matches!(err, SkillPathError::Escape { .. }));
    }

    #[test]
    fn rejects_absolute_escape() {
        let err = validate_skill_path("/etc/passwd", &[cwd_root()]).unwrap_err();
        assert!(matches!(err, SkillPathError::Escape { .. }));
    }

    #[test]
    fn rejects_nonexistent_within_form() {
        // No `..` in the string, but the target is missing: canonicalize fails.
        let root = cwd_root();
        let err = validate_skill_path("does-not-exist-xyz", &[root]).unwrap_err();
        assert!(matches!(err, SkillPathError::Canonicalize { .. }));
    }

    #[test]
    fn accepts_in_root_existing_path() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("skills/demo")).unwrap();
        let root = AllowedRoot::new(dir.path()).unwrap();
        let resolved = validate_skill_path("skills/demo", std::slice::from_ref(&root)).unwrap();
        assert_eq!(resolved, root.as_path().join("skills/demo"));
    }

    #[test]
    fn dot_resolves_within_allowed_root() {
        // Relative paths resolve against each allowed root (not the process
        // CWD), so `.` lands on the root itself and must validate when the
        // caller permits that root.
        let dir = tempfile::TempDir::new().unwrap();
        let root = AllowedRoot::new(dir.path()).unwrap();
        let resolved = validate_skill_path(".", &[root]).unwrap();
        assert_eq!(resolved, dir.path().canonicalize().unwrap());
    }

    #[test]
    fn symlink_escape_is_rejected() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("skills")).unwrap();
        std::fs::create_dir_all(std::env::temp_dir().join("ckx-outside")).unwrap();
        let root = AllowedRoot::new(dir.path()).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(
            std::env::temp_dir().join("ckx-outside"),
            dir.path().join("skills/link"),
        )
        .unwrap();
        let err = validate_skill_path("skills/link", &[root]).unwrap_err();
        // Rejection reason is resolution-based: canonicalize resolves the
        // symlink, containment then sees the outside target.
        assert!(matches!(err, SkillPathError::Escape { .. }));
    }

    #[test]
    fn empty_roots_rejects() {
        assert!(validate_skill_path(".", &[]).is_err());
    }

    #[test]
    fn cwd_helper_accepts_package_file_and_rejects_escape() {
        // Test harness CWD is the package root: Cargo.toml exists there.
        let ok = validate_skill_path_cwd("Cargo.toml").unwrap();
        assert!(ok.ends_with("Cargo.toml"));
        let bad = validate_skill_path_cwd("../..").unwrap_err();
        assert!(matches!(bad, SkillPathError::Escape { .. }));
    }
}
