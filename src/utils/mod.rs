//! Utility modules

mod config;
mod validation;
mod sorting;
mod merge_patterns;
pub mod cache;
pub mod scoring;
pub mod extraction;

pub use config::ConfigManager;
pub use validation::DependencyChecker;
pub use sorting::{natural_sort, natural_sort_by};
pub use cache::{AudibleCache, CacheStats};
pub use merge_patterns::{detect_merge_pattern, MergePatternResult, MergePatternType};

// Re-export Config for convenience
pub use crate::models::Config;
