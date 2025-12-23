//! Configuration model

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub directories: DirectoryConfig,
    #[serde(default)]
    pub performance: PerformanceConfig,
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
            performance: PerformanceConfig::default(),
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

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum number of files to encode in parallel
    /// "auto" = use all CPU cores, or specify a number
    #[serde(default = "default_max_concurrent_encodes")]
    pub max_concurrent_encodes: String,
    /// Enable parallel file encoding (faster but more CPU/memory)
    #[serde(default = "default_true")]
    pub enable_parallel_encoding: bool,
    /// Encoding quality preset: "fast", "balanced", "high"
    #[serde(default = "default_encoding_preset")]
    pub encoding_preset: String,
    /// Maximum concurrent file encodings per book (prevents resource exhaustion)
    #[serde(default = "default_max_concurrent_files_per_book")]
    pub max_concurrent_files_per_book: String,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_encodes: "auto".to_string(),
            enable_parallel_encoding: true,
            encoding_preset: "balanced".to_string(),
            max_concurrent_files_per_book: "8".to_string(),
        }
    }
}

fn default_max_concurrent_encodes() -> String {
    "auto".to_string()
}

fn default_encoding_preset() -> String {
    "balanced".to_string()
}

fn default_max_concurrent_files_per_book() -> String {
    "8".to_string()
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
    /// Maximum number of retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u8,
    /// Initial retry delay in seconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            parallel_workers: 2,
            skip_existing: true,
            force_reprocess: false,
            normalize_existing: false,
            keep_temp_files: false,
            max_retries: 2,
            retry_delay: 1,
        }
    }
}

fn default_max_retries() -> u8 {
    2
}

fn default_retry_delay() -> u64 {
    1
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
    /// Default bitrate in kbps ("auto" or specific: 64, 128, 256)
    #[serde(default = "default_bitrate")]
    pub default_bitrate: String,
    /// Default sample rate in Hz ("auto" or specific: 44100, 48000)
    #[serde(default = "default_sample_rate")]
    pub default_sample_rate: String,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            prefer_stereo: true,
            chapter_source: "auto".to_string(),
            default_bitrate: "auto".to_string(),
            default_sample_rate: "auto".to_string(),
        }
    }
}

fn default_chapter_source() -> String {
    "auto".to_string()
}

fn default_bitrate() -> String {
    "auto".to_string()
}

fn default_sample_rate() -> String {
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
    /// Auto-extract embedded cover art from audio files as fallback
    #[serde(default = "default_auto_extract_cover")]
    pub auto_extract_cover: bool,
    /// Audible metadata integration
    #[serde(default)]
    pub audible: AudibleConfig,
    /// Matching mode for build command
    #[serde(default)]
    pub match_mode: MatchMode,
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
            auto_extract_cover: true,
            audible: AudibleConfig::default(),
            match_mode: MatchMode::default(),
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

fn default_auto_extract_cover() -> bool {
    true
}

/// Matching mode for interactive metadata matching during build
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MatchMode {
    /// Disabled - don't match during build
    Disabled,
    /// Auto - automatically select best match (non-interactive)
    Auto,
    /// Interactive - prompt user for each file
    Interactive,
}

impl Default for MatchMode {
    fn default() -> Self {
        MatchMode::Disabled
    }
}

/// Audible metadata integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudibleConfig {
    /// Enable Audible metadata fetching
    #[serde(default)]
    pub enabled: bool,
    /// Default Audible region for queries
    #[serde(default = "default_audible_region")]
    pub region: String,
    /// Auto-match books by folder name during build
    #[serde(default)]
    pub auto_match: bool,
    /// Download and embed cover art from Audible
    #[serde(default = "default_true")]
    pub download_covers: bool,
    /// Cache metadata locally (hours, 0 = no cache)
    #[serde(default = "default_cache_duration")]
    pub cache_duration_hours: u64,
    /// Rate limit (requests per minute)
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: u32,
    /// Maximum retry attempts for API failures (0 = no retry)
    #[serde(default = "default_api_max_retries")]
    pub api_max_retries: u8,
    /// Initial retry delay in seconds
    #[serde(default = "default_api_retry_delay")]
    pub api_retry_delay_secs: u64,
    /// Maximum retry delay in seconds (for exponential backoff)
    #[serde(default = "default_api_max_retry_delay")]
    pub api_max_retry_delay_secs: u64,
}

impl Default for AudibleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            region: "us".to_string(),
            auto_match: false,
            download_covers: true,
            cache_duration_hours: 168, // 7 days
            rate_limit_per_minute: 100,
            api_max_retries: 3,
            api_retry_delay_secs: 1,
            api_max_retry_delay_secs: 30,
        }
    }
}

fn default_audible_region() -> String {
    "us".to_string()
}

fn default_cache_duration() -> u64 {
    168 // 7 days
}

fn default_rate_limit() -> u32 {
    100
}

fn default_api_max_retries() -> u8 {
    3
}

fn default_api_retry_delay() -> u64 {
    1
}

fn default_api_max_retry_delay() -> u64 {
    30
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

fn default_aac_encoder() -> String {
    "auto".to_string()
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
    /// DEPRECATED: Use aac_encoder instead
    /// Use Apple Silicon hardware encoder (aac_at)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_apple_silicon_encoder: Option<bool>,
    /// AAC encoder preference: "auto", "aac_at", "libfdk_aac", "aac"
    #[serde(default = "default_aac_encoder")]
    pub aac_encoder: String,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            ffmpeg_path: None,
            atomic_parsley_path: None,
            mp4box_path: None,
            temp_directory: None,
            use_apple_silicon_encoder: None,
            aac_encoder: default_aac_encoder(),
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
