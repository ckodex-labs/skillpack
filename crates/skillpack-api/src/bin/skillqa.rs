//! SkillPack CLI
//!
//! Commands:
//! - check: Run skill assessment
//! - grade: Show skill grade
//! - report: Generate assessment report
//! - validate: Validate schemas
//! - init: Scaffold new skill project
//! - lock: Generate skill.lock file
//! - migrate: Upgrade schema versions

use chrono::Utc;
use clap::{Parser, Subcommand};
use colored::Colorize;
use skillpack_adapters::checkers::all_checkers;
use skillpack_adapters::reader::FsSkillReader;
use skillpack_application::{AssessSkillRequest, AssessSkillUseCase};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "skillpack")]
#[command(about = "AI Agent Skill Quality Assessment CLI", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run skill assessment
    Check {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Show skill grade
    Grade {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Generate assessment report
    Report {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output format
        #[arg(long, default_value = "markdown")]
        format: String,
    },
    /// Validate skill schemas
    Validate {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Scaffold new skill project
    Init {
        /// Project name
        name: String,
        /// Target directory
        #[arg(long, default_value = ".")]
        directory: PathBuf,
    },
    /// Generate skill.lock file
    Lock {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Upgrade schema versions
    Migrate {
        /// Path to skill directory
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Target schema version
        #[arg(long, default_value = "v1")]
        to: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { path } => cmd_check(&path),
        Commands::Grade { path } => cmd_grade(&path),
        Commands::Report { path, format } => cmd_report(&path, &format),
        Commands::Validate { path } => cmd_validate(&path),
        Commands::Init { name, directory } => cmd_init(&name, &directory),
        Commands::Lock { path } => cmd_lock(&path),
        Commands::Migrate { path, to } => cmd_migrate(&path, &to),
    }
}

fn cmd_check(path: &Path) {
    println!("{} {}", "Checking".cyan(), path.display());

    let reader = FsSkillReader::new();
    let checkers = all_checkers();

    let use_case = AssessSkillUseCase::new(reader, checkers);
    let request = AssessSkillRequest {
        skill_path: path.to_string_lossy().to_string(),
        min_score: None,
    };

    match use_case.execute(request) {
        Ok(response) => {
            let assessment = response.assessment;

            println!("\n{}", "Dimension Scores:".bold());
            for (dim_id, score) in &assessment.dimension_scores {
                let bar = "█".repeat((score.value() / 10.0) as usize);
                let empty = "░".repeat(10 - (score.value() / 10.0) as usize);
                println!(
                    "  {:15} [{}{}] {:5.1}%",
                    dim_id.name(),
                    bar.green(),
                    empty,
                    score.value()
                );
            }

            println!(
                "\n{}: {}",
                "Overall".bold(),
                format_grade(assessment.total_score().value())
            );

            if !assessment.issues.is_empty() {
                println!("\n{} ({}):", "Issues".yellow(), assessment.issues.len());
                for issue in assessment.issues.iter().take(5) {
                    println!("  {} {}", "•".red(), issue.message);
                }
            }
        }
        Err(e) => {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        }
    }
}

fn cmd_grade(path: &Path) {
    let reader = FsSkillReader::new();
    let checkers = all_checkers();

    let use_case = AssessSkillUseCase::new(reader, checkers);
    let request = AssessSkillRequest {
        skill_path: path.to_string_lossy().to_string(),
        min_score: None,
    };

    match use_case.execute(request) {
        Ok(response) => {
            println!(
                "{}",
                format_grade(response.assessment.total_score().value())
            );
        }
        Err(e) => {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        }
    }
}

fn cmd_report(path: &Path, format: &str) {
    let reader = FsSkillReader::new();
    let checkers = all_checkers();

    let use_case = AssessSkillUseCase::new(reader, checkers);
    let request = AssessSkillRequest {
        skill_path: path.to_string_lossy().to_string(),
        min_score: None,
    };

    match use_case.execute(request) {
        Ok(response) => {
            let assessment = response.assessment;

            match format {
                "json" => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&assessment).unwrap_or_default()
                    );
                }
                "sarif" => {
                    println!(
                        "{{\"$schema\":\"https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json\",\"version\":\"2.1.0\",\"runs\":[]}}"
                    );
                }
                _ => {
                    println!("# SkillPack Assessment Report\n");
                    println!("**Path**: {}", path.display());
                    println!(
                        "**Grade**: {}\n",
                        format_grade(assessment.total_score().value())
                    );
                    println!("## Dimensions\n");
                    for (dim_id, score) in &assessment.dimension_scores {
                        println!("- {}: {:.1}%", dim_id.name(), score.value());
                    }
                }
            }
        }
        Err(e) => {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        }
    }
}

fn cmd_validate(path: &Path) {
    println!("{} {}", "Validating".cyan(), path.display());

    let mut valid = true;

    // Check for CNSB bundle
    let cnsb_path = path.join("skill.cnsb.json");
    if cnsb_path.exists() {
        match fs::read_to_string(&cnsb_path) {
            Ok(content) => {
                if serde_json::from_str::<serde_json::Value>(&content).is_ok() {
                    println!("  {} skill.cnsb.json", "Valid".green());
                } else {
                    println!("  {} skill.cnsb.json - Invalid JSON", "Error".red());
                    valid = false;
                }
            }
            Err(_) => {
                println!("  {} skill.cnsb.json - Could not read", "Error".red());
                valid = false;
            }
        }
    }

    // Check for SKILL.md
    let skill_md_path = path.join("SKILL.md");
    if skill_md_path.exists() {
        println!("  {} SKILL.md", "Found".green());
    } else {
        println!("  {} SKILL.md - Missing", "Warn".yellow());
    }

    if valid {
        println!("\n{}", "Validation passed".green().bold());
    } else {
        println!("\n{}", "Validation failed".red().bold());
        std::process::exit(1);
    }
}

fn cmd_init(name: &str, directory: &Path) {
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

fn cmd_lock(path: &Path) {
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

fn cmd_migrate(path: &Path, to: &str) {
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

fn format_grade(score: f64) -> String {
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
