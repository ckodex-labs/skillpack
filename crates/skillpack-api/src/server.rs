//! gRPC Server Implementation

use crate::conversions;
use crate::proto::{
    AssessEvent, AssessRequest, AssessResponse, GradeRequest, GradeResponse, ReportRequest,
    ReportResponse,
    skill_pack_service_server::{SkillPackService, SkillPackServiceServer},
};
use skillpack_adapters::checkers::all_checkers;
use skillpack_adapters::filesystem::FilesystemReader;
use skillpack_application::{AssessSkillRequest, AssessSkillUseCase};
use skillpack_domain::SkillReader;
use std::path::{Path, PathBuf};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

pub struct SkillPackServer;

impl Default for SkillPackServer {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillPackServer {
    pub fn new() -> Self {
        Self
    }

    pub fn into_service(self) -> SkillPackServiceServer<Self> {
        SkillPackServiceServer::new(self)
    }

    /// Sanitize and canonicalize a user-provided path to prevent path traversal attacks.
    /// Returns an error if the path attempts to escape the current working directory.
    fn sanitize_path(&self, path: &str) -> Result<String, Status> {
        // Convert to absolute path
        let absolute = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            std::env::current_dir()
                .map_err(|e| Status::internal(format!("Failed to get current directory: {}", e)))?
                .join(path)
        };

        // Canonicalize to resolve any .. or . components
        let canonical = absolute.canonicalize().map_err(|e| {
            Status::invalid_argument(format!("Failed to canonicalize path '{}': {}", path, e))
        })?;

        // Ensure the canonical path doesn't escape the current working directory
        let current_dir = std::env::current_dir()
            .map_err(|e| Status::internal(format!("Failed to get current directory: {}", e)))?
            .canonicalize()
            .map_err(|e| {
                Status::internal(format!("Failed to canonicalize current directory: {}", e))
            })?;

        if !canonical.starts_with(&current_dir) {
            return Err(Status::permission_denied(format!(
                "Path '{}' attempts to escape the current working directory",
                path
            )));
        }

        canonical
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Status::invalid_argument("Path contains invalid UTF-8"))
    }
}

