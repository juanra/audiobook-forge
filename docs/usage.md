# Usage Guide

This guide covers all commands and common usage patterns for Audiobook Forge.

## Quick Start

The fastest way to convert an audiobook:

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

## Command Reference

### `build` - Convert Audiobooks to M4B

Convert one or more audiobooks to M4B format with chapters and metadata.

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
- `--fetch-audible` - Enable Audible metadata fetching during build
- `--audible-region <REGION>` - Specify Audible region for build
- `--audible-auto-match` - Auto-match books by folder name
- `-v, --verbose` - Verbose logging

**Examples:**

```bash
# Auto-detect current directory
cd ~/Downloads/My\ Audiobook
audiobook-forge build

# Process single audiobook
audiobook-forge build --root "/audiobooks/The Great Gatsby"

# Batch process with parallel workers
audiobook-forge build --root /audiobooks --parallel 4

# With Audible metadata
audiobook-forge build --root /audiobooks --fetch-audible --audible-region us

# Force high quality
audiobook-forge build --root /audiobooks --quality high

# Verbose output for debugging
audiobook-forge build --root /audiobooks --verbose
```

---

### `organize` - Organize Audiobook Library

Organize your audiobook library by moving M4B files to an `M4B/` folder and unconverted books to `To_Convert/`.

```bash
audiobook-forge organize [OPTIONS] --root <PATH>
```

**Options:**

- `--root <PATH>` - Root directory to organize (required)
- `--dry-run` - Show what would be done without making changes
- `-v, --verbose` - Verbose logging

**Examples:**

```bash
# Organize library
audiobook-forge organize --root /audiobooks

# Preview what will be done
audiobook-forge organize --root /audiobooks --dry-run

# Organize with verbose output
audiobook-forge organize --root /audiobooks --verbose
```

---

### `match` - Interactive Metadata Matching

BEETS-inspired interactive matching - Search Audible and interactively select the best metadata match for M4B files with visual scoring and confidence levels.

**New in v2.3.0 | Enhanced in v2.4.1 | Fixed in v2.4.0**

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

# Dry run to preview matches
audiobook-forge match --dir /path --dry-run

# Match with specific region
audiobook-forge match --file "Book.m4b" --region uk
```

---

### `metadata` - Fetch and Manage Audible Metadata

Fetch comprehensive metadata from Audible's catalog or enrich existing M4B files.

**New in v2.2.0**

```bash
audiobook-forge metadata <SUBCOMMAND>
```

#### Subcommand: `fetch`

Fetch metadata from Audible by ASIN, title, or author.

```bash
audiobook-forge metadata fetch [OPTIONS]
```

**Options:**

- `--asin <ASIN>` - Audible ASIN (e.g., B00B5HZGUG)
- `--title <TITLE>` - Search by title
- `--author <AUTHOR>` - Search by author
- `--region <REGION>` - Audible region: us, uk, ca, au, fr, de, jp, it, in, es (default: us)
- `--output <PATH>` - Save metadata to JSON file

**Examples:**

```bash
# Fetch by ASIN
audiobook-forge metadata fetch --asin B00B5HZGUG

# Search by title/author
audiobook-forge metadata fetch --title "The Martian" --author "Andy Weir"

# Search with specific region
audiobook-forge metadata fetch --title "Book" --region uk

# Save to file
audiobook-forge metadata fetch --asin B00B5HZGUG --output metadata.json
```

#### Subcommand: `enrich`

Enrich an existing M4B file with Audible metadata.

```bash
audiobook-forge metadata enrich [OPTIONS]
```

**Options:**

- `--file <PATH>` - M4B file to enrich
- `--asin <ASIN>` - Audible ASIN
- `--auto-detect` - Auto-detect ASIN from filename
- `--region <REGION>` - Audible region (default: us)

**Examples:**

```bash
# Enrich with explicit ASIN
audiobook-forge metadata enrich --file "The Martian.m4b" --asin B00B5HZGUG

# Auto-detect ASIN from filename (e.g., "The Martian [B00B5HZGUG].m4b")
audiobook-forge metadata enrich --file "The Martian [B00B5HZGUG].m4b" --auto-detect

