//! IPGuard — Intellectual Property Boundary Protection
//!
//! Prevents paths and skill names containing restricted patterns
//! from entering the canonical store or being synced to agents.

use regex::Regex;
use std::path::Path;
use thiserror::Error;

/// Error raised when an IP boundary violation is detected.
#[derive(Debug, Error, PartialEq)]
pub enum IPGuardError {
    #[error("IP boundary violation: {0}")]
    BoundaryViolation(String),
}

/// Intellectual Property boundary guard.
///
/// Blocks paths or folder names containing restricted patterns:
/// - `thales`
/// - `cortaix-csr`
/// - `ppt-thales`
/// - `ip-pending`
#[derive(Debug, Clone)]
pub struct IPGuard {
    pattern: Regex,
}

impl Default for IPGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl IPGuard {
    /// Create a new `IPGuard` with the default restricted pattern.
    pub fn new() -> Self {
        let pattern = Regex::new("(?i)thales|cortaix-csr|ppt-thales|ip-pending")
            .expect("default IPGuard regex is valid");
        Self { pattern }
    }

    /// Check a file-system path for boundary violations.
    pub fn guard_path(&self, path: &Path) -> Result<(), IPGuardError> {
        let path_str = path.to_string_lossy();
        if self.pattern.is_match(&path_str) {
            return Err(IPGuardError::BoundaryViolation(format!(
                "'{}' matches a restricted IP pattern. Resolve IP counsel clearance and use a different path.",
                path_str
            )));
        }
        Ok(())
    }

    /// Check a skill name for boundary violations.
    pub fn guard_skill_name(&self, name: &str) -> Result<(), IPGuardError> {
        if self.pattern.is_match(name) {
            return Err(IPGuardError::BoundaryViolation(format!(
                "Skill '{}' matches a restricted IP pattern. Remove from canonical store.",
                name
            )));
        }
        Ok(())
    }

    /// Recursively scan a directory for boundary violations up to `max_depth`.
    pub fn scan_directory(&self, root: &Path, max_depth: usize) -> Result<(), IPGuardError> {
        self.scan_directory_inner(root, root, 0, max_depth)
    }

    fn scan_directory_inner(
        &self,
        root: &Path,
        current: &Path,
        depth: usize,
        max_depth: usize,
    ) -> Result<(), IPGuardError> {
        if depth > max_depth {
            return Ok(());
        }

        let mut violations = Vec::new();

        if let Ok(entries) = std::fs::read_dir(current) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                if self.pattern.is_match(&name_str) {
                    violations.push(entry.path().to_string_lossy().to_string());
                }

                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    self.scan_directory_inner(root, &entry.path(), depth + 1, max_depth)?;
                }
            }
        }

        if !violations.is_empty() {
            let hits = violations.join("\n         ");
            return Err(IPGuardError::BoundaryViolation(format!(
                "{} contains restricted IP paths:\n         {}\n         Move them out before syncing.",
                root.display(),
                hits
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_skill_name_safe() {
        let guard = IPGuard::new();
        assert!(guard.guard_skill_name("azure-deploy").is_ok());
        assert!(guard.guard_skill_name("security-best-practices").is_ok());
    }

    #[test]
    fn test_guard_skill_name_violation() {
        let guard = IPGuard::new();
        assert!(guard.guard_skill_name("thales-csr").is_err());
        assert!(guard.guard_skill_name("CORTAIX-CSR").is_err());
        assert!(guard.guard_skill_name("ppt-thales").is_err());
        assert!(guard.guard_skill_name("ip-pending-review").is_err());
    }

    #[test]
    fn test_guard_path_violation() {
        let guard = IPGuard::new();
        let path = Path::new("/Users/dev/skills/thales-internal");
        assert!(guard.guard_path(path).is_err());
    }
}
