//! SkillPack MCP Server
//!
//! Model Context Protocol server for AI agent integration.

use crate::checkers::all_checkers;
use crate::filesystem::FilesystemReader;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use skillpack_application::{AssessSkillRequest, AssessSkillUseCase, validate_skill_path_cwd};
use skillpack_domain::ReportGenerator;

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub version: String,
    pub description: String,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            name: "skillpack".to_string(),
            version: "1.0.0".to_string(),
            description: "AI Agent Skill Quality Assessment".to_string(),
        }
    }
}

/// MCP Tool definitions for SkillPack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// SkillPack MCP tools
pub fn tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "assess_skill".to_string(),
            description: "Assess a skill pack's quality score".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path or URI to skill pack"
                    },
                    "min_score": {
                        "type": "number",
                        "description": "Minimum score threshold (0-150)"
                    }
                },
                "required": ["path"]
            }),
        },
        McpTool {
            name: "grade_skill".to_string(),
            description: "Get letter grade for a skill pack".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path or URI to skill pack"
                    },
                    "minimum_grade": {
                        "type": "string",
                        "enum": ["F", "D", "C", "B", "A", "S", "S+"],
                        "description": "Minimum required grade"
                    }
                },
                "required": ["path"]
            }),
        },
        McpTool {
            name: "generate_report".to_string(),
            description: "Generate quality assessment report".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path or URI to skill pack"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["json", "sarif", "markdown", "badge"],
                        "description": "Report output format"
                    }
                },
                "required": ["path"]
            }),
        },
        McpTool {
            name: "list_dimensions".to_string(),
            description: "List all quality dimensions and their weights".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}

/// MCP Resource definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
}

/// SkillPack MCP resources
pub fn resources() -> Vec<McpResource> {
    vec![
        McpResource {
            uri: "skillpack://dimensions".to_string(),
            name: "Quality Dimensions".to_string(),
            description: "All quality dimensions with weights and criteria".to_string(),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "skillpack://grades".to_string(),
            name: "Grading Scale".to_string(),
            description: "Grade thresholds and descriptions".to_string(),
            mime_type: "application/json".to_string(),
        },
    ]
}

/// Handle MCP tool call
pub async fn handle_tool_call(
    name: &str,
    arguments: serde_json::Value,
) -> Result<serde_json::Value> {
    match name {
        "assess_skill" => {
            let raw_path = arguments["path"].as_str().unwrap_or(".");
            let path = validate_skill_path_cwd(raw_path)?.display().to_string();
            let reader = FilesystemReader::new();
            let use_case = AssessSkillUseCase::new(reader, all_checkers());
            let response = use_case.execute(AssessSkillRequest {
                skill_path: path,
                min_score: arguments["min_score"].as_f64(),
            })?;

            let assessment = response.assessment;
            let mut dims = serde_json::Map::new();
            for (dim_id, score) in &assessment.dimension_scores {
                dims.insert(dim_id.name().to_string(), score.value().into());
            }

            Ok(serde_json::json!({
                "score": assessment.total_score().value(),
                "grade": assessment.grade().to_string(),
                "profile": assessment.profile.name(),
                "dimensions": dims,
                "passed": assessment.total_score().value() >= arguments["min_score"].as_f64().unwrap_or(0.0),
            }))
        }
        "grade_skill" => {
            let raw_path = arguments["path"].as_str().unwrap_or(".");
            let path = validate_skill_path_cwd(raw_path)?.display().to_string();
            let minimum = arguments["minimum_grade"].as_str().unwrap_or("C");
            let reader = FilesystemReader::new();
            let use_case = AssessSkillUseCase::new(reader, all_checkers());
            let response = use_case.execute(AssessSkillRequest {
                skill_path: path,
                min_score: None,
            })?;

            let assessment = response.assessment;
            let grade = assessment.grade();
            let minimum_enum = match minimum {
                "S+" => skillpack_domain::score::Grade::SPlus,
                "S" => skillpack_domain::score::Grade::S,
                "A" => skillpack_domain::score::Grade::A,
                "B" => skillpack_domain::score::Grade::B,
                "C" => skillpack_domain::score::Grade::C,
                "D" => skillpack_domain::score::Grade::D,
                _ => skillpack_domain::score::Grade::F,
            };

            Ok(serde_json::json!({
                "grade": grade.to_string(),
                "score": assessment.total_score().value(),
                "meets_minimum": grade >= minimum_enum,
            }))
        }
        "generate_report" => {
            let raw_path = arguments["path"].as_str().unwrap_or(".");
            let path = validate_skill_path_cwd(raw_path)?.display().to_string();
            let format = arguments["format"].as_str().unwrap_or("json");

            let reader = FilesystemReader::new();
            let use_case = AssessSkillUseCase::new(reader, all_checkers());
            let response = use_case.execute(AssessSkillRequest {
                skill_path: path,
                min_score: None,
            })?;
            let assessment = response.assessment;

            let report = match format {
                "sarif" => generate_sarif(&assessment, raw_path)?,
                "markdown" | "md" => generate_markdown(&assessment)?,
                "badge" => crate::reporters::BadgeReporter.generate(&assessment)?,
                _ => serde_json::to_string_pretty(&assessment).map_err(|e| anyhow::anyhow!(e))?,
            };

            Ok(serde_json::json!({
                "content": [{"type": "text", "text": report}],
                "isError": false,
            }))
        }
        "list_dimensions" => Ok(serde_json::json!([
            {"id": "IdentityAndManifest", "weight": 11, "description": "Skill identity and manifest completeness"},
            {"id": "Security", "weight": 18, "description": "Supply chain security and secret scanning"},
            {"id": "Provenance", "weight": 14, "description": "Artifact traceability and signing"},
            {"id": "Documentation", "weight": 11, "description": "User and developer documentation"},
            {"id": "Testing", "weight": 10, "description": "Test coverage and quality gates"},
            {"id": "Compatibility", "weight": 8, "description": "Platform and version compatibility"},
            {"id": "Lifecycle", "weight": 10, "description": "Skill lifecycle and deprecation management"},
            {"id": "Governance", "weight": 9, "description": "Organizational standards and compliance"},
            {"id": "EvalsHitl", "weight": 9, "description": "Evaluation suites and human-in-the-loop validation"}
        ])),
        _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
    }
}

