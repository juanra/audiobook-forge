# Audiobook Forge 🎧

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-77%20passing-brightgreen.svg)](tests/)

A blazing-fast CLI tool for converting audiobook directories to M4B format with chapters and metadata. Written in Rust for maximum performance and reliability.

## 📑 Table of Contents

- [Why Audiobook Forge?](#-why-audiobook-forge)
- [Features](#-features)
- [Installation](#-installation)
- [Dependencies](#-dependencies)
- [Usage](#-usage)
  - [Quick Start](#quick-start)
  - [Command Reference](#command-reference)
  - [Usage Examples](#usage-examples)
- [Configuration](#-configuration)
- [Input Directory Structure](#-input-directory-structure)
- [Supported Formats](#-supported-formats)
- [Performance](#-performance)
- [Troubleshooting](#-troubleshooting)
- [Contributing](#-contributing)
- [License](#-license)
- [Acknowledgments](#-acknowledgments)

---

## 🎯 Why Audiobook Forge?

When downloading audiobooks, they often come as **multiple separate MP3 files** — one for each chapter or section. While this works perfectly fine with various audiobook players like AudiobookShelf and others, managing your audiobook library becomes significantly easier when **each audiobook is consolidated into a single file**.

**Audiobook Forge** solves this problem by taking those scattered audio files and merging them into a single **M4B file** (MPEG-4 Audiobook format), which has become the standard for audiobook applications and offers superior portability and management.

### 📚 Single File vs. Multiple Files: The Benefits

**With a single M4B file, you get:**

✅ **Simplified Library Management**
- One file per audiobook instead of dozens or hundreds of individual files
- Easier to organize, rename, move, and backup
- Cleaner directory structure

✅ **Better Metadata & Chapter Support**
- Embedded chapter markers for easy navigation
- Complete metadata (title, author, narrator, cover art) in one place
- Chapter information persists across devices and players

✅ **Improved Portability**
- Transfer entire audiobooks with a single file copy
- Sync seamlessly across devices (phone, tablet, computer)
- No risk of missing files or broken chapter sequences

✅ **Enhanced Playback Experience**
- Resume exactly where you left off, even across different apps
- Navigate chapters with built-in chapter markers
- Cover art displays properly in all compatible players

✅ **Reduced Storage Overhead**
- Eliminates file system overhead from multiple small files
- Efficient compression while preserving audio quality
- Optional copy mode for instant merging without re-encoding

✅ **Universal Compatibility**
- M4B format supported by Apple Books, Audiobookshelf, Plex, and most modern audiobook players
- Works across iOS, Android, macOS, Windows, and Linux
- Industry-standard format for audiobook distribution

---

## ✨ Features

- **📁 Auto-Detect Current Directory**: Run from inside audiobook folders without `--root` parameter
- **⚡ Parallel File Encoding**: Encode files concurrently for **3.8x faster** processing (121s → 32s)
- **🚀 Parallel Book Processing**: Convert multiple audiobooks simultaneously with intelligent resource management
- **🎯 Smart Quality Detection**: Automatically detects and preserves source audio quality
- **📖 Chapter Generation**: Multiple sources (files, CUE sheets, auto-detection)
- **🎨 Metadata Management**: Extracts and enhances metadata from ID3 and M4A tags
- **🎭 Interactive Metadata Matching** (v2.4.1 - Enhanced, v2.4.0 - Fixed, v2.3.0 - Introduced): BEETS-inspired interactive matching system
  - Fuzzy string matching with weighted scoring (Title 40%, Author 30%, Duration 20%, Year 10%)
  - Color-coded confidence levels (Green >96%, Yellow 88-96%, Red 80-88%)
  - Visual percentage display for each candidate
  - Interactive selection with Skip/Manual Entry/Custom Search options
  - Batch processing with progress tracking
  - Auto mode for non-interactive matching
  - **Note**: v2.4.0 fixed search functionality, v2.4.1 enhanced metadata extraction from filenames with underscores
- **🎧 Audible Metadata Integration** (v2.2.0): Fetch comprehensive metadata from Audible's catalog
  - Automatic ASIN detection from folder names
  - 10 regional stores (US, CA, UK, AU, FR, DE, JP, IT, IN, ES)
  - High-quality cover art download
  - Smart caching (7-day default TTL)
  - Rate-limited API integration
- **🖼️ Cover Art**: Automatically detects and embeds cover images
- **🔄 Batch Operations**: Process entire libraries with a single command
- **⚡ Copy Mode**: Ultra-fast concatenation without re-encoding when possible
- **🎯 Smart Encoder Selection**: Automatic AAC encoder detection with fallback (aac_at → libfdk_aac → aac)
- **🔁 Error Recovery**: Automatic retry with configurable settings
- **📊 Progress Tracking**: Real-time progress with ETA calculation
- **🗂️ Auto-Organization**: Organize books into M4B and To_Convert folders
- **⚙️ Configuration**: Comprehensive YAML-based configuration with CLI overrides

---

## 📦 Installation

### Prerequisites

- **Rust 1.75+**: Install from [rustup.rs](https://rustup.rs/)
- **FFmpeg**: Required for audio processing
- **AtomicParsley**: Required for metadata embedding
- **MP4Box** (from GPAC): Required for M4B container creation

### Via Cargo (Recommended)

```bash
cargo install --git https://github.com/juanra/audiobook-forge
```

### From Source

```bash
# Clone the repository
git clone https://github.com/juanra/audiobook-forge
cd audiobook-forge

# Build and install
cargo build --release
cargo install --path .

# Or just build (binary at: target/release/audiobook-forge)
cargo build --release
```

---

## 🔧 Dependencies

Install the required external tools:

### macOS

```bash
brew install ffmpeg atomicparsley gpac
```

### Ubuntu/Debian

```bash
sudo apt update
sudo apt install ffmpeg atomicparsley gpac
```

### Fedora/RHEL

```bash
sudo dnf install ffmpeg atomicparsley gpac
```

### Verify Installation

```bash
audiobook-forge check
```

**Expected output:**

```
→ Checking system dependencies...

  ✓ FFmpeg
  ✓ AtomicParsley
  ✓ MP4Box

✓ All dependencies found
```

---

## 🚀 Usage

### Quick Start

```bash
# Auto-detect: Run from inside an audiobook folder (NEW in v2.1.0!)
cd "/path/to/My Audiobook"
audiobook-forge build

# Or specify the root directory explicitly
audiobook-forge build --root "/path/to/My Audiobook"

# Process multiple audiobooks in parallel
audiobook-forge build --root "/path/to/audiobooks" --parallel 4

# Organize library (move M4B files to M4B/ folder)
audiobook-forge organize --root "/path/to/audiobooks"
```

### Command Reference

#### `build` - Convert audiobooks to M4B

```bash
audiobook-forge build [OPTIONS]
```

**Options:**
- `--root <PATH>` - Root directory containing audiobook(s) (optional; auto-detects current directory if omitted)
- `--parallel <N>` - Number of parallel workers (default: CPU cores / 2)
- `--skip-existing` - Skip audiobooks that already have M4B files (default: true)
- `--quality <PRESET>` - Quality preset: low, medium, high, ultra, maximum, source (default: source)
  - `low` - 64kbps, 22050Hz, mono (smallest file size)
  - `medium` - 96kbps, 44100Hz, stereo (balanced quality/size)
  - `high` - 128kbps, 48000Hz, stereo (premium audiobook quality)
  - `ultra` - 192kbps, 48000Hz, stereo (for music/theatrical productions)
  - `maximum` - 256kbps, 48000Hz, stereo (near-lossless quality)
  - `source` - Auto-detect from source files (default)
- `--output <PATH>` - Output directory (default: same as source)
- `-v, --verbose` - Verbose logging

#### `organize` - Organize audiobook library

```bash
audiobook-forge organize [OPTIONS] --root <PATH>
```

**Options:**
- `--root <PATH>` - Root directory to organize (required)
- `--dry-run` - Show what would be done without making changes
- `-v, --verbose` - Verbose logging

#### `match` - Interactive metadata matching (v2.4.1 - Enhanced, v2.4.0 - Fixed, v2.3.0 - Introduced)

```bash
audiobook-forge match [OPTIONS]
```

**BEETS-inspired interactive matching** - Search Audible and interactively select the best metadata match for M4B files with visual scoring and confidence levels.

> **Note:** v2.4.0 fixed critical search API bug (404 errors). v2.4.1 enhanced metadata extraction to correctly parse filenames with underscores (e.g., `Author_-_Title.m4b`), dramatically improving match accuracy.

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

**Features:**
- 🎯 Fuzzy string matching with weighted scoring (Title: 40%, Author: 30%, Duration: 20%, Year: 10%)
- 🎨 Color-coded confidence levels (Green: >96%, Yellow: 88-96%, Red: 80-88%)
- 📊 Visual percentage display for each candidate
- 🔄 Multiple options: Skip, Manual Entry, Custom Search
- 📝 Before/after metadata comparison
- 🚀 Batch processing with progress tracking

**Examples:**
```bash
# Interactive match single file
audiobook-forge match --file "Book.m4b"

# Batch process directory
audiobook-forge match --dir /path/to/audiobooks

# Auto mode (non-interactive)
audiobook-forge match --dir /path --auto

# Manual override
audiobook-forge match --file "Book.m4b" --title "Title" --author "Author"

# Dry run
audiobook-forge match --dir /path --dry-run
```

#### `metadata` - Fetch and manage Audible metadata (NEW in v2.2.0)

```bash
audiobook-forge metadata <SUBCOMMAND>
```

**Subcommands:**

**`fetch`** - Fetch metadata from Audible
```bash
audiobook-forge metadata fetch [OPTIONS]
```
- `--asin <ASIN>` - Audible ASIN (e.g., B00B5HZGUG)
- `--title <TITLE>` - Search by title
- `--author <AUTHOR>` - Search by author
- `--region <REGION>` - Audible region: us, uk, ca, au, fr, de, jp, it, in, es (default: us)
- `--output <PATH>` - Save metadata to JSON file

**`enrich`** - Enrich M4B file with Audible metadata
```bash
audiobook-forge metadata enrich [OPTIONS]
```
- `--file <PATH>` - M4B file to enrich
- `--asin <ASIN>` - Audible ASIN
- `--auto-detect` - Auto-detect ASIN from filename
- `--region <REGION>` - Audible region (default: us)

**Build Integration:**
- `--fetch-audible` - Enable Audible metadata fetching during build
- `--audible-region <REGION>` - Specify Audible region for build
- `--audible-auto-match` - Auto-match books by folder name

**Examples:**
```bash
# Fetch by ASIN
audiobook-forge metadata fetch --asin B00B5HZGUG

# Search by title/author
audiobook-forge metadata fetch --title "The Martian" --author "Andy Weir"

# Enrich existing M4B
audiobook-forge metadata enrich --file book.m4b --asin B00B5HZGUG

# Auto-fetch during build
audiobook-forge build --root /audiobooks --fetch-audible --audible-region us
```

See [AUDIBLE_METADATA.md](AUDIBLE_METADATA.md) for comprehensive documentation.

#### `config` - Manage configuration

```bash
audiobook-forge config <SUBCOMMAND>
```

**Subcommands:**
- `init` - Create default config file
- `show` - Display current configuration
- `validate` - Validate config file
- `path` - Show config file location
- `edit` - Open config in default editor

#### `check` - Verify dependencies

```bash
audiobook-forge check
```

Checks for FFmpeg, AtomicParsley, and MP4Box installation.

#### `--version` - Show version

```bash
audiobook-forge --version
```

---

### Usage Examples

#### Example 1: Auto-Detect from Current Directory (NEW in v2.1.0!)

**Directory structure:**

```
~/Downloads/My Audiobook/
├── 01 - Introduction.mp3
├── 02 - Chapter One.mp3
├── 03 - Chapter Two.mp3
└── cover.jpg
```

**Command:**

```bash
cd ~/Downloads/My\ Audiobook
audiobook-forge build
```

**Output:**

```
→ Auto-detected audiobook folder: /Users/you/Downloads/My Audiobook
→ Scanning audiobooks in: /Users/you/Downloads/My Audiobook

✓ Found 1 audiobook(s)

→ Analyzing tracks...
✓ Analysis complete

→ Processing 1 audiobook(s)...

  ✓ My Audiobook (32.1s, transcode)

✓ Batch complete: 1 successful, 0 failed
```

**Result:**

```
~/Downloads/My Audiobook/
├── 01 - Introduction.mp3
├── 02 - Chapter One.mp3
├── 03 - Chapter Two.mp3
├── cover.jpg
└── My Audiobook.m4b  ← Created in the same directory!
```

**Note:** Auto-detect requires 2+ audio files (MP3/M4A) in the current directory.

---

#### Example 2: Convert a Single Audiobook (Explicit Path)

**Directory structure:**

```
/audiobooks/The Great Gatsby/
├── 01 - Chapter 1.mp3
├── 02 - Chapter 2.mp3
├── 03 - Chapter 3.mp3
├── cover.jpg
└── info.txt
```

**Command:**

```bash
audiobook-forge build --root "/audiobooks/The Great Gatsby"
```

**Output:**

```
Audiobook Forge v2.1.0

→ Discovering audiobooks...
  Found 1 audiobook to process

→ Processing: The Great Gatsby
  Case: B (MP3 files - requires conversion)
  Tracks: 3 files
  Quality: 128kbps, 44.1kHz, Stereo
  Duration: 4h 23m

  [00:02:15] ████████████████████ 100% | ETA: 0s

  ✓ Created: The Great Gatsby.m4b
  Size: 246 MB
  Time: 2m 15s

✓ Complete: 1 success, 0 failed
```

**Result:**

```
/audiobooks/The Great Gatsby/
├── 01 - Chapter 1.mp3
├── 02 - Chapter 2.mp3
├── 03 - Chapter 3.mp3
├── cover.jpg
├── info.txt
└── The Great Gatsby.m4b  ← New file
```

---

#### Example 3: Batch Convert Multiple Audiobooks

**Directory structure:**

```
/audiobooks/
├── Book 1/
│   ├── chapter1.mp3
│   └── chapter2.mp3
├── Book 2/
│   ├── part1.m4a
│   └── part2.m4a
└── Book 3/
    └── audiobook.m4b  (already converted)
```

**Command:**

```bash
audiobook-forge build --root /audiobooks --parallel 2
```

**Output:**

```
Audiobook Forge v2.1.0

→ Discovering audiobooks...
  Found 3 audiobooks (1 already converted, skipped)

→ Processing 2 audiobooks with 2 workers...

[Book 1] Case B: MP3 → M4B (transcode)
[Book 2] Case C: M4A → M4B (copy mode - fast!)

[Book 1] [00:01:30] ████████████████████ 100%
[Book 2] [00:00:15] ████████████████████ 100%

Results:
  ✓ Book 1.m4b - 156 MB (1m 30s)
  ✓ Book 2.m4b - 203 MB (15s) [copy mode]
  ⊘ Book 3 - Already exists (skipped)

✓ Complete: 2 success, 0 failed, 1 skipped
Total time: 1m 45s
```

---

#### Example 4: Organize Library

**Before:**

```
/audiobooks/
├── Book 1/
│   ├── chapter1.mp3
│   └── Book 1.m4b
├── Book 2/
│   ├── part1.mp3
│   └── Book 2.m4b
└── Book 3/
    ├── 01.mp3
    └── 02.mp3
```

**Command:**

```bash
audiobook-forge organize --root /audiobooks
```

**Output:**

```
→ Organizing audiobook library...

  Moving completed audiobooks to M4B/
  ✓ Book 1.m4b → M4B/Book 1.m4b
  ✓ Book 2.m4b → M4B/Book 2.m4b

  Organizing unconverted books to To_Convert/
  ✓ Book 3/ → To_Convert/Book 3/

✓ Organization complete
  2 M4B files moved
  1 folder moved to To_Convert
```

**After:**

```
/audiobooks/
├── M4B/
│   ├── Book 1.m4b
│   └── Book 2.m4b
└── To_Convert/
    └── Book 3/
        ├── 01.mp3
        └── 02.mp3
```

---

#### Example 5: Fetch Audible Metadata (NEW in v2.2.0)

**Fetch metadata by ASIN:**

```bash
audiobook-forge metadata fetch --asin B00B5HZGUG
```

**Output:**

```
→ Fetching Audible metadata...
  → Looking up ASIN: B00B5HZGUG

============================================================
Title: The Martian
Subtitle: null
Author(s): Andy Weir
Narrator(s): R. C. Bray
Publisher: Podium Publishing
Published: 2013
Duration: 10h 53m
Language: english
Genres: Science Fiction & Fantasy
ASIN: B00B5HZGUG
============================================================
```

**Search by title and author:**

```bash
audiobook-forge metadata fetch --title "Project Hail Mary" --author "Andy Weir" --region us
```

**Enrich existing M4B file:**

```bash
# With explicit ASIN
audiobook-forge metadata enrich --file "The Martian.m4b" --asin B00B5HZGUG

# Auto-detect ASIN from filename (e.g., "The Martian [B00B5HZGUG].m4b")
audiobook-forge metadata enrich --file "The Martian [B00B5HZGUG].m4b" --auto-detect
```

**Auto-fetch during build (recommended workflow):**

```bash
# Rename folders with ASINs for automatic detection
# Format: Book Title [B00G3L6JMS]

cd /audiobooks
mv "The Martian" "The Martian [B00B5HZGUG]"
mv "Project Hail Mary" "Project Hail Mary [B00G3L6JMS]"

# Build with Audible metadata
audiobook-forge build --root /audiobooks --fetch-audible --audible-region us
```

**Output:**

```
→ Scanning audiobooks in: /audiobooks
✓ Found 2 audiobook(s)

→ Analyzing tracks...
✓ Analysis complete

→ Fetching Audible metadata...
  ✓ The Martian (ASIN: B00B5HZGUG)
  ✓ Project Hail Mary (ASIN: B00G3L6JMS)
✓ Fetched metadata for 2/2 books

→ Processing 2 audiobook(s)...
  ✓ The Martian (32.1s, transcode)
  ✓ Project Hail Mary (28.5s, transcode)

✓ Batch complete: 2 successful, 0 failed
```

See [AUDIBLE_METADATA.md](AUDIBLE_METADATA.md) for comprehensive documentation on ASIN detection, regional stores, caching, and troubleshooting.

---

#### Example 6: Configuration Management

**Initialize config:**

```bash
audiobook-forge config init
```

**Output:**

```
✓ Config file created at: /Users/you/.config/audiobook-forge/config.yaml

Edit the file to customize settings, or use:
  audiobook-forge config edit
```

**Show current config:**

```bash
audiobook-forge config show
```

**Output:**

```yaml
directories:
  source: null
  output: same_as_source

processing:
  parallel_workers: 4
  skip_existing: true
  quality_preset: source
  use_copy_mode: true

metadata:
  extract_from_files: true
  prefer_embedded: true
  fallback_to_folder_name: true

chapters:
  generate_from_files: true
  parse_cue_files: true
  auto_chapters: false

output:
  format: m4b
  naming_pattern: "{title}"
  include_metadata: true

performance:
  ffmpeg_threads: 0
  buffer_size: 8192
```

---

## ⚙️ Configuration

Configuration file location: `~/.config/audiobook-forge/config.yaml`

### Create Config File

```bash
# Initialize default configuration
audiobook-forge config init

# Edit configuration
nano ~/.config/audiobook-forge/config.yaml
```

### Key Settings

```yaml
# Performance Settings (NEW in v2.1.0!)
performance:
  max_concurrent_encodes: "auto"  # Parallel file encoding: "auto" or number (1-16)
  enable_parallel_encoding: true  # Enable parallel encoding (3.8x faster!)
  encoding_preset: "balanced"     # Encoding preset: fast, balanced, high

# Processing Settings
processing:
  parallel_workers: 4        # Concurrent audiobooks to process
  skip_existing: true        # Skip if M4B already exists
  max_retries: 3             # Retry attempts for failed operations
  retry_delay: 1             # Delay between retries (seconds)
  keep_temp_files: false     # Keep temporary files for debugging

# Quality Settings
quality:
  chapter_source: "auto"     # Chapter source: auto, files, cue, none
  default_bitrate: "auto"    # Audio bitrate: auto or specific (e.g., "128k")
  default_sample_rate: "auto" # Sample rate: auto or specific (e.g., "44100")
  prefer_stereo: false       # Prefer stereo over mono

# Metadata Settings
metadata:
  extract_from_files: true   # Extract metadata from audio files
  prefer_embedded: true      # Prefer embedded tags over filenames
  fallback_to_folder_name: true  # Use folder name as fallback
  default_language: "en"     # Default language code

  # Interactive Matching Mode (NEW in v2.3.0)
  match_mode: disabled       # Options: disabled, auto, interactive
                             # - disabled: Don't match during build (default)
                             # - auto: Automatically select best match (non-interactive)
                             # - interactive: Prompt user for selection

  # Audible Metadata Integration (v2.2.0)
  audible:
    enabled: false           # Enable automatic fetching during build
    region: "us"             # Audible region: us, ca, uk, au, fr, de, jp, it, in, es
    auto_match: false        # Auto-match books by folder name (may have false positives)
    download_covers: true    # Download and embed cover art from Audible
    cache_duration_hours: 168  # Cache metadata for 7 days (0 = no cache)
    rate_limit_per_minute: 100 # API rate limit (do not exceed 100)

# Advanced Settings
advanced:
  aac_encoder: "auto"  # AAC encoder: "auto", "aac_at", "libfdk_aac", "aac" (auto-detects best available)
```

**Override config with CLI flags:**

```bash
# Override parallel workers
audiobook-forge build --root /path --parallel 8

# Force reprocessing
audiobook-forge build --root /path --force

# Keep temporary files for debugging
audiobook-forge build --root /path --keep-temp
```

---

## 📁 Input Directory Structure

Audiobook Forge automatically detects and processes various directory structures:

### Supported Structures

✅ **Flat structure** (all files in one folder)
```
My Audiobook/
├── 01.mp3
├── 02.mp3
├── 03.mp3
└── cover.jpg
```

✅ **Nested chapters** (subfolders)
```
My Audiobook/
├── Part 1/
│   ├── Chapter 01.mp3
│   └── Chapter 02.mp3
└── Part 2/
    ├── Chapter 03.mp3
    └── Chapter 04.mp3
```

✅ **With CUE file** (single audio + chapters)
```
My Audiobook/
├── audiobook.mp3
├── audiobook.cue
└── cover.png
```

✅ **Mixed formats** (MP3 + M4A)
```
My Audiobook/
├── intro.mp3
├── chapter1.m4a
├── chapter2.m4a
└── outro.mp3
```

### Required Files

- **Audio files**: At least one `.mp3` or `.m4a` file
- **Cover art** (optional): `cover.jpg`, `cover.png`, `folder.jpg`, etc.
- **CUE file** (optional): `*.cue` for chapter information

---

## 🎵 Supported Formats

### Input Audio Formats

- **MP3** (`.mp3`) - MPEG Audio Layer III
- **M4A** (`.m4a`) - MPEG-4 Audio
- **AAC** (`.aac`) - Advanced Audio Coding

### Output Format

- **M4B** (`.m4b`) - MPEG-4 Audiobook with embedded chapters and metadata

### Metadata Sources

- **Audible** (NEW in v2.2.0) - Comprehensive metadata from Audible's catalog via Audnexus API
  - Title, subtitle, authors, narrators, publisher, year
  - Description, language, duration, series information
  - Genres, tags, ISBN, customer ratings
  - High-quality cover artwork
- **ID3 tags** (MP3 files)
- **M4A atoms** (M4A/M4B files)
- **CUE sheets** (`.cue` files)
- **Filenames** (fallback)
- **Folder names** (fallback)

---

## 📊 Performance

### Benchmarks

#### v2.1.0 Performance Improvements

**Parallel File Encoding** (NEW in v2.1.0):

| Mode | Time | CPU Usage | Speedup |
|------|------|-----------|---------|
| Serial encoding (v2.0.0) | 121.5s | 13% | Baseline |
| Parallel encoding (v2.1.0) | 32.1s | 590% | **3.8x faster** 🚀 |

*Test: 10-file audiobook (~276MB) on 8-core CPU*

#### Overall Performance vs Python Version

| Operation | Python Version | Rust v2.0.0 | Rust v2.1.0 | Total Speedup |
|-----------|---------------|-------------|-------------|---------------|
| Startup time | ~500ms | ~10ms | ~10ms | **50x faster** |
| Single book (copy mode) | 45s | 12s | 12s | **3.8x faster** |
| Single book (transcode) | 180s | 65s | 17s | **10.6x faster** 🚀 |
| Batch (10 books, parallel) | 25m | 8m | 2.5m | **10x faster** 🚀 |
| Memory usage | ~250 MB | ~25 MB | ~25 MB | **10x less** |

### Performance Tips

1. **Enable parallel file encoding** (default in v2.1.0): Encodes files concurrently for massive speedup
2. **Use parallel book processing**: `--parallel 4` (or more based on CPU cores)
3. **Enable copy mode**: Automatic when input is already AAC/M4A
4. **Use SSD storage**: Significantly faster I/O for large libraries
5. **Apple Silicon**: Automatic hardware acceleration with `aac_at` encoder
6. **Skip existing**: Use `--skip-existing` for incremental processing
7. **Adjust concurrent encodes**: Set `performance.max_concurrent_encodes` in config to match your CPU cores

---

## 🔧 Troubleshooting

### Common Issues

#### FFmpeg not found

**Error:**
```
✗ FFmpeg not found in PATH
```

**Solution:**
```bash
# macOS
brew install ffmpeg

# Ubuntu/Debian
sudo apt install ffmpeg

# Verify
audiobook-forge check
```

---

#### Permission denied

**Error:**
```
Error: Permission denied (os error 13)
```

**Solution:**
```bash
# Check file permissions
ls -la /path/to/audiobooks

# Fix permissions
chmod -R u+rw /path/to/audiobooks
```

---

#### Out of memory (large libraries)

**Error:**
```
Error: Cannot allocate memory
```

**Solution:**
```bash
# Reduce parallel workers
audiobook-forge build --root /path --parallel 1

# Process in smaller batches
audiobook-forge build --root "/path/Book 1"
audiobook-forge build --root "/path/Book 2"
```

---

#### Quality is worse than original

**Issue:** Output file sounds compressed

**Solution:**
```bash
# First, check your source file quality
ffmpeg -i input.mp3

# The default preserves source quality (no upsampling/downsampling)
audiobook-forge build --root /path --quality source

# Note: Using a higher quality preset won't improve quality
# that doesn't exist in the source files
```

**Important:** The tool preserves whatever quality exists in your source files. If your source is already compressed (e.g., 64kbps), encoding at a higher bitrate won't improve quality - it will just create a larger file with the same audio quality.

---

#### MP3 files not converting (Fixed in v2.1.0)

**Previous Error (v2.0.0):**
```
✗ Failed to concatenate audio files
Could not find tag for codec mp3 in stream #0
```

**Solution:** This issue has been fixed in v2.1.0! The tool now automatically:
- Forces AAC transcoding for MP3 files (MP3 codec cannot be copied into M4B container)
- Skips video streams (embedded cover art) with `-vn` flag
- Uses parallel encoding for faster MP3 to M4B conversion

If you're still on v2.0.0, upgrade to v2.1.0:
```bash
cargo install audiobook-forge --force
```

---

#### Audible Metadata Issues (NEW in v2.2.0)

**Match command fails with 404 errors (FIXED in v2.4.0):**

**Error in v2.3.0:**
```
✗ Error: Search API returned status: 404 Not Found
```

**Solution:** This was a critical bug in v2.3.0 where the match command used a non-existent API endpoint. **Upgrade to v2.4.1** to fix:
```bash
cargo install audiobook-forge --force
```

After upgrading, the match command will work correctly:
```bash
audiobook-forge match --file "Book.m4b"
# or
audiobook-forge match --dir ~/Downloads/m4b/ --auto
```

---

**Poor match results with underscore filenames (FIXED in v2.4.1):**

**Issue:** Files named like `Author_-_Title.m4b` return irrelevant search results.

**Example:**
- File: `Adam_Phillips_-_On_Giving_Up.m4b`
- Search: Only title "On Giving Up", missing author
- Results: Irrelevant books (Barndominium Bible, Reparenting Myself, etc.)

**Solution:** v2.4.1 now correctly parses underscored filenames and extracts both author and title:
```bash
# After upgrading to v2.4.1
audiobook-forge match --file "Adam_Phillips_-_On_Giving_Up.m4b"
# Now searches: title="On Giving Up" + author="Adam Phillips"
# Match accuracy: 85-95% (vs 60-70% before)
```

The fix automatically handles common patterns:
- `Author_-_Title.m4b` → ✅ Works
- `Author - Title.m4b` → ✅ Works
- `Author_ -_Title.m4b` → ✅ Works
- Mixed patterns → ✅ Works

---

**No metadata found:**

**Error:**
```
No results found for search query
```

**Solutions:**
```bash
# Try different search terms (partial titles work better)
audiobook-forge metadata fetch --title "Hail Mary" --author "Weir"

# Use ASIN directly (more reliable)
audiobook-forge metadata fetch --asin B00G3L6JMS

# Try different region
audiobook-forge metadata fetch --title "Book" --region uk

# Check spelling of author/title
```

---

**ASIN not detected:**

**Issue:** Folder has ASIN but not auto-detected

**Solutions:**
```bash
# Ensure ASIN format is correct: B + 9 alphanumeric characters
# Supported formats:
#   ✓ "Book Title [B00G3L6JMS]"
#   ✓ "B00G3L6JMS - Book Title"
#   ✓ "Book - B00G3L6JMS - Author"

# Use explicit ASIN instead
audiobook-forge metadata fetch --asin B00G3L6JMS

# Or use auto-detect on filename
audiobook-forge metadata enrich --file "Book [B00G3L6JMS].m4b" --auto-detect
```

---

**API rate limiting:**

**Issue:** Too many requests

**Solution:**
```yaml
# In config.yaml, reduce rate limit or wait a few minutes
metadata:
  audible:
    rate_limit_per_minute: 50  # Reduce from 100

# Or process in smaller batches
```

---

**Cache issues:**

**Issue:** Stale or corrupted cache data

**Solution:**
```bash
# Clear cache for specific ASIN
rm ~/.cache/audiobook-forge/audible/B00G3L6JMS.json

# Clear entire cache
rm -rf ~/.cache/audiobook-forge/audible/

# Or disable caching temporarily in config.yaml
metadata:
  audible:
    cache_duration_hours: 0  # Disable cache
```

See [AUDIBLE_METADATA.md](AUDIBLE_METADATA.md) for comprehensive troubleshooting and advanced usage.

---

### Getting Help

- **Check logs**: Run with `--verbose` flag for detailed output
- **Verify dependencies**: `audiobook-forge check`
- **Report issues**: [GitHub Issues](https://github.com/juanra/audiobook-forge/issues)
- **Documentation**: See `docs/` folder for detailed guides
- **Audible metadata**: See [AUDIBLE_METADATA.md](AUDIBLE_METADATA.md) for comprehensive documentation

---

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Start for Contributors

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Run linter: `cargo clippy`
6. Format code: `cargo fmt`
7. Commit: `git commit -m "feat: add my feature"`
8. Push: `git push origin feature/my-feature`
9. Open a Pull Request

---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

### MIT License Summary

- ✅ Commercial use
- ✅ Modification
- ✅ Distribution
- ✅ Private use
- ⚠️ Liability and warranty disclaimer

---

## 🙏 Acknowledgments

- **Original Python version**: This Rust rewrite is based on the original Python implementation, delivering 3-4x better performance
- **FFmpeg**: The backbone of audio processing
- **Rust community**: For excellent crates and tooling
- **Contributors**: Thanks to all who have contributed to this project

### Built With

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Tokio](https://tokio.rs/) - Async runtime
- [Clap](https://github.com/clap-rs/clap) - CLI framework
- [FFmpeg](https://ffmpeg.org/) - Audio/video processing
- [AtomicParsley](https://github.com/wez/atomicparsley) - Metadata embedding
- [MP4Box/GPAC](https://github.com/gpac/gpac) - MP4 container tools

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/juanra/audiobook-forge/issues)
- **Discussions**: [GitHub Discussions](https://github.com/juanra/audiobook-forge/discussions)
- **Documentation**: [`docs/`](docs/) folder

---

Made with ❤️ and 🦀 (Rust)
