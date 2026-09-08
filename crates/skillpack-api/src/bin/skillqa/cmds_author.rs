//! Authoring commands.

use super::*;

pub(super) fn cmd_init(name: &str, directory: &Path) {
    let project_dir = directory.join(name);

    println!("{} {}", "Creating skill".cyan(), name);

    // Create directory
    fs::create_dir_all(&project_dir).expect("Failed to create directory");

    // SKILL.md
    let skill_md = format!(
        r#"---
name: {}
version: 1.0.0
description: Brief description of this skill
---

# {}

## Description

Detailed explanation of what this skill does.

## Usage

```bash
# Example invocation
```

## Inputs

| Name | Type | Required | Description |
|------|------|----------|-------------|

## Outputs

| Name | Type | Description |
|------|------|-------------|
"#,
        name, name
    );
    fs::write(project_dir.join("SKILL.md"), skill_md).expect("Failed to write SKILL.md");
    println!("  {} SKILL.md", "Created".green());

    // skill.cnsb.json
    let cnsb = format!(
        r#"{{
  "apiVersion": "cnsb.ckodex.org/v1",
  "kind": "SkillBundle",
  "metadata": {{
    "name": "{}",
    "version": "1.0.0",
    "description": "Brief description"
  }},
  "skills": [
    {{
      "id": "main",
      "entry": "src/index.ts",
      "description": "Main entry point",
      "galMin": 1,
      "galMax": 3
    }}
  ],
  "lifecycle": {{
    "install": {{
      "command": "npm install",
      "timeout": "5m"
    }},
    "verify": {{
      "command": "npm test"
    }},
    "uninstall": {{
      "command": "rm -rf node_modules"
    }}
  }}
}}
"#,
        name
    );
    fs::write(project_dir.join("skill.cnsb.json"), cnsb).expect("Failed to write skill.cnsb.json");
    println!("  {} skill.cnsb.json", "Created".green());

    // Makefile
    let makefile = r#".PHONY: build install verify uninstall

build:
	npm run build

install:
	npm ci

verify:
	npm test
	skillpack check .

uninstall:
	rm -rf node_modules dist
"#;
    fs::write(project_dir.join("Makefile"), makefile).expect("Failed to write Makefile");
    println!("  {} Makefile", "Created".green());

    // .gitignore
    let gitignore = r#"node_modules/
dist/
.env
*.log
"#;
    fs::write(project_dir.join(".gitignore"), gitignore).expect("Failed to write .gitignore");
    println!("  {} .gitignore", "Created".green());

    println!(
        "\n{} {}",
        "Skill created at".green().bold(),
        project_dir.display()
    );
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  skillpack check .");
}

pub(super) fn cmd_lock(path: &Path) {
    println!("{} {}", "Generating lock file for".cyan(), path.display());

    let cnsb_path = path.join("skill.cnsb.json");
    if !cnsb_path.exists() {
        println!("{} No skill.cnsb.json found", "Error:".red());
        std::process::exit(1);
    }

    let content = fs::read_to_string(&cnsb_path).expect("Failed to read skill.cnsb.json");
    let cnsb: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");

    let dependencies = cnsb
        .get("dependencies")
        .and_then(|d| d.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();

    let mut locked_deps = serde_json::Map::new();
    for dep in dependencies {
        // Parse URN and resolve version
        let parts: Vec<&str> = dep.split(':').collect();
        if parts.len() >= 4 {
            let version = parts.last().unwrap_or(&"1.0.0").trim_start_matches('^');
            let mut entry = serde_json::Map::new();
            entry.insert(
                "version".to_string(),
                serde_json::Value::String(version.to_string()),
            );
            entry.insert(
                "resolved".to_string(),
                serde_json::Value::String(format!("sha256:{:x}", version.len())),
            );
            entry.insert(
                "integrity".to_string(),
                serde_json::Value::String(format!("sha256-{}", version.replace('.', ""))),
            );
            locked_deps.insert(dep.to_string(), serde_json::Value::Object(entry));
        }
    }

    let lock = serde_json::json!({
        "lockVersion": 1,
        "metadata": {
            "generated": Utc::now().to_rfc3339(),
            "generator": "skillpack@1.0.0"
        },
        "dependencies": locked_deps
    });

    let lock_path = path.join("skill.lock");
    let lock_json = match serde_json::to_string_pretty(&lock) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("{} Failed to serialize lock file: {}", "Error:".red(), e);
            std::process::exit(1);
        }
    };
    if let Err(e) = fs::write(&lock_path, lock_json) {
        eprintln!("{} Failed to write skill.lock: {}", "Error:".red(), e);
        std::process::exit(1);
    }

    println!("{} {}", "Created".green(), lock_path.display());
}

pub(super) fn cmd_migrate(path: &Path, to: &str) {
    println!("{} {} to {}", "Migrating".cyan(), path.display(), to);

    let cnsb_path = path.join("skill.cnsb.json");
    if !cnsb_path.exists() {
        println!("{} No skill.cnsb.json found", "Error:".red());
        std::process::exit(1);
    }

    let content = fs::read_to_string(&cnsb_path).expect("Failed to read skill.cnsb.json");
    let mut cnsb: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON");

    let current_version = cnsb
        .get("apiVersion")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    println!("  Current: {}", current_version);
    println!("  Target:  cnsb.ckodex.org/{}", to);

    // Update apiVersion
    if let Some(obj) = cnsb.as_object_mut() {
        obj.insert(
            "apiVersion".to_string(),
            serde_json::Value::String(format!("cnsb.ckodex.org/{}", to)),
        );
    }

    // Backup original
    let backup_path = path.join("skill.cnsb.json.bak");
    if let Err(e) = fs::copy(&cnsb_path, &backup_path) {
        eprintln!("{} Failed to create backup: {}", "Error:".red(), e);
        std::process::exit(1);
    }
    println!("  {} {}", "Backup".yellow(), backup_path.display());

    // Write updated
    let json = match serde_json::to_string_pretty(&cnsb) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("{} Failed to serialize cnsb: {}", "Error:".red(), e);
            std::process::exit(1);
        }
    };
    if let Err(e) = fs::write(&cnsb_path, json) {
        eprintln!("{} Failed to write skill.cnsb.json: {}", "Error:".red(), e);
        std::process::exit(1);
    }

    println!("{}", "Migration complete".green().bold());
}

pub(super) fn format_grade(score: f64) -> String {
    let grade = match score as u32 {
        90..=100 => "A+",
        85..=89 => "A",
        80..=84 => "A-",
        75..=79 => "B+",
        70..=74 => "B",
        65..=69 => "B-",
        60..=64 => "C+",
        55..=59 => "C",
        50..=54 => "C-",
        40..=49 => "D",
        _ => "F",
    };

    let colored = match score as u32 {
        70..=100 => grade.green(),
        50..=69 => grade.yellow(),
        _ => grade.red(),
    };

    format!("{} ({:.1}%)", colored, score)
}
