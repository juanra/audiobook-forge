//! Utility modules

pub mod cache;
mod config;
pub mod extraction;
mod merge_patterns;
pub mod scoring;
mod sorting;
mod validation;

pub use cache::{AudibleCache, CacheStats};
pub use config::ConfigManager;
pub use merge_patterns::{
    are_sequential_part_directories, detect_merge_pattern, MergePatternResult, MergePatternType,
};
pub use sorting::{natural_sort, natural_sort_by};
pub use validation::DependencyChecker;

// Re-export Config for convenience
pub use crate::models::Config;
