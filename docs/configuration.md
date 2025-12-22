# Configuration

Audiobook Forge can be configured through a YAML configuration file and CLI flags. This guide covers all available configuration options.

## Configuration File Location

The configuration file is located at:

```
~/.config/audiobook-forge/config.yaml
```

On macOS/Linux, this expands to `/Users/yourusername/.config/audiobook-forge/config.yaml`.

## Creating the Configuration File

### Initialize Default Configuration

To create a configuration file with default settings:

```bash
audiobook-forge config init
```

This creates the config file with all default values that you can customize.

### Configuration Management Commands

```bash
# Show current configuration
audiobook-forge config show

# Validate configuration file
audiobook-forge config validate

# Show config file path
audiobook-forge config path

# Edit config in default editor
audiobook-forge config edit
```

## Complete Configuration Reference

Here's a comprehensive example with all available options:

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
  cover_filenames:           # Cover art filenames to search for
    - "cover.jpg"
    - "folder.jpg"
    - "cover.png"
    - "folder.png"
  auto_extract_cover: true   # Auto-extract embedded cover art (v2.8.0)

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

## Configuration Sections Explained

### Performance Settings

Controls parallel processing and encoding performance.

**`max_concurrent_encodes`** (NEW in v2.1.0)
- **Default:** `"auto"` (uses CPU cores / 2)
- **Options:** `"auto"` or number between 1-16
- **Description:** Number of audio files to encode in parallel. Higher values = faster processing but more CPU usage.
- **Recommendation:** Leave at "auto" unless you have specific needs

**`enable_parallel_encoding`** (NEW in v2.1.0)
- **Default:** `true`
- **Description:** Enable parallel file encoding for 3.8x faster processing
- **Impact:** Reduces encoding time from 121s → 32s (in benchmarks)

**`encoding_preset`**
- **Default:** `"balanced"`
- **Options:** `"fast"`, `"balanced"`, `"high"`
- **Description:** Encoder speed/quality tradeoff
  - `fast`: Faster encoding, slightly lower efficiency
  - `balanced`: Good balance of speed and quality
  - `high`: Slower encoding, better compression

### Processing Settings

Controls how audiobooks are processed.

**`parallel_workers`**
- **Default:** 4 (or number of CPU cores / 2)
- **Description:** Number of audiobooks to process simultaneously
- **Recommendation:** Set to number of CPU cores / 2 for optimal performance

**`skip_existing`**
- **Default:** `true`
- **Description:** Skip audiobooks that already have M4B files
- **Use case:** Set to `false` to force reprocessing

**`max_retries`** (NEW in v2.5.0)
- **Default:** 3
- **Description:** Number of retry attempts for failed operations
- **Use case:** Increase if you have flaky network or storage

**`retry_delay`** (NEW in v2.5.0)
- **Default:** 1 (second)
- **Description:** Delay between retry attempts
- **Use case:** Increase if operations need more time to recover

**`keep_temp_files`**
- **Default:** `false`
- **Description:** Keep temporary files after processing
- **Use case:** Set to `true` for debugging encoding issues

### Quality Settings

Controls audio quality and chapter detection.

**`chapter_source`**
- **Default:** `"auto"`
- **Options:** `"auto"`, `"files"`, `"cue"`, `"none"`
- **Description:**
  - `auto`: Try CUE files first, fall back to file-based chapters
  - `files`: Generate chapters from individual files
  - `cue`: Use CUE sheet only
  - `none`: No chapter markers

**`default_bitrate`**
- **Default:** `"auto"` (preserves source bitrate)
- **Examples:** `"128k"`, `"192k"`, `"256k"`
- **Description:** Audio bitrate for encoding
- **Note:** Higher bitrate doesn't improve quality beyond source quality

**`default_sample_rate`**
- **Default:** `"auto"` (preserves source sample rate)
- **Examples:** `"44100"`, `"48000"`
- **Description:** Audio sample rate in Hz
- **Recommendation:** Leave at "auto" unless you have specific requirements

**`prefer_stereo`**
- **Default:** `false`
- **Description:** Convert mono to stereo
- **Note:** Usually not needed for audiobooks

### Metadata Settings

Controls how metadata is extracted and managed.

**`extract_from_files`**
- **Default:** `true`
- **Description:** Extract metadata from audio file tags (ID3, M4A atoms)

**`prefer_embedded`**
- **Default:** `true`
- **Description:** Prefer embedded metadata tags over filenames

**`fallback_to_folder_name`**
- **Default:** `true`
- **Description:** Use folder name as metadata if no tags found

**`default_language`**
- **Default:** `"en"`
- **Description:** ISO 639-1 language code for audiobooks
- **Examples:** `"en"`, `"es"`, `"fr"`, `"de"`, `"ja"`

