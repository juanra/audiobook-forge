//! Configuration file management

use crate::models::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Configuration manager for loading and saving config files
pub struct ConfigManager;

impl ConfigManager {
    /// Get the default config file path
    pub fn default_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Cannot determine config directory")?
            .join("audiobook-forge");

        Ok(config_dir.join("config.yaml"))
    }

    /// Ensure config directory exists
    pub fn ensure_config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Cannot determine config directory")?
            .join("audiobook-forge");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .context("Failed to create config directory")?;
        }

        Ok(config_dir)
    }

    /// Load configuration from file or create default
    pub fn load_or_default(path: Option<&PathBuf>) -> Result<Config> {
        let config_path = match path {
            Some(p) => p.clone(),
            None => Self::default_config_path()?,
        };

        if config_path.exists() {
            Self::load(&config_path)
        } else {
            Ok(Config::default())
        }
    }

    /// Load configuration from file
    pub fn load(path: &PathBuf) -> Result<Config> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(config: &Config, path: Option<&PathBuf>) -> Result<()> {
        let config_path = match path {
            Some(p) => p.clone(),
            None => Self::default_config_path()?,
        };

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .context("Failed to create config directory")?;
            }
        }

        let yaml = serde_yaml::to_string(config)
            .context("Failed to serialize config to YAML")?;

        fs::write(&config_path, yaml)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        Ok(())
    }

    /// Initialize config file with defaults and comprehensive comments
    pub fn init(force: bool) -> Result<PathBuf> {
        let config_path = Self::default_config_path()?;

        if config_path.exists() && !force {
            anyhow::bail!(
                "Config file already exists at: {}\nUse --force to overwrite",
                config_path.display()
            );
        }

        // Create documented config
        let config_content = include_str!("../../templates/config.yaml");

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(&config_path, config_content)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        Ok(config_path)
    }

    /// Validate configuration
    pub fn validate(config: &Config) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Validate parallel workers
        if config.processing.parallel_workers < 1 || config.processing.parallel_workers > 8 {
            warnings.push(format!(
                "parallel_workers ({}) should be between 1 and 8",
                config.processing.parallel_workers
            ));
        }

        // Validate chapter source
        let valid_chapter_sources = ["auto", "files", "cue", "id3", "none"];
        if !valid_chapter_sources.contains(&config.quality.chapter_source.as_str()) {
            warnings.push(format!(
                "chapter_source '{}' is not recognized. Valid options: {}",
                config.quality.chapter_source,
                valid_chapter_sources.join(", ")
            ));
        }

        // Validate log level
        let valid_log_levels = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];
        if !valid_log_levels.contains(&config.logging.log_level.to_uppercase().as_str()) {
            warnings.push(format!(
                "log_level '{}' is not recognized. Valid options: {}",
                config.logging.log_level,
                valid_log_levels.join(", ")
            ));
        }

        // Check if custom paths exist
        if let Some(ref path) = config.directories.source {
            if !path.exists() {
                warnings.push(format!(
                    "source directory does not exist: {}",
                    path.display()
                ));
            }
        }

        Ok(warnings)
    }

    /// Show current configuration
    pub fn show(path: Option<&PathBuf>) -> Result<String> {
        let config = Self::load_or_default(path)?;
        let yaml = serde_yaml::to_string(&config)
            .context("Failed to serialize config")?;
        Ok(yaml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_save_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");

        let config = Config::default();
        ConfigManager::save(&config, Some(&config_path)).unwrap();

        let loaded = ConfigManager::load(&config_path).unwrap();
        assert_eq!(loaded.processing.parallel_workers, 2);
    }

    #[test]
    fn test_validate_config() {
        let mut config = Config::default();
        config.processing.parallel_workers = 10; // Invalid

        let warnings = ConfigManager::validate(&config).unwrap();
        assert!(!warnings.is_empty());
    }
}
