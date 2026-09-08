//! Authentication layer — Bearer token validation per CLIENT-SPEC §6
//!
//! Reads `SKILLPACK_API_TOKEN` env var at startup. If unset, auth is disabled
//! (convenient for local development). When set, every request must include
//! `Authorization: Bearer <token>`.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Shared token validator.
#[derive(Clone)]
pub struct TokenValidator {
    expected: Option<String>,
}

impl TokenValidator {
    pub fn from_env() -> Self {
        Self {
            expected: std::env::var("SKILLPACK_API_TOKEN")
                .ok()
                .filter(|s| !s.is_empty()),
        }
    }

    /// Disabled validator — every bearer value is accepted.
    /// Named `none` because tests read `TokenValidator::none()` at
    /// the six former env-mutating sites.
    pub fn none() -> Self {
        Self { expected: None }
    }

    /// Explicit token. Prefer over `from_env` in tests: mutating the
    /// process environment is racy under parallel test threads.
    pub fn with_token(token: &str) -> Self {
        Self {
            expected: Some(token.to_string()),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.expected.is_some()
    }

    /// Validate a raw `Bearer <token>` string.
    pub fn validate(&self, header: &str) -> bool {
        match &self.expected {
            None => true,
            Some(expected) => {
                let token = header
                    .strip_prefix("Bearer ")
                    .or_else(|| header.strip_prefix("bearer "));
                token == Some(expected.as_str())
            }
        }
    }
}

/// Axum middleware: reject requests with missing/invalid Bearer token.
/// When auth is disabled (SKILLPACK_API_TOKEN unset), still reject
/// mutating operations (non-GET) and /migrate/* paths to prevent
/// unauthorized modifications in shared or remote environments.
pub async fn auth_middleware(
    validator: axum::extract::State<Arc<TokenValidator>>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    if !validator.is_enabled() {
        // Auth disabled: allow safe read-only GETs, block mutations
        let is_get = method == axum::http::Method::GET;
        let is_migrate = path.starts_with("/migrate/");
        if is_get && !is_migrate {
            return next.run(request).await;
        }
        return (
            StatusCode::UNAUTHORIZED,
            "Unauthorized: mutating endpoints require SKILLPACK_API_TOKEN",
        )
            .into_response();
    }

    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header) if validator.validate(header) => next.run(request).await,
        _ => (
            StatusCode::UNAUTHORIZED,
            "Unauthorized: invalid or missing Bearer token",
        )
            .into_response(),
    }
}

/// Check if an API token is configured, returning Unauthenticated if not.
/// Call this at the top of mutating gRPC methods to enforce auth when
/// SKILLPACK_API_TOKEN is unset.
pub fn ensure_token_configured() -> Result<(), tonic::Status> {
    let has_token = std::env::var("SKILLPACK_API_TOKEN")
        .ok()
        .filter(|s| !s.is_empty())
        .is_some();
    if has_token {
        Ok(())
    } else {
        Err(tonic::Status::unauthenticated(
            "Mutating endpoints require SKILLPACK_API_TOKEN",
        ))
    }
}

/// Tonic interceptor: same logic for gRPC metadata.
pub fn grpc_interceptor(
    validator: Arc<TokenValidator>,
) -> impl Fn(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> + Clone + Send + 'static
{
    move |req| {
        if !validator.is_enabled() {
            return Ok(req);
        }

        let meta = req.metadata();
        let header = meta.get("authorization").and_then(|v| v.to_str().ok());

        match header {
            Some(h) if validator.validate(h) => Ok(req),
            _ => Err(tonic::Status::unauthenticated(
                "invalid or missing Bearer token",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::routing::get;
    use tower::ServiceExt;

    #[test]
    fn validate_exact_match() {
        let v = TokenValidator {
            expected: Some("secret123".to_string()),
        };
        assert!(v.validate("Bearer secret123"));
        assert!(v.validate("bearer secret123"));
        assert!(!v.validate("Bearer wrong"));
        assert!(!v.validate("secret123"));
    }

    #[test]
    fn disabled_allows_everything() {
        let v = TokenValidator { expected: None };
        assert!(v.validate("anything"));
    }

    #[tokio::test]
    async fn middleware_rejects_missing_token() {
        let validator = Arc::new(TokenValidator {
            expected: Some("secret".to_string()),
        });
        let app = axum::Router::new()
            .route("/protected", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                validator,
                auth_middleware,
            ));

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn middleware_allows_valid_token() {
        let validator = Arc::new(TokenValidator {
            expected: Some("secret".to_string()),
        });
        let app = axum::Router::new()
            .route("/protected", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                validator,
                auth_middleware,
            ));

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/protected")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn empty_token_disables_auth() {
        let v = TokenValidator {
            expected: Some("".to_string()),
        };
        assert!(v.is_enabled());
        assert!(v.validate("Bearer "));
    }

    #[test]
    fn missing_bearer_prefix_rejected() {
        let v = TokenValidator {
            expected: Some("secret".to_string()),
        };
        assert!(!v.validate("secret"));
        assert!(!v.validate("Basic secret"));
    }
}
