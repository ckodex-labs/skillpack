//! Assessment commands.

use super::*;
use super::cmds_author::format_grade;

pub(super) fn cmd_check(path: &Path) {
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

pub(super) fn cmd_grade(path: &Path) {
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

pub(super) fn cmd_report(path: &Path, format: &str) {
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

pub(super) fn cmd_validate(path: &Path) {
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
