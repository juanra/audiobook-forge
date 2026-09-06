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
    pub fn new(
        bitrate: u32,
        sample_rate: u32,
        channels: u8,
        codec: String,
        duration: f64,
    ) -> anyhow::Result<Self> {
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

    /// Whether this profile describes a lossless codec.
    ///
    /// Lossless bitrates describe the *source* file's size, not a perceptual
    /// quality target, so they must never be handed to a lossy encoder as-is.
    pub fn is_lossless(&self) -> bool {
        matches!(
            self.codec.to_lowercase().as_str(),
            "flac" | "alac" | "wav" | "pcm_s16le" | "pcm_s24le" | "ape" | "wavpack"
        )
    }

    /// AAC target for a stereo lossless source, in kbps.
    pub const LOSSLESS_AAC_TARGET_STEREO: u32 = 192;
    /// AAC target for a mono lossless source, in kbps.
    pub const LOSSLESS_AAC_TARGET_MONO: u32 = 128;

    /// The bitrate this profile should actually be encoded at.
    ///
    /// Lossy sources keep their own bitrate (transcoding to a higher one only
    /// wastes space). Lossless sources probe at 900-1000+ kbps, which AAC
    /// cannot meaningfully use; mirroring that produced enormous output files
    /// at a nonsensical bitrate (issue #18), so they are clamped to a sane
    /// perceptual target instead.
    pub fn to_encode_target(&self) -> QualityProfile {
        let bitrate = if self.is_lossless() {
            if self.channels == 1 {
                Self::LOSSLESS_AAC_TARGET_MONO
            } else {
                Self::LOSSLESS_AAC_TARGET_STEREO
            }
        } else {
            self.bitrate
        };

        QualityProfile {
            bitrate,
            sample_rate: self.sample_rate,
            channels: self.channels,
            codec: self.codec.clone(),
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
            "ultra" => Some(QualityProfile {
                bitrate: 192,
                sample_rate: 48000,
                channels: 2, // stereo
                codec: "aac".to_string(),
                duration: source.duration,
            }),
            "maximum" => Some(QualityProfile {
                bitrate: 256,
                sample_rate: 48000,
                channels: 2, // stereo
                codec: "aac".to_string(),
                duration: source.duration,
            }),
            "source" | _ => None, // Use auto-detected quality from source
        }
    }

    /// Resolve the profile to encode with, honouring an optional preset override.
    ///
    /// An explicit preset (`low`..`maximum`) is the user's decision and wins.
    /// Every other path — no preset at all, or the `source` preset, which means
    /// "auto-detect from the source" — falls through to [`Self::to_encode_target`],
    /// so a lossless source is always clamped to a sane AAC bitrate (issue #18).
    pub fn apply_preset(&self, preset: Option<&str>) -> QualityProfile {
        preset
            .and_then(|p| Self::from_preset(p, self))
            .unwrap_or_else(|| self.to_encode_target())
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
    fn test_lossless_source_is_clamped_to_sane_aac_target() {
        // FLAC rips probe at ~900-1000 kbps. Mirroring that into the AAC encoder
        // produces enormous files at a bitrate AAC cannot use (issue #18).
        let flac = QualityProfile::new(920, 44100, 2, "flac".to_string(), 3600.0).unwrap();
        let target = flac.to_encode_target();

        assert_eq!(
            target.bitrate, 192,
            "stereo lossless should target 192k AAC"
        );
        assert_eq!(target.sample_rate, 44100, "sample rate must be preserved");
        assert_eq!(target.channels, 2);
    }

    #[test]
    fn test_mono_lossless_source_uses_lower_target() {
        let flac = QualityProfile::new(460, 44100, 1, "flac".to_string(), 3600.0).unwrap();
        assert_eq!(flac.to_encode_target().bitrate, 128);
    }

    #[test]
    fn test_lossy_source_bitrate_is_preserved() {
        // Lossy sources must keep their existing behaviour: no clamping.
        let mp3 = QualityProfile::new(128, 44100, 2, "mp3".to_string(), 3600.0).unwrap();
        assert_eq!(mp3.to_encode_target().bitrate, 128);

        let mp3_high = QualityProfile::new(320, 44100, 2, "mp3".to_string(), 3600.0).unwrap();
        assert_eq!(mp3_high.to_encode_target().bitrate, 320);
    }

    #[test]
    fn test_source_preset_still_clamps_lossless() {
        // "source" is a documented --quality value meaning "auto-detect from the
        // source". It must still clamp a lossless bitrate, or it reintroduces the
        // enormous-output bug of issue #18.
        let flac = QualityProfile::new(920, 44100, 2, "flac".to_string(), 3600.0).unwrap();
        assert_eq!(flac.apply_preset(Some("source")).bitrate, 192);
    }

    #[test]
    fn test_source_preset_preserves_lossy_bitrate() {
        let mp3 = QualityProfile::new(320, 44100, 2, "mp3".to_string(), 3600.0).unwrap();
        assert_eq!(mp3.apply_preset(Some("source")).bitrate, 320);
    }

    #[test]
    fn test_explicit_preset_overrides_lossless_clamp() {
        // An explicit preset is the user's decision and must win over the clamp.
        let flac = QualityProfile::new(920, 44100, 2, "flac".to_string(), 3600.0).unwrap();
        assert_eq!(flac.apply_preset(Some("low")).bitrate, 64);
        assert_eq!(flac.apply_preset(Some("maximum")).bitrate, 256);
    }

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
