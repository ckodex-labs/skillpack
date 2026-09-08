//! Check Boundary Use Case
//!
//! Delegates IP boundary checks to the domain IPGuard.

use skillpack_domain::{IPGuard, IPGuardError};
use std::path::Path;

/// Request for boundary check.
pub struct CheckBoundaryRequest {
    pub target_path: String,
    pub skill_name: Option<String>,
}

/// Response from boundary check.
pub struct CheckBoundaryResponse {
    pub is_safe: bool,
    pub message: String,
    pub violations: Vec<String>,
}

/// Use case for checking IP boundaries.
pub struct CheckBoundaryUseCase {
    guard: IPGuard,
}

impl Default for CheckBoundaryUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckBoundaryUseCase {
    pub fn new() -> Self {
        Self {
            guard: IPGuard::new(),
        }
    }

    pub fn execute(&self, request: CheckBoundaryRequest) -> CheckBoundaryResponse {
        let mut violations = Vec::new();
        let mut is_safe = true;

        let path = Path::new(&request.target_path);
        if let Err(IPGuardError::BoundaryViolation(msg)) = self.guard.guard_path(path) {
            violations.push(msg);
            is_safe = false;
        }

        if let Some(name) = request.skill_name
            && let Err(IPGuardError::BoundaryViolation(msg)) = self.guard.guard_skill_name(&name)
        {
            violations.push(msg);
            is_safe = false;
        }

        let message = if is_safe {
            "Path and skill name are safe.".to_string()
        } else {
            format!("{} violation(s) detected.", violations.len())
        };

        CheckBoundaryResponse {
            is_safe,
            message,
            violations,
        }
    }
}
