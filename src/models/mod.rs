//! Data models for audiobook processing

mod audible;
mod book;
mod config;
mod match_models;
mod quality;
mod result;
mod track;

pub use audible::{
    AudibleAuthor, AudibleChapter, AudibleMetadata, AudibleRegion, AudibleSearchResult,
    AudibleSeries, AudnexChaptersResponse,
};
pub use book::{BookCase, BookFolder};
pub use config::{
    AdvancedConfig, AudibleConfig, Config, DirectoryConfig, LoggingConfig, MatchMode,
    MetadataConfig, OrganizationConfig, ProcessingConfig, QualityConfig,
};
pub use match_models::{
    CurrentMetadata, MatchCandidate, MatchConfidence, MetadataDistance, MetadataSource,
};
pub use quality::QualityProfile;
pub use result::ProcessingResult;
pub use track::Track;
