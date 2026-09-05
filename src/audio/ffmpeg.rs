//! FFmpeg wrapper for audio operations

use crate::audio::AacEncoder;
use crate::models::QualityProfile;
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Audio file metadata extracted from ffprobe
#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub composer: Option<String>,
    pub comment: Option<String>,
}

/// FFmpeg operations wrapper
#[derive(Clone)]
pub struct FFmpeg {
    /// Path to ffmpeg binary
    ffmpeg_path: String,
    /// Path to ffprobe binary
    ffprobe_path: String,
}


/// Read a numeric ffprobe field that may be encoded as a JSON string or number.
///
/// ffprobe emits most numeric fields as strings, but this varies across builds
/// and output options; assuming strings caused spurious "No bitrate found"
/// failures (issue #18).
fn parse_u64_field(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|s| s.trim().parse::<u64>().ok()))
}

/// Read a floating-point ffprobe field encoded as either a JSON string or number.
fn parse_f64_field(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|s| s.trim().parse::<f64>().ok()))
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

        // Extract bitrate. Lossless codecs (FLAC, ALAC, WAV) carry no bitrate in
        // their stream headers, so ffprobe omits `bit_rate` on the audio stream
        // and only libavformat's computed `format.bit_rate` is available. Fall
        // back to it, and fail with a message that names the missing field
        // rather than the bare "No bitrate found" (issue #18).
        let bitrate_bps = parse_u64_field(&audio_stream["bit_rate"])
            .or_else(|| parse_u64_field(&json["format"]["bit_rate"]))
            .context(
                "ffprobe reported no bit_rate on either the audio stream or the \
                 container format. The file may be truncated or written without \
                 duration metadata; re-encoding or remuxing it usually fixes this",
            )?;

        if bitrate_bps < 1000 {
            anyhow::bail!(
                "ffprobe reported an implausible bit_rate of {} bps",
                bitrate_bps
            );
        }
        let bitrate = (bitrate_bps / 1000) as u32; // Convert to kbps

        // Extract sample rate
        let sample_rate = parse_u64_field(&audio_stream["sample_rate"])
            .context("No sample rate found")? as u32;

        // Extract channels
        let channels = parse_u64_field(&audio_stream["channels"])
            .context("No channels found")? as u8;

        // Extract codec
        let codec = audio_stream["codec_name"]
            .as_str()
            .context("No codec found")?
            .to_string();

        // Extract duration
        let duration = parse_f64_field(&audio_stream["duration"])
            .or_else(|| parse_f64_field(&json["format"]["duration"]))
            .context("No duration found")?;

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

    /// Concatenate M4B files losslessly (copy mode only)
    pub async fn concat_m4b_files(
        &self,
        concat_file: &Path,
        output_file: &Path,
    ) -> Result<()> {
        let mut cmd = Command::new(&self.ffmpeg_path);

        cmd.args([
            "-y",
            "-f", "concat",
            "-safe", "0",
            "-i",
        ])
        .arg(concat_file)
        .args([
            // Drop video streams. Source M4B files may embed cover art as an
            // mjpeg video stream; the ipod/M4B muxer rejects mjpeg in copy mode
            // ("Could not find tag for codec mjpeg"), so carrying it breaks the
            // mux (issue #17). Cover art is re-injected downstream via
            // AtomicParsley from the scanner's extracted/standalone cover file.
            "-vn",
            "-c", "copy",           // Lossless copy
            "-movflags", "+faststart",
        ])
        .arg(output_file);

        tracing::debug!("FFmpeg M4B concat command: {:?}", cmd.as_std());
        tracing::info!("Concatenating M4B files (lossless copy mode)");

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute ffmpeg")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("FFmpeg M4B concatenation failed: {}", stderr);
        }

        Ok(())
    }

    /// Probe metadata from audio file
    pub async fn probe_metadata(&self, path: &Path) -> Result<AudioMetadata> {
        let output = Command::new(&self.ffprobe_path)
            .args([
                "-v", "quiet",
                "-print_format", "json",
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

        let tags = &json["format"]["tags"];

        Ok(AudioMetadata {
            title: tags["title"].as_str().map(String::from),
            artist: tags["artist"].as_str().map(String::from),
            album: tags["album"].as_str().map(String::from),
            album_artist: tags["album_artist"].as_str().map(String::from),
            year: tags["date"].as_str().and_then(|s| s.get(..4).and_then(|y| y.parse().ok())),
            genre: tags["genre"].as_str().map(String::from),
            composer: tags["composer"].as_str().map(String::from),
            comment: tags["comment"].as_str().map(String::from),
        })
    }

    /// Probe an audio file's duration (ms) and embedded title in one ffprobe call.
    ///
    /// Used when merging chapterless M4B files: each file's duration becomes the
    /// span, and its title the label, of a synthesized one-chapter-per-file entry
    /// (issue #15). A single ffprobe invocation reads both from `format`.
    pub async fn probe_duration_and_title(&self, path: &Path) -> Result<(u64, Option<String>)> {
        let output = Command::new(&self.ffprobe_path)
            .args([
                "-v", "quiet",
                "-print_format", "json",
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

        let duration_secs = json["format"]["duration"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .with_context(|| format!("No duration found for {}", path.display()))?;
        let duration_ms = (duration_secs * 1000.0).round() as u64;

        let title = json["format"]["tags"]["title"]
            .as_str()
            .map(String::from)
            .filter(|t| !t.trim().is_empty());

        Ok((duration_ms, title))
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

    /// FLAC stores no bitrate in STREAMINFO, so ffprobe omits `bit_rate` on the
    /// audio stream and only libavformat's computed `format.bit_rate` is present
    /// (issue #18).
    #[test]
    fn test_parse_ffprobe_flac_stream_without_bitrate() {
        let json_str = r#"{
            "streams": [{
                "codec_type": "audio",
                "codec_name": "flac",
                "sample_rate": "44100",
                "channels": 2,
                "duration": "1800.0"
            }],
            "format": {
                "bit_rate": "920000",
                "duration": "1800.0"
            }
        }"#;

        let json: Value = serde_json::from_str(json_str).unwrap();
        let ffmpeg = FFmpeg::new().unwrap();
        let profile = ffmpeg.parse_ffprobe_output(&json).unwrap();

        assert_eq!(profile.codec, "flac");
        assert_eq!(profile.bitrate, 920);
    }

    /// Some ffprobe builds emit numeric JSON values rather than strings. The
    /// parser must accept both rather than reporting "No bitrate found".
    #[test]
    fn test_parse_ffprobe_numeric_fields() {
        let json_str = r#"{
            "streams": [{
                "codec_type": "audio",
                "codec_name": "flac",
                "sample_rate": 48000,
                "channels": 2,
                "bit_rate": 960000,
                "duration": 1200.0
            }],
            "format": {
                "bit_rate": 960000,
                "duration": 1200.0
            }
        }"#;

        let json: Value = serde_json::from_str(json_str).unwrap();
        let ffmpeg = FFmpeg::new().unwrap();
        let profile = ffmpeg.parse_ffprobe_output(&json).unwrap();

        assert_eq!(profile.bitrate, 960);
        assert_eq!(profile.sample_rate, 48000);
        assert!((profile.duration - 1200.0).abs() < 0.1);
    }

    /// When neither the stream nor the format carries a bitrate, the error must
    /// name the file and the missing field instead of the bare "No bitrate found"
    /// that sent the reporter of #18 looking at his FLAC install.
    #[test]
    fn test_parse_ffprobe_missing_bitrate_has_actionable_error() {
        let json_str = r#"{
            "streams": [{
                "codec_type": "audio",
                "codec_name": "flac",
                "sample_rate": "44100",
                "channels": 2,
                "duration": "1800.0"
            }],
            "format": {
                "duration": "1800.0"
            }
        }"#;

        let json: Value = serde_json::from_str(json_str).unwrap();
        let ffmpeg = FFmpeg::new().unwrap();
        let err = ffmpeg.parse_ffprobe_output(&json).unwrap_err().to_string();

        assert!(
            err.contains("bit_rate"),
            "error should name the missing field, got: {err}"
        );
    }
}
