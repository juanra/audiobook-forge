# Changelog

> ❤️ **Support this project**: If you find audiobook-forge useful, consider [sponsoring development](https://github.com/sponsors/juanra)!

All notable changes to audiobook-forge (Rust version) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.6.4] - 2025-12-20

### 📝 Documentation

#### Quality Preservation Clarifications
- **Updated v2.6.3 documentation** - Republished with clearer quality preservation behavior
  - No code changes from v2.6.3, only documentation improvements
  - Published to crates.io (v2.6.3 couldn't be republished per crates.io policy)

## [2.6.3] - 2025-12-20

### 📝 Documentation

#### README Updates
- **Quality presets documentation** - Updated README to document `ultra` and `maximum` presets
  - Added detailed descriptions of all quality presets with bitrate/sample rate specs
  - Clarified troubleshooting section to explain quality presets don't improve source quality
  - Fixed misleading language: "Smart Quality Detection" now says "preserves source audio quality"
  - Added important note: Higher bitrate presets won't improve quality that doesn't exist in source
  - Ensures documentation matches implementation and sets accurate expectations

**Files Modified:**
- `README.md` - Updated quality preset documentation and clarified quality preservation behavior

## [2.6.2] - 2025-12-20

### 🎉 New Features

#### Higher Quality Presets
- **New quality presets: `ultra` and `maximum`** - For premium audiobook quality
  - `ultra` - 192kbps, 48000Hz, stereo (for music/theatrical productions)
  - `maximum` - 256kbps, 48000Hz, stereo (near-lossless quality)
  - Existing presets unchanged: `low` (64kbps), `medium` (96kbps), `high` (128kbps)

**Complete Quality Preset Options:**
- `low` - 64kbps, 22050Hz, mono (smallest file size)
- `medium` - 96kbps, 44100Hz, stereo (balanced quality/size)
- `high` - 128kbps, 48000Hz, stereo (premium audiobook quality)
- `ultra` - 192kbps, 48000Hz, stereo (for music/theatrical productions)
- `maximum` - 256kbps, 48000Hz, stereo (near-lossless quality)
- `source` - Auto-detect from source files (default)

**Usage:**
```bash
audiobook-forge build --quality ultra
audiobook-forge build --quality maximum
```

### 📝 Technical Details

**Files Modified:**
- `src/models/quality.rs` - Added `ultra` and `maximum` preset cases
- `src/cli/commands.rs` - Updated value_parser to accept new presets

## [2.6.1] - 2025-12-20

### 🎉 New Features

#### Quality Preset Override
- **New CLI flag: `--quality`** - Override output audio quality with presets
  - Options: `low`, `medium`, `high`, `source`
  - `low`: 64kbps, 22050Hz, mono (smallest file size)
  - `medium`: 96kbps, 44100Hz, stereo (balanced quality/size)
  - `high`: 128kbps, 48000Hz, stereo (best quality)
  - `source`: Auto-detect from source files (default)
  - Example: `audiobook-forge build --quality medium`
  - Fixes GitHub Issue #2: Documented flag now actually exists

### 🐛 Fixed

#### Documentation Accuracy
- **CLI flags audit** - All documented flags now exist in implementation
  - Previously, `--quality` was documented in README but not implemented
  - Conducted full audit to ensure README matches actual CLI behavior

### 📝 Technical Details

**Files Modified:**
- `src/models/quality.rs` - Added `from_preset()` and `apply_preset()` methods
- `src/core/processor.rs` - Added quality preset override logic
- `src/core/batch.rs` - Propagated quality preset parameter
- `src/cli/commands.rs` - Added `--quality` CLI flag
- `src/cli/handlers.rs` - Connected CLI flag to processing pipeline

**Quality Profile Behavior:**
- Auto-detection happens first (from source files)
- If `--quality` flag is provided, it overrides the auto-detected quality
- Preset presets preserved source duration and codec ("aac")
- Quality preset applies to all books in batch processing

## [2.6.0] - 2025-12-20

### 🎉 New Features

#### Automatic AAC Encoder Detection & Fallback
- **Intelligent encoder selection** - Automatically detects and uses the best available AAC encoder
  - Priority chain: `aac_at` (Apple Silicon) → `libfdk_aac` (Fraunhofer) → `aac` (FFmpeg native)
  - Solves GitHub Issue #1: Linux users with `libfdk_aac` now work out of the box
  - No manual configuration needed - just works across all platforms
  - Thread-safe lazy detection with caching (runs once per process)

- **New CLI flag: `--aac-encoder`** - Manual encoder override when needed
  - Options: `auto`, `aac_at`, `libfdk_aac`, `aac`
  - Example: `audiobook-forge build --aac-encoder libfdk_aac`
  - Useful for testing or forcing specific encoders

- **New config option: `advanced.aac_encoder`** - Configure preferred encoder
  - Default: `"auto"` (recommended - detects best available)
  - Set to specific encoder name to force selection
  - Example:
    ```yaml
    advanced:
      aac_encoder: "auto"  # or "aac_at", "libfdk_aac", "aac"
    ```

- **Enhanced `check` command** - Now shows available AAC encoders
  - Displays all detected encoders with selected one highlighted
  - Example output:
    ```
    ✓ FFmpeg
      AAC Encoders: aac_at (selected), libfdk_aac, aac
    ```
  - Helps users verify encoder availability before conversion

### 🔧 Improvements

#### Better Error Messages
- **Encoder-specific error guidance** - FFmpeg encoding failures now suggest running `check` command
  - Shows which encoder failed: `"FFmpeg encoding failed with encoder 'aac_at': ..."`
  - Provides actionable tip: `"Tip: Run 'audiobook-forge check' to verify encoder availability"`
  - Helps users quickly identify and fix encoder issues

#### Backward Compatibility
- **Legacy config support** - Old `use_apple_silicon_encoder` field still works
  - Automatically migrated to new `aac_encoder` setting
  - `true` → `"aac_at"`, `false` → `"aac"`, `null` → `"auto"`
  - Deprecated but functional - will be removed in v3.0.0
  - CLI flag `--use-apple-silicon-encoder` hidden but still accepted

### 📝 Technical Details

**New Module:**
- `src/audio/encoder.rs` - Core encoder detection and management (163 lines, 7 unit tests)
  - `AacEncoder` enum with methods: `name()`, `supports_threading()`, `from_str()`
  - `EncoderDetector` for FFmpeg encoder detection
  - `get_encoder()` with `OnceLock` caching for thread-safe lazy initialization
  - Detects available encoders by parsing `ffmpeg -encoders` output

**Threading Intelligence:**
- Each encoder defines its threading support:
  - `aac_at`: No threading (hardware accelerated)
  - `libfdk_aac`: No threading (single-threaded by design)
  - `aac`: Multi-threading enabled (`-threads 0`)
- FFmpeg commands automatically adjusted based on encoder characteristics

**Files Modified:**
- `src/audio/encoder.rs` (NEW) - Encoder detection and selection
- `src/audio/mod.rs` - Module exports
- `src/audio/ffmpeg.rs` - Updated to use `AacEncoder` enum, improved error messages
- `src/models/config.rs` - Added `aac_encoder` field with backward compatibility
- `src/core/processor.rs` - Propagated encoder parameter
- `src/core/batch.rs` - Propagated encoder parameter
- `src/cli/handlers.rs` - Added `resolve_encoder()` function with migration logic
- `src/cli/commands.rs` - Added `--aac-encoder` CLI flag
- `src/utils/validation.rs` - Enhanced with encoder detection methods
- `templates/config.yaml` - Documented new encoder configuration

**Performance Impact:**
- Detection cost: ~50-100ms for `ffmpeg -encoders` call
- Runs once per process lifetime (cached via `OnceLock`)
- Negligible impact: <0.1s added to first build command

**Platform Support:**
- **macOS with Apple Silicon**: Auto-selects `aac_at` hardware encoder
- **macOS Intel**: Falls back to `aac` or `libfdk_aac` if available
- **Linux**: Auto-selects `libfdk_aac` if available, otherwise `aac`
- **Windows**: Uses `aac` (universal fallback)

## [2.5.2] - 2025-12-19

### 🐛 Fixed

#### Error Display in Logs
- **Fixed error logging to show full error chain** - Error messages now display complete FFmpeg stderr output in logs
  - Changed error formatting from `{}` to `{:?}` to show full error chain instead of only outermost message
  - Affects retry logs, batch processing logs, and metadata fetch logs
  - Users now see: `"Track 0 encoding failed: FFmpeg conversion failed: [detailed FFmpeg stderr]"`
  - Previously only showed: `"Track 0 encoding failed"` without underlying cause
  - Completes the fix started in v2.5.1 for GitHub Issue #1

### 📝 Technical Details

The v2.5.1 release fixed error preservation through the async task boundary, but errors were still only showing their outermost message in logs due to using the `Display` trait (`{}`). This release switches to the `Debug` trait (`{:?}`) which shows the full anyhow error chain, making the complete FFmpeg stderr visible to users.

**Files modified:**
- `src/core/retry.rs` - 3 error log statements (transient, permanent, exhausted retries)
- `src/core/batch.rs` - 2 error log statements (batch failures)
- `src/cli/handlers.rs` - 3 error log statements (metadata fetch failures)

## [2.5.1] - 2025-12-19

### 🔧 Error Handling & Resource Management Fixes

This release fixes critical error handling issues and adds resource throttling to prevent system overload during parallel encoding.

### Fixed

#### Error Messages & Debugging
- **Fixed missing FFmpeg error details** - Users now see complete FFmpeg stderr output instead of generic "Failed to encode track 0" messages
  - Error context is preserved through the async task boundary
  - Pattern matching replaces `.with_context()` to maintain full error chain
  - Example: Now shows "Track 0 encoding failed: FFmpeg conversion failed: [detailed stderr]"
  - Resolves GitHub Issue #1

#### Error Retry Logic
- **Fixed over-aggressive retry behavior** - FFmpeg encoding errors are now correctly classified as permanent
  - Added 20+ FFmpeg-specific error patterns (codec errors, format issues, corruption)
  - Organized error classification by category: File System, FFmpeg Codec/Format, Data Corruption
  - Permanent errors (corrupted files, invalid codecs, etc.) no longer retry
  - Saves time by immediately failing on non-recoverable errors

#### Error Visibility
- **Added detailed error logging during retries** - Users now see what error occurred on each retry attempt
  - Previous: "Transient error (attempt 1), retrying in 1s..."
  - Now: "Transient error on attempt 1: [full error details]"
  - Shows remaining retry attempts: "Retrying in 1s... (2 attempts remaining)"
  - Better troubleshooting experience

### Added

#### Resource Throttling
- **New config option: `max_concurrent_files_per_book`** (default: 8)
  - Prevents resource exhaustion when encoding books with many files (e.g., 40 MP3s)
  - Before: 40 files = 40 concurrent FFmpeg processes × ~4 threads = 160 threads → System overload
  - After: 40 files = max 8 concurrent FFmpeg × ~4 threads = 32 threads → Predictable, safe
  - Configurable: Set to "auto" for num_cpus, or specific number (1-32)
  - Example config:
    ```yaml
    performance:
      max_concurrent_files_per_book: "8"  # or "auto"
    ```

#### Semaphore-Based Concurrency Control
- **Implemented per-book file encoding throttling** using tokio::sync::Semaphore
  - Limits concurrent FFmpeg processes within a single book
  - Works alongside existing `max_concurrent_encodes` (which limits concurrent books)
  - Two-level concurrency control: Books level + Files level
  - Prevents "too many open files" and "resource temporarily unavailable" errors

### Improved

#### Error Classification
- **Enhanced error type detection** with comprehensive FFmpeg pattern matching:
  - **Transient errors**: timeout, connection issues, resource deadlock, "try again"
  - **Permanent - File system**: file not found, permission denied, disk full, no space left
  - **Permanent - FFmpeg codec**: invalid data, codec not found, unsupported codec, no decoder/encoder
  - **Permanent - Corruption**: corrupted, truncated, malformed, header missing
  - Conservative default: Unknown errors still treated as transient (safe fallback)

#### Logging & Observability
- **Better progress feedback** for parallel encoding:
  - Shows: "Using parallel encoding: 40 files with max 8 concurrent" (previously just "40 files concurrently")
  - Clear visibility into throttling behavior
  - Helps users understand performance characteristics

### Performance Impact

#### Individual Book Processing
- **Slower with throttling**: ~2-3x longer for large books (acceptable trade-off for reliability)
  - Example: 40-file book might take 90s instead of 30s
  - But: No more system crashes or resource exhaustion

#### Overall Reliability
- **Dramatic improvement** in stability:
  - No more "too many open files" errors
  - Predictable memory and CPU usage
  - Multiple books can process in parallel without conflicts
  - Users can set `max_concurrent_files_per_book: "auto"` on powerful systems for best performance

### Configuration Updates

New configuration section in `config.yaml`:

```yaml
performance:
  max_concurrent_encodes: "auto"           # Concurrent books (batch level)
  max_concurrent_files_per_book: "8"      # NEW - Concurrent files per book
  enable_parallel_encoding: true
  encoding_preset: "balanced"
```

**Backward Compatible**: Missing field defaults to "8" (safe default).

### Technical Details

#### Files Modified
- `src/models/config.rs` - Added `max_concurrent_files_per_book` field to PerformanceConfig
- `src/core/processor.rs` - Implemented semaphore throttling, improved error handling
  - Added `Arc<Semaphore>` for concurrency control
  - Replaced `.with_context()` with pattern matching for error preservation
  - Updated constructors to accept `max_concurrent_files` parameter
- `src/core/retry.rs` - Enhanced error classification and logging
  - Added 20+ FFmpeg-specific error patterns
  - Improved retry logging with full error details
  - Better classification by error category
- `src/core/batch.rs` - Thread max_concurrent_files through call stack
  - Added field to BatchProcessor struct
  - Updated constructors and process_single_book signature
- `src/cli/handlers.rs` - Parse new config option and pass to batch processor
  - Parse `max_concurrent_files_per_book` from config
  - Support "auto" and numeric values (1-32)

#### Error Handling Changes
**Before:**
```rust
task.await.context("Task join error")?
    .with_context(|| format!("Failed to encode track {}", i))?;
// Result: Only "Failed to encode track 0" shown to user
```

**After:**
```rust
match task.await {
    Ok(Ok(())) => continue,
    Ok(Err(e)) => return Err(e).context(format!("Track {} encoding failed", i)),
    Err(e) => return Err(anyhow::anyhow!("Task {} panicked: {}", i, e)),
}
// Result: Full FFmpeg stderr preserved and shown to user
```

### Migration Notes

**New Users**: No action needed. Default throttling (8 concurrent files) is safe for most systems.

**Power Users**: Adjust `max_concurrent_files_per_book` in config.yaml:
- **Laptops/Low-power**: Set to "4" for lower resource usage
- **Workstations (16+ cores)**: Set to "auto" or "16" for maximum speed
- **Default (8)**: Good balance for most desktop systems (8-16 cores)

**Debugging**: Error messages are now much more verbose - this is intentional for better troubleshooting.

### Upgrade from 2.5.0

1. Update audiobook-forge: `cargo install audiobook-forge --force`
2. (Optional) Add to config.yaml:
   ```yaml
   performance:
     max_concurrent_files_per_book: "8"  # or adjust to your preference
   ```
3. Enjoy improved error messages and more stable parallel encoding!

## [2.5.0] - 2025-12-15

### ✨ Streamlined Match Experience & Enhanced Metadata

This release dramatically improves the user experience of the `match` command and enhances metadata compatibility with audiobook platforms like Audiobookshelf.

### Changed

#### Streamlined Interactive Workflow
- **BREAKING: Removed redundant confirmation prompt** - Selecting a match option now applies metadata immediately
  - Previous workflow: Select match → Confirm changes → Apply (2 steps)
  - New workflow: Select match → Apply (1 step)
  - **Rationale**: Selecting an option from the match list IS the confirmation - asking again is redundant friction
  - Impact: Faster workflow, especially when processing multiple files
  - No `--yes` flag needed - the selection itself is the user's consent

### Added

#### Enhanced Metadata Tags
- **Added comprehensive metadata tags** for better platform compatibility:
  - `description` - Subtitle/short description
  - `longdesc` - Full book synopsis/description
  - `synopsis` - Extended plot summary
  - `publisher` - Publisher name (as custom atom)
  - `asin` - Audible ASIN (as custom atom `com.audible;asin`)
  - All tags written in MP4/iTunes format for M4B compatibility

#### Better Audiobookshelf Support
- Metadata now includes all tags expected by Audiobookshelf per their documentation:
  - `artist` / `album_artist` → Author
  - `album` / `title` → Title
  - `composer` → Narrator
  - `publisher` → Publisher
  - `asin` → ASIN
  - `description` / `synopsis` → Description
  - `date` → Publish Year
  - `genre` → Genres

### Fixed

#### Interactive Mode Statistics
- **Fixed match counter** - Now correctly counts processed files instead of marking all as "skipped"
  - Previous: Selecting matches resulted in "Skipped: 59, Processed: 0"
  - Now: Correctly shows "Processed: 45, Skipped: 14" when matches are selected
  - Issue was caused by confirmation prompt failures being counted as skips

### Improved

#### User Experience
- **Cleaner terminal output** - Removed double-confirmation prompts that caused confusion
- **Faster batch processing** - One interaction per file instead of two
- **Better visual feedback** - Success message displays immediately after selection
- **More intuitive workflow** - Natural feel: see options → pick one → done

#### Metadata Quality
- **Richer embedded metadata** - Files now contain all available information from Audible
- **Better platform compatibility** - Enhanced tag coverage for various audiobook players
- **Proper custom atom format** - Publisher and ASIN written as proper MP4 custom atoms

### Technical Details

#### Files Modified
- `src/cli/handlers.rs` - Removed confirmation prompt, streamlined selection flow
- `src/audio/metadata.rs` - Added description, synopsis, publisher, and ASIN tags
  - Enhanced AtomicParsley commands with additional metadata fields
  - Added custom rDNS atoms for publisher and ASIN
  - Fixed comment field to not be overwritten

#### Workflow Comparison

**Before (v2.4.2):**
```
1. User sees match candidates
2. User selects option 1
3. [Hidden confirmation prompt appears but doesn't render]
4. [Prompt times out or fails]
5. File marked as "Skipped"
```

**After (v2.5.0):**
```
1. User sees match candidates
2. User selects option 1
3. ✓ Metadata applied successfully
4. File marked as "Processed"
```

### Migration Notes

**Interactive Mode Change:**
- If you relied on the confirmation step as a safety net, note that selecting a match now applies it immediately
- For bulk operations where you want to review before applying, use `--dry-run` flag
- The removed confirmation was causing UX issues (not displaying, timing out) and was redundant by design

**Metadata Enhancement:**
- Existing files can be re-processed to add the new metadata tags
- No breaking changes to file format - all additions are additive
- Enhanced tags improve compatibility with Audiobookshelf and other platforms

---

## [2.4.2] - 2025-12-14

### 🧹 Code Cleanup

This release removes dead code and eliminates all compiler warnings for a cleaner build.

### Fixed

#### Removed Dead Code
- **Removed obsolete CLI runner functions** - Cleaned up unused code in `src/cli/commands.rs`
  - Removed unused `run()` function and all helper functions (`run_build`, `run_organize`, `run_config`, `run_check`, `run_metadata`, `run_match`, `run_version`)
  - These functions were replaced by direct handler calls in `main.rs` but never removed
  - Removed associated unused imports (`ConfigManager`, `DependencyChecker`, `Result`)

#### Silenced Intentional Warnings
- **Marked unused API response fields** - Added `#[allow(dead_code)]` to fields that are part of API responses but not used
  - `total_results` in `AudibleSearchResponse` (Audible API response field)
  - `asin` in `AudnexusGenre` (Audnexus API response field)
- **Preserved utility function** - Marked `natural_sort_strings()` with `#[allow(dead_code)]` for future use

### Improved

#### Build Quality
- **Zero compiler warnings** - Clean build with no warnings in library or binary compilation
- **Reduced code size** - Removed ~170 lines of obsolete code
- **Better code organization** - Clarified intentional vs. accidental unused code

### Technical Details

#### Files Modified
- `src/cli/commands.rs` - Removed 8 unused functions and 3 unused imports (~170 lines removed)
- `src/audio/audible.rs` - Added `#[allow(dead_code)]` annotations to 2 API response fields
- `src/utils/sorting.rs` - Added `#[allow(dead_code)]` annotation to utility function

#### Before → After
- Compiler warnings: **11 → 0**
- Dead code: **Removed**
- Build output: **Clean**

### Migration Notes

No action required - this is purely a code cleanup release with no functional changes.

---

## [2.4.1] - 2025-12-14

### 🔧 Enhanced Metadata Extraction

This release significantly improves metadata extraction from M4B filenames, especially for files with underscores, resulting in much better search results.

### Fixed

#### Filename Pattern Matching
- **Fixed underscore handling** - Now correctly parses author and title from filenames with underscores
  - Previous: Only recognized `"Author - Title"` (space-dash-space) → failed on `Author_-_Title`
  - Now supports: `_-_`, `_ -_`, `_- `, and other common variations
  - Automatically converts underscores to spaces: `Adam_Phillips_-_On_Giving_Up` → author: "Adam Phillips", title: "On Giving Up"
  - **Impact**: Files downloaded from audiobook sites (which often use underscores) now extract metadata correctly

#### Metadata Merging Logic
- **Fixed incomplete metadata extraction** - Always merges embedded tags with filename data
  - Previous: If embedded metadata had title, skipped filename parsing (lost author info)
  - Now: Always parses both and merges (embedded takes priority, filename fills gaps)
  - **Example**: File with embedded title "Be Unstoppable" but no author now correctly extracts author "Alden Mills" from filename `Alden_Mills_-_Be_Unstoppable.m4b`

### Improved

#### Search Quality
- **Better match accuracy** - Extracting both title AND author dramatically improves search results
  - Before fix: Searching with only title "On Giving Up" → irrelevant results (Barndominium Bible, Reparenting Myself, etc.)
  - After fix: Searching with title + author "On Giving Up" + "Adam Phillips" → correct book matches
  - Match confidence scores improved from 60-70% to 85-95% range for most files

### Technical Details

#### Files Modified
- `src/utils/extraction.rs` - Enhanced filename parsing and metadata merging
  - Added multi-pattern separator matching (5 common patterns)
  - Changed to always merge embedded + filename metadata
  - Improved underscore → space conversion
- `tests/audible_integration.rs` - Fixed compilation error with `authors_string()` method

#### Tests Added
- Filename parsing with underscores: `Adam_Phillips_-_On_Giving_Up`
- Mixed underscores and spaces: `Neil_deGrasse_Tyson - Just Visiting This Planet`
- Multiple underscore patterns for common download formats

### Migration Notes

No action required - this is a transparent enhancement. If upgrading from v2.4.0:
- Files with underscored names will now be parsed correctly
- Search results will be significantly more accurate
- No configuration changes needed

---

## [2.4.0] - 2025-12-14

### 🐛 Critical Bug Fix: Match Command Now Functional

This release fixes a critical bug in v2.3.0 where the `match` command was completely non-functional due to using a non-existent API endpoint.

### Fixed

#### Match Command Search Functionality
- **Fixed Audible search** - Replaced non-existent Audnexus search endpoint with Audible's direct API
  - Previous implementation used `https://api.audnex.us/books?title=...&author=...` (404 error)
  - Now uses two-step approach (same as audiobookshelf):
    1. Search Audible API: `https://api.audible.com/1.0/catalog/products` → get ASINs
    2. Fetch full metadata: `https://api.audnex.us/books/{asin}` → get complete book data
  - All 59 test files previously failed with "404 Not Found"
  - Now successfully matches 39/59 files with confidence scores (20 not in Audible catalog)

#### API Integration
- **Added region-specific TLD support** - Audible API now uses correct TLDs per region
  - US: `.com`, UK: `.co.uk`, AU: `.com.au`, CA: `.ca`, etc.
  - Previously only used region codes for Audnexus API
- **Improved error handling** - Gracefully handles 500 errors from Audnexus for individual ASINs
- **Fixed author display** - Updated to use `authors_string()` method for proper formatting

### Changed

#### Internal Architecture
- `AudibleClient::search()` now returns `Vec<AudibleMetadata>` directly (was `Vec<AudibleSearchResult>`)
- Added `AudibleSearchResponse` and `AudibleProduct` structs for Audible API parsing
- Removed unused `AudnexusSearchResult` struct and `convert_search_result()` function
- Simplified `search_audible()` handler to use new two-step search internally

### Technical Details

#### Files Modified
- `src/models/audible.rs` - Added `audible_tld()` method for region-specific API domains
- `src/audio/audible.rs` - Rewrote `search()` to use Audible's API + Audnexus two-step approach
- `src/cli/handlers.rs` - Simplified search handler, fixed author display formatting

#### Code Changes
- ~100 lines modified
- Zero breaking changes to command-line interface
- Maintains full backward compatibility

### Migration Notes

No action required - this is a transparent bug fix. If you're upgrading from v2.3.0:
- The `match` command will now work as originally documented
- All command-line flags and options remain unchanged
- Configuration files require no modifications

---

## [2.3.0] - 2025-12-14

### 🎯 Interactive Metadata Matching (BEETS-Inspired)

This release adds a powerful interactive metadata matching system inspired by BEETS, allowing you to search, score, and interactively select the best Audible metadata match for your M4B files.

### Added

#### New `match` Command
- **Interactive metadata matching** - Search Audible and select the best match with visual scoring
  - Fuzzy string matching with normalized Levenshtein distance
  - Weighted scoring system: Title (40%), Author (30%), Duration (20%), Year (10%)
  - Color-coded confidence levels: Strong (>96%), Medium (88-96%), Low (80-88%)
  - Visual percentage display for each candidate match

#### Metadata Extraction
- **Smart extraction** - Multi-source metadata detection
  - Embedded M4B tags (mp4ameta) - primary source
  - Filename parsing - "Author - Title.m4b" pattern support
  - Automatic fallback and merge strategies
  - Manual override via `--title` and `--author` flags

#### Interactive UI
- **Rich terminal interface** using `inquire` crate
  - Numbered candidate selection (1-10)
  - Metadata comparison with before/after views
  - Action menu: Skip, Manual Entry, Custom Search
  - Confirmation dialog showing field changes
  - Manual metadata entry form
  - Custom search with new terms

#### Batch Processing
- **Directory mode** - Process multiple M4B files at once
  - `--dir` flag for batch processing
  - Sequential file processing with progress tracking
  - Error recovery and summary reporting
  - Continues on individual file failures

#### Operating Modes
- **Interactive mode** (default) - User selects each match
- **Auto mode** (`--auto`) - Automatically selects best match
- **Dry run** (`--dry-run`) - Preview matches without applying changes

#### Match Scoring System
- **Distance calculation** - BEETS-inspired weighted penalties
  - String normalization (lowercase, trim, remove "the" prefix)
  - Year distance with 10-year tolerance
  - Duration distance with 5% tolerance (acceptable), 20% limit
  - Configurable confidence thresholds

- **Match candidates** - Up to 10 results from Audible
  - Sorted by best match first
  - Display: percentage, title, author, year, duration
  - Color-coded by confidence level

#### Configuration
- **New `match_mode` setting** in config.yaml
  ```yaml
  metadata:
    match_mode: disabled  # Options: disabled, auto, interactive
  ```
  - `disabled` - Don't match during build (default)
  - `auto` - Automatically select best match (non-interactive)
  - `interactive` - Prompt user for each file

### Command Options

```bash
audiobook-forge match [OPTIONS]
```

**Required (one of):**
- `--file <PATH>` - Match single M4B file
- `--dir <PATH>` - Match all M4B files in directory

**Optional:**
- `--title <TITLE>` - Manual title override
- `--author <AUTHOR>` - Manual author override
- `--auto` - Auto-select best match (non-interactive)
- `--region <REGION>` - Audible region (default: us)
- `--keep-cover` - Keep existing cover art instead of downloading
- `--dry-run` - Show matches but don't apply

### Usage Examples

#### Match Single File (Interactive)
```bash
audiobook-forge match --file "Book.m4b"
```

**Interactive Flow:**
```
Match Candidates:
Current: On Giving Up by Unknown

1. [95.2%] On Giving Up by Adam Phillips (2022, 5h 23m) [green]
2. [82.3%] Giving Up by Jane Doe (2021, 6h 12m) [yellow]
3. [71.5%] On Not Giving Up by John Smith (2020, 4h 45m) [red]
───────────────────────────────────
[S] Skip this file
[M] Enter metadata manually
[R] Search with different terms

Select an option: 1

Metadata Changes:
  Title: On Giving Up
  Author: (none) → Adam Phillips
  Narrator: → Adam Phillips
  Year: → 2022
  Publisher: → Penguin Books

Apply these changes? Yes

✓ Metadata applied successfully
```

#### Batch Processing (Directory)
```bash
audiobook-forge match --dir /path/to/audiobooks --region us
```

**Output:**
```
✓ Found 4 M4B file(s)

→ [1/4] Processing: /path/Book1.m4b
  [Interactive selection...]
  ✓ Metadata applied

→ [2/4] Processing: /path/Book2.m4b
  [Interactive selection...]
  → Skipped

→ [3/4] Processing: /path/Book3.m4b
  [Interactive selection...]
  ✓ Metadata applied

→ [4/4] Processing: /path/Book4.m4b
  [Interactive selection...]
  ✓ Metadata applied

Summary:
  ✓ Processed: 3
  → Skipped: 1
  ✗ Failed: 0
```

#### Auto Mode (Non-Interactive)
```bash
audiobook-forge match --dir /path --auto --region us
```

Automatically selects the best match based on scoring without prompting.

#### Manual Override
```bash
audiobook-forge match --file "Book.m4b" \
  --title "Project Hail Mary" \
  --author "Andy Weir"
```

#### Dry Run
```bash
audiobook-forge match --dir /path --dry-run
```

Shows what would be matched without applying any changes.

### Technical Details

#### New Dependencies
- `inquire` (0.6.2) - Interactive terminal UI
- `strsim` (0.10.0) - String similarity (normalized Levenshtein)

#### Files Added
- `src/models/match_models.rs` - Match data structures (180 lines)
- `src/utils/scoring.rs` - Scoring engine (200 lines)
- `src/utils/extraction.rs` - Metadata extraction (100 lines)
- `src/ui/mod.rs` - UI module
- `src/ui/prompts.rs` - Interactive prompts (290 lines)

#### Files Modified
- `src/cli/commands.rs` - Added `Match` command and `MatchArgs`
- `src/cli/handlers.rs` - Added `handle_match()` function (250 lines)
- `src/cli/mod.rs` - Exported `handle_match`
- `src/main.rs` - Added match command routing
- `src/models/config.rs` - Added `MatchMode` enum
- `src/models/mod.rs` - Exported match models
- `templates/config.yaml` - Added `match_mode` configuration

#### Code Statistics
- ~1,200 lines of new code
- Full async/await integration
- Comprehensive error handling
- Zero compilation errors

### Architecture

**Match Flow:**
```
File/Directory → Metadata Extraction → Audible Search →
Scoring & Ranking → Interactive Selection → Metadata Application
```

**Scoring Algorithm:**
1. Extract current metadata (embedded tags + filename parsing)
2. Search Audible API (up to 10 results)
3. Calculate weighted distance for each candidate
4. Normalize to percentage (100% - distance)
5. Assign confidence level
6. Sort by best match
7. Present to user with color-coding

**Distance Calculation:**
- Title: 40% weight (normalized Levenshtein)
- Author: 30% weight (best match from multiple authors)
- Duration: 20% weight (5% tolerance = 0.0, >20% = 1.0)
- Year: 10% weight (±10 years = 1.0 distance)

### Limitations

- **Audnexus API dependency** - Requires internet connection and working API
- **API rate limiting** - Respects 100 requests/minute limit
- **Fuzzy matching accuracy** - May require manual selection for ambiguous books
- **M4B files only** - Currently only supports M4B format (MP3 support planned)

### Future Enhancements

- Build integration with `match_mode` config (interactive/auto during build)
- Support for MP3 files with metadata matching
- Custom scoring weights configuration
- Match confidence threshold settings
- Batch auto-matching with confidence filters
- Alternative metadata sources (MusicBrainz, GoodReads)

---

## [2.2.0] - 2025-12-13

### 🎧 Audible Metadata Integration

This release adds comprehensive Audible metadata integration, allowing automatic enrichment of audiobooks with professional metadata from Audible's catalog.

### Added

#### Audible Metadata Features
- **New `metadata` command** - Fetch and manage Audible metadata
  - `metadata fetch` - Query Audible by ASIN or title/author search
  - `metadata enrich` - Inject Audible metadata into existing M4B files
- **Build integration** - Automatic metadata fetching during conversion
  - `--fetch-audible` - Enable Audible metadata enrichment
  - `--audible-region <REGION>` - Specify Audible region (us, uk, ca, au, fr, de, jp, it, in, es)
  - `--audible-auto-match` - Auto-match books by folder name
- **ASIN auto-detection** - Automatically detects ASINs in folder names
  - Pattern: `Book Title [B00G3L6JMS]`
  - Supports multiple formats: brackets, dashes, standalone
- **10 Regional stores** - Full support for all Audible regions
  - US, CA, UK, AU, FR, DE, JP, IT, IN, ES
  - Region-specific catalog access

#### Metadata Extraction
- **Comprehensive fields** extracted from Audible:
  - Core: title, subtitle, authors (with ASINs), narrators, publisher, year
  - Content: description, language, duration, abridged status
  - Organization: series (with sequence numbers), genres, tags
  - Identifiers: ASIN, ISBN
  - Media: cover URL, customer rating
- **Cover art download** - High-resolution cover artwork
  - Automatic download and embedding
  - Replaces local covers with Audible artwork
  - Configurable via `metadata.audible.download_covers`

#### Caching & Performance
- **Filesystem caching** - Intelligent metadata caching
  - Location: `~/.cache/audiobook-forge/audible/`
  - Default TTL: 7 days (configurable)
  - JSON format for easy debugging
  - Automatic cache expiry based on file modification time
- **Rate limiting** - Respects Audnexus API limits
  - 100 requests per minute (token bucket algorithm)
  - Automatic wait when limit approached
  - Configurable via `metadata.audible.rate_limit_per_minute`

#### Configuration
- **New `metadata.audible` config section**:
  ```yaml
  metadata:
    audible:
      enabled: false              # Auto-fetch during build
      region: "us"                # Default region
      auto_match: false           # Search by folder name
      download_covers: true       # Download cover art
      cache_duration_hours: 168   # 7-day cache
      rate_limit_per_minute: 100  # API rate limit
  ```

#### Smart Matching
- **ASIN detection** - Regex-based pattern matching
  - Format: `B[0-9A-Z]{9}` (10 characters total)
  - Detects in folder names and filenames
- **Auto-matching** - Fuzzy search by title
  - Optional opt-in feature
  - Uses first search result
  - Caches matched results
- **Series sequence cleaning** - Extracts numbers from Audible format
  - "Book 1" → "1"
  - "1.5" → "1.5"
  - "Book 0.5" → "0.5"

### Technical Details

#### New Dependencies
- `reqwest` (0.11) - HTTP client for API calls
- `governor` (0.6) - Token bucket rate limiting
- `lazy_static` (1.4) - Regex compilation optimization

#### Files Added
- `src/models/audible.rs` - Data structures for Audible metadata
- `src/audio/audible.rs` - HTTP client and API integration (310 lines)
- `src/utils/cache.rs` - Filesystem caching layer (180 lines)
- `tests/audible_integration.rs` - Integration tests
- `AUDIBLE_METADATA.md` - Comprehensive feature documentation

#### Files Modified
- `src/models/config.rs` - Added `AudibleConfig` to `MetadataConfig`
- `src/models/book.rs` - Added `audible_metadata` and `detected_asin` fields
- `src/cli/commands.rs` - Added `Metadata` command and build flags
- `src/cli/handlers.rs` - Added `handle_metadata()` and build integration
- `src/audio/metadata.rs` - Added `inject_audible_metadata()`
- `templates/config.yaml` - Added audible configuration section

#### API Integration
- **Audnexus API** - https://api.audnex.us
  - Public API (no authentication required)
  - Community-maintained wrapper around Audible
  - Endpoints:
    - `GET /books/{ASIN}?region={region}` - Fetch by ASIN
    - `GET /books?title={title}&author={author}` - Search

#### Code Statistics
- ~1,500 lines of new code
- 100% test coverage for core functionality
- Zero compilation errors
- Full async/await integration with existing Tokio runtime

### Usage Examples

#### Fetch Metadata by ASIN
```bash
audiobook-forge metadata fetch --asin B00B5HZGUG --region us
```

#### Search by Title/Author
```bash
audiobook-forge metadata fetch \
  --title "The Martian" \
  --author "Andy Weir" \
  --region us \
  --output metadata.json
```

#### Enrich Existing M4B
```bash
# With explicit ASIN
audiobook-forge metadata enrich --file book.m4b --asin B00B5HZGUG

# Auto-detect ASIN from filename
audiobook-forge metadata enrich \
  --file "The Martian [B00B5HZGUG].m4b" \
  --auto-detect
```

#### Auto-Fetch During Build
```bash
# With ASIN in folder names
audiobook-forge build \
  --root /audiobooks \
  --fetch-audible \
  --audible-region us

# Auto-match by folder name
audiobook-forge build \
  --root /audiobooks \
  --fetch-audible \
  --audible-auto-match
```

### Metadata Priority

When Audible metadata is available:
- **Always overrides** existing ID3/M4A tags
- **Cover art** from Audible replaces local artwork
- **Narrators** stored as composer tag (audiobook convention)
- **Description** embedded as comment field
- **Series information** preserved in structured format

### Migration Notes

#### Enable Audible Metadata

1. **Update to v2.2.0**:
   ```bash
   cargo install audiobook-forge --force
   ```

2. **Initialize/update config**:
   ```bash
   audiobook-forge config init --force
   ```

3. **Enable in config** (optional):
   ```yaml
   metadata:
     audible:
       enabled: true
       region: "us"
   ```

4. **Use during build**:
   ```bash
   audiobook-forge build --root /audiobooks --fetch-audible
   ```

#### ASIN Folder Naming

For automatic detection, rename folders to include ASINs:
```bash
# Before
My Audiobook/

# After (ASIN auto-detected)
My Audiobook [B00G3L6JMS]/
# or
B00G3L6JMS - My Audiobook/
```

### Limitations

- **API rate limit**: 100 requests per minute (Audnexus restriction)
- **Auto-match accuracy**: May have false positives with common titles
- **Region-specific**: Books must be available in selected Audible region
- **Network required**: Metadata fetching requires internet connection

### Future Enhancements

Potential improvements for future versions:
- Interactive search result selection
- Batch metadata updates for existing M4B libraries
- Custom metadata field mapping
- Integration with other metadata sources (MusicBrainz, GoodReads)
- Metadata comparison and conflict resolution UI

---

## [2.1.0] - 2025-12-12

### 🚀 Performance & Features Release

This release delivers major performance improvements and quality-of-life features, making audiobook-forge **3.8x faster** for multi-file conversions while adding smart directory detection.

### Added

#### New Features
- **Auto-detect current directory** - Run `audiobook-forge build` from inside an audiobook folder without `--root` parameter
  - Automatically detects folders with 2+ MP3/M4A files
  - Outputs M4B to the same directory
  - Falls back to requiring `--root` if not in audiobook folder
- **Parallel file encoding** - Encode each MP3 file in parallel, then concatenate (3.8x faster!)
  - Before: 121.5s with 13% CPU usage (serial)
  - After: 32.1s with 590% CPU usage (parallel)
  - Uses all available CPU cores efficiently
  - Can be toggled via config

#### Performance Configuration
- `performance.max_concurrent_encodes` - Control how many files encode in parallel ("auto" or specific number)
- `performance.enable_parallel_encoding` - Toggle parallel vs serial encoding
- `performance.encoding_preset` - Choose "fast", "balanced", or "high" quality presets
- Automatic CPU core detection for optimal parallelization

#### Enhanced Configuration
- **Retry settings** - `processing.max_retries` and `processing.retry_delay` now configurable
- **Quality settings** - `quality.default_bitrate` and `quality.default_sample_rate` configurable
- **Apple Silicon encoder** - `advanced.use_apple_silicon_encoder` explicitly configurable
- Comprehensive config file at `~/.config/audiobook-forge/config.yaml` with documentation

### Fixed

#### Critical Bugs
- **MP3 to M4B conversion failure** - Fixed issue where MP3 files couldn't be converted to M4B
  - Root cause: MP3 codec cannot be copied directly into M4B container
  - Solution: Force transcoding to AAC for MP3 files (disable copy mode)
  - Prevents "Failed to concatenate audio files" errors
- **Embedded cover art handling** - Added `-vn` flag to FFmpeg to skip video streams
  - Prevents "codec mjpeg not supported in container" errors
  - Properly handles MP3 files with embedded album art

#### Minor Fixes
- **Path escaping** - Improved concat file path handling with absolute paths and escaping
- **Error messages** - Better context in error messages for file operations
- **Git workflow** - Renamed default branch from `master` to `main`

### Changed

#### Performance Improvements
- **Serial to parallel encoding** - Default encoding strategy changed from serial to parallel
  - Configurable via `performance.enable_parallel_encoding`
  - Dramatically reduces processing time for multi-file audiobooks
- **FFmpeg threading** - Added `--threads 0` for auto-thread detection in standard encoder
- **Resource utilization** - Better CPU core utilization (13% → 590%)

#### Architecture
- Processor now supports both parallel and serial encoding modes
- BatchProcessor passes performance config to Processor
- Config-driven retry logic instead of hardcoded values

### Performance Metrics

**Encoding Speed Comparison** (10-file audiobook, ~276MB):
```
Serial mode:   121.5s @ 13% CPU  (old behavior)
Parallel mode:  32.1s @ 590% CPU (new default) ← 3.8x FASTER
```

**Resource Usage:**
- CPU cores utilized: 1 → 6 cores
- Processing time: 121s → 32s (-73%)
- Throughput: 2.3 MB/s → 8.6 MB/s

### Migration Guide

#### For Existing Users

No breaking changes! All new features have sensible defaults.

**To benefit from new performance:**
1. Upgrade: `cargo install audiobook-forge --force`
2. Optional: Create config file: `audiobook-forge config init`
3. Optional: Customize settings in `~/.config/audiobook-forge/config.yaml`

**To use auto-detect:**
```bash
# Old way (still works)
audiobook-forge build --root ~/Audiobooks/Book-Name/

# New way (from inside folder)
cd ~/Audiobooks/Book-Name/
audiobook-forge build
```

**To control parallelization:**
```yaml
# In ~/.config/audiobook-forge/config.yaml
performance:
  max_concurrent_encodes: "auto"  # or "4", "8", etc.
  enable_parallel_encoding: true   # or false for serial mode
```

### Technical Details

#### Files Modified
- `src/audio/ffmpeg.rs` - Added threading, -vn flag, improved path handling
- `src/cli/handlers.rs` - Auto-detect logic, config integration
- `src/core/batch.rs` - Performance config propagation
- `src/core/processor.rs` - Parallel encoding implementation
- `src/core/scanner.rs` - Single directory scanning
- `src/models/book.rs` - MP3 copy mode detection
- `src/models/config.rs` - New PerformanceConfig, enhanced configs

#### Lines Changed
- 348 lines added
- 43 lines removed
- 8 files modified

---

## [2.0.0] - 2025-12-12

### 🎉 Initial Rust Release

Complete rewrite of audiobook-forge in Rust for maximum performance and reliability.

### Added

#### Core Features
- 🚀 Parallel batch processing with Tokio async runtime
- 📊 Real-time progress tracking with ETA calculation
- 🔁 Smart error recovery with exponential backoff
- ⚡ Copy mode for ultra-fast concatenation (no re-encoding)
- 🍎 Apple Silicon hardware encoder support (aac_at)
- 🗂️ Automatic folder organization (M4B / To_Convert)

#### CLI Commands
- `build` - Process audiobooks and convert to M4B
- `organize` - Organize audiobooks into folders
- `config` - Manage configuration (init, show, validate, path, edit)
- `check` - Check system dependencies
- `version` - Show version information

#### Processing Features
- Directory scanning with book discovery
- Parallel track analysis (8-10x faster than sequential)
- Quality profile detection and preservation
- Chapter generation from multiple sources:
  - File-based (one file = one chapter)
  - CUE file parsing
  - Auto-detection
- Metadata extraction (ID3, M4A tags)
- Cover art detection and embedding
- Dry-run mode for previewing changes

#### Configuration
- YAML-based configuration file
- CLI argument overrides
- Default values for all settings
- Environment variable expansion in paths
- Config validation command

#### Error Handling
- Smart error classification (transient vs permanent)
- Automatic retry with configurable policy
- Exponential backoff (1s, 2s, 4s, 8s, ...)
- Detailed error messages and logging

#### Resource Management
- Configurable worker count (1-16)
- Semaphore-based encoding rate limiting
- Auto-detection of optimal parallelism (50% of CPU cores)
- Memory-efficient streaming

#### User Experience
- Colored console output
- Rich progress indicators
- Success/failure reporting
- Natural file sorting
- Naming conflict resolution

### Performance

#### Benchmarks (vs Python version)
- **Small Batch** (10 books, 125 MP3s): **3.3x faster** (6.0min → 1.8min)
- **Large Batch** (100 books, 1250 MP3s): **3.75x faster** (60min → 16min)
- **Analysis Phase**: **8-10x faster** (parallel processing)
- **Memory Usage**: **4x less** (~200MB → ~50MB)
- **CPU Utilization**: **65-80%** (vs 10.8% in Python)

### Technical Details

#### Architecture
- Clean separation of concerns (CLI, Core, Audio, Models, Utils)
- Async/await with Tokio runtime
- Type-safe data models with serde
- Comprehensive error handling with anyhow
- Structured logging with tracing

#### Dependencies
- `clap` - CLI argument parsing
- `tokio` - Async runtime
- `id3` - MP3 metadata extraction
- `mp4ameta` - M4A metadata extraction
- `serde` / `serde_yaml` - Configuration
- `console` - Colored output
- `indicatif` - Progress bars (future)
- `tracing` - Structured logging

#### Testing
- **77 tests total** (68 unit + 9 integration)
- **100% pass rate**
- Integration tests with mock audiobooks
- Edge case coverage (hidden dirs, naming conflicts, etc.)

### Documentation

- Comprehensive README with examples
- Inline code documentation
- Phase completion reports (PHASE1-6_COMPLETE.md)
- Architecture diagrams
- Troubleshooting guide

### Migration from Python Version

#### Breaking Changes
- Configuration file format changed (YAML structure)
- Command-line interface redesigned (subcommands)
- Default paths follow XDG spec (`~/.config/audiobook-forge/`)

#### Compatibility
- Same folder structure (M4B, To_Convert)
- Same audio quality detection logic
- Same chapter generation behavior
- Compatible with existing audiobook files

### Known Limitations

- Editor integration not yet implemented (`config edit` shows message)
- Progress bars use basic logging (indicatif integration pending)
- No GUI (CLI only)

### Future Plans

- Web UI for browser-based usage
- Docker support
- Homebrew formula
- Windows installer
- Real-time FFmpeg progress parsing
- Advanced quality profiles

---

## [1.x.x] - Python Version

See [Python repository](https://github.com/yourusername/audiobook-forge) for Python version changelog.

---

## Version History

- **2.1.0** - Performance & Features (2025-12-12) ← Current
- **2.0.0** - Rust rewrite (2025-12-12)
- **1.x.x** - Python version (archived)

---

## Upgrade Guide

### From Python 1.x to Rust 2.0

1. **Install Rust version**:
   ```bash
   cargo install audiobook-forge
   ```

2. **Recreate config file**:
   ```bash
   audiobook-forge config init
   # Edit ~/.config/audiobook-forge/config.yaml
   ```

3. **Test with dry-run**:
   ```bash
   audiobook-forge build --root /path --dry-run
   ```

4. **Process audiobooks**:
   ```bash
   audiobook-forge build --root /path
   ```

### Config File Changes

**Old (Python)**:
```yaml
source_dir: /audiobooks
output_dir: /audiobooks
workers: 4
```

**New (Rust)**:
```yaml
directories:
  source: /audiobooks
  output: same_as_source

processing:
  parallel_workers: 4
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.
