//! Audio quality profile model

use serde::{Deserialize, Serialize};
use std::fmt;

/// Audio quality profile with bitrate, sample rate, channels, and codec
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityProfile {
    /// Bitrate in kbps
    pub bitrate: u32,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels (1=mono, 2=stereo)
    pub channels: u8,
    /// Audio codec (e.g., "mp3", "aac")
    pub codec: String,
    /// Duration in seconds
    pub duration: f64,
}

impl QualityProfile {
    /// Create a new quality profile
    pub fn new(bitrate: u32, sample_rate: u32, channels: u8, codec: String, duration: f64) -> anyhow::Result<Self> {
        if bitrate == 0 {
            anyhow::bail!("Bitrate must be positive, got {}", bitrate);
        }
        if sample_rate == 0 {
            anyhow::bail!("Sample rate must be positive, got {}", sample_rate);
        }
        if channels != 1 && channels != 2 {
            anyhow::bail!("Channels must be 1 or 2, got {}", channels);
        }

        Ok(Self {
            bitrate,
            sample_rate,
            channels,
            codec,
            duration,
        })
    }

    /// Compare quality profiles to determine which is better
    pub fn is_better_than(&self, other: &QualityProfile, prefer_stereo: bool) -> bool {
        // Priority order:
        // 1. Bitrate (higher is better)
        if self.bitrate != other.bitrate {
            return self.bitrate > other.bitrate;
        }

        // 2. Sample rate (higher is better)
        if self.sample_rate != other.sample_rate {
            return self.sample_rate > other.sample_rate;
        }

        // 3. Channels (stereo > mono if prefer_stereo, else mono > stereo)
        if self.channels != other.channels {
            if prefer_stereo {
                return self.channels > other.channels;
            } else {
                return self.channels < other.channels;
            }
        }

        // 4. Codec preference: AAC > MP3
        let codec_priority = |codec: &str| match codec.to_lowercase().as_str() {
            "aac" => 2,
            "mp3" => 1,
            _ => 0,
        };

        codec_priority(&self.codec) > codec_priority(&other.codec)
    }

    /// Check if two profiles can be concatenated without re-encoding
    pub fn is_compatible_for_concat(&self, other: &QualityProfile) -> bool {
        self.bitrate == other.bitrate
            && self.sample_rate == other.sample_rate
            && self.channels == other.channels
            && self.codec.to_lowercase() == other.codec.to_lowercase()
    }

    /// Convert to AAC profile with equivalent or better quality
    pub fn to_aac_equivalent(&self) -> QualityProfile {
        // For MP3->AAC conversion, use same or higher bitrate
        let aac_bitrate = self.bitrate.max(128); // Minimum AAC quality

        QualityProfile {
            bitrate: aac_bitrate,
            sample_rate: self.sample_rate,
            channels: self.channels,
            codec: "aac".to_string(),
            duration: self.duration,
        }
    }

    /// Create a quality profile from a preset
    /// Returns None for "source" preset (auto-detect from source files)
    pub fn from_preset(preset: &str, source: &QualityProfile) -> Option<QualityProfile> {
        match preset.to_lowercase().as_str() {
            "low" => Some(QualityProfile {
                bitrate: 64,
                sample_rate: 22050,
                channels: 1, // mono
                codec: "aac".to_string(),
                duration: source.duration,
            }),
            "medium" => Some(QualityProfile {
                bitrate: 96,
                sample_rate: 44100,
                channels: 2, // stereo
                codec: "aac".to_string(),
                duration: source.duration,
            }),
            "high" => Some(QualityProfile {
                bitrate: 128,
                sample_rate: 48000,
                channels: 2, // stereo
                codec: "aac".to_string(),
                duration: source.duration,
            }),
            "source" | _ => None, // Use auto-detected quality from source
        }
    }

    /// Apply quality preset override if specified
    pub fn apply_preset(&self, preset: Option<&str>) -> QualityProfile {
        preset
            .and_then(|p| Self::from_preset(p, self))
            .unwrap_or_else(|| self.clone())
    }
}

impl fmt::Display for QualityProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}kbps, {}Hz, {}ch, {}, {:.1}s",
            self.bitrate, self.sample_rate, self.channels, self.codec, self.duration
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_creation() {
        let profile = QualityProfile::new(128, 44100, 2, "aac".to_string(), 3600.0).unwrap();
        assert_eq!(profile.bitrate, 128);
        assert_eq!(profile.sample_rate, 44100);
        assert_eq!(profile.channels, 2);
        assert_eq!(profile.codec, "aac");
    }

    #[test]
    fn test_quality_validation() {
        assert!(QualityProfile::new(0, 44100, 2, "aac".to_string(), 3600.0).is_err());
        assert!(QualityProfile::new(128, 0, 2, "aac".to_string(), 3600.0).is_err());
        assert!(QualityProfile::new(128, 44100, 3, "aac".to_string(), 3600.0).is_err());
    }

    #[test]
    fn test_is_better_than() {
        let high = QualityProfile::new(256, 44100, 2, "aac".to_string(), 3600.0).unwrap();
        let low = QualityProfile::new(128, 44100, 2, "aac".to_string(), 3600.0).unwrap();

        assert!(high.is_better_than(&low, true));
        assert!(!low.is_better_than(&high, true));
    }

    #[test]
    fn test_compatibility() {
        let profile1 = QualityProfile::new(128, 44100, 2, "aac".to_string(), 3600.0).unwrap();
        let profile2 = QualityProfile::new(128, 44100, 2, "aac".to_string(), 1800.0).unwrap();
        let profile3 = QualityProfile::new(256, 44100, 2, "aac".to_string(), 3600.0).unwrap();

        assert!(profile1.is_compatible_for_concat(&profile2));
        assert!(!profile1.is_compatible_for_concat(&profile3));
    }
}
