//! FFmpeg wrapper for audio operations

use crate::audio::AacEncoder;
use crate::models::QualityProfile;
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// FFmpeg operations wrapper
#[derive(Clone)]
pub struct FFmpeg {
    /// Path to ffmpeg binary
    ffmpeg_path: String,
    /// Path to ffprobe binary
    ffprobe_path: String,
}

impl FFmpeg {
    /// Create a new FFmpeg wrapper with default paths
    pub fn new() -> Result<Self> {
        let ffmpeg_path = which::which("ffmpeg")
            .context("FFmpeg not found in PATH")?
            .to_string_lossy()
            .to_string();

        let ffprobe_path = which::which("ffprobe")
            .context("FFprobe not found in PATH")?
            .to_string_lossy()
            .to_string();

        Ok(Self {
            ffmpeg_path,
            ffprobe_path,
        })
    }

    /// Create FFmpeg wrapper with custom paths
    pub fn with_paths(ffmpeg_path: String, ffprobe_path: String) -> Self {
        Self {
            ffmpeg_path,
            ffprobe_path,
        }
    }

    /// Probe audio file and extract quality information
    pub async fn probe_audio_file(&self, path: &Path) -> Result<QualityProfile> {
        let output = Command::new(&self.ffprobe_path)
            .args(&[
                "-v", "quiet",
                "-print_format", "json",
                "-show_streams",
                "-show_format",
            ])
            .arg(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute ffprobe")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("FFprobe failed: {}", stderr);
        }

        let json: Value = serde_json::from_slice(&output.stdout)
            .context("Failed to parse ffprobe JSON output")?;

        self.parse_ffprobe_output(&json)
    }

    /// Parse ffprobe JSON output into QualityProfile
    fn parse_ffprobe_output(&self, json: &Value) -> Result<QualityProfile> {
        // Find audio stream
        let streams = json["streams"]
            .as_array()
            .context("No streams in ffprobe output")?;

        let audio_stream = streams
            .iter()
            .find(|s| s["codec_type"] == "audio")
            .context("No audio stream found")?;

        // Extract bitrate
        let bitrate = if let Some(bit_rate) = audio_stream["bit_rate"].as_str() {
            bit_rate.parse::<u32>()? / 1000 // Convert to kbps
        } else {
            // Fallback to format bitrate
            json["format"]["bit_rate"]
                .as_str()
                .context("No bitrate found")?
                .parse::<u32>()? / 1000
        };

        // Extract sample rate
        let sample_rate = audio_stream["sample_rate"]
            .as_str()
            .context("No sample rate found")?
            .parse::<u32>()?;

        // Extract channels
        let channels = audio_stream["channels"]
            .as_u64()
            .context("No channels found")? as u8;

        // Extract codec
        let codec = audio_stream["codec_name"]
            .as_str()
            .context("No codec found")?
            .to_string();

        // Extract duration
        let duration = if let Some(dur) = audio_stream["duration"].as_str() {
            dur.parse::<f64>()?
        } else {
            json["format"]["duration"]
                .as_str()
                .context("No duration found")?
                .parse::<f64>()?
        };

        QualityProfile::new(bitrate, sample_rate, channels, codec, duration)
    }

    /// Concatenate audio files using FFmpeg
    pub async fn concat_audio_files(
        &self,
        concat_file: &Path,
        output_file: &Path,
        quality: &QualityProfile,
        use_copy: bool,
        encoder: AacEncoder,
    ) -> Result<()> {
        let mut cmd = Command::new(&self.ffmpeg_path);

        cmd.args(&[
            "-y",
            "-f", "concat",
            "-safe", "0",
            "-i",
        ])
        .arg(concat_file);

        // Skip video streams (embedded cover art in MP3s)
        cmd.arg("-vn");

        if use_copy {
            // Copy mode - no re-encoding
            cmd.args(&["-c", "copy"]);
        } else {
            // Transcode mode
            cmd.args(&[
                "-c:a", encoder.name(),
                "-b:a", &format!("{}k", quality.bitrate),
                "-ar", &quality.sample_rate.to_string(),
                "-ac", &quality.channels.to_string(),
            ]);

            // Use multiple threads for encoding if encoder supports it
            if encoder.supports_threading() {
                cmd.args(&["-threads", "0"]); // 0 = auto-detect optimal thread count
            }
        }

        // Add faststart flag for better streaming
        cmd.args(&["-movflags", "+faststart"]);
        cmd.arg(output_file);

        // Log command for debugging
        tracing::debug!("FFmpeg concat command: {:?}", cmd.as_std());
        tracing::info!(
            "Concatenating {} ({}mode)",
            concat_file.display(),
            if use_copy { "copy " } else { "transcode " }
        );

        // Execute command
        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute ffmpeg")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Provide helpful error message if encoder is the issue
            if stderr.to_lowercase().contains("encoder") {
                anyhow::bail!(
                    "FFmpeg encoding failed with encoder '{}': {}\nTip: Run 'audiobook-forge check' to verify encoder availability",
                    encoder.name(),
                    stderr
                );
            }
            anyhow::bail!("FFmpeg concatenation failed: {}", stderr);
        }

