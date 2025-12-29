# Changelog

All notable changes to audiobook-forge will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.9.0] - 2025-12-29

### Added
- **Chapter Update System**: Comprehensive system for updating M4B chapter names from multiple sources
  - Fetch chapters from Audnex API by ASIN
  - Parse chapters from text files (simple, timestamped, MP4Box formats)
  - Extract chapters from EPUB table of contents
  - Read existing chapters from M4B files using ffprobe
- **Chapter Merge Strategies**: Flexible strategies for merging new chapters with existing ones
  - `keep-timestamps`: Update names while preserving existing timestamps
  - `replace-all`: Replace entire chapter list
  - `skip-on-mismatch`: Error if chapter counts don't match
  - `interactive`: Prompt for each file (default)
- **CLI Enhancements**: Extended `metadata enrich` command with chapter update options
  - `--chapters <FILE>`: Import from text/EPUB file
  - `--chapters-asin <ASIN>`: Fetch from Audnex API
  - `--update-chapters-only`: Skip metadata, only update chapters
  - `--merge-strategy <STRATEGY>`: Choose merge approach
- **Build Integration**: Automatic chapter fetching during build process
  - New config option: `metadata.audible.fetch_chapters` (default: false)
  - Fetches chapters from Audnex API after successful metadata fetch
- **New Dependencies**: Added `epub = "2.0"` for EPUB parsing support

### Changed
- Enhanced `AudibleClient` with `fetch_chapters()` method for Audnex API integration
- Updated documentation with comprehensive chapter update examples and usage

### Technical Details
- **New Models**: `AudibleChapter`, `AudnexChaptersResponse`, `ChapterSource`, `ChapterMergeStrategy`, `ChapterComparison`
- **New Functions**: `parse_text_chapters()`, `parse_epub_chapters()`, `merge_chapters()`, `read_m4b_chapters()`
- **Integration Tests**: Added 8 comprehensive integration tests for chapter functionality
- **Files Modified**: 11 files, +1200 lines added across models, audio processing, CLI, and tests

### Reddit Community Request
This feature was implemented based on community feedback addressing the common issue of M4B files with generic chapter names ("Chapter 1", "Chapter 2"). Users can now easily replace these with meaningful titles from Audible chapter data, text files, or EPUB table of contents.

## [2.8.2] - Previous Release

See git history for details on earlier versions.
