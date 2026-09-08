//! Tier-3 Behavioral Evaluation — runner + orchestration (IO layer).
//!
//! The pure fixture/parse/grade logic lives in `skillpack_domain::behavioral`.
//! This layer runs the agent and isolates it. Every mitigation from the risk
//! analysis (`docs/tier3-behavioral-risk-analysis.md`) is encoded here:
//!
//! - **Isolation**: each case runs in a fresh throwaway workspace that is
//!   deleted afterward; the skill is *copied* in (symlinks skipped) so the run
//!   cannot mutate the real skill or escape via a link.
//! - **Bounded cost**: a per-case wall-clock timeout kills a runaway agent;
//!   `max_cases` caps how many run.
//! - **Safe by default**: there is no default runner command. Behavioral evals
//!   only run when the operator explicitly configures a command AND opts in.
//! - **Mechanical grading**: the trace is graded by asserting on tool calls
//!   (domain layer) — no model reads it, so there is no grader-injection path.

use anyhow::{Context, Result};
use skillpack_domain::behavioral::{
    AgentTrace, BehavioralCase, BehavioralFixture, CaseGrade, grade_case, parse_stream_json,
};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Runs an agent on a task in an isolated workspace, returning the raw
/// line-delimited trace (e.g. Claude Code `--output-format stream-json`).
pub trait BehavioralRunner {
    fn run(&self, workspace: &Path, prompt: &str, timeout: Duration) -> Result<String>;
}

/// Options bounding a behavioral run.
#[derive(Debug, Clone)]
pub struct BehavioralOptions {
    pub timeout: Duration,
    pub max_cases: usize,
}

impl Default for BehavioralOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(120),
            max_cases: 20,
        }
    }
}

/// Runs an operator-configured agent as a subprocess. The command receives the
/// task prompt on stdin and runs with the throwaway workspace as its cwd; the
/// skill under test is copied to `./skill/` within that workspace.
pub struct SubprocessRunner {
    program: String,
    args: Vec<String>,
}

impl SubprocessRunner {
    /// Build from a command spec split into program + args. Empty spec is an
    /// error — there is no default command (safe by default).
    pub fn from_command(spec: &[String]) -> Result<Self> {
        let (program, args) = spec
            .split_first()
            .context("empty behavioral command (set SKILLPACK_BEHAVIORAL_CMD)")?;
        Ok(Self {
            program: program.clone(),
            args: args.to_vec(),
        })
    }
}

