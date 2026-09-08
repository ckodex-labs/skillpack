//! Skill frontmatter migration helpers
//!
//! Shared logic for reading/writing SKILL.md YAML frontmatter and ensuring
//! required fields are present. Used by both the CLI and the gRPC service.

use anyhow::Result;

/// Read YAML frontmatter and body from a markdown file.
/// Returns `(Some(mapping), body)` if frontmatter exists and parses as a mapping,
/// `(None, full_content)` otherwise.
pub fn read_skill_md(path: &std::path::Path) -> Result<(Option<serde_yaml::Mapping>, String)> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.split('\n').collect();
    if lines.len() >= 2
        && lines[0].trim() == "---"
        && let Some(end_idx) = lines.iter().skip(1).position(|l| l.trim() == "---")
    {
        let fm_text = lines[1..=end_idx].join("\n");
        let body = lines[end_idx + 2..].join("\n");
        let fm: serde_yaml::Value = serde_yaml::from_str(&fm_text)?;
        return Ok((fm.as_mapping().cloned(), body));
    }
    Ok((None, content))
}

/// Write frontmatter mapping and body back to a markdown file.
pub fn write_skill_md(path: &std::path::Path, fm: &serde_yaml::Mapping, body: &str) -> Result<()> {
    let fm_text = serde_yaml::to_string(fm)?;
    let content = format!("---\n{}---\n{}", fm_text, body);
    std::fs::write(path, content)?;
    Ok(())
}

/// Ensure a top-level field exists in the frontmatter mapping.
/// Returns `true` if the field was inserted.
pub fn ensure_field(fm: &mut serde_yaml::Mapping, key: &str, value: serde_yaml::Value) -> bool {
    let k = serde_yaml::Value::String(key.to_string());
    if !fm.contains_key(&k) {
        fm.insert(k, value);
        true
    } else {
        false
    }
}

/// Ensure a field exists inside the `metadata` mapping.
/// Creates the `metadata` mapping if absent.
/// Returns `true` if the field was inserted (or if metadata was created).
pub fn ensure_metadata_field(
    fm: &mut serde_yaml::Mapping,
    key: &str,
    value: serde_yaml::Value,
) -> bool {
    let meta_key = serde_yaml::Value::String("metadata".to_string());
    let mut created = false;
    let meta = fm
        .entry(meta_key)
        .or_insert_with(|| {
            created = true;
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
        })
        .as_mapping_mut()
        .unwrap();
    let k = serde_yaml::Value::String(key.to_string());
    if !meta.contains_key(&k) {
        meta.insert(k, value);
        true
    } else {
        created
    }
}

/// Migrate a single SKILL.md (or agent/harness .md) file by ensuring required fields.
///
/// * `path` — path to the markdown file.
/// * `name` — value for the `name` field.
/// * `default_ns` — default namespace if missing.
/// * `default_license` — default license if missing.
/// * `default_author` — optional default metadata.author.
///
/// Returns a list of human-readable change descriptions.
pub fn migrate_skill_file(
    path: &std::path::Path,
    name: &str,
    default_ns: &str,
    default_license: &str,
    default_author: Option<&str>,
) -> Result<Vec<String>> {
    let (maybe_fm, body) = read_skill_md(path)?;
    let mut changes = Vec::new();
    let mut fm = match maybe_fm {
        Some(m) => m,
        None => {
            let mut m = serde_yaml::Mapping::new();
            m.insert(
                serde_yaml::Value::String("name".to_string()),
                serde_yaml::Value::String(name.to_string()),
            );
            m.insert(
                serde_yaml::Value::String("description".to_string()),
                serde_yaml::Value::String(format!("Skill for {}.", name.replace('-', " "))),
            );
            m.insert(
                serde_yaml::Value::String("license".to_string()),
                serde_yaml::Value::String(default_license.to_string()),
            );
            m.insert(
                serde_yaml::Value::String("namespace".to_string()),
                serde_yaml::Value::String(default_ns.to_string()),
            );
            let mut meta = serde_yaml::Mapping::new();
            meta.insert(
                serde_yaml::Value::String("version".to_string()),
                serde_yaml::Value::String("1.0.0".to_string()),
            );
            meta.insert(
                serde_yaml::Value::String("status".to_string()),
                serde_yaml::Value::String("active".to_string()),
            );
            if let Some(author) = default_author {
                meta.insert(
                    serde_yaml::Value::String("author".to_string()),
                    serde_yaml::Value::String(author.to_string()),
                );
            }
            m.insert(
                serde_yaml::Value::String("metadata".to_string()),
                serde_yaml::Value::Mapping(meta),
            );
            write_skill_md(path, &m, &body)?;
            changes.push("created frontmatter".to_string());
            return Ok(changes);
        }
    };

    // Ensure namespace (check metadata.namespace fallback)
    let ns_key = serde_yaml::Value::String("namespace".to_string());
    if !fm.contains_key(&ns_key) {
        let meta_ns = fm
            .get(serde_yaml::Value::String("metadata".to_string()))
            .and_then(|v| v.as_mapping())
            .and_then(|m| m.get(&ns_key))
            .and_then(|v| v.as_str());
        if let Some(ns) = meta_ns {
            fm.insert(ns_key.clone(), serde_yaml::Value::String(ns.to_string()));
            if let Some(meta) = fm
                .get_mut(serde_yaml::Value::String("metadata".to_string()))
                .and_then(|v| v.as_mapping_mut())
            {
                meta.remove(&ns_key);
            }
            changes.push("moved metadata.namespace to namespace".to_string());
        } else {
            fm.insert(ns_key, serde_yaml::Value::String(default_ns.to_string()));
            changes.push(format!("added namespace: {}", default_ns));
        }
    }

    // Ensure metadata section exists and is a mapping
    let meta_key = serde_yaml::Value::String("metadata".to_string());
    match fm.get(&meta_key) {
        Some(serde_yaml::Value::Mapping(_)) => {}
        Some(other) => {
            let mut new_meta = serde_yaml::Mapping::new();
            new_meta.insert(
                serde_yaml::Value::String("version".to_string()),
                serde_yaml::Value::String(other.as_str().unwrap_or("1.0.0").to_string()),
            );
            fm.insert(meta_key, serde_yaml::Value::Mapping(new_meta));
            changes.push("normalized metadata to dict".to_string());
        }
        None => {
            fm.insert(
                meta_key,
                serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
            );
            changes.push("added metadata section".to_string());
        }
    }

    if ensure_metadata_field(
        &mut fm,
        "version",
        serde_yaml::Value::String("1.0.0".to_string()),
    ) {
        changes.push("added metadata.version".to_string());
    }
    if ensure_metadata_field(
        &mut fm,
        "status",
        serde_yaml::Value::String("active".to_string()),
    ) {
        changes.push("added metadata.status".to_string());
    }

    if let Some(author) = default_author
        && ensure_metadata_field(
            &mut fm,
            "author",
            serde_yaml::Value::String(author.to_string()),
        )
    {
        changes.push("added metadata.author".to_string());
    }

    if ensure_field(
        &mut fm,
        "license",
        serde_yaml::Value::String(default_license.to_string()),
    ) {
        changes.push("added license".to_string());
    }
    if ensure_field(&mut fm, "name", serde_yaml::Value::String(name.to_string())) {
        changes.push("added name".to_string());
    }

    if !changes.is_empty() {
        write_skill_md(path, &fm, &body)?;
    }
    Ok(changes)
}

