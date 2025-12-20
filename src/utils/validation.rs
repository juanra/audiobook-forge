//! Dependency validation utilities

use std::process::Command;
use which::which;

/// Dependency checker for external tools
pub struct DependencyChecker;

#[derive(Debug, Clone)]
pub struct DependencyStatus {
    pub name: String,
    pub found: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

impl DependencyChecker {
    /// Check if FFmpeg is installed and get version
    pub fn check_ffmpeg() -> DependencyStatus {
        match which("ffmpeg") {
            Ok(path) => {
                let version = Self::get_ffmpeg_version();
                DependencyStatus {
                    name: "ffmpeg".to_string(),
                    found: true,
                    version,
                    path: Some(path.display().to_string()),
                }
            }
            Err(_) => DependencyStatus {
                name: "ffmpeg".to_string(),
                found: false,
                version: None,
                path: None,
            },
        }
    }

    /// Check if AtomicParsley is installed
    pub fn check_atomic_parsley() -> DependencyStatus {
        match which("AtomicParsley") {
            Ok(path) => {
                let version = Self::get_atomic_parsley_version();
                DependencyStatus {
                    name: "AtomicParsley".to_string(),
                    found: true,
                    version,
                    path: Some(path.display().to_string()),
                }
            }
            Err(_) => DependencyStatus {
                name: "AtomicParsley".to_string(),
                found: false,
                version: None,
                path: None,
            },
        }
    }

    /// Check if MP4Box is installed
    pub fn check_mp4box() -> DependencyStatus {
        match which("MP4Box") {
            Ok(path) => {
                let version = Self::get_mp4box_version();
                DependencyStatus {
                    name: "MP4Box".to_string(),
                    found: true,
                    version,
                    path: Some(path.display().to_string()),
                }
            }
            Err(_) => DependencyStatus {
                name: "MP4Box".to_string(),
                found: false,
                version: None,
                path: None,
            },
        }
    }

    /// Check all dependencies
    pub fn check_all() -> Vec<DependencyStatus> {
        vec![
            Self::check_ffmpeg(),
            Self::check_atomic_parsley(),
            Self::check_mp4box(),
        ]
    }

    /// Check if all dependencies are satisfied
    pub fn all_dependencies_met() -> bool {
        Self::check_all().iter().all(|dep| dep.found)
    }

    /// Get FFmpeg version
    fn get_ffmpeg_version() -> Option<String> {
        let output = Command::new("ffmpeg")
            .arg("-version")
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(2))
            .map(|s| s.to_string())
    }

    /// Get AtomicParsley version
    fn get_atomic_parsley_version() -> Option<String> {
        let output = Command::new("AtomicParsley")
            .arg("--version")
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .find(|line| line.contains("version"))
            .and_then(|line| line.split_whitespace().last())
            .map(|s| s.to_string())
    }

    /// Get MP4Box version
    fn get_mp4box_version() -> Option<String> {
        let output = Command::new("MP4Box")
            .arg("-version")
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().find(|s| s.contains("version")))
            .map(|s| s.to_string())
    }

    /// Check if Apple Silicon AAC encoder is available
    pub fn check_aac_at_support() -> bool {
        // Check if ffmpeg supports aac_at encoder
        let output = Command::new("ffmpeg")
            .args(&["-encoders"])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("aac_at")
        } else {
            false
        }
    }

    /// Get all available AAC encoders
    pub fn get_available_encoders() -> Vec<String> {
        crate::audio::EncoderDetector::get_available_encoders()
            .into_iter()
            .map(|e| e.name().to_string())
            .collect()
    }

    /// Get the currently selected AAC encoder
    pub fn get_selected_encoder() -> String {
        crate::audio::get_encoder().name().to_string()
    }
}

impl std::fmt::Display for DependencyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.found {
            write!(f, "✓ {}", self.name)?;
            if let Some(ref version) = self.version {
                write!(f, " ({})", version)?;
            }
            if let Some(ref path) = self.path {
                write!(f, "\n  Path: {}", path)?;
            }
            Ok(())
        } else {
            write!(f, "✗ {} - NOT FOUND", self.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_dependencies() {
        let deps = DependencyChecker::check_all();
        assert_eq!(deps.len(), 3);

        // At least ffmpeg should be found for tests to pass
        let ffmpeg = deps.iter().find(|d| d.name == "ffmpeg");
        assert!(ffmpeg.is_some());
    }
}
