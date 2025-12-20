//! Audio processing modules
//!
//! This module contains audio-specific functionality:
//! - FFmpeg: Subprocess management for audio operations
//! - Metadata: Extraction and injection of audio metadata
//! - Chapters: Chapter generation and management
//! - Audible: Audible metadata fetching and integration
//! - Encoder: AAC encoder detection and selection

mod ffmpeg;
mod metadata;
mod chapters;
pub mod audible;
pub mod encoder;

pub use ffmpeg::FFmpeg;
pub use metadata::{extract_metadata, extract_mp3_metadata, extract_m4a_metadata, inject_metadata_atomicparsley, inject_audible_metadata};
pub use chapters::{Chapter, generate_chapters_from_files, parse_cue_file, write_mp4box_chapters, inject_chapters_mp4box};
pub use audible::{AudibleClient, detect_asin, clean_sequence};
pub use encoder::{AacEncoder, get_encoder, EncoderDetector};