impl BehavioralRunner for SubprocessRunner {
    fn run(&self, workspace: &Path, prompt: &str, timeout: Duration) -> Result<String> {
        let mut command = Command::new(&self.program);
        command
            .args(&self.args)
            .current_dir(workspace)
            .env("SKILLPACK_SKILL_DIR", workspace.join("skill"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        // Put the agent in its own process group so a timeout can kill the
        // whole tree — the agent AND any tool subprocesses it spawned — rather
        // than orphaning grandchildren that keep running (and hold the pipe).
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let mut child = command
            .spawn()
            .with_context(|| format!("failed to spawn behavioral runner `{}`", self.program))?;
        let pid = child.id();

        if let Some(mut stdin) = child.stdin.take() {
            // Prompt goes over stdin (avoids arg-length limits / E2BIG) and
            // is dropped to signal EOF.
            let _ = stdin.write_all(prompt.as_bytes());
        }

        // Drain stdout on a thread so the pipe buffer can't deadlock the child.
        let mut stdout = child.stdout.take().context("no stdout pipe")?;
        let (tx, rx) = mpsc::channel();
        let reader = thread::spawn(move || {
            let mut buf = String::new();
            let _ = stdout.read_to_string(&mut buf);
            let _ = tx.send(buf);
        });

        // Enforce the wall-clock timeout: kill a runaway agent and bail
        // immediately (we discard its output on timeout, so we do not wait for
        // the stdout drain — which an orphaned grandchild could otherwise hold
        // open).
        let start = Instant::now();
        loop {
            match child.try_wait()? {
                Some(_) => break,
                None => {
                    if start.elapsed() > timeout {
                        kill_tree(&mut child, pid);
                        let _ = child.wait();
                        anyhow::bail!("behavioral runner exceeded {}s timeout", timeout.as_secs());
                    }
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }

        let out = rx.recv_timeout(Duration::from_secs(5)).unwrap_or_default();
        let _ = reader.join();
        Ok(out)
    }
}

/// Kill the agent and, on unix, its whole process group (the agent is its own
/// group leader, so `kill -KILL -<pid>` reaps its tool subprocesses too).
fn kill_tree(child: &mut std::process::Child, pid: u32) {
    let _ = child.kill();
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .arg("-KILL")
            .arg(format!("-{}", pid))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(not(unix))]
    let _ = pid;
}

/// Run a skill's behavioral fixture. Each case executes in its own throwaway
/// workspace with the skill copied to `./skill/`; the workspace is removed when
/// the case completes.
pub fn run_behavioral(
    skill_dir: &Path,
    fixture: &BehavioralFixture,
    runner: &dyn BehavioralRunner,
    opts: &BehavioralOptions,
) -> Result<Vec<CaseGrade>> {
    let mut grades = Vec::new();
    for case in fixture.cases.iter().take(opts.max_cases) {
        let grade = run_one_case(skill_dir, case, runner, opts)?;
        grades.push(grade);
    }
    Ok(grades)
}

fn run_one_case(
    skill_dir: &Path,
    case: &BehavioralCase,
    runner: &dyn BehavioralRunner,
    opts: &BehavioralOptions,
) -> Result<CaseGrade> {
    let ws = tempfile::TempDir::new().context("could not create throwaway workspace")?;
    copy_dir_no_symlinks(skill_dir, &ws.path().join("skill"))
        .context("could not stage skill into workspace")?;

    let raw = runner.run(ws.path(), &case.prompt, opts.timeout)?;
    let trace: AgentTrace = parse_stream_json(&raw);
    Ok(grade_case(case, &trace))
    // ws (TempDir) drops here → workspace removed.
}

/// Recursively copy a directory, skipping symlinks so a hostile skill cannot
/// use a link to reach outside the copy.
fn copy_dir_no_symlinks(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_symlink() {
            continue;
        } else if ty.is_dir() {
            copy_dir_no_symlinks(&from, &to)?;
        } else if ty.is_file() {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Deterministic runner: returns a canned trace, ignores the real agent.
    struct MockRunner {
        trace: String,
    }
    impl BehavioralRunner for MockRunner {
        fn run(&self, workspace: &Path, _prompt: &str, _t: Duration) -> Result<String> {
            // The skill must have been staged into the workspace.
            assert!(workspace.join("skill/SKILL.md").exists());
            Ok(self.trace.clone())
        }
    }

    fn skill_fixture() -> TempDir {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("SKILL.md"), "---\nname: t\n---\n# T").unwrap();
        dir
    }

    #[test]
    fn orchestrator_stages_grades_and_cleans_up() {
        let skill = skill_fixture();
        let fixture = BehavioralFixture {
            cases: vec![BehavioralCase {
                name: "c1".into(),
                prompt: "do it".into(),
                expect_tools: vec!["Read".into()],
                forbid_tools: vec!["WebFetch".into()],
                expect_output_contains: vec!["done".into()],
            }],
        };
        let runner = MockRunner {
            trace: "{\"type\":\"assistant\",\"message\":{\"content\":[{\"type\":\"tool_use\",\"name\":\"Read\"}]}}\n{\"type\":\"result\",\"result\":\"done\"}".into(),
        };
        let grades = run_behavioral(
            skill.path(),
            &fixture,
            &runner,
            &BehavioralOptions::default(),
        )
        .unwrap();
        assert_eq!(grades.len(), 1);
        assert!(grades[0].passed, "reasons: {:?}", grades[0].reasons);
    }

    #[test]
    fn orchestrator_reports_failure_on_forbidden_tool() {
        let skill = skill_fixture();
        let fixture = BehavioralFixture {
            cases: vec![BehavioralCase {
                name: "egress".into(),
                prompt: "do it".into(),
                expect_tools: vec![],
                forbid_tools: vec!["WebFetch".into()],
                expect_output_contains: vec![],
            }],
        };
        let runner = MockRunner {
            trace: "{\"type\":\"assistant\",\"message\":{\"content\":[{\"type\":\"tool_use\",\"name\":\"WebFetch\"}]}}".into(),
        };
        let grades = run_behavioral(
            skill.path(),
            &fixture,
            &runner,
            &BehavioralOptions::default(),
        )
        .unwrap();
        assert!(!grades[0].passed);
        assert!(grades[0].reasons.iter().any(|r| r.contains("WebFetch")));
    }

    #[test]
    fn max_cases_caps_execution() {
        let skill = skill_fixture();
        let cases = (0..5)
            .map(|i| BehavioralCase {
                name: format!("c{}", i),
                prompt: "p".into(),
                expect_tools: vec![],
                forbid_tools: vec![],
                expect_output_contains: vec![],
            })
            .collect();
        let fixture = BehavioralFixture { cases };
        let runner = MockRunner {
            trace: "{\"type\":\"result\",\"result\":\"ok\"}".into(),
        };
        let opts = BehavioralOptions {
            max_cases: 2,
            ..Default::default()
        };
        let grades = run_behavioral(skill.path(), &fixture, &runner, &opts).unwrap();
        assert_eq!(grades.len(), 2);
    }

    #[test]
    fn from_command_rejects_empty() {
        assert!(SubprocessRunner::from_command(&[]).is_err());
    }

    #[test]
    fn copy_skips_symlinks() {
        let src = TempDir::new().unwrap();
        std::fs::write(src.path().join("real.txt"), "x").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("/etc/hosts", src.path().join("escape")).unwrap();
        let dst = TempDir::new().unwrap();
        copy_dir_no_symlinks(src.path(), &dst.path().join("out")).unwrap();
        assert!(dst.path().join("out/real.txt").exists());
        assert!(!dst.path().join("out/escape").exists());
    }
}