# Enrich with UK region metadata
audiobook-forge metadata enrich --file "Book.m4b" --asin B00XXXXX --region uk
```

See [Metadata Guide](metadata.md) for comprehensive Audible documentation.

---

### `config` - Manage Configuration

Manage Audiobook Forge configuration file.

```bash
audiobook-forge config <SUBCOMMAND>
```

**Subcommands:**

- `init` - Create default config file
- `show` - Display current configuration
- `validate` - Validate config file
- `path` - Show config file location
- `edit` - Open config in default editor

**Examples:**

```bash
# Create default config
audiobook-forge config init

# Show current settings
audiobook-forge config show

# Validate config file
audiobook-forge config validate

# Find config location
audiobook-forge config path

# Edit in default editor
audiobook-forge config edit
```

---

### `check` - Verify Dependencies

Check for required external dependencies (FFmpeg, AtomicParsley, MP4Box).

```bash
audiobook-forge check
```

**Output:**

```
→ Checking system dependencies...

  ✓ FFmpeg
  ✓ AtomicParsley
  ✓ MP4Box

✓ All dependencies found
```

---

### `--version` - Show Version

Display the current version of Audiobook Forge.

```bash
audiobook-forge --version
```

---

## Usage Examples

### Example 1: Auto-Detect from Current Directory

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
└── My Audiobook.m4b  ← Created!
```

---

### Example 2: Convert a Single Audiobook

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

---

### Example 3: Batch Convert Multiple Audiobooks

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

### Example 4: Organize Library

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

### Example 5: Fetch Audible Metadata

**Fetch by ASIN:**

```bash
audiobook-forge metadata fetch --asin B00B5HZGUG
```

**Output:**

```
→ Fetching Audible metadata...
  → Looking up ASIN: B00B5HZGUG

============================================================
Title: The Martian
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

**Auto-fetch during build:**

```bash
# Rename folders with ASINs for automatic detection
cd /audiobooks
mv "The Martian" "The Martian [B00B5HZGUG]"
mv "Project Hail Mary" "Project Hail Mary [B00G3L6JMS]"

# Build with Audible metadata
audiobook-forge build --root /audiobooks --fetch-audible
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

---

### Example 6: Configuration Management

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
performance:
  max_concurrent_encodes: "auto"
  enable_parallel_encoding: true

processing:
  parallel_workers: 4
  skip_existing: true

metadata:
  audible:
    enabled: false
    region: "us"
```

---

## Common Workflows

### Workflow 1: Quick Conversion

For quick, one-off conversions:

```bash
cd /path/to/audiobook
audiobook-forge build
```

### Workflow 2: Batch Processing

For processing multiple audiobooks:

```bash
# Process all audiobooks with maximum parallelism
audiobook-forge build --root /audiobooks --parallel 8

# Organize after processing
audiobook-forge organize --root /audiobooks
```

### Workflow 3: Audible-Integrated

For audiobooks with ASIN tracking:

```bash
# 1. Rename folders with ASINs
mv "Book" "Book [B00XXXXX]"

# 2. Build with Audible metadata
audiobook-forge build --root /audiobooks --fetch-audible --audible-region us

# Result: M4B files with complete Audible metadata and cover art
```

### Workflow 4: Quality-Focused

For high-quality archival:

```bash
# Use maximum quality preset
audiobook-forge build --root /audiobooks --quality maximum --parallel 4
```

### Workflow 5: Interactive Matching

For existing M4B files needing metadata:

```bash
# Interactive matching with visual confidence scoring
audiobook-forge match --dir /path/to/m4b/files

# Auto-match for large batches
audiobook-forge match --dir /path/to/m4b/files --auto
```

## Tips and Best Practices

1. **Use auto-detect**: Run `audiobook-forge build` from inside audiobook folders for convenience
2. **Enable parallel processing**: Use `--parallel` to process multiple audiobooks simultaneously
3. **Leverage Audible metadata**: Add ASINs to folder names for automatic metadata enrichment
4. **Check before processing**: Use `--dry-run` with organize command to preview changes
5. **Keep source files**: The tool doesn't delete source files, so you can always reprocess
6. **Use verbose mode for debugging**: Add `--verbose` to see detailed processing logs
7. **Configure once**: Set up `config.yaml` for your preferences to avoid repetitive flags

## Next Steps

- [Configuration Guide](configuration.md) - Customize settings and defaults
- [Metadata Guide](metadata.md) - Comprehensive metadata and Audible integration
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
