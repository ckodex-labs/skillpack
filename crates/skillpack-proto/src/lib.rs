//! SkillPack gRPC Protobuf Definitions
//!
//! This crate contains only the generated protobuf/gRPC code.
//! It has no dependencies on other skillpack workspace crates,
//! allowing both server and client crates to depend on it without cycles.

pub mod proto {
    tonic::include_proto!("skillpack.v1");
}

pub use proto::*;
