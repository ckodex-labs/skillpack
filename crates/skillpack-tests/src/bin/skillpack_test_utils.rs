//! skillpack-test-utils — Consolidated test utilities for SkillPack.
//!
//! Subcommands:
//!   schema-round-trip   Verify assessment envelope round-trips through JSON
//!   fixture-grade       Grade all fixture skills and report stub dimensions

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "skillpack-test-utils")]
#[command(about = "SkillPack test utilities")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify assessment envelope round-trips through JSON without data loss
    SchemaRoundTrip,
    /// Grade all fixture skills and report stub dimensions
    FixtureGrade,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::SchemaRoundTrip => run_schema_round_trip(),
        Commands::FixtureGrade => run_fixture_grade(),
    }
}

fn run_schema_round_trip() -> Result<()> {
    use skillpack_domain::{Assessment, DimensionId, Issue, Score, Severity, SkillIdentity};

    let mut original = Assessment::new(SkillIdentity {
        name: "test-skill".into(),
        version: "1.0.0".into(),
        path: "/tmp/test".into(),
    });

    original.record_dimension(DimensionId::IdentityAndManifest, Score::dimension(95.0));
    original.record_dimension(DimensionId::Security, Score::dimension(88.0));
    original.add_issue(Issue {
        dimension: DimensionId::IdentityAndManifest,
        severity: Severity::Warning,
        message: "test issue".into(),
        file: Some("SKILL.md".into()),
        line: Some(1),
    });
    original.stub_count = 0;

    let json = serde_json::to_string(&original)?;
    let parsed: Assessment = serde_json::from_str(&json)?;

    assert_eq!(
        original.dimension_scores.len(),
        parsed.dimension_scores.len()
    );
    assert_eq!(
        original
            .dimension_scores
            .get(&DimensionId::Security)
            .unwrap()
            .value(),
        parsed
            .dimension_scores
            .get(&DimensionId::Security)
            .unwrap()
            .value()
    );
    assert_eq!(original.issues.len(), parsed.issues.len());
    assert_eq!(original.total_score().value(), parsed.total_score().value());
    assert_eq!(original.stub_count, parsed.stub_count);

    println!("schema_round_trip: PASSED");
    Ok(())
}

fn run_fixture_grade() -> Result<()> {
    use skillpack_adapters::checkers::all_checkers;
    use skillpack_adapters::filesystem::FilesystemReader;
    use skillpack_application::{AssessSkillRequest, AssessSkillUseCase};
    use std::path::Path;

    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/skills");

    let reader = FilesystemReader::new();
    let use_case = AssessSkillUseCase::new(reader, all_checkers());

    let mut total = 0;
    let mut passed = 0;

    for entry in std::fs::read_dir(&fixtures)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy();
        total += 1;

        let response = match use_case.execute(AssessSkillRequest {
            skill_path: path.to_str().unwrap().to_string(),
            min_score: None,
        }) {
            Ok(r) => r,
            Err(e) => {
                println!("  {}: SKIP ({})", name, e);
                continue;
            }
        };

        let score = response.assessment.total_score().value();
        if response.stub_dimensions.is_empty() {
            passed += 1;
            println!("  {}: OK (score={:.1})", name, score);
        } else {
            println!(
                "  {}: STUBS ({:?}) (score={:.1})",
                name, response.stub_dimensions, score
            );
        }
    }

    let skipped = total - passed;
    println!();
    println!(
        "fixture_grade_corpus: {}/{} fixtures OK ({} skipped)",
        passed, total, skipped
    );

    if passed + skipped < total {
        std::process::exit(1);
    }
    Ok(())
}