**`cover_filenames`**
- **Default:** `["cover.jpg", "folder.jpg", "cover.png", "folder.png"]`
- **Description:** List of filenames to search for cover art
- **Customization:** Add your own patterns (case-insensitive matching)

**`auto_extract_cover`** (NEW in v2.8.0)
- **Default:** `true`
- **Description:** Automatically extract embedded cover art from MP3/M4A files as fallback
- **Behavior:** Only activates if no standalone cover file found
- **Priority:** Standalone covers checked first, embedded art as fallback

**`match_mode`** (NEW in v2.3.0)
- **Default:** `disabled`
- **Options:** `disabled`, `auto`, `interactive`
- **Description:**
  - `disabled`: Manual metadata matching only (use `match` command)
  - `auto`: Automatically select best match during build (non-interactive)
  - `interactive`: Prompt user to select match during build
- **See also:** [Interactive Matching](metadata.md#interactive-matching)

### Audible Integration Settings

See [Metadata Guide](metadata.md#audible-integration) for comprehensive Audible documentation.

**`audible.enabled`**
- **Default:** `false`
- **Description:** Enable automatic Audible metadata fetching during build

**`audible.region`**
- **Default:** `"us"`
- **Options:** `us`, `ca`, `uk`, `au`, `fr`, `de`, `jp`, `it`, `in`, `es`
- **Description:** Audible store region for metadata queries

**`audible.auto_match`**
- **Default:** `false`
- **Description:** Automatically match books by folder name
- **Warning:** May produce false positives - use with caution

**`audible.download_covers`**
- **Default:** `true`
- **Description:** Download high-quality cover art from Audible

**`audible.cache_duration_hours`**
- **Default:** 168 (7 days)
- **Description:** How long to cache metadata locally (0 = no cache)

**`audible.rate_limit_per_minute`**
- **Default:** 100
- **Warning:** Do not exceed 100 requests/minute
- **Description:** API rate limiting to respect Audnexus service limits

### Advanced Settings

**`aac_encoder`**
- **Default:** `"auto"`
- **Options:** `"auto"`, `"aac_at"`, `"libfdk_aac"`, `"aac"`
- **Description:** AAC encoder selection
  - `auto`: Auto-detect best available encoder
  - `aac_at`: Apple's hardware-accelerated encoder (macOS/iOS)
  - `libfdk_aac`: High-quality Fraunhofer FDK AAC (if available)
  - `aac`: FFmpeg's built-in AAC encoder (fallback)
- **Recommendation:** Leave at "auto" for best results

## Overriding Configuration with CLI Flags

CLI flags take precedence over configuration file settings:

```bash
# Override parallel workers
audiobook-forge build --root /path --parallel 8

# Override quality preset
audiobook-forge build --root /path --quality high

# Force reprocessing (skip skip_existing setting)
audiobook-forge build --root /path --force

# Keep temporary files (override keep_temp_files)
audiobook-forge build --root /path --keep-temp

# Enable Audible metadata
audiobook-forge build --root /path --fetch-audible --audible-region uk

# Enable verbose logging
audiobook-forge build --root /path --verbose
```

## Example Configurations

### Minimal Configuration (Fast Processing)

```yaml
performance:
  max_concurrent_encodes: "auto"
  enable_parallel_encoding: true

processing:
  parallel_workers: 8
  skip_existing: true

quality:
  chapter_source: "auto"
  default_bitrate: "auto"
```

### High-Quality Configuration

```yaml
performance:
  encoding_preset: "high"
  enable_parallel_encoding: true

quality:
  chapter_source: "cue"  # Prefer CUE sheets
  default_bitrate: "192k"
  default_sample_rate: "48000"

metadata:
  prefer_embedded: true
  audible:
    enabled: true
    download_covers: true
```

### Audible-Integrated Workflow

```yaml
metadata:
  audible:
    enabled: true
    region: "us"
    auto_match: false  # Safer: requires ASIN in folder names
    download_covers: true
    cache_duration_hours: 168

  match_mode: interactive  # Prompt for metadata selection
```

## Troubleshooting Configuration

### Invalid Configuration

If your config file has errors:

```bash
# Validate configuration
audiobook-forge config validate
```

This will show any syntax errors or invalid values.

### Reset to Defaults

To reset configuration to defaults:

```bash
# Backup current config
cp ~/.config/audiobook-forge/config.yaml ~/.config/audiobook-forge/config.yaml.backup

# Reinitialize
audiobook-forge config init --force
```

### Find Config Location

If you're not sure where your config file is:

```bash
audiobook-forge config path
```

## Next Steps

- [Usage Guide](usage.md) - Learn how to use Audiobook Forge
- [Metadata Guide](metadata.md) - Metadata features and Audible integration
- [Troubleshooting](troubleshooting.md) - Common configuration issues