        Ok(())
    }

    /// Convert a single audio file to M4A/M4B
    pub async fn convert_single_file(
        &self,
        input_file: &Path,
        output_file: &Path,
        quality: &QualityProfile,
        use_copy: bool,
        encoder: AacEncoder,
    ) -> Result<()> {
        let mut cmd = Command::new(&self.ffmpeg_path);

        cmd.args(&["-y", "-i"])
            .arg(input_file);

        // Skip video streams (embedded cover art in MP3s)
        cmd.arg("-vn");

        if use_copy {
            cmd.args(&["-c", "copy"]);
        } else {
            cmd.args(&[
                "-c:a", encoder.name(),
                "-b:a", &format!("{}k", quality.bitrate),
                "-ar", &quality.sample_rate.to_string(),
                "-ac", &quality.channels.to_string(),
            ]);

            // Use multiple threads for encoding if encoder supports it
            if encoder.supports_threading() {
                cmd.args(&["-threads", "0"]); // 0 = auto-detect optimal thread count
            }
        }

        cmd.args(&["-movflags", "+faststart"]);
        cmd.arg(output_file);

        // Log command for debugging
        tracing::debug!("FFmpeg convert command: {:?}", cmd.as_std());
        tracing::info!(
            "Converting {} → {} (encoder: {}, {}kbps)",
            input_file.file_name().unwrap().to_string_lossy(),
            output_file.file_name().unwrap().to_string_lossy(),
            encoder.name(),
            quality.bitrate
        );

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute ffmpeg")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Provide helpful error message if encoder is the issue
            if stderr.to_lowercase().contains("encoder") {
                anyhow::bail!(
                    "FFmpeg encoding failed with encoder '{}': {}\nTip: Run 'audiobook-forge check' to verify encoder availability",
                    encoder.name(),
                    stderr
                );
            }
            anyhow::bail!("FFmpeg conversion failed: {}", stderr);
        }

        Ok(())
    }

    /// Create concat file for FFmpeg with proper path escaping
    pub fn create_concat_file(files: &[&Path], output: &Path) -> Result<()> {
        let mut content = String::new();
        for file in files {
            // Verify file exists before adding to concat list
            if !file.exists() {
                anyhow::bail!("File not found: {}", file.display());
            }

            // Get absolute path for better compatibility
            let abs_path = file.canonicalize()
                .with_context(|| format!("Failed to resolve path: {}", file.display()))?;

            // Escape the path for FFmpeg concat format
            // FFmpeg concat format requires:
            // - Single quotes around path
            // - Single quotes within path must be escaped as '\''
            // - Backslashes should be forward slashes (even on Windows for -safe 0)
            let path_str = abs_path.to_string_lossy();
            let escaped = path_str.replace('\'', r"'\''");

            content.push_str(&format!("file '{}'\n", escaped));
        }

        std::fs::write(output, content)
            .context("Failed to write concat file")?;

        Ok(())
    }
}

impl Default for FFmpeg {
    fn default() -> Self {
        Self::new().expect("FFmpeg not found")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffmpeg_initialization() {
        let ffmpeg = FFmpeg::new();
        assert!(ffmpeg.is_ok());
    }

    #[test]
    fn test_parse_ffprobe_json() {
        let json_str = r#"{
            "streams": [{
                "codec_type": "audio",
                "codec_name": "mp3",
                "sample_rate": "44100",
                "channels": 2,
                "bit_rate": "128000",
                "duration": "3600.5"
            }],
            "format": {
                "bit_rate": "128000",
                "duration": "3600.5"
            }
        }"#;

        let json: Value = serde_json::from_str(json_str).unwrap();
        let ffmpeg = FFmpeg::new().unwrap();
        let profile = ffmpeg.parse_ffprobe_output(&json).unwrap();

        assert_eq!(profile.bitrate, 128);
        assert_eq!(profile.sample_rate, 44100);
        assert_eq!(profile.channels, 2);
        assert_eq!(profile.codec, "mp3");
        assert!((profile.duration - 3600.5).abs() < 0.1);
    }
}
