//! Audio metadata extraction and manipulation

use crate::models::{Track, AudibleMetadata};
use anyhow::{Context, Result};
use id3::TagLike;
use std::path::Path;

/// Extract metadata from MP3 file using ID3 tags
pub fn extract_mp3_metadata(track: &mut Track) -> Result<()> {
    let tag = id3::Tag::read_from_path(&track.file_path)
        .context("Failed to read ID3 tags")?;

    // Extract basic metadata
    track.title = tag.title().map(|s| s.to_string());
    track.artist = tag.artist().map(|s| s.to_string());
    track.album = tag.album().map(|s| s.to_string());
    track.album_artist = tag.album_artist().map(|s| s.to_string());
    track.genre = tag.genre().map(|s| s.to_string());
    track.year = tag.year().map(|y| y as u32);
    track.comment = tag.comments().next().map(|c| c.text.clone());
    track.composer = tag.get("TCOM").and_then(|frame| frame.content().text()).map(|s| s.to_string());

    // Extract track number
    track.track_number = tag.track();

    Ok(())
}

/// Extract metadata from M4A/M4B file
pub fn extract_m4a_metadata(track: &mut Track) -> Result<()> {
    let tag = mp4ameta::Tag::read_from_path(&track.file_path)
        .context("Failed to read M4A metadata")?;

    // Extract basic metadata
    track.title = tag.title().map(|s| s.to_string());
    track.artist = tag.artist().map(|s| s.to_string());
    track.album = tag.album().map(|s| s.to_string());
    track.album_artist = tag.album_artist().map(|s| s.to_string());
    track.genre = tag.genre().map(|s| s.to_string());
    track.year = tag.year().map(|s| s.parse::<u32>().ok()).flatten();
    track.comment = tag.comment().map(|s| s.to_string());
    track.composer = tag.composer().map(|s| s.to_string());

    // Extract track number
    if let Some(track_num) = tag.track_number() {
        track.track_number = Some(track_num as u32);
    }

    Ok(())
}

