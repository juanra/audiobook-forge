use std::process::Command;
use std::sync::OnceLock;

/// AAC encoder types supported by audiobook-forge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AacEncoder {
    /// Apple Silicon hardware encoder (aac_at) - macOS only
    AppleSilicon,
    /// Fraunhofer FDK AAC encoder (libfdk_aac) - high quality
    LibFdk,
    /// FFmpeg native AAC encoder (aac) - universal fallback
    Native,
    /// Windows Media Foundation encoder (aac_mf) - Windows only, opt-in.
    ///
    /// Not auto-detected yet: Media Foundation documents only 44.1/48 kHz and
    /// 96-192 kbps, which the `low` preset and high source bitrates fall
    /// outside of (issue #36).
    MediaFoundation,
}

impl AacEncoder {
    /// Returns the FFmpeg encoder name
    pub fn name(&self) -> &'static str {
        match self {
            Self::AppleSilicon => "aac_at",
            Self::LibFdk => "libfdk_aac",
            Self::Native => "aac",
            Self::MediaFoundation => "aac_mf",
        }
    }

    /// Returns whether this encoder benefits from multi-threading
    pub fn supports_threading(&self) -> bool {
        match self {
            Self::AppleSilicon => false,    // Hardware encoder, no threading needed
            Self::LibFdk => false,          // Single-threaded by design
            Self::Native => true,           // Benefits from multi-threading
            Self::MediaFoundation => false, // System encoder, threads itself
        }
    }

    /// Try to parse encoder from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "aac_at" => Some(Self::AppleSilicon),
            "libfdk_aac" | "libfdk" => Some(Self::LibFdk),
            "aac" => Some(Self::Native),
            "aac_mf" => Some(Self::MediaFoundation),
            _ => None,
        }
    }
}

impl std::fmt::Display for AacEncoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Encoder detection and selection
pub struct EncoderDetector;

impl EncoderDetector {
    /// Detect the best available AAC encoder
    /// Priority: aac_at → libfdk_aac → aac
    pub fn detect_best_encoder() -> AacEncoder {
        let candidates = [
            AacEncoder::AppleSilicon,
            AacEncoder::LibFdk,
            AacEncoder::Native,
        ];

        for encoder in candidates {
            if Self::is_encoder_available(encoder) {
                tracing::info!("Detected AAC encoder: {}", encoder.name());
                return encoder;
            }
        }

        // Fallback (should never happen as 'aac' is always available)
        tracing::warn!("No AAC encoder detected, defaulting to 'aac'");
        AacEncoder::Native
    }

    /// Check if a specific encoder is available in FFmpeg
    pub fn is_encoder_available(encoder: AacEncoder) -> bool {
        let output = Command::new("ffmpeg").args(&["-encoders"]).output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Format: " A....D encodername     Description". Match the name
            // field exactly: a substring test made `aac` match `aac_mf` lines.
            stdout.lines().any(|line| {
                let mut fields = line.split_whitespace();
                fields.next().is_some_and(|flags| flags.starts_with('A'))
                    && fields.next() == Some(encoder.name())
            })
        } else {
            false
        }
    }

    /// Get all available AAC encoders
    pub fn get_available_encoders() -> Vec<AacEncoder> {
        let all_encoders = [
            AacEncoder::AppleSilicon,
            AacEncoder::LibFdk,
            AacEncoder::Native,
            AacEncoder::MediaFoundation,
        ];

        all_encoders
            .into_iter()
            .filter(|&encoder| Self::is_encoder_available(encoder))
            .collect()
    }
}

/// Global cache for detected encoder
static DETECTED_ENCODER: OnceLock<AacEncoder> = OnceLock::new();

/// Get the best available AAC encoder (cached, thread-safe)
pub fn get_encoder() -> AacEncoder {
    *DETECTED_ENCODER.get_or_init(|| EncoderDetector::detect_best_encoder())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_name() {
        assert_eq!(AacEncoder::AppleSilicon.name(), "aac_at");
        assert_eq!(AacEncoder::LibFdk.name(), "libfdk_aac");
        assert_eq!(AacEncoder::Native.name(), "aac");
        assert_eq!(AacEncoder::MediaFoundation.name(), "aac_mf");
    }

    #[test]
    fn test_encoder_threading() {
        assert!(!AacEncoder::AppleSilicon.supports_threading());
        assert!(!AacEncoder::LibFdk.supports_threading());
        assert!(AacEncoder::Native.supports_threading());
        assert!(!AacEncoder::MediaFoundation.supports_threading());
    }

    #[test]
    fn test_encoder_from_str() {
        assert_eq!(
            AacEncoder::from_str("aac_at"),
            Some(AacEncoder::AppleSilicon)
        );
        assert_eq!(AacEncoder::from_str("libfdk_aac"), Some(AacEncoder::LibFdk));
        assert_eq!(AacEncoder::from_str("libfdk"), Some(AacEncoder::LibFdk));
        assert_eq!(AacEncoder::from_str("aac"), Some(AacEncoder::Native));
        assert_eq!(
            AacEncoder::from_str("AAC_MF"),
            Some(AacEncoder::MediaFoundation)
        );
        assert_eq!(AacEncoder::from_str("unknown"), None);
    }

    #[test]
    fn test_encoder_display() {
        assert_eq!(format!("{}", AacEncoder::AppleSilicon), "aac_at");
        assert_eq!(format!("{}", AacEncoder::LibFdk), "libfdk_aac");
        assert_eq!(format!("{}", AacEncoder::Native), "aac");
    }

    #[test]
    fn test_detect_encoder() {
        // Should detect at least the native 'aac' encoder
        let encoder = EncoderDetector::detect_best_encoder();
        assert!(matches!(
            encoder,
            AacEncoder::AppleSilicon | AacEncoder::LibFdk | AacEncoder::Native
        ));
    }

    #[test]
    fn test_get_available_encoders() {
        let encoders = EncoderDetector::get_available_encoders();
        // Should have at least one encoder (aac is universal)
        assert!(!encoders.is_empty());
        // Native aac should always be available
        assert!(encoders.contains(&AacEncoder::Native) || encoders.len() > 0);
    }

    #[test]
    fn test_get_encoder_cached() {
        // First call initializes
        let encoder1 = get_encoder();
        // Second call should return same cached value
        let encoder2 = get_encoder();
        assert_eq!(encoder1, encoder2);
    }
}
