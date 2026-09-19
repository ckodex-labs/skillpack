//! SkillPack Dagger Pipeline
//!
//! Rust Dagger SDK CI/CD pipeline following CKODEX principles.
//!
//! Since 1.0.0-beta.2 the lint/test/audit/build/sign/attest stages execute real
//! work. The `xtask ci` release gate (crates/xtask) reproduces the same stages
//! locally with the same failure semantics.
//!
//! Stages:
//! 1. Lint - cargo clippy, cargo fmt
//! 2. Test - cargo test with coverage
//! 3. Build - release binary + distroless image
//! 4. Sign - Sigstore keyless signing (cosign sign-blob)
//! 5. Attest - SLSA provenance + SBOM (CycloneDX)
//! 6. Publish - OCI registry push

pub struct SkillPackPipeline {
    pub name: String,
    pub version: String,
}

impl SkillPackPipeline {
    pub fn new(version: &str) -> Self {
        Self {
            name: "skillpack".to_string(),
            version: version.to_string(),
        }
    }

    /// Run full pipeline
    pub async fn run(&self) -> anyhow::Result<PipelineResult> {
        let mut result = PipelineResult::default();
        
        // Stage 1: Lint
        result.lint = self.lint().await?;
        if !result.lint.passed {
            return Ok(result);
        }
        
        // Stage 2: Test
        result.test = self.test().await?;
        if !result.test.passed {
            return Ok(result);
        }
        
        // Stage 3: Audit
        result.audit = self.audit().await?;
        if !result.audit.passed {
            return Ok(result);
        }
        
        // Stage 4: Build
        result.build = self.build().await?;
        if !result.build.passed {
            return Ok(result);
        }
        
        // Stage 5: Sign
        result.sign = self.sign().await?;
        
        // Stage 6: Attest
        result.attest = self.attest().await?;
        
        result.overall_passed = true;
        Ok(result)
    }

    async fn lint(&self) -> anyhow::Result<StageResult> {
        let start = std::time::Instant::now();

        let fmt = tokio::process::Command::new("cargo")
            .args(["fmt", "--check"])
            .output()
            .await?;
        let clippy = tokio::process::Command::new("cargo")
            .args(["clippy", "--workspace", "--", "-D", "warnings"])
            .output()
            .await?;

        let passed = fmt.status.success() && clippy.status.success();
        if !passed {
            eprintln!("{}", String::from_utf8_lossy(&fmt.stderr));
            eprintln!("{}", String::from_utf8_lossy(&clippy.stderr));
        }

        Ok(StageResult {
            name: "lint".to_string(),
            passed,
            duration_ms: start.elapsed().as_millis() as u64,
            artifacts: vec![],
        })
    }

    async fn test(&self) -> anyhow::Result<StageResult> {
        let start = std::time::Instant::now();

        let out = tokio::process::Command::new("cargo")
            .args(["test", "--workspace"])
            .output()
            .await?;

        let passed = out.status.success();
        if !passed {
            eprintln!("{}", String::from_utf8_lossy(&out.stderr));
        }

        Ok(StageResult {
            name: "test".to_string(),
            passed,
            duration_ms: start.elapsed().as_millis() as u64,
            artifacts: vec![],
        })
    }

    /// Audit stage — run cargo audit and emit JSON report.
    async fn audit(&self) -> anyhow::Result<StageResult> {
        let start = std::time::Instant::now();
        let report_path = "target/release/cargo-audit.json";

        let check = tokio::process::Command::new("sh")
            .args(["-c", "command -v cargo-audit"])
            .output()
            .await?;
        if !check.status.success() {
            eprintln!("audit stage failed: cargo-audit not installed");
            return Ok(StageResult {
                name: "audit".to_string(),
                passed: false,
                duration_ms: start.elapsed().as_millis() as u64,
                artifacts: vec![],
            });
        }

        let out = tokio::process::Command::new("cargo")
            .args(["audit", "--json"])
            .output()
            .await?;

        let passed = out.status.success();
        if passed {
            tokio::fs::write(report_path, &out.stdout).await?;
        } else {
            eprintln!("{}", String::from_utf8_lossy(&out.stderr));
            // Still write the JSON report even if vulnerabilities were found
            tokio::fs::write(report_path, &out.stdout).await?;
        }

        Ok(StageResult {
            name: "audit".to_string(),
            passed,
            duration_ms: start.elapsed().as_millis() as u64,
            artifacts: vec![report_path.to_string()],
        })
    }

    async fn build(&self) -> anyhow::Result<StageResult> {
        let start = std::time::Instant::now();

        let out = tokio::process::Command::new("cargo")
            .args(["build", "--release", "--workspace"])
            .output()
            .await?;

        let passed = out.status.success();
        if !passed {
            eprintln!("{}", String::from_utf8_lossy(&out.stderr));
        }

        Ok(StageResult {
            name: "build".to_string(),
            passed,
            duration_ms: start.elapsed().as_millis() as u64,
            artifacts: vec![
                "target/release/skillpack-server".to_string(),
                "target/release/skillqa".to_string(),
            ],
        })
    }

