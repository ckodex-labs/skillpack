//! SkillPack gRPC API - Presentation Space
//!
//! gRPC services implementing skillpack.v1.*

pub mod auth;
pub mod cache;
pub mod canonical_service;
pub mod conversions;
pub mod http_server;
pub mod index_service;
pub mod query_service;
pub mod ratings_service;
pub mod server;
pub mod telemetry;
pub mod watcher;

pub use skillpack_proto::proto;

pub use canonical_service::CanonicalStoreServiceImpl;
pub use index_service::IndexServiceImpl;
pub use query_service::QueryServiceImpl;
pub use ratings_service::RatingsServiceImpl;
pub use server::SkillPackServer;