// ---------------------------------------------------------------------------
// Stdio JSON-RPC 2.0 server loop
// ---------------------------------------------------------------------------

use std::io::{self, BufRead, Write};

/// Run the MCP server over stdio using JSON-RPC 2.0.
pub async fn run_stdio_server() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match handle_jsonrpc(&line).await {
            Ok(Some(response)) => {
                let json = serde_json::to_string(&response)?;
                writeln!(stdout, "{}", json)?;
                stdout.flush()?;
            }
            Ok(None) => {}
            Err(e) => {
                let err = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: format!("Internal error: {}", e),
                        data: None,
                    }),
                };
                let json = serde_json::to_string(&err)?;
                writeln!(stdout, "{}", json)?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}

async fn handle_jsonrpc(line: &str) -> anyhow::Result<Option<JsonRpcResponse>> {
    let request: JsonRpcRequest = serde_json::from_str(line)?;

    match request.method.as_str() {
        "initialize" => Ok(Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "skillpack-mcp",
                    "version": env!("CARGO_PKG_VERSION"),
                },
                "capabilities": { "tools": {} }
            })),
            error: None,
        })),
        "notifications/initialized" => Ok(None),
        "tools/list" => Ok(Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({ "tools": tools() })),
            error: None,
        })),
        "tools/call" => {
            let params = request.params.unwrap_or(serde_json::json!({}));
            let name = params["name"].as_str().unwrap_or("");
            let args = &params["arguments"];
            match handle_tool_call(name, args.clone()).await {
                Ok(result) => Ok(Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({
                        "content": [{"type": "text", "text": serde_json::to_string(&result)?}],
                        "isError": false
                    })),
                    error: None,
                })),
                Err(e) => Ok(Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({
                        "content": [{"type": "text", "text": format!("Error: {}", e)}],
                        "isError": true
                    })),
                    error: None,
                })),
            }
        }
        _ => Ok(Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        })),
    }
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct JsonRpcRequest {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: Option<serde_json::Value>,
}

#[derive(serde::Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(serde::Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

