//! Configuration model

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub directories: DirectoryConfig,
    #[serde(default)]
    pub processing: ProcessingConfig,
    #[serde(default)]
    pub quality: QualityConfig,
    #[serde(default)]
    pub metadata: MetadataConfig,
    #[serde(default)]
    pub organization: OrganizationConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub advanced: AdvancedConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            directories: DirectoryConfig::default(),
            processing: ProcessingConfig::default(),
            quality: QualityConfig::default(),
            metadata: MetadataConfig::default(),
            organization: OrganizationConfig::default(),
            logging: LoggingConfig::default(),
            advanced: AdvancedConfig::default(),
        }
    }
}

/// Directory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryConfig {
    /// Source directory for audiobooks (overrides --root)
    pub source: Option<PathBuf>,
    /// Output directory ("same_as_source" or custom path)
    #[serde(default = "default_output")]
    pub output: String,
}

impl Default for DirectoryConfig {
    fn default() -> Self {
        Self {
            source: None,
            output: "same_as_source".to_string(),
        }
    }
}

fn default_output() -> String {
    "same_as_source".to_string()
}

/// Processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    /// Number of parallel workers (1-8)
    #[serde(default = "default_parallel_workers")]
    pub parallel_workers: u8,
    /// Skip folders with existing M4B files
    #[serde(default = "default_true")]
    pub skip_existing: bool,
    /// Always reprocess, overwriting existing files
    #[serde(default)]
    pub force_reprocess: bool,
    /// Normalize existing M4B files (fix metadata)
    #[serde(default)]
    pub normalize_existing: bool,
    /// Keep temporary files for debugging
    #[serde(default)]
    pub keep_temp_files: bool,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            parallel_workers: 2,
            skip_existing: true,
            force_reprocess: false,
            normalize_existing: false,
            keep_temp_files: false,
        }
    }
}

fn default_parallel_workers() -> u8 {
    2
}

fn default_true() -> bool {
    true
}

/// Quality configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// Prefer stereo over mono when quality is equal
    #[serde(default = "default_true")]
    pub prefer_stereo: bool,
    /// Chapter source priority ("auto", "files", "cue", etc.)
    #[serde(default = "default_chapter_source")]
    pub chapter_source: String,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            prefer_stereo: true,
            chapter_source: "auto".to_string(),
        }
    }
}

fn default_chapter_source() -> String {
    "auto".to_string()
}

/// Metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    /// Default language for metadata (ISO 639-1)
    #[serde(default = "default_language")]
    pub default_language: String,
    /// Cover art filenames to search for
    #[serde(default = "default_cover_filenames")]
    pub cover_filenames: Vec<String>,
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            default_language: "es".to_string(),
            cover_filenames: vec![
                "cover.jpg".to_string(),
                "folder.jpg".to_string(),
                "cover.png".to_string(),
                "folder.png".to_string(),
            ],
        }
    }
}

fn default_language() -> String {
    "es".to_string()
}

fn default_cover_filenames() -> Vec<String> {
    vec![
        "cover.jpg".to_string(),
        "folder.jpg".to_string(),
        "cover.png".to_string(),
        "folder.png".to_string(),
    ]
}

/// Organization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationConfig {
    /// Name for completed audiobooks folder
    #[serde(default = "default_m4b_folder")]
    pub m4b_folder: String,
    /// Name for conversion queue folder
    #[serde(default = "default_convert_folder")]
    pub convert_folder: String,
}

impl Default for OrganizationConfig {
    fn default() -> Self {
        Self {
            m4b_folder: "M4B".to_string(),
            convert_folder: "To_Convert".to_string(),
        }
    }
}

fn default_m4b_folder() -> String {
    "M4B".to_string()
}

fn default_convert_folder() -> String {
    "To_Convert".to_string()
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Enable automatic log file creation
    #[serde(default)]
    pub log_to_file: bool,
    /// Custom log file path
    pub log_file: Option<PathBuf>,
    /// Log level ("INFO", "DEBUG", "WARNING", "ERROR")
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_to_file: false,
            log_file: None,
            log_level: "INFO".to_string(),
        }
    }
}

fn default_log_level() -> String {
    "INFO".to_string()
}

/// Advanced configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    /// Custom FFmpeg binary path
    pub ffmpeg_path: Option<PathBuf>,
    /// Custom AtomicParsley binary path
    pub atomic_parsley_path: Option<PathBuf>,
    /// Custom MP4Box binary path
    pub mp4box_path: Option<PathBuf>,
    /// Custom temporary files location
    pub temp_directory: Option<PathBuf>,
    /// Use Apple Silicon hardware encoder (aac_at)
    pub use_apple_silicon_encoder: Option<bool>,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            ffmpeg_path: None,
            atomic_parsley_path: None,
            mp4box_path: None,
            temp_directory: None,
            use_apple_silicon_encoder: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.processing.parallel_workers, 2);
        assert_eq!(config.quality.prefer_stereo, true);
        assert_eq!(config.metadata.default_language, "es");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.processing.parallel_workers, 2);
    }
}