/// Extract metadata from a FLAC file via ffprobe (reads Vorbis comments).
///
/// FLAC tags are Vorbis comments, which id3/mp4ameta cannot read. ffprobe is
/// already a runtime dependency and exposes them under `format.tags`, so we reuse
/// it rather than pulling in a new crate. Vorbis comment keys are conventionally
/// uppercase but not case-canonical, so lookups are case-insensitive.
pub fn extract_flac_metadata(track: &mut Track) -> Result<()> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
        ])
        .arg(&track.file_path)
        .output()
        .context("Failed to execute ffprobe for FLAC metadata")?;

    if !output.status.success() {
        anyhow::bail!("ffprobe failed to read FLAC metadata");
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("Failed to parse ffprobe JSON output")?;

    // Vorbis comment keys can be any case; collect them lowercased for lookup.
    let tags: std::collections::HashMap<String, String> = json["format"]["tags"]
        .as_object()
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.to_lowercase(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    let get = |key: &str| tags.get(key).map(|s| s.to_string());

    track.title = get("title");
    track.artist = get("artist");
    track.album = get("album");
    // Vorbis convention is ALBUMARTIST; ffmpeg also maps to album_artist.
    track.album_artist = get("albumartist").or_else(|| get("album_artist"));
    track.genre = get("genre");
    // Vorbis DATE is often a full date or just a year; take the leading 4 digits.
    track.year = get("date")
        .as_deref()
        .and_then(|s| s.get(..4))
        .and_then(|y| y.parse::<u32>().ok());
    track.comment = get("comment").or_else(|| get("description"));
    track.composer = get("composer");

    // TRACKNUMBER may be "5" or "5/12"; take the part before the slash.
    track.track_number = get("tracknumber")
        .or_else(|| get("track"))
        .and_then(|s| s.split('/').next().and_then(|n| n.trim().parse::<u32>().ok()));

    Ok(())
}

/// Extract metadata from any audio file (auto-detect format)
pub fn extract_metadata(track: &mut Track) -> Result<()> {
    if track.is_mp3() {
        extract_mp3_metadata(track)
    } else if track.is_m4a() {
        extract_m4a_metadata(track)
    } else if track.is_flac() {
        extract_flac_metadata(track)
    } else {
        // Unknown format - skip metadata extraction
        Ok(())
    }
}

/// Extract embedded cover art from MP3 file (APIC frame)
pub fn extract_mp3_cover_art(file_path: &Path, output_path: &Path) -> Result<bool> {
    let tag = id3::Tag::read_from_path(file_path)
        .context("Failed to read ID3 tag")?;

    // Collect pictures to avoid borrow checker issues
    let pictures: Vec<_> = tag.pictures().collect();

    // Get first picture (APIC frame)
    if let Some(picture) = pictures.first() {
        tracing::debug!(
            "Extracting embedded cover from MP3: {} ({} bytes, type: {:?})",
            file_path.display(),
            picture.data.len(),
            picture.picture_type
        );

        std::fs::write(output_path, &picture.data)
            .context("Failed to write extracted cover")?;

        Ok(true)
    } else {
        tracing::debug!("No embedded cover found in MP3: {}", file_path.display());
        Ok(false)
    }
}

/// Extract embedded cover art from M4A/M4B file
pub fn extract_m4a_cover_art(file_path: &Path, output_path: &Path) -> Result<bool> {
    let tag = mp4ameta::Tag::read_from_path(file_path)
        .context("Failed to read M4A tag")?;

    // Get artwork (first image)
    if let Some(artwork) = tag.artwork() {
        tracing::debug!(
            "Extracting embedded cover from M4A: {} ({} bytes)",
            file_path.display(),
            artwork.data.len()
        );

        std::fs::write(output_path, &artwork.data)
            .context("Failed to write extracted cover")?;

        Ok(true)
    } else {
        tracing::debug!("No embedded cover found in M4A: {}", file_path.display());
        Ok(false)
    }
}

/// Extract embedded cover art from any audio file (auto-detect format)
pub fn extract_embedded_cover(file_path: &Path, output_path: &Path) -> Result<bool> {
    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension.to_lowercase().as_str() {
        "mp3" => extract_mp3_cover_art(file_path, output_path),
        "m4a" | "m4b" => extract_m4a_cover_art(file_path, output_path),
        _ => {
            tracing::debug!("Unsupported format for cover extraction: {}", extension);
            Ok(false)
        }
    }
}

/// Build the AtomicParsley artwork arguments for a cover image.
///
/// Emits `--artwork REMOVE_ALL` before the new artwork so repeated runs replace
/// the cover instead of appending duplicate image streams (issue #11). Returns an
/// empty vec when there is no cover to embed.
fn artwork_args(cover_art: Option<&Path>) -> Vec<String> {
    match cover_art {
        Some(cover) => vec![
            "--artwork".to_string(),
            "REMOVE_ALL".to_string(),
            "--artwork".to_string(),
            cover.display().to_string(),
        ],
        None => Vec::new(),
    }
}

/// Inject metadata into M4B file using AtomicParsley
pub async fn inject_metadata_atomicparsley(
    file_path: &Path,
    title: Option<&str>,
    artist: Option<&str>,
    album: Option<&str>,
    album_artist: Option<&str>,
    year: Option<u32>,
    genre: Option<&str>,
    composer: Option<&str>,
    comment: Option<&str>,
    cover_art: Option<&Path>,
) -> Result<()> {
    let mut cmd = tokio::process::Command::new("AtomicParsley");
    cmd.arg(file_path);

    if let Some(title) = title {
        cmd.args(&["--title", title]);
    }
    if let Some(artist) = artist {
        cmd.args(&["--artist", artist]);
    }
    if let Some(album) = album {
        cmd.args(&["--album", album]);
    }
    if let Some(album_artist) = album_artist {
        cmd.args(&["--albumArtist", album_artist]);
    }
    if let Some(year) = year {
        cmd.args(&["--year", &year.to_string()]);
    }
    if let Some(genre) = genre {
        cmd.args(&["--genre", genre]);
    }
    if let Some(composer) = composer {
        cmd.args(&["--composer", composer]);
    }
    if let Some(comment) = comment {
        let comment = comment.replace('\0', "");
        if !comment.is_empty() {
            cmd.args(&["--comment", &comment]);
        }
    }
    cmd.args(artwork_args(cover_art));

    cmd.args(&["--overWrite"]);

    // Log command for debugging
    tracing::debug!("AtomicParsley command: {:?}", cmd.as_std());

    let output = cmd
        .output()
        .await
        .context("Failed to execute AtomicParsley")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("AtomicParsley failed: {}", stderr);
    }

    Ok(())
}

