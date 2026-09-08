// xtask: workspace automation gate. See CONTRIBUTING.md for usage.

mod ci;
mod common;
mod provenance;
mod stamp;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask", version, about = "CKODEX skillpack workspace gate")]
struct Cli {
    #[command(subcommand)]
    command: CommandCli,
}

#[derive(Subcommand)]
enum CommandCli {
    /// Full CI-parity gate: fmt, clippy (-D warnings, -W unit_cmp), test, smoke.
    Ci,
    /// Check or repair the version triple (Cargo.toml, release manifest, CHANGELOG).
    Stamp {
        /// Repair mismatches instead of failing on them.
        #[arg(long)]
        write: bool,
    },
    /// Verify or fill the SLSA provenance file.
    Provenance {
        /// Fill invocationId/startedOn/finishedOn placeholders.
        #[arg(long)]
        write: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let root = common::workspace_root(std::path::Path::new("."))?;
    std::env::set_current_dir(&root)?;
    match cli.command {
        CommandCli::Ci => {
            let passed = ci::run()?;
            std::process::exit(i32::from(!passed));
        }
        CommandCli::Stamp { write } => {
            let ok = stamp::run(write)?;
            std::process::exit(i32::from(!ok));
        }
        CommandCli::Provenance { write } => {
            let ok = provenance::run(write)?;
            std::process::exit(i32::from(!ok));
        }
    }
}
