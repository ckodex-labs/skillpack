//! Testing Checker
//!
//! Assesses presence and quality of tests, evals, and CI configuration.

use crate::checkers::common;
use skillpack_domain::{DimensionChecker, DimensionId, Issue, Score, Severity, SkillReader};
use std::path::Path;

pub struct TestingChecker;

impl DimensionChecker for TestingChecker {
    fn dimension(&self) -> DimensionId {
        DimensionId::Testing
    }

    fn check(&self, reader: &dyn SkillReader, path: &Path) -> (Score, Vec<Issue>) {
        if common::is_agentskills(reader, path) {
            // AgentSkills rubric: doc skills rarely carry test suites; what
            // is testable is fixtures, runnable scripts, and verification
            // guidance in the body.
            let mut score = 0.0f64;
            let mut issues = Vec::new();
            let test_files = reader.list_files(path, r"(tests|evals)/.+");
            if !test_files.is_empty() {
                score += 50.0;
            } else {
                issues.push(Issue {
                    dimension: DimensionId::Testing,
                    severity: Severity::Note,
                    message: "no tests/ or evals/ fixtures".into(),
                    file: None,
                    line: None,
                });
            }
            if !reader.list_files(path, r"scripts/.+").is_empty() {
                score += 25.0;
            }
            let body = common::skill_body(reader, path)
                .unwrap_or_default()
                .to_lowercase();
            if body.contains("verify") || body.contains("validate") || body.contains("test") {
                score += 25.0;
            }
            return (Score::dimension(score.min(100.0)), issues);
        }

        let mut score = 0.0f64;
        let mut issues = Vec::new();

        // 1. Tests directory with actual test files (40 pts)
        let test_files = reader.list_files(path, r"tests/.*");
        let has_test_files = test_files.iter().any(|f| {
            f.ends_with(".rs") || f.ends_with(".py") || f.ends_with(".js") || f.ends_with(".ts")
        });
        if has_test_files {
            score += 40.0;
        } else if reader.file_exists(path, "tests") {
            issues.push(Issue {
                dimension: DimensionId::Testing,
                severity: Severity::Warning,
                message: "tests/ directory exists but contains no recognised test files".into(),
                file: Some("tests/".into()),
                line: None,
            });
            score += 10.0;
        } else {
            issues.push(Issue {
                dimension: DimensionId::Testing,
                severity: Severity::Warning,
                message: "No tests directory found".into(),
                file: None,
                line: None,
            });
        }

        // 2. Eval harness with runner config (30 pts)
        if reader.file_exists(path, "evals/runner.yaml") {
            if let Ok(content) = reader.read_file(path, "evals/runner.yaml") {
                if content.contains("tests:") || content.contains("scenarios:") {
                    score += 30.0;
                } else {
                    issues.push(Issue {
                        dimension: DimensionId::Testing,
                        severity: Severity::Warning,
                        message: "evals/runner.yaml missing 'tests:' or 'scenarios:' section"
                            .into(),
                        file: Some("evals/runner.yaml".into()),
                        line: None,
                    });
                    score += 10.0;
                }
            }
        } else if reader.file_exists(path, "evals") {
            score += 10.0;
            issues.push(Issue {
                dimension: DimensionId::Testing,
                severity: Severity::Note,
                message: "evals/ directory exists but no runner.yaml config found".into(),
                file: Some("evals/".into()),
                line: None,
            });
        }

        // 3. CI workflow with actual test jobs (30 pts)
        let workflow_paths = [
            ".github/workflows/test.yml",
            ".github/workflows/ci.yml",
            ".github/workflows/eval.yml",
        ];
        let mut ci_found = false;
        let mut ci_has_tests = false;
        for wf in &workflow_paths {
            if let Ok(content) = reader.read_file(path, wf) {
                ci_found = true;
                if workflow_has_test_job(&content) {
                    ci_has_tests = true;
                    break;
                }
            }
        }
        if ci_has_tests {
            score += 30.0;
        } else if ci_found {
            issues.push(Issue {
                dimension: DimensionId::Testing,
                severity: Severity::Warning,
                message: "CI workflow found but missing test job or run step".into(),
                file: Some(".github/workflows/".into()),
                line: None,
            });
            score += 10.0;
        }

        (Score::dimension(score), issues)
    }
}