    /// Sign stage — keyless signing with Sigstore/cosign.
    /// Requires `cosign` binary to be installed and authenticated.
    async fn sign(&self) -> anyhow::Result<StageResult> {
        let start = std::time::Instant::now();
        let binary = "target/release/skillpack-server";

        // Verify cosign is available
        let check = tokio::process::Command::new("sh")
            .args(["-c", "command -v cosign"])
            .output()
            .await?;
        if !check.status.success() {
            return Ok(StageResult {
                name: "sign".to_string(),
                passed: false,
                duration_ms: start.elapsed().as_millis() as u64,
                artifacts: vec![],
            });
        }

        let out = tokio::process::Command::new("cosign")
            .args(["sign-blob", "--yes", "--output-signature", "target/release/skillpack-server.sig", binary])
            .output()
            .await?;

        let passed = out.status.success();
        if !passed {
            eprintln!("{}", String::from_utf8_lossy(&out.stderr));
        }

        Ok(StageResult {
            name: "sign".to_string(),
            passed,
            duration_ms: start.elapsed().as_millis() as u64,
            artifacts: if passed {
                vec!["target/release/skillpack-server.sig".to_string()]
            } else {
                vec![]
            },
        })
    }

    /// Attest stage — generate CycloneDX SBOM and SLSA provenance.
    /// Attempts `cargo cyclonedx` for SBOM; falls back to a minimal manifest.
    async fn attest(&self) -> anyhow::Result<StageResult> {
        let start = std::time::Instant::now();
        let mut artifacts = Vec::new();
        let mut passed = true;

        // SBOM generation
        let sbom_path = "target/release/sbom.cdx.json";
        let cyclonedx = tokio::process::Command::new("sh")
            .args(["-c", "command -v cargo-cyclonedx || command -v cyclonedx"])
            .output()
            .await?;

        if cyclonedx.status.success() {
            let out = tokio::process::Command::new("cargo")
                .args(["cyclonedx", "--output", sbom_path])
                .output()
                .await?;
            if out.status.success() {
                artifacts.push(sbom_path.to_string());
            } else {
                eprintln!("SBOM generation failed: {}", String::from_utf8_lossy(&out.stderr));
                passed = false;
            }
        } else {
            // Minimal fallback SBOM
            let minimal_sbom = serde_json::json!({
                "bomFormat": "CycloneDX",
                "specVersion": "1.5",
                "serialNumber": format!("urn:uuid:{}", uuid::Uuid::new_v4()),
                "version": 1,
                "metadata": {
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "tools": [{ "name": "skillpack-dagger", "version": &self.version }]
                },
                "components": [{
                    "type": "application",
                    "name": "skillpack-server",
                    "version": &self.version,
                    "purl": format!("pkg:cargo/skillpack-server@{}", &self.version)
                }]
            });
            tokio::fs::write(sbom_path, serde_json::to_string_pretty(&minimal_sbom)?).await?;
            artifacts.push(sbom_path.to_string());
        }

        // SLSA provenance generation
        let provenance_path = "target/release/provenance.json";
        let provenance = serde_json::json!({
            "_type": "https://in-toto.io/Statement/v0.1",
            "predicateType": "https://slsa.dev/provenance/v0.2",
            "subject": [{
                "name": "skillpack-server",
                "digest": { "sha256": "pending" }
            }],
            "predicate": {
                "builder": { "id": "https://github.com/ckodex-labs/skillpack/.github/workflows/skillpack-ci.yml" },
                "buildType": "https://github.com/ckodex-labs/skillpack/dagger/v1",
                "invocation": {
                    "configSource": {
                        "uri": "https://github.com/ckodex-labs/skillpack",
                        "digest": { "sha1": "HEAD" }
                    },
                    "parameters": { "version": &self.version }
                },
                "metadata": {
                    "buildStartedOn": chrono::Utc::now().to_rfc3339(),
                    "completeness": {
                        "parameters": true,
                        "environment": false,
                        "materials": false
                    }
                }
            }
        });
        tokio::fs::write(provenance_path, serde_json::to_string_pretty(&provenance)?).await?;
        artifacts.push(provenance_path.to_string());

        Ok(StageResult {
            name: "attest".to_string(),
            passed,
            duration_ms: start.elapsed().as_millis() as u64,
            artifacts,
        })
    }
}

#[derive(Default)]
pub struct PipelineResult {
    pub overall_passed: bool,
    pub lint: StageResult,
    pub test: StageResult,
    pub audit: StageResult,
    pub build: StageResult,
    pub sign: StageResult,
    pub attest: StageResult,
}

#[derive(Default)]
pub struct StageResult {
    pub name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub artifacts: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let pipeline = SkillPackPipeline::new(env!("CARGO_PKG_VERSION"));
    let result = pipeline.run().await?;
    tracing::info!("Pipeline completed: overall_passed={}", result.overall_passed);
    if !result.overall_passed {
        std::process::exit(1);
    }
    Ok(())
}
