# Changelog

All notable changes to audiobook-forge will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.11.3] - 2026-09-07

### Fixed
- **M4B files merged in the wrong order** (#31): audiobooks whose track number is
  a filename *prefix* (`001 Author (Year) Title.m4b`) were concatenated in raw
  filesystem order, producing an audiobook that started partway through. The sort
  key was extracted with a suffix-anchored regex that matched nothing on these
  names, so every file collapsed to key `0` and the stable sort silently did
  nothing. M4B files are now ordered with the same natural sort already used for
  MP3 tracks, which handles prefixes, `Part`/`Disc`/`CD` suffixes, and unpadded
  numbers alike. M4B files are also sorted for every book, not only for folders
  matching a merge pattern: `--merge-m4b` can force a merge of any folder, and
  `read_dir` order is never the correct playback order.
- **Merge results missing from the batch summary** (#31): a run consisting only of
  M4B merges reported `Batch complete: 0 successful, 0 failed` even when the merge
  succeeded, because merges were printed but never recorded. Merge outcomes now
  feed the same tally as conversions.
- **Build aborted on a library containing a directory named like an audio file**
  (#30): a subdirectory whose name ends in `.mp3`, `.m4a` or `.m4b` (as produced by
  some usenet downloaders) made the library root look like a single audiobook, and
  the run stopped with the misleading `Current directory does not contain valid
  audiobook files` before any book was processed. The detection helpers counted
  directory entries by extension without checking that they were files, while the
  scanner did check — the two disagreed. Both now share one `is_audio_track_file`
  helper and one extension list, so they cannot drift apart again. This also fixes
  `.flac` folders not being recognised as single audiobooks.
- **FLAC and other lossless sources failing with "No bitrate found"** (#18):
  numeric ffprobe fields are now accepted whether they are reported as JSON strings
  or numbers, and when no bitrate is available at all the error names the missing
  field and suggests a remedy instead of sending users to check their FLAC install.
- **M4B duration probe shared the FLAC bitrate defect** (#18): the duration
  reader parsed `format.duration` as a JSON string only, the same assumption
  that made FLAC sources fail, so a build reporting numeric fields would fail
  when merging. Both readers now accept either shape.
- **Analyzer re-sorted tracks lexicographically** (#25): after analysis, tracks
  were re-sorted with a plain path comparison, which places `track10` before
  `track2` and could silently reorder a book the scanner had already ordered
  correctly. The analyzer probes concurrently with `buffer_unordered`, so this
  sort is what establishes final track order. It now uses the same natural
  ordering as the scanner. Reported and independently fixed by @JKamsker in #25,
  which predates this change.
- **A single unreadable track aborted the whole batch** (#18): one bad file in the
  first book stopped a 60-book run. Unreadable tracks are now skipped with a
  warning; a book is only failed when none of its tracks can be read.

### Changed
- **Lossless sources no longer transcode at the source bitrate**: FLAC, ALAC and
  WAV probe at 900-1000+ kbps, which was passed straight to the AAC encoder,
  producing enormous output files at a bitrate AAC cannot meaningfully use.
  Lossless sources now target 192 kbps AAC (stereo) or 128 kbps (mono) unless an
  explicit `--quality` preset is given. Lossy sources keep their existing bitrate,
  and `--quality source` (which means "auto-detect from the source") clamps too.
- **Skipped tracks are reported in the batch summary**: a book that completed with
  unreadable tracks now says so on its result line, instead of only in the log.

### Internal
- **The codebase is formatted with rustfmt.** This touched most files without
  changing behaviour, so the formatting commit is listed in
  `.git-blame-ignore-revs` and `git blame` skips it.

### Contributors
Thanks to the contributors who reported the bugs fixed in this release:
- [@virtualistic](https://github.com/virtualistic) — reported the M4B merge ordering bug (#31)
- [@christopherpross](https://github.com/christopherpross) — reported the library scan abort, with a precise reproduction (#30)
- [@dkuester](https://github.com/dkuester) — reported the FLAC transcode failure (#18)
- [@JKamsker](https://github.com/JKamsker) — diagnosed the analyzer track-ordering
  bug and proposed the same fix in #25, weeks before it was rediscovered and
  shipped here

## [2.11.2] - 2026-07-09

### Fixed
- **M4B merge failed on sources with embedded cover art** (#17): source M4B files
  that embed cover art as an mjpeg video stream broke the lossless concat step
  (`Could not find tag for codec mjpeg ... Could not write header for output
  file #0`), because the concat command carried all input streams into the
  ipod/M4B muxer in `-c copy` mode. The merge concat now drops video streams
  (`-vn`), matching the existing MP3 concat and single-file conversion paths.
  Cover art is unaffected: it is re-embedded downstream via AtomicParsley from
  the scanner's extracted or standalone cover file.

### Changed
- **Warn when a coverless build has `auto_extract_cover` disabled**: if a book
  has no standalone cover file and `auto_extract_cover` is turned off in config,
  the scanner now logs a warning that the output will have no cover art (the
  embedded source cover is dropped during conversion/merge).

### Contributors
Thanks to the contributors whose work shipped in this release:
- [@virtualistic](https://github.com/virtualistic) — reported the M4B merge cover-art regression (#17)

## [2.11.1] - 2026-07-03

### Added
- **FLAC source support** (#13): `.flac` files are now accepted as source tracks
  and transcoded to AAC like MP3/M4A sources, enabling workflows that rip audio
  CDs to FLAC for maximum fidelity. Track metadata (title, artist, album, track
  number, etc.) is read from FLAC Vorbis comments via ffprobe. Thanks to
  @xanium4332 for the initial contribution.

### Fixed
- **AtomicParsley crash on NUL characters in comment tags** (#14): comment
  metadata is now sanitized (embedded NUL characters stripped) before being
  passed to AtomicParsley, which previously failed on such tags. Thanks to
  @Haysdp for the fix.
- **Concat copy-mode codec check now uses an allowlist**: stream copy into the
  M4B container is only attempted for codecs that live natively in MP4 (AAC,
  ALAC). Previously only MP3 was blocklisted, so other non-MP4 codecs (e.g. FLAC)
  would fall through to a failing `-c copy`.
- **`normalize_string` left trailing whitespace after stripping symbols**: titles
  like `"Title! @ # $"` normalized to `"title   "` instead of `"title"`, which
  degraded metadata match scoring. Whitespace runs are now collapsed.

### Contributors
Thanks to the contributors whose work shipped in this release:
- [@xanium4332](https://github.com/xanium4332) — FLAC source support (#13)
- [@Haysdp](https://github.com/Haysdp) — AtomicParsley NUL comment crash fix (#14)

## [2.11.0] - 2026-07-03

### Fixed
- **Interactive `match` selection applied the wrong candidate** (#12): the selected
  option was recovered by scanning the color-escaped menu label for its first
  digit, which misread ANSI color codes (e.g. `\x1b[32m`) as the option number.
  Selection is now dispatched by the chosen list position via `inquire`'s
  `raw_prompt()`.
- **`metadata enrich` accumulated duplicate cover images** (#11): every run
  appended another artwork stream because AtomicParsley's `--artwork` appends.
  Existing artwork is now stripped with `--artwork REMOVE_ALL` before embedding
  the new cover.
- **`build --fetch-audible` did not embed cover art** (#10): the fetched Audible
  cover URL was discarded. The cover is now downloaded and embedded during the
  build, without requiring a separate `enrich` run. Respects an existing local
  cover and the `metadata.audible.download_covers` config.
- **M4B merge of chapterless files failed with 0 chapters** (#15): merging
  incremental audiobooks (one file per chapter, no internal chapters) now
  synthesizes a single chapter per source file from its duration and title. The
  underlying ffmpeg error is also surfaced in full instead of being hidden behind
  a generic "Failed to concatenate M4B files" message.

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
