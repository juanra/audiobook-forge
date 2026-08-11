//! Audio processing modules
//!
//! This module contains audio-specific functionality:
//! - FFmpeg: Subprocess management for audio operations
//! - Metadata: Extraction and injection of audio metadata
//! - Chapters: Chapter generation and management
//! - Audible: Audible metadata fetching and integration
//! - Encoder: AAC encoder detection and selection

pub mod audible;
mod chapter_import;
mod chapters;
pub mod encoder;
mod ffmpeg;
mod metadata;

pub use audible::{clean_sequence, detect_asin, AudibleClient};
pub use chapter_import::{
    merge_chapter_lists, merge_chapters, parse_epub_chapters, parse_text_chapters,
    read_m4b_chapters, ChapterComparison, ChapterMergeStrategy, ChapterSource,
};
pub use chapters::{
    default_chapters_output, generate_chapters, generate_chapters_from_files,
    inject_chapters_mp4box, parse_cue_file, write_ffmetadata, write_json, write_mp4box_chapters,
    Chapter,
};
pub use encoder::{get_encoder, AacEncoder, EncoderDetector};
pub use ffmpeg::{AudioMetadata, FFmpeg};
pub use metadata::{
    extract_embedded_cover, extract_flac_metadata, extract_m4a_metadata, extract_metadata,
    extract_mp3_metadata, inject_audible_metadata, inject_metadata_atomicparsley,
};
