//! SkillPack Domain Layer - Kernel Space
//!
//! Pure domain model following DDD principles.
//! No infrastructure dependencies - only domain concepts.

pub mod asc;
pub use asc::*;
pub mod assessment;
pub mod behavioral;
pub mod bundle;
pub mod caniuse;
pub mod agent_registry;
pub mod canonical_store;
pub mod collision;
pub mod cost;
pub mod dimension;
pub mod envelope;
pub mod exemption;
pub mod index;
pub mod index_repository;
pub mod ip_guard;
pub mod migrate;
pub mod ports;
pub mod profile;
pub mod query;
pub mod ratings;
pub mod reputation;
pub mod schema_validation;
pub mod score;
pub mod source;
pub mod sri;

pub use assessment::*;
pub use behavioral::*;
pub use bundle::*;
pub use caniuse::*;
pub use canonical_store::*;
pub use collision::*;
pub use cost::*;
pub use dimension::*;
pub use envelope::*;
pub use exemption::*;
pub use index::*;
pub use index_repository::*;
pub use ip_guard::*;
pub use migrate::*;
pub use ports::*;
pub use profile::*;
pub use query::*;
pub use ratings::*;
pub use reputation::*;
pub use score::*;
pub use source::*;
pub use sri::*;
