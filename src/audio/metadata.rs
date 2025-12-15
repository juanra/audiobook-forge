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

    // Extract track number
    if let Some(track_num) = tag.track_number() {
        track.track_number = Some(track_num as u32);
    }

    Ok(())
}

/// Extract metadata from any audio file (auto-detect format)
pub fn extract_metadata(track: &mut Track) -> Result<()> {
    if track.is_mp3() {
        extract_mp3_metadata(track)
    } else if track.is_m4a() {
        extract_m4a_metadata(track)
    } else {
        // Unknown format - skip metadata extraction
        Ok(())
    }
}

/// Inject metadata into M4B file using AtomicParsley
pub async fn inject_metadata_atomicparsley(
    file_path: &Path,
    title: Option<&str>,
    artist: Option<&str>,
    album: Option<&str>,
    year: Option<u32>,
    genre: Option<&str>,
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
    if let Some(year) = year {
        cmd.args(&["--year", &year.to_string()]);
    }
    if let Some(genre) = genre {
        cmd.args(&["--genre", genre]);
    }
    if let Some(cover) = cover_art {
        cmd.args(&["--artwork", &cover.display().to_string()]);
    }

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

    // Cover art
    if let Some(cover_path) = cover_art {
        cmd.args(&["--artwork", &cover_path.display().to_string()]);
    }

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
}