#[tonic::async_trait]
impl SkillPackService for SkillPackServer {
    async fn assess(
        &self,
        request: Request<AssessRequest>,
    ) -> Result<Response<AssessResponse>, Status> {
        let req = request.into_inner();
        let skill_path = self.sanitize_path(&req.skill_path)?;

        let reader = FilesystemReader::new();
        let checkers = all_checkers();
        let use_case = AssessSkillUseCase::new(reader, checkers);

        let assess_req = AssessSkillRequest {
            skill_path,
            min_score: req.min_score,
        };

        let response = use_case
            .execute(assess_req)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(AssessResponse {
            assessment: Some(conversions::assessment_to_proto(&response.assessment)),
            meets_minimum: response.meets_minimum,
        }))
    }

    async fn grade(
        &self,
        request: Request<GradeRequest>,
    ) -> Result<Response<GradeResponse>, Status> {
        let req = request.into_inner();
        let skill_path = self.sanitize_path(&req.skill_path)?;

        let reader = FilesystemReader::new();
        let checkers = all_checkers();
        let use_case = AssessSkillUseCase::new(reader, checkers);

        let assess_req = AssessSkillRequest {
            skill_path,
            min_score: None,
        };

        let response = use_case
            .execute(assess_req)
            .map_err(|e| Status::internal(e.to_string()))?;

        let assessment = &response.assessment;

        Ok(Response::new(GradeResponse {
            grade: assessment.grade().as_str().to_string(),
            total_score: assessment.total_score().value(),
            meets_minimum: assessment.grade().as_str() >= req.minimum_grade.as_str(),
        }))
    }

    async fn report(
        &self,
        request: Request<ReportRequest>,
    ) -> Result<Response<ReportResponse>, Status> {
        let req = request.into_inner();

        let reader = FilesystemReader::new();
        let checkers = all_checkers();
        let use_case = AssessSkillUseCase::new(reader, checkers);

        let assess_req = AssessSkillRequest {
            skill_path: req.skill_path,
            min_score: None,
        };

        let response = use_case
            .execute(assess_req)
            .map_err(|e| Status::internal(e.to_string()))?;

        let content = serde_json::to_string_pretty(&response.assessment)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ReportResponse {
            content,
            format: req.format,
        }))
    }

    type AssessStreamStream = ReceiverStream<Result<AssessEvent, Status>>;

    async fn assess_stream(
        &self,
        request: Request<AssessRequest>,
    ) -> Result<Response<Self::AssessStreamStream>, Status> {
        let req = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(16);

        tokio::spawn(async move {
            let reader = FilesystemReader::new();
            let checkers = all_checkers();
            let path = std::path::Path::new(&req.skill_path);

            // Read identity (single step, not streamed)
            let identity = match reader.read_identity(path) {
                Ok(id) => id,
                Err(e) => {
                    let _ = tx.send(Err(Status::invalid_argument(e.to_string()))).await;
                    return;
                }
            };

            let mut assessment = skillpack_domain::Assessment::new(identity);

            // Stream each dimension sequentially so clients see real-time progress
            for checker in checkers {
                let dimension_name = checker.dimension().name().to_string();

                // 1. Progress event
                let progress_sent = tx
                    .send(Ok(AssessEvent {
                        event: Some(crate::proto::assess_event::Event::DimensionProgress(
                            Box::new(crate::proto::DimensionProgress {
                                dimension: dimension_name.clone(),
                                status: "Checking...".to_string(),
                            }),
                        )),
                    }))
                    .await;
                if progress_sent.is_err() {
                    return; // Client disconnected
                }

                // 2. Run the check
                let (score, issues) = checker.check(&reader, path);
                if checker.is_stub() {
                    assessment.stub_count += 1;
                }
                assessment.record_dimension(checker.dimension(), score);
                for issue in issues.clone() {
                    assessment.add_issue(issue);
                }

                // 3. Complete event
                let proto_issues: Vec<crate::proto::Issue> = issues
                    .iter()
                    .map(|i| crate::proto::Issue {
                        dimension: i.dimension.name().to_string(),
                        severity: match i.severity {
                            skillpack_domain::Severity::Error => {
                                crate::proto::Severity::Error as i32
                            }
                            skillpack_domain::Severity::Warning => {
                                crate::proto::Severity::Warning as i32
                            }
                            skillpack_domain::Severity::Note => crate::proto::Severity::Note as i32,
                        },
                        message: i.message.clone(),
                        file: i.file.clone(),
                        line: i.line,
                    })
                    .collect();

                let complete_sent = tx
                    .send(Ok(AssessEvent {
                        event: Some(crate::proto::assess_event::Event::DimensionComplete(
                            Box::new(crate::proto::DimensionComplete {
                                dimension: dimension_name,
                                score: score.value(),
                                issues: proto_issues,
                            }),
                        )),
                    }))
                    .await;
                if complete_sent.is_err() {
                    return; // Client disconnected
                }
            }

            // 4. Final assessment event
            let _ = tx
                .send(Ok(AssessEvent {
                    event: Some(crate::proto::assess_event::Event::AssessmentComplete(
                        Box::new(crate::proto::AssessmentComplete {
                            assessment: Some(conversions::assessment_to_proto(&assessment)),
                        }),
                    )),
                }))
                .await;
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type AssessBatchStreamStream = ReceiverStream<Result<AssessEvent, Status>>;

    async fn assess_batch_stream(
        &self,
        request: Request<tonic::Streaming<AssessRequest>>,
    ) -> Result<Response<Self::AssessBatchStreamStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(16);

        tokio::spawn(async move {
            while let Ok(Some(req)) = stream.message().await {
                let skill_path = match sanitize_path_grpc(&req.skill_path) {
                    Ok(p) => p,
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        continue;
                    }
                };

                let reader = FilesystemReader::new();
                let checkers = all_checkers();
                let path = std::path::Path::new(&skill_path);

                // Read identity
                let identity = match reader.read_identity(path) {
                    Ok(id) => id,
                    Err(e) => {
                        let _ = tx.send(Err(Status::invalid_argument(e.to_string()))).await;
                        continue;
                    }
                };

                let mut assessment = skillpack_domain::Assessment::new(identity);

                // Stream each dimension
                for checker in checkers {
                    let dimension_name = checker.dimension().name().to_string();

                    let progress_sent = tx
                        .send(Ok(AssessEvent {
                            event: Some(crate::proto::assess_event::Event::DimensionProgress(
                                Box::new(crate::proto::DimensionProgress {
                                    dimension: dimension_name.clone(),
                                    status: "Checking...".to_string(),
                                }),
                            )),
                        }))
                        .await;
                    if progress_sent.is_err() {
                        return;
                    }

                    let (score, issues) = checker.check(&reader, path);
                    if checker.is_stub() {
                        assessment.stub_count += 1;
                    }
                    assessment.record_dimension(checker.dimension(), score);
                    for issue in issues.clone() {
                        assessment.add_issue(issue);
                    }

                    let proto_issues: Vec<crate::proto::Issue> = issues
                        .iter()
                        .map(|i| crate::proto::Issue {
                            dimension: i.dimension.name().to_string(),
                            severity: match i.severity {
                                skillpack_domain::Severity::Error => {
                                    crate::proto::Severity::Error as i32
                                }
                                skillpack_domain::Severity::Warning => {
                                    crate::proto::Severity::Warning as i32
                                }
                                skillpack_domain::Severity::Note => {
                                    crate::proto::Severity::Note as i32
                                }
                            },
                            message: i.message.clone(),
                            file: i.file.clone(),
                            line: i.line,
                        })
                        .collect();

                    let complete_sent = tx
                        .send(Ok(AssessEvent {
                            event: Some(crate::proto::assess_event::Event::DimensionComplete(
                                Box::new(crate::proto::DimensionComplete {
                                    dimension: dimension_name,
                                    score: score.value(),
                                    issues: proto_issues,
                                }),
                            )),
                        }))
                        .await;
                    if complete_sent.is_err() {
                        return;
                    }
                }

                let _ = tx
                    .send(Ok(AssessEvent {
                        event: Some(crate::proto::assess_event::Event::AssessmentComplete(
                            Box::new(crate::proto::AssessmentComplete {
                                assessment: Some(conversions::assessment_to_proto(&assessment)),
                            }),
                        )),
                    }))
                    .await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

fn sanitize_path_grpc(path: &str) -> Result<String, Status> {
    let absolute = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        std::env::current_dir()
            .map_err(|e| Status::internal(format!("Failed to get current directory: {}", e)))?
            .join(path)
    };

    let canonical = absolute.canonicalize().map_err(|e| {
        Status::invalid_argument(format!("Failed to canonicalize path '{}': {}", path, e))
    })?;

    let current_dir = std::env::current_dir()
        .map_err(|e| Status::internal(format!("Failed to get current directory: {}", e)))?
        .canonicalize()
        .map_err(|e| {
            Status::internal(format!("Failed to canonicalize current directory: {}", e))
        })?;

    if !canonical.starts_with(&current_dir) {
        return Err(Status::permission_denied(format!(
            "Path '{}' attempts to escape the current working directory",
            path
        )));
    }

    canonical
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| Status::invalid_argument("Path contains invalid UTF-8"))
}