/// Structured summary returned by all migration runners.
#[derive(Debug, serde::Serialize)]
pub struct MigrationSummary {
    pub total_scanned: usize,
    pub migrated_count: usize,
    pub already_compliant_count: usize,
    pub migrated_names: Vec<String>,
    pub message: String,
}

impl MigrationSummary {
    pub fn from_results(results: &[(String, Vec<String>)]) -> Self {
        let total_scanned = results.len();
        let migrated: Vec<_> = results.iter().filter(|(_, c)| !c.is_empty()).collect();
        let already_compliant: Vec<_> = results.iter().filter(|(_, c)| c.is_empty()).collect();
        let migrated_names = migrated.iter().map(|(n, _)| n.clone()).collect();
        Self {
            total_scanned,
            migrated_count: migrated.len(),
            already_compliant_count: already_compliant.len(),
            migrated_names,
            message: format!(
                "Scanned {} skills, migrated {}, already compliant {}",
                total_scanned,
                migrated.len(),
                already_compliant.len()
            ),
        }
    }
}

/// Migrate canonical skills under `canonical_root` (or `~/Skills/shared`).
pub fn run_migrate_skills(canonical_root: Option<&str>) -> Result<MigrationSummary> {
    let root: std::path::PathBuf = canonical_root.map(|s| s.into()).unwrap_or_else(|| {
        dirs::home_dir()
            .map(|h| h.join("Skills/shared"))
            .unwrap_or_else(|| std::path::PathBuf::from("~/Skills/shared"))
    });

    if !root.exists() {
        anyhow::bail!("Canonical root not found: {}", root.display());
    }

    let mut results: Vec<(String, Vec<String>)> = Vec::new();

    // Root-level skills
    for entry in std::fs::read_dir(&root)?.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").exists() {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let changes =
                migrate_skill_file(&path.join("SKILL.md"), &name, "default", "Apache-2.0", None)?;
            results.push((name, changes));
        }
    }

    // Namespace-level skills
    for entry in std::fs::read_dir(&root)?.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let ns = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        for sub in std::fs::read_dir(&path)?.flatten() {
            let sub_path = sub.path();
            if sub_path.is_dir() && sub_path.join("SKILL.md").exists() {
                let name = sub_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let changes =
                    migrate_skill_file(&sub_path.join("SKILL.md"), &name, &ns, "Apache-2.0", None)?;
                results.push((name, changes));
            }
        }
    }

    Ok(MigrationSummary::from_results(&results))
}

