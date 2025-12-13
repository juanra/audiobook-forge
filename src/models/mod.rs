//! Data models for audiobook processing

mod book;
mod track;
mod quality;
mod config;
mod result;
mod audible;

pub use book::{BookFolder, BookCase};
pub use track::Track;
pub use quality::QualityProfile;
pub use config::{Config, DirectoryConfig, ProcessingConfig, QualityConfig, MetadataConfig, AudibleConfig, OrganizationConfig, LoggingConfig, AdvancedConfig};
pub use result::ProcessingResult;
pub use audible::{AudibleMetadata, AudibleAuthor, AudibleSeries, AudibleRegion, AudibleSearchResult};
