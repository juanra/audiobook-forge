//! Data models for audiobook processing

mod book;
mod track;
mod quality;
mod config;
mod result;
mod audible;
mod match_models;

pub use book::{BookFolder, BookCase};
pub use track::Track;
pub use quality::QualityProfile;
pub use config::{Config, DirectoryConfig, ProcessingConfig, QualityConfig, MetadataConfig, AudibleConfig, OrganizationConfig, LoggingConfig, AdvancedConfig, MatchMode};
pub use result::ProcessingResult;
pub use audible::{AudibleMetadata, AudibleAuthor, AudibleSeries, AudibleRegion, AudibleSearchResult};
pub use match_models::{MatchCandidate, MetadataDistance, MatchConfidence, CurrentMetadata, MetadataSource};
