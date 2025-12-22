# Metadata Management

Audiobook Forge provides comprehensive metadata management through multiple sources and methods. This guide covers all metadata features including local extraction, cover art management, Audible integration, and interactive matching.

## Table of Contents

- [Overview](#overview)
- [Local Metadata Extraction](#local-metadata-extraction)
- [Cover Art Management](#cover-art-management)
- [Audible Integration](#audible-integration)
- [Interactive Matching](#interactive-matching)
- [Configuration Reference](#configuration-reference)
- [Metadata Priority & Precedence](#metadata-priority--precedence)
- [Troubleshooting](#troubleshooting)

---

## Overview

Audiobook Forge extracts and manages metadata from multiple sources:

1. **Local Sources**:
   - ID3 tags (MP3 files)
   - M4A atoms (M4A/M4B files)
   - CUE sheets for chapter information
   - Filenames and folder names (fallback)
   - Embedded cover art (NEW in v2.8.0)

2. **External Sources**:
   - Audible catalog via Audnexus API
   - Interactive matching with fuzzy search

3. **Features**:
   - Automatic metadata extraction
   - Cover art detection and auto-extraction
   - Audible metadata integration with caching
   - Interactive matching with confidence scoring
   - Configurable priority and fallback behavior

---

## Local Metadata Extraction

Audiobook Forge automatically extracts metadata from audio files.

### ID3 Tags (MP3 Files)

Extracts standard ID3v2 tags:
- Title (TIT2)
- Artist/Author (TPE1)
- Album (TALB)
- Year (TDRC/TYER)
- Genre (TCON)
- Track Number (TRCK)
- Comment (COMM)
- Embedded cover art (APIC frames)

### M4A Atoms (M4A/M4B Files)

Extracts MP4 metadata atoms:
- Title (©nam)
- Artist (©ART)
- Album Artist (aART)
- Album (©alb)
- Year (©day)
- Genre (©gen)
- Track (trkn)
- Comment (©cmt)
- Narrator/Composer (©wrt)
- Embedded artwork

### CUE Sheets

Parses CUE files for:
- Chapter markers
- Track titles
- Performer information
- Album/title metadata

**Supported CUE commands**:
- `PERFORMER` - Artist/author
- `TITLE` - Album/track titles
- `INDEX` - Chapter timing
- `TRACK` - Chapter numbering

### Filename Parsing

When no embedded metadata exists, the tool parses filenames:

**Patterns recognized**:
- `01 - Chapter Title.mp3`
- `Author - Title.mp3`
- `Title (Author).mp3`
- `[Track##] Title.mp3`

**Enhanced in v2.4.1**:
- Correctly handles underscores: `Author_-_Title.m4b`
- Mixed patterns: `Author_ -_Title.m4b`

### Folder Name Fallback

If no other metadata exists, uses folder name as title:
- Sanitizes folder name
- Removes common patterns: `[ASIN]`, `(Year)`, etc.
- Uses as album/title metadata

---

## Cover Art Management

### Cover File Detection

Audiobook Forge searches for cover art files in this order:

**Default filenames** (case-insensitive):
1. `cover.jpg`
2. `folder.jpg`
3. `cover.png`
4. `folder.png`

**Custom filenames**: Configure in `config.yaml`:

```yaml
metadata:
  cover_filenames:
    - "cover.jpg"
    - "folder.jpg"
    - "cover.png"
    - "cover_art.jpg"
    - "front.jpg"
```

### Auto-Extract Embedded Cover Art

**NEW in v2.8.0** - Automatically extracts cover art from audio files as a fallback.

**How it works**:
1. Scanner checks for standalone cover files first
2. If none found, extracts embedded art from **first audio file**
3. Saves as temporary `.extracted_cover.jpg` in book folder
4. Embeds into final M4B file
5. Cleans up temporary file after processing

**Supported formats**:
- **MP3**: APIC frames (ID3v2)
- **M4A/M4B**: MP4 artwork atoms

**Configuration**:

```yaml
metadata:
  auto_extract_cover: true  # Default: enabled
```

**Disable auto-extraction**:

```yaml
metadata:
  auto_extract_cover: false
```

**Benefits**:
- Audiobooks from Audible/iTunes with embedded covers "just work"
- No need to manually extract cover.jpg
- Fallback ensures cover art when possible

**Priority**:
1. Standalone cover files (cover.jpg, etc.)
2. Embedded cover art (auto-extracted)
3. No cover (if neither exists)

---

## Audible Integration

Comprehensive metadata integration with Audible's catalog via the Audnexus API.

### Features

✅ **Complete Metadata** - Fetches title, subtitle, authors, narrators, publisher, year, description, cover art, series info, genres, rating, and more

✅ **10 Regional Stores** - Supports US, CA, UK, AU, FR, DE, JP, IT, IN, and ES Audible regions

✅ **ASIN Auto-Detection** - Automatically detects ASINs in folder names (e.g., `Book Title [B00G3L6JMS]`)

✅ **Smart Search** - Search by title/author when ASIN isn't available

✅ **Intelligent Caching** - Reduces API calls with 7-day filesystem cache

✅ **Rate Limiting** - Respects Audnexus API limits (100 req/min)

✅ **Cover Art** - Downloads and embeds high-quality cover artwork

### Quick Start

#### Fetch Metadata by ASIN

```bash
audiobook-forge metadata fetch --asin B00B5HZGUG
```

#### Search by Title/Author

```bash
audiobook-forge metadata fetch --title "The Martian" --author "Andy Weir" --region us
```

#### Enrich Existing M4B File

```bash
# With explicit ASIN
audiobook-forge metadata enrich --file book.m4b --asin B00B5HZGUG

# Auto-detect ASIN from filename
audiobook-forge metadata enrich --file "The Martian [B00B5HZGUG].m4b" --auto-detect
```

#### Auto-Fetch During Build

```bash
# Fetch metadata during conversion (requires folder names with ASINs)
audiobook-forge build --root /audiobooks --fetch-audible

# Auto-match by folder name (may have false positives)
audiobook-forge build --root /audiobooks --fetch-audible --audible-auto-match

# Specify region
audiobook-forge build --root /audiobooks --fetch-audible --audible-region uk
```

### Configuration

Add Audible settings to your `~/.config/audiobook-forge/config.yaml`:

```yaml
metadata:
  audible:
    # Enable automatic fetching during build
    enabled: false

    # Default region for metadata queries
    # Options: us, ca, uk, au, fr, de, jp, it, in, es
    region: "us"

    # Auto-match books by folder name
    # Warning: May produce false positives
    auto_match: false

    # Download and embed cover art from Audible
    download_covers: true

    # Cache metadata locally (in hours, 0 = no cache)
    cache_duration_hours: 168  # 7 days

    # API rate limit (requests per minute)
    # Do not exceed 100
    rate_limit_per_minute: 100
```

### ASIN Detection

The tool automatically detects Audible ASINs in folder names using pattern: `B[0-9A-Z]{9}`

**Supported formats:**
- `Book Title [B00G3L6JMS]`
- `B00G3L6JMS - Book Title`
- `The Martian - B00G3L6JMS - Andy Weir`
- `Book [B00G3L6JMS].m4b`

**ASIN Requirements:**
- Must start with the letter `B`
- Must be exactly 10 characters total
- Can only contain uppercase letters and numbers

### Metadata Fields

The following metadata is extracted from Audible:

**Core Fields**:
- Title, Subtitle
- Authors (with ASINs)
- Narrators
- Publisher
- Published Year

**Content Information**:
- Description (synopsis)
- Language
- Duration (minutes/hours)
- Is Abridged

**Organization**:
- Series (name and sequence)
- Genres (primary)
- Tags (categorization)

**Identifiers**:
- ASIN (Audible Standard Identification Number)
- ISBN (when available)

**Media**:
- Cover URL (high-resolution artwork)
- Rating (customer rating)

### How It Works

#### API Integration

Uses the **Audnexus API** (https://api.audnex.us):
- No authentication required
- Comprehensive data from Audible's catalog
- Multi-regional support
- Free tier

#### Caching Strategy

Metadata is cached locally to minimize API calls:
- **Location**: `~/.cache/audiobook-forge/audible/`
- **Format**: JSON files named by ASIN (e.g., `B00G3L6JMS.json`)
- **Default TTL**: 7 days (configurable)
- **Validation**: Automatic expiry based on file modification time

#### Rate Limiting

To respect the Audnexus API:
- **Default limit**: 100 requests per minute
- **Implementation**: Token bucket algorithm via `governor` crate
- **Behavior**: Automatic wait when limit reached
- **Configurable**: Adjust via config (don't exceed 100)

#### Metadata Priority

When enriching audiobooks:
1. **Audible metadata always wins** - Overrides existing ID3/M4A tags
2. **Cover art replacement** - Audible covers replace local artwork
3. **Comprehensive fields** - Narrator added as composer tag
4. **Description preservation** - Full synopsis stored in comment field

### Regional Support

| Region | Code | Audible Store |
|--------|------|---------------|
| United States | `us` | audible.com |
| Canada | `ca` | audible.ca |
| United Kingdom | `uk` | audible.co.uk |
| Australia | `au` | audible.com.au |
| France | `fr` | audible.fr |
| Germany | `de` | audible.de |
| Japan | `jp` | audible.co.jp |
| Italy | `it` | audible.it |
| India | `in` | audible.in |
| Spain | `es` | audible.es |

**Specify region**:

```bash
# Via CLI flag
audiobook-forge metadata fetch --asin B00B5HZGUG --region uk

# Via config
metadata:
  audible:
    region: "uk"
```

---

## Interactive Matching

**NEW in v2.3.0 | Enhanced in v2.4.1 | Fixed in v2.4.0**

BEETS-inspired interactive matching system with fuzzy search and confidence scoring.

### Features

- 🎯 **Fuzzy String Matching**: Weighted scoring algorithm (Title 40%, Author 30%, Duration 20%, Year 10%)
- 🎨 **Color-Coded Confidence**: Visual indicators (Green >96%, Yellow 88-96%, Red 80-88%)
- 📊 **Percentage Display**: Clear match confidence for each candidate
- 🔄 **Multiple Options**: Skip, Manual Entry, Custom Search
- 📝 **Before/After Comparison**: See metadata changes before applying
- 🚀 **Batch Processing**: Process entire directories with progress tracking
- ⚡ **Auto Mode**: Non-interactive mode for scripting

### Usage

```bash
# Interactive match single file
audiobook-forge match --file "Book.m4b"

# Batch process directory
audiobook-forge match --dir /path/to/audiobooks

# Auto mode (non-interactive)
audiobook-forge match --dir /path --auto

# Manual override
audiobook-forge match --file "Book.m4b" --title "Title" --author "Author"

# Dry run to preview
audiobook-forge match --dir /path --dry-run

# Keep existing cover
audiobook-forge match --file "Book.m4b" --keep-cover

# Specific region
audiobook-forge match --file "Book.m4b" --region uk
```

### Matching Algorithm

**Weighted Scoring**:
- **Title**: 40% weight - Most important identifier
- **Author**: 30% weight - Secondary identifier
- **Duration**: 20% weight - Validates correct edition
- **Year**: 10% weight - Helps distinguish versions

**Confidence Levels**:
- **Green (>96%)**: Excellent match, high confidence
- **Yellow (88-96%)**: Good match, review recommended
- **Red (80-88%)**: Possible match, verification needed
- **Below 80%**: Not shown (low confidence)

### Configuration

```yaml
metadata:
  match_mode: disabled  # Options: disabled, auto, interactive
```

**Match Modes**:
- `disabled` - Manual matching only (use `match` command)
- `auto` - Automatically select best match during build (non-interactive)
- `interactive` - Prompt user to select match during build

### Enhanced in v2.4.1

**Improved Filename Parsing**:
- Correctly handles underscores: `Author_-_Title.m4b`
- Mixed patterns: `Author_ -_Title.m4b`
- Match accuracy improved from 60-70% → 85-95%

**Example**:
```
File: Adam_Phillips_-_On_Giving_Up.m4b
Before v2.4.1: Only title "On Giving Up" → Poor results
After v2.4.1: title="On Giving Up" + author="Adam Phillips" → Excellent results
```

---

## Configuration Reference

Complete metadata-related configuration options:

```yaml
metadata:
  # Local Metadata Extraction
  extract_from_files: true          # Extract from ID3/M4A tags
  prefer_embedded: true              # Prefer embedded tags over filenames
  fallback_to_folder_name: true     # Use folder name as fallback
  default_language: "en"             # ISO 639-1 language code

  # Cover Art
  cover_filenames:                   # Files to search for covers
    - "cover.jpg"
    - "folder.jpg"
    - "cover.png"
    - "folder.png"
  auto_extract_cover: true           # Auto-extract embedded art (v2.8.0)

  # Interactive Matching (v2.3.0)
  match_mode: disabled               # disabled | auto | interactive

  # Audible Integration (v2.2.0)
  audible:
    enabled: false                   # Auto-fetch during build
    region: "us"                     # Audible region code
    auto_match: false                # Match by folder name
    download_covers: true            # Download Audible covers
    cache_duration_hours: 168        # Cache TTL (7 days)
    rate_limit_per_minute: 100       # API rate limit
```

---

## Metadata Priority & Precedence

Understanding how metadata sources are prioritized:

### Priority Hierarchy

1. **Audible metadata** (if enabled and matched)
   - Highest priority when using `--fetch-audible` or match command
   - Overrides all local metadata

2. **Embedded audio tags** (ID3/M4A atoms)
   - Second priority for local metadata
   - Preferred when `prefer_embedded: true`

3. **Filename parsing**
   - Third priority
   - Activated when no embedded tags exist

4. **Folder name fallback**
   - Lowest priority
   - Used when `fallback_to_folder_name: true` and no other sources available

### Cover Art Priority

1. **Standalone cover files** (cover.jpg, etc.)
   - First priority, always used if found
   - Checked against `cover_filenames` list

2. **Embedded cover art** (auto-extracted)
   - Second priority
   - Only if `auto_extract_cover: true` and no standalone file

3. **Audible cover download**
   - Third priority
   - Only if `audible.download_covers: true` and metadata fetched

4. **No cover**
   - If none of the above sources provide artwork

### Metadata Merge Behavior

When multiple sources exist:
- **Audible**: Replaces all fields completely
- **Local tags**: Fill in missing fields only
- **Filenames**: Lowest priority, only for empty fields
- **Folder names**: Absolute fallback

---

## Troubleshooting

### Audible Metadata Issues

**No metadata found:**

```
Error: No results found for search query
```

**Solutions:**
```bash
# Try partial titles
audiobook-forge metadata fetch --title "Hail Mary" --author "Weir"

# Use ASIN directly (more reliable)
audiobook-forge metadata fetch --asin B00G3L6JMS

# Try different region
audiobook-forge metadata fetch --title "Book" --region uk
```

**ASIN not detected:**

Ensure format is correct:
- `Book Title [B00G3L6JMS]` ✓
- `B00G3L6JMS - Book Title` ✓
- `book-B00G3L6JMS` ✗ (missing brackets/separation)

**API rate limiting:**

```yaml
# Reduce rate in config
metadata:
  audible:
    rate_limit_per_minute: 50
```

**Cache issues:**

```bash
# Clear cache for specific ASIN
rm ~/.cache/audiobook-forge/audible/B00G3L6JMS.json

# Clear entire cache
rm -rf ~/.cache/audiobook-forge/audible/
```

### Match Command Issues

**Poor match results (Fixed in v2.4.1):**

Upgrade to v2.4.1 for improved filename parsing:

```bash
cargo install audiobook-forge --force
```

**No matches found:**

- Verify author/title in filename
- Try manual override: `--title "Title" --author "Author"`
- Use different region: `--region uk`

### Cover Art Issues

**No cover detected:**

Check cover filenames:
```yaml
metadata:
  cover_filenames:
    - "cover.jpg"
    - "Cover.jpg"  # Add case variations
    - "cover.png"
```

**Embedded cover not extracted:**

Verify feature is enabled:
```yaml
metadata:
  auto_extract_cover: true
```

Check if audio file has embedded art:
```bash
ffprobe -v error -show_entries stream=codec_name,codec_type file.mp3
```

---

## Next Steps

- [Configuration Guide](configuration.md) - Detailed metadata configuration
- [Usage Guide](usage.md) - Using metadata commands
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
