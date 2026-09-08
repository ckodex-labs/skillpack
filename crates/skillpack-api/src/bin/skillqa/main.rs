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

mod cmds_assess;
mod cmds_author;

use cmds_assess::{cmd_check, cmd_grade, cmd_report, cmd_validate};
use cmds_author::{cmd_init, cmd_lock, cmd_migrate};

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
