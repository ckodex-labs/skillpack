//! SkillPack CLI Entry Point

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    skillpack_adapters::cli::run()
}
