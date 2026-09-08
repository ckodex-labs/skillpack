//! SkillPack CNI — Container/Node Interface
//!
//! Headless programmatic client for CI/CD pipelines.
//! Reads skill path from env or stdin, calls the gRPC server, outputs JSON.
//!
//! Usage:
//!   SKILLPACK_SERVER_URL=http://localhost:50051 \
//!   SKILLPACK_SKILL_PATH=./my-skill \
//!   skillpack-cni assess
//!
//!   echo '{"skill_path":"./my-skill"}' | skillpack-cni assess --stdin

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use skillpack_proto::proto::{
    AssessRequest, GetCompatibilityRequest, query_service_client::QueryServiceClient,
    skill_pack_service_client::SkillPackServiceClient,
};
use std::io::{self, Read};
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "skillpack-cni")]
#[command(about = "SkillPack Container/Node Interface — headless CI/CD client")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// gRPC server URL (overrides SKILLPACK_SERVER_URL env var)
    #[arg(long, env = "SKILLPACK_SERVER_URL")]
    server_url: Option<String>,

    /// Skill path to assess (overrides SKILLPACK_SKILL_PATH env var)
    #[arg(long, env = "SKILLPACK_SKILL_PATH")]
    skill_path: Option<String>,

    /// Read skill path from stdin as JSON {"skill_path":"..."}
    #[arg(long)]
    stdin: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Assess a skill and output results as JSON
    Assess,
    /// Check compatibility with the canonical store
    Check,
}

#[derive(serde::Serialize)]
struct CniOutput {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        error!("{:#}", e);
        let output = CniOutput {
            success: false,
            result: None,
            error: Some(format!("{:#}", e)),
        };
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    let skill_path = resolve_skill_path(&cli)?;
    let server_url = cli
        .server_url
        // Accept the spec-wide SKILLPACK_API_URL (CLIENT-SPEC §3.1) as a
        // fallback so CNI and CLI agree on discovery. Literal loopback, not
        // `localhost`: resolver stalls escape connect timeouts.
        .or_else(|| std::env::var("SKILLPACK_API_URL").ok())
        .unwrap_or_else(|| "http://127.0.0.1:50051".to_string());

    info!("Connecting to {}", server_url);
    let mut client = SkillPackServiceClient::connect(server_url.clone())
        .await
        .context("Failed to connect to SkillPack gRPC server")?;

    match cli.command {
        Commands::Assess => {
            info!("Assessing skill at {}", skill_path);
            let request = tonic::Request::new(AssessRequest {
                skill_path: skill_path.clone(),
                ..Default::default()
            });
            let response = client
                .assess(request)
                .await
                .context("gRPC assess call failed")?;
            let inner = response.into_inner();
            let mut result = serde_json::json!({
                "meets_minimum": inner.meets_minimum,
            });
            // Surface the verdict itself, not just the boolean — CI logs
            // need grade/score/profile to be actionable.
            if let Some(a) = &inner.assessment {
                result["grade"] = serde_json::json!(a.grade);
                result["total_score"] = serde_json::json!(a.total_score);
                result["profile"] = serde_json::json!(a.profile);
            }
            let output = CniOutput {
                success: true,
                result: Some(result),
                error: None,
            };
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        Commands::Check => {
            info!("Checking compatibility for skill at {}", skill_path);
            let request = tonic::Request::new(GetCompatibilityRequest {
                skill_ref: skill_path.clone(),
            });
            let mut query_client = QueryServiceClient::connect(server_url)
                .await
                .context("Failed to connect to QueryService")?;
            let response = query_client
                .get_compatibility(request)
                .await
                .context("gRPC get_compatibility call failed")?;
            let inner = response.into_inner();
            let result = serde_json::json!({
                "has_compatibility": inner.compatibility.is_some(),
            });
            let output = CniOutput {
                success: true,
                result: Some(result),
                error: None,
            };
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
    }

    Ok(())
}

fn resolve_skill_path(cli: &Cli) -> Result<String> {
    if cli.stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read stdin")?;
        let parsed: serde_json::Value =
            serde_json::from_str(&buffer).context("Invalid JSON from stdin")?;
        parsed["skill_path"]
            .as_str()
            .map(|s| s.to_string())
            .context("Missing 'skill_path' field in stdin JSON")
    } else {
        cli.skill_path
            .clone()
            .context("Missing skill path. Set SKILLPACK_SKILL_PATH or use --skill-path")
    }
}