/// Generate SARIF 2.1.0 report for GitHub/GitLab integration
fn generate_sarif(assessment: &skillpack_domain::Assessment, path: &str) -> Result<String> {
    let results: Vec<serde_json::Value> = assessment
        .dimension_scores
        .iter()
        .filter(|(_, score)| score.value() < 70.0)
        .map(|(id, score)| {
            serde_json::json!({
                "ruleId": format!("skillpack/{}", id.name().to_lowercase().replace(" ", "-")),
                "level": if score.value() < 50.0 { "error" } else { "warning" },
                "message": {
                    "text": format!("{} score is {:.0}/100 (minimum: 70)", id.name(), score.value())
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": format!("{}/SKILL.md", path)
                        }
                    }
                }]
            })
        })
        .collect();

    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "SkillPack",
                    "version": "1.0.0",
                    "informationUri": "https://ckodex.org/skillpack",
                    "rules": [
                        { "id": "skillpack/structure", "name": "Structure", "shortDescription": { "text": "Skill structure completeness" } },
                        { "id": "skillpack/documentation", "name": "Documentation", "shortDescription": { "text": "Documentation quality" } },
                        { "id": "skillpack/security", "name": "Security", "shortDescription": { "text": "Security posture" } },
                        { "id": "skillpack/performance", "name": "Performance", "shortDescription": { "text": "Performance characteristics" } },
                        { "id": "skillpack/testing", "name": "Testing", "shortDescription": { "text": "Test coverage" } },
                        { "id": "skillpack/compatibility", "name": "Compatibility", "shortDescription": { "text": "Platform compatibility" } },
                        { "id": "skillpack/governance", "name": "Governance", "shortDescription": { "text": "Governance compliance" } },
                        { "id": "skillpack/observability", "name": "Observability", "shortDescription": { "text": "Observability instrumentation" } }
                    ]
                }
            },
            "results": results,
            "invocations": [{
                "executionSuccessful": assessment.grade().as_str() != "F"
            }]
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}

/// Generate Markdown report
fn generate_markdown(assessment: &skillpack_domain::Assessment) -> Result<String> {
    let mut md = String::new();
    md.push_str("# SkillPack Assessment Report\n\n");
    md.push_str(&format!(
        "**Overall Grade:** {}\n",
        assessment.grade().as_str()
    ));
    md.push_str(&format!(
        "**Total Score:** {:.0}/150\n\n",
        assessment.total_score().value()
    ));

    md.push_str("## Dimension Scores\n\n");
    md.push_str("| Dimension | Score | Status |\n");
    md.push_str("|-----------|-------|--------|\n");

    for (id, score) in &assessment.dimension_scores {
        let status = if score.value() >= 90.0 {
            "Excellent"
        } else if score.value() >= 70.0 {
            "Good"
        } else {
            "Needs Work"
        };
        md.push_str(&format!(
            "| {} | {:.0}/100 | {} |\n",
            id.name(),
            score.value(),
            status
        ));
    }

    Ok(md)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_dimensions_returns_nine() {
        let result = handle_tool_call("list_dimensions", serde_json::json!({}))
            .await
            .unwrap();
        let dims = result.as_array().expect("should be array");
        assert_eq!(dims.len(), 9, "expected 9 dimensions");

        let ids: Vec<_> = dims.iter().map(|d| d["id"].as_str().unwrap()).collect();
        assert!(ids.contains(&"IdentityAndManifest"));
        assert!(ids.contains(&"Security"));
        assert!(ids.contains(&"Provenance"));
        assert!(ids.contains(&"Documentation"));
        assert!(ids.contains(&"Testing"));
        assert!(ids.contains(&"Compatibility"));
        assert!(ids.contains(&"Lifecycle"));
        assert!(ids.contains(&"Governance"));
        assert!(ids.contains(&"EvalsHitl"));
    }

    #[tokio::test]
    async fn generate_report_returns_report() {
        let tmp = std::path::PathBuf::from("target/test-skill-report");
        std::fs::create_dir_all(&tmp).ok();
        std::fs::write(
            tmp.join("SKILL.md"),
            r#"# test-skill

A test skill for MCP report generation.

## Metadata
- **Version:** 1.0.0
- **Author:** test
- **License:** MIT
"#,
        )
        .unwrap();

        let result = handle_tool_call(
            "generate_report",
            serde_json::json!({"path": tmp.to_str().unwrap(), "format": "json"}),
        )
        .await;
        assert!(result.is_ok(), "generate_report failed: {:?}", result.err());
        let value = result.unwrap();
        let content = value["content"].as_array().expect("content array");
        assert!(!content.is_empty());
        let text = content[0]["text"].as_str().expect("text");
        assert!(text.contains("skill"));
        assert!(text.contains("dimension_scores"));
        assert!(text.contains("issues"));
    }

    #[tokio::test]
    async fn assess_skill_known_fixture() {
        let tmp = std::path::PathBuf::from("target/test-skill");
        std::fs::create_dir_all(&tmp).ok();
        std::fs::write(
            tmp.join("SKILL.md"),
            r#"# test-skill

A test skill for MCP assessment.

## Metadata
- **Version:** 1.0.0
- **Author:** test
- **License:** MIT
"#,
        )
        .unwrap();

        let result = handle_tool_call(
            "assess_skill",
            serde_json::json!({"path": tmp.to_str().unwrap()}),
        )
        .await;
        assert!(result.is_ok(), "assess_skill failed: {:?}", result.err());
        let assessment = result.unwrap();
        assert!(!assessment["dimensions"].as_object().unwrap().is_empty());
    }
}
