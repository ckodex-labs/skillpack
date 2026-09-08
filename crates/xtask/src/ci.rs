//! CI gate pipeline: stages fan out over a fanring MPSC channel.

use std::process::{Command, Stdio};
use std::time::Instant;

use fanring::mpsc::{Receiver, channel};
use serde::Serialize;

use crate::common::tail_lines;

const OUTPUT_TAIL_LIMIT: usize = 40;
const CHANNEL_CAPACITY: usize = 64;
const STAGES: [&str; 4] = ["fmt", "clippy", "test", "smoke"];

#[derive(Debug, Serialize)]
pub struct StageResult {
    pub stage: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub output_tail: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SkippedEvent {
    stage: String,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct GateSummary {
    gate: &'static str,
    passed: bool,
    ran: usize,
    failed: usize,
    skipped: usize,
}

pub fn run() -> anyhow::Result<bool> {
    let (tx, mut rx): (fanring::mpsc::Sender<StageResult>, Receiver<StageResult>) =
        channel(CHANNEL_CAPACITY);

    let mut results = Vec::with_capacity(STAGES.len());
    let mut overall = true;
    let mut saw_failure = false;
    let mut ran = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;

    for stage in STAGES {
        if saw_failure {
            let marker = SkippedEvent {
                stage: (*stage).to_string(),
                status: "skipped",
            };
            println!("{}", serde_json::to_string(&marker).unwrap_or_default());
            results.push(skipped_result(stage));
            skipped += 1;
            continue;
        }
        let mut sender = tx
            .try_register()
            .map_err(|e| anyhow::anyhow!("stage lane registration failed: {e}"))?;
        let stage_name = (*stage).to_string();
        let handle = std::thread::spawn(move || {
            let started = Instant::now();
            let (passed, tail) = execute(&stage_name);
            let done = StageResult {
                stage: stage_name,
                passed,
                duration_ms: started.elapsed().as_millis() as u64,
                output_tail: tail,
            };
            let _ = sender.send(done);
        });
        let event = rx
            .recv()
            .map_err(|_| anyhow::anyhow!("result channel closed"))?;
        handle
            .join()
            .map_err(|_| anyhow::anyhow!("stage thread panicked"))?;
        println!("{}", serde_json::to_string(&event).unwrap_or_default());
        if event.passed {
            ran += 1;
        } else {
            failed += 1;
            saw_failure = true;
            overall = false;
        }
        results.push(event);
    }
    drop(tx);
    println!(
        "{}",
        serde_json::to_string(&GateSummary {
            gate: "ci",
            passed: overall,
            ran,
            failed,
            skipped
        })
        .unwrap_or_default()
    );
    Ok(overall)
}

fn skipped_result(stage: &str) -> StageResult {
    StageResult {
        stage: stage.to_string(),
        passed: false,
        duration_ms: 0,
        output_tail: vec!["skipped".to_string()],
    }
}

fn execute(stage: &str) -> (bool, Vec<String>) {
    match stage {
        "fmt" => exec(command("cargo", &["fmt", "--all", "--", "--check"])),
        "clippy" => exec(command(
            "cargo",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
                "-W",
                "clippy::unit_cmp",
                "-W",
                "clippy::large_enum_variant",
            ],
        )),
        "test" => exec(command("cargo", &["test", "--workspace"])),
        "smoke" => smoke(),
        _ => (false, vec![format!("unknown stage {stage}")]),
    }
}

fn command(program: &str, args: &[&str]) -> Command {
    let mut cmd = Command::new(program);
    cmd.args(args);
    cmd
}

fn exec(mut cmd: Command) -> (bool, Vec<String>) {
    match cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).output() {
        Ok(out) => {
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            (out.status.success(), tail_lines(&text, OUTPUT_TAIL_LIMIT))
        }
        Err(err) => (false, vec![format!("spawn failed: {err}")]),
    }
}

fn smoke() -> (bool, Vec<String>) {
    // Honor CARGO_TARGET_DIR exactly as cargo resolves artifacts; fall back
    // to the in-tree target directory when the variable is unset.
    let target_dir = match std::env::var("CARGO_TARGET_DIR") {
        Ok(dir) if !dir.is_empty() => std::path::PathBuf::from(dir),
        Ok(_) | Err(_) => std::path::PathBuf::from("target"),
    };
    let binary = target_dir.join("debug").join("skillpack");
    if !binary.exists() {
        return (
            false,
            vec![format!(
                "smoke binary missing at {}: run cargo build first",
                binary.display()
            )],
        );
    }
    let mut cmd = Command::new(&binary);
    cmd.current_dir("examples/agentic-skill-template/skills/agentic-skill-template")
        .arg("check");
    exec(cmd)
}