/// Inject Audible metadata into M4B file
pub async fn inject_audible_metadata(
    file_path: &Path,
    audible: &AudibleMetadata,
    cover_art: Option<&Path>,
) -> Result<()> {
    let mut cmd = tokio::process::Command::new("AtomicParsley");
    cmd.arg(file_path);

    // Title (with subtitle if present)
    let full_title = if let Some(subtitle) = &audible.subtitle {
        format!("{}: {}", audible.title, subtitle)
    } else {
        audible.title.clone()
    };
    cmd.args(&["--title", &full_title]);

    // Album (use title for audiobooks)
    cmd.args(&["--album", &audible.title]);

    // Artist (primary author)
    if let Some(author) = audible.primary_author() {
        cmd.args(&["--artist", author]);
        cmd.args(&["--albumArtist", author]);
    }

    // Narrator as composer (audiobook convention)
    if let Some(narrator) = audible.primary_narrator() {
        cmd.args(&["--composer", narrator]);
    }

    // Subtitle (if present and not already in title)
    if let Some(subtitle) = &audible.subtitle {
        cmd.args(&["--description", subtitle]);
    }

    // Description/Summary
    if let Some(desc) = &audible.description {
        // Limit description length to avoid issues with AtomicParsley
        let truncated_desc = if desc.len() > 4000 {
            format!("{}...", &desc[..4000])
        } else {
            desc.clone()
        };
        cmd.args(&["--longdesc", &truncated_desc]);
        cmd.args(&["--comment", &truncated_desc]);
    }

    // Publisher
    if let Some(publisher) = &audible.publisher {
        cmd.args(&["--rDNSatom", &format!("{}", publisher), "name=publisher", "domain=com.apple.iTunes"]);
    }

    // Year
    if let Some(year) = audible.published_year {
        cmd.args(&["--year", &year.to_string()]);
    }

    // Genre (first genre)
    if let Some(genre) = audible.genres.first() {
        cmd.args(&["--genre", genre]);
    }

    // ASIN as custom atom (for Audiobookshelf)
    cmd.args(&["--rDNSatom", &audible.asin, "name=asin", "domain=com.audible"]);

    // Cover art (strips existing artwork first — issue #11)
    cmd.args(artwork_args(cover_art));

    // Overwrite
    cmd.args(&["--overWrite"]);

    let output = cmd
        .output()
        .await
        .context("Failed to execute AtomicParsley")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("AtomicParsley failed: {}", stderr);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::QualityProfile;
    use std::path::PathBuf;

    #[test]
    fn test_extract_metadata_mp3() {
        // This test requires an actual MP3 file with metadata
        // For now, just test that the function signature is correct
        let quality = QualityProfile::new(128, 44100, 2, "mp3".to_string(), 3600.0).unwrap();
        let mut track = Track::new(PathBuf::from("test.mp3"), quality);

        // If file doesn't exist, this will fail - that's OK for now
        let _ = extract_mp3_metadata(&mut track);
    }

    #[test]
    fn test_extract_metadata_m4a() {
        let quality = QualityProfile::new(128, 44100, 2, "aac".to_string(), 3600.0).unwrap();
        let mut track = Track::new(PathBuf::from("test.m4a"), quality);

        // If file doesn't exist, this will fail - that's OK for now
        let _ = extract_m4a_metadata(&mut track);
    }

    // Regression test for issue #11: cover embedding must strip existing artwork
    // before adding the new image, otherwise every run appends a duplicate cover.
    #[test]
    fn artwork_args_strips_existing_before_adding() {
        let cover = PathBuf::from("/tmp/cover.jpg");
        let args = artwork_args(Some(&cover));
        assert_eq!(
            args,
            vec![
                "--artwork".to_string(),
                "REMOVE_ALL".to_string(),
                "--artwork".to_string(),
                "/tmp/cover.jpg".to_string(),
            ]
        );
        // REMOVE_ALL must come before the new artwork path.
        let remove_pos = args.iter().position(|a| a == "REMOVE_ALL").unwrap();
        let path_pos = args.iter().position(|a| a == "/tmp/cover.jpg").unwrap();
        assert!(remove_pos < path_pos);
    }

    #[test]
    fn artwork_args_empty_without_cover() {
        assert!(artwork_args(None).is_empty());
    }
}
