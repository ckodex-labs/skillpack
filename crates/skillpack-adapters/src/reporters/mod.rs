//! Reporter Adapters

pub mod badge;
pub mod json;
pub mod markdown;
pub mod sarif;

pub use badge::BadgeReporter;
pub use json::JsonReporter;
pub use markdown::MarkdownReporter;
pub use sarif::SarifReporter;
