//! Telemetry Module
//!
//! Observability for SkillPack:
//! - Structured logging with tracing
//! - Prometheus /metrics endpoint
//! - Custom metrics for skill assessments
//! - Carbon scoring

use opentelemetry::propagation::{Extractor, Injector};
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{Registry, layer::SubscriberExt, util::SubscriberInitExt};

/// HTTP header carrier for trace context extraction.
pub struct HttpHeaderCarrier<'a> {
    headers: &'a axum::http::HeaderMap,
}

impl<'a> HttpHeaderCarrier<'a> {
    pub fn new(headers: &'a axum::http::HeaderMap) -> Self {
        Self { headers }
    }
}

impl<'a> Extractor for HttpHeaderCarrier<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|k| k.as_str()).collect()
    }
}

/// gRPC metadata carrier for trace context injection.
pub struct GrpcMetadataCarrier<'a> {
    metadata: &'a mut tonic::metadata::MetadataMap,
}

impl<'a> GrpcMetadataCarrier<'a> {
    pub fn new(metadata: &'a mut tonic::metadata::MetadataMap) -> Self {
        Self { metadata }
    }
}

impl<'a> Injector for GrpcMetadataCarrier<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(k) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes())
            && let Ok(v) = tonic::metadata::MetadataValue::try_from(&value)
        {
            self.metadata.insert(k, v);
        }
    }
}

/// Extract a W3C traceparent from HTTP headers and return the trace ID (or None).
pub fn extract_trace_id(headers: &axum::http::HeaderMap) -> Option<String> {
    let carrier = HttpHeaderCarrier::new(headers);

    opentelemetry::global::get_text_map_propagator(|propagator| {
        let _context = propagator.extract(&carrier);
    });

    // Fall back to current tracing span if available
    let span = tracing::Span::current();
    span.id().map(|id| id.into_u64().to_string())
}

/// Initialize telemetry with structured logging and optional OTLP export.
///
/// OTLP is enabled when either the `otlp_endpoint` argument is provided
/// or the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable is set.
/// If OTLP setup fails, the function falls back to stdout-only logging.
pub fn init_telemetry(
    service_name: &str,
    otlp_endpoint: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = otlp_endpoint
        .map(|s| s.to_string())
        .or_else(|| std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok())
        .filter(|s| !s.is_empty());

    let telemetry_layer = if let Some(endpoint) = endpoint {
        tracing::info!(
            target: "skillpack",
            "Attempting OTLP export to {} for service {}",
            endpoint,
            service_name
        );

        let resource = opentelemetry_sdk::Resource::builder_empty()
            .with_service_name(service_name.to_string())
            .build();

        match opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()
        {
            Ok(exporter) => {
                let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
                    .with_resource(resource)
                    .with_simple_exporter(exporter)
                    .build();
                opentelemetry::global::set_tracer_provider(provider);
                let tracer = opentelemetry::global::tracer("skillpack");
                tracing::info!(target: "skillpack", "OTLP tracer installed");
                Some(tracing_opentelemetry::layer().with_tracer(tracer))
            }
            Err(e) => {
                tracing::warn!(
                    target: "skillpack",
                    "Failed to build OTLP exporter ({}). Falling back to stdout-only logging.",
                    e
                );
                None
            }
        }
    } else {
        None
    };

    let registry = Registry::default()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true),
        )
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("skillpack=info".parse()?),
        );

    if let Some(telemetry) = telemetry_layer {
        registry.with(telemetry).init();
    } else {
        registry.init();
    }

    Ok(())
}

/// Shutdown telemetry gracefully.
pub fn shutdown_telemetry() {
    tracing::info!("Shutting down SkillPack telemetry");
}

// ============================================================================
// Custom Metrics
// ============================================================================

use metrics::{counter, gauge, histogram};

/// Record an assessment operation
pub fn record_assessment(skill_name: &str, grade: &str, score: f64, duration_ms: u64) {
    counter!("skillpack_assessments_total", "skill" => skill_name.to_string(), "grade" => grade.to_string()).increment(1);
    histogram!("skillpack_assessment_score", "skill" => skill_name.to_string()).record(score);
    histogram!("skillpack_assessment_duration_ms", "skill" => skill_name.to_string())
        .record(duration_ms as f64);
}

/// Record dimension score
pub fn record_dimension_score(dimension: &str, score: f64) {
    gauge!("skillpack_dimension_score", "dimension" => dimension.to_string()).set(score);
}

/// Record issue count by severity
pub fn record_issues(severity: &str, count: u64) {
    counter!("skillpack_issues_total", "severity" => severity.to_string()).increment(count);
}

/// Record OCI operation
pub fn record_oci_operation(operation: &str, success: bool) {
    let status = if success { "success" } else { "failure" };
    counter!("skillpack_oci_operations_total", "operation" => operation.to_string(), "status" => status.to_string()).increment(1);
}

// ============================================================================
// Prometheus Metrics Server
// ============================================================================

use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;

/// Start Prometheus metrics server on specified address
pub fn start_prometheus_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let builder = PrometheusBuilder::new();
    builder.with_http_listener(addr).install()?;

    tracing::info!("Prometheus metrics available at http://{}/metrics", addr);
    Ok(())
}

// ============================================================================
// Carbon Scoring
// ============================================================================

/// Carbon rating for skills (A-F based on estimated compute requirements)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CarbonRating {
    A, // Minimal compute (static data, simple logic)
    B, // Low compute (small models, cached results)
    C, // Moderate compute (medium models, some API calls)
    D, // High compute (large models, frequent API calls)
    E, // Very high compute (multiple large models)
    F, // Extreme compute (training, batch processing)
}

impl CarbonRating {
    pub fn as_str(&self) -> &'static str {
        match self {
            CarbonRating::A => "A",
            CarbonRating::B => "B",
            CarbonRating::C => "C",
            CarbonRating::D => "D",
            CarbonRating::E => "E",
            CarbonRating::F => "F",
        }
    }

    /// Estimate carbon rating based on skill characteristics
    pub fn estimate(
        uses_llm: bool,
        model_size: Option<&str>,
        api_calls_per_invocation: u32,
        caches_results: bool,
    ) -> Self {
        let mut score = 0;

        if uses_llm {
            score += match model_size {
                Some("small") => 2,
                Some("medium") => 4,
                Some("large") => 6,
                Some("xlarge") => 8,
                _ => 4, // Default to medium
            };
        }

        score += (api_calls_per_invocation / 5) as i32;

        if caches_results {
            score = (score as f64 * 0.7) as i32;
        }

        match score {
            0..=1 => CarbonRating::A,
            2..=3 => CarbonRating::B,
            4..=5 => CarbonRating::C,
            6..=7 => CarbonRating::D,
            8..=9 => CarbonRating::E,
            _ => CarbonRating::F,
        }
    }
}

/// Record carbon rating metric
pub fn record_carbon_rating(skill_name: &str, rating: CarbonRating) {
    let score = match rating {
        CarbonRating::A => 100.0,
        CarbonRating::B => 80.0,
        CarbonRating::C => 60.0,
        CarbonRating::D => 40.0,
        CarbonRating::E => 20.0,
        CarbonRating::F => 0.0,
    };
    gauge!("skillpack_carbon_score", "skill" => skill_name.to_string(), "rating" => rating.as_str().to_string()).set(score);
}