/// Migrate agent skills under `~/.config/agents/skills`.
pub fn run_migrate_agents() -> Result<MigrationSummary> {
    let root = dirs::home_dir()
        .map(|h| h.join(".config/agents/skills"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config/agents/skills"));

    if !root.exists() {
        anyhow::bail!("Agents root not found: {}", root.display());
    }

    let mut results: Vec<(String, Vec<String>)> = Vec::new();

    for entry in std::fs::read_dir(&root)?.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").exists() {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let changes = migrate_skill_file(
                &path.join("SKILL.md"),
                &name,
                "agents",
                "MIT",
                Some("vercel"),
            )?;
            results.push((name, changes));
        }
    }

    Ok(MigrationSummary::from_results(&results))
}

/// Migrate Claude agent markdown files under `agents_dir` (or `~/.claude/agents`).
pub fn run_migrate_claude_agents(agents_dir: Option<&str>) -> Result<MigrationSummary> {
    let root: std::path::PathBuf = agents_dir.map(|s| s.into()).unwrap_or_else(|| {
        dirs::home_dir()
            .map(|h| h.join(".claude/agents"))
            .unwrap_or_else(|| std::path::PathBuf::from("~/.claude/agents"))
    });

    let mut results: Vec<(String, Vec<String>)> = Vec::new();

    if root.exists() {
        for entry in std::fs::read_dir(&root)?.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                let name = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let changes = migrate_skill_file(&path, &name, "claude", "MIT", Some("ckodex"))?;
                results.push((name, changes));
            }
        }
    }

    // Also migrate example agentic-engineer
    let example = std::path::Path::new(
        "/Users/mchorfa/Documents/projects/runbase/ckodex-skill-pack/examples/agentic-skill-template/agents/agentic-engineer.md",
    );
    if example.exists() {
        let name = example
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let changes = migrate_skill_file(example, &name, "claude", "MIT", Some("ckodex"))?;
        results.push((name, changes));
    }

    Ok(MigrationSummary::from_results(&results))
}

/// Migrate harness rule files under `~/.config/agents/skills/*/rules/*.md`.
pub fn run_migrate_harnesses() -> Result<MigrationSummary> {
    let root = dirs::home_dir()
        .map(|h| h.join(".config/agents/skills"))
        .unwrap_or_else(|| std::path::PathBuf::from("~/.config/agents/skills"));

    if !root.exists() {
        anyhow::bail!("Agents root not found: {}", root.display());
    }

    let mut results: Vec<(String, Vec<String>)> = Vec::new();

    for skill_dir in std::fs::read_dir(&root)?.flatten() {
        let path = skill_dir.path();
        if !path.is_dir() {
            continue;
        }
        let rules_dir = path.join("rules");
        if !rules_dir.exists() {
            continue;
        }
        let skill_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        for rule_entry in std::fs::read_dir(&rules_dir)?.flatten() {
            let rule_path = rule_entry.path();
            if rule_path.is_file() && rule_path.extension().map(|e| e == "md").unwrap_or(false) {
                let name = rule_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let (maybe_fm, body) = read_skill_md(&rule_path)?;
                let mut changes = Vec::new();
                let mut fm = match maybe_fm {
                    Some(m) => m,
                    None => {
                        let mut m = serde_yaml::Mapping::new();
                        m.insert(
                            serde_yaml::Value::String("name".to_string()),
                            serde_yaml::Value::String(name.clone()),
                        );
                        m.insert(
                            serde_yaml::Value::String("description".to_string()),
                            serde_yaml::Value::String(format!(
                                "Rule for {}.",
                                name.replace('-', " ")
                            )),
                        );
                        m.insert(
                            serde_yaml::Value::String("namespace".to_string()),
                            serde_yaml::Value::String(skill_name.clone()),
                        );
                        let mut meta = serde_yaml::Mapping::new();
                        meta.insert(
                            serde_yaml::Value::String("version".to_string()),
                            serde_yaml::Value::String("1.0.0".to_string()),
                        );
                        meta.insert(
                            serde_yaml::Value::String("status".to_string()),
                            serde_yaml::Value::String("active".to_string()),
                        );
                        meta.insert(
                            serde_yaml::Value::String("tags".to_string()),
                            serde_yaml::Value::Sequence(Vec::new()),
                        );
                        m.insert(
                            serde_yaml::Value::String("metadata".to_string()),
                            serde_yaml::Value::Mapping(meta),
                        );
                        write_skill_md(&rule_path, &m, &body)?;
                        changes.push("created frontmatter".to_string());
                        results.push((name, changes));
                        continue;
                    }
                };

                if ensure_field(
                    &mut fm,
                    "namespace",
                    serde_yaml::Value::String(skill_name.clone()),
                ) {
                    changes.push(format!("added namespace: {}", skill_name));
                }
                if ensure_metadata_field(
                    &mut fm,
                    "version",
                    serde_yaml::Value::String("1.0.0".to_string()),
                ) {
                    changes.push("added metadata.version".to_string());
                }
                if ensure_metadata_field(
                    &mut fm,
                    "status",
                    serde_yaml::Value::String("active".to_string()),
                ) {
                    changes.push("added metadata.status".to_string());
                }

                if !changes.is_empty() {
                    write_skill_md(&rule_path, &fm, &body)?;
                }
                results.push((name, changes));
            }
        }
    }

    Ok(MigrationSummary::from_results(&results))
}
