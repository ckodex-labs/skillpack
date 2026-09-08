fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Regenerate whenever the canonical proto definitions change — without
    // this, edits to proto/ silently ship stale generated code.
    println!("cargo:rerun-if-changed=../../proto/skillpack/v1");
    tonic_prost_build::configure().compile_with_config(
        {
            let mut config = tonic_prost_build::Config::new();
            config.boxed(".skillpack.v1.AssessEvent.event");
            config
        },
        &[
            "../../proto/skillpack/v1/skillpack.proto",
            "../../proto/skillpack/v1/canonical.proto",
        ],
        &["../../proto/skillpack/v1"],
    )?;
    Ok(())
}
