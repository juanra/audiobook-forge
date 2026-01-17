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
pub use sorting::natural_sort;
pub use cache::{AudibleCache, CacheStats};
pub use merge_patterns::{detect_merge_pattern, sort_by_part_number, MergePatternResult, MergePatternType};

// Re-export Config for convenience
pub use crate::models::Config;
