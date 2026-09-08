//! Canonical store filesystem discovery.
//!
//! Pure I/O: enumerates the skill directories in a canonical store root. Kept in
//! the adapter layer so the application/transport layers stay free of direct
//! filesystem walking.

use std::path::Path;

/// List the immediate skill directories under `root`: subdirectories that
/// directly contain a `SKILL.md` or `skill.cnsb.json`. Returns absolute path
/// strings. On an unreadable root, returns an empty list.
pub fn list_skill_dirs(root: &Path) -> Vec<String> {
    let has_skill = |d: &Path| d.join("SKILL.md").exists() || d.join("skill.cnsb.json").exists();
    match std::fs::read_dir(root) {
        Ok(entries) => entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir() && has_skill(p))
            .map(|p| p.display().to_string())
            .collect(),
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn lists_only_dirs_with_a_skill_manifest() {
        let root = TempDir::new().unwrap();
        // a real skill
        let a = root.path().join("alpha");
        fs::create_dir(&a).unwrap();
        fs::write(a.join("SKILL.md"), "---\nname: alpha\n---\n#").unwrap();
        // a cnsb skill
        let b = root.path().join("beta");
        fs::create_dir(&b).unwrap();
        fs::write(b.join("skill.cnsb.json"), "{}").unwrap();
        // a non-skill dir
        let c = root.path().join("gamma");
        fs::create_dir(&c).unwrap();
        fs::write(c.join("README.md"), "no skill here").unwrap();

        let mut names: Vec<String> = list_skill_dirs(root.path())
            .iter()
            .map(|p| {
                Path::new(p)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        names.sort();
        assert_eq!(names, vec!["alpha".to_string(), "beta".to_string()]);
    }

    #[test]
    fn empty_on_missing_root() {
        assert!(list_skill_dirs(Path::new("/nonexistent/skillpack/root")).is_empty());
    }
}