/// Heuristic: a GitHub Actions workflow YAML has a test job if it contains
/// `jobs:` block with at least one job name, and a `run:` step that executes
/// a test-related command (cargo test, pytest, jest, npm test, etc.).
fn workflow_has_test_job(content: &str) -> bool {
    let lower = content.to_lowercase();
    if !lower.contains("jobs:") {
        return false;
    }
    let test_cmds = [
        "cargo test",
        "pytest",
        "jest",
        "npm test",
        "yarn test",
        "go test",
        "dotnet test",
        "mvn test",
        "gradle test",
        "bundle exec rspec",
    ];
    test_cmds.iter().any(|cmd| lower.contains(cmd))
}

#[cfg(test)]
mod tests {
    use super::*;
    use skillpack_domain::{SkillIdentity, SkillReaderError};
    use std::collections::HashMap;
    use std::path::Path;

    struct MockReader {
        files: HashMap<String, String>,
    }

    impl SkillReader for MockReader {
        fn read_identity(&self, _path: &Path) -> Result<SkillIdentity, SkillReaderError> {
            Ok(SkillIdentity::new("test", "1.0.0", ""))
        }
        fn file_exists(&self, _path: &Path, relative: &str) -> bool {
            if self.files.contains_key(relative) {
                return true;
            }
            let prefix = format!("{}/", relative);
            self.files.keys().any(|k| k.starts_with(&prefix))
        }
        fn read_file(&self, _path: &Path, relative: &str) -> Result<String, SkillReaderError> {
            self.files
                .get(relative)
                .cloned()
                .ok_or_else(|| SkillReaderError::NotFound(relative.into()))
        }
        fn list_files(&self, _path: &Path, _pattern: &str) -> Vec<String> {
            self.files.keys().cloned().collect()
        }
    }

    #[test]
    fn workflow_has_test_job_detects_cargo_test() {
        let yaml = "name: CI\njobs:\n  test:\n    steps:\n      - run: cargo test\n";
        assert!(workflow_has_test_job(yaml));
    }

    #[test]
    fn workflow_has_test_job_detects_pytest() {
        let yaml = "name: CI\njobs:\n  test:\n    steps:\n      - run: pytest\n";
        assert!(workflow_has_test_job(yaml));
    }

    #[test]
    fn workflow_has_test_job_rejects_no_jobs() {
        let yaml = "name: CI\n";
        assert!(!workflow_has_test_job(yaml));
    }

    #[test]
    fn workflow_has_test_job_rejects_no_test_cmd() {
        let yaml = "name: CI\njobs:\n  build:\n    steps:\n      - run: cargo build\n";
        assert!(!workflow_has_test_job(yaml));
    }

    #[test]
    fn checker_full_score_with_tests_ci_and_evals() {
        let reader = MockReader {
            files: [
                ("tests/unit.rs".into(), "".into()),
                (
                    ".github/workflows/ci.yml".into(),
                    "jobs:\n  test:\n    run: cargo test\n".into(),
                ),
                ("evals/runner.yaml".into(), "tests:\n  - foo\n".into()),
            ]
            .into(),
        };
        let checker = TestingChecker;
        let (score, issues) = checker.check(&reader, Path::new("."));
        assert_eq!(score.value(), 100.0);
        assert!(issues.is_empty());
    }

    #[test]
    fn checker_warns_empty_tests_dir() {
        let reader = MockReader {
            files: [("tests/README.md".into(), "".into())].into(),
        };
        let checker = TestingChecker;
        let (score, issues) = checker.check(&reader, Path::new("."));
        assert_eq!(score.value(), 10.0);
        assert!(
            issues
                .iter()
                .any(|i| i.message.contains("no recognised test files"))
        );
    }

    #[test]
    fn checker_warns_ci_without_test_job() {
        let reader = MockReader {
            files: [
                ("tests/unit.rs".into(), "".into()),
                (
                    ".github/workflows/ci.yml".into(),
                    "jobs:\n  build:\n    run: cargo build\n".into(),
                ),
            ]
            .into(),
        };
        let checker = TestingChecker;
        let (score, issues) = checker.check(&reader, Path::new("."));
        assert!(score.value() >= 50.0); // 40 tests + 10 ci partial
        assert!(
            issues
                .iter()
                .any(|i| i.message.contains("missing test job"))
        );
    }

    #[test]
    fn checker_warns_eval_runner_missing_tests_section() {
        let reader = MockReader {
            files: [
                ("tests/unit.rs".into(), "".into()),
                ("evals/runner.yaml".into(), "name: runner\n".into()),
            ]
            .into(),
        };
        let checker = TestingChecker;
        let (score, issues) = checker.check(&reader, Path::new("."));
        assert!(score.value() >= 50.0); // 40 tests + 10 eval partial
        assert!(issues.iter().any(|i| i.message.contains("tests:")));
    }
}
