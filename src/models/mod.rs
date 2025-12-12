//! Data models for audiobook processing

mod book;
mod track;
mod quality;
mod config;
mod result;

pub use book::{BookFolder, BookCase};
pub use track::Track;
pub use quality::QualityProfile;
pub use config::{Config, DirectoryConfig, ProcessingConfig, QualityConfig, MetadataConfig, OrganizationConfig, LoggingConfig, AdvancedConfig};
pub use result::ProcessingResult;
