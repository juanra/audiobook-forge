# Configuration

Audiobook Forge uses a YAML config file. Some CLI flags override it (see [CLI Overrides](#cli-overrides)).

## Config File

**Location** (from your OS config directory):

| OS | Path |
|----|------|
| Linux | `~/.config/audiobook-forge/config.yaml` |
| macOS | `~/Library/Application Support/audiobook-forge/config.yaml` |
| Windows | `%APPDATA%\audiobook-forge\config.yaml` |

Run `audiobook-forge config path` to see yours. If the file does not exist, built-in defaults are used.

```bash
audiobook-forge config init       # Create a file with every key at its default (no comments)
audiobook-forge config show       # Print current settings (fails if no config file exists)
audiobook-forge config validate   # Check that the file parses (no range or unknown-key checks)
audiobook-forge config path       # Show file path
```

`config edit` exists but is not implemented yet: it only prints the path. Open the file in your editor directly.

> Unknown keys are silently ignored, so a misspelled key passes `config validate` and has no effect.

---

## Settings Reference

### Directories

| Setting | Default | Description |
|---------|---------|-------------|
| `source` | `null` | Library to process when `--root` is not given |
| `output` | `"same_as_source"` | Output directory: `"same_as_source"` or a path. `--out` overrides it |

```yaml
directories:
  source: null
  output: "same_as_source"
```

### Performance

| Setting | Default | Description |
|---------|---------|-------------|
| `enable_parallel_encoding` | `true` | Encode the files of one book concurrently |
| `max_concurrent_files_per_book` | `"8"` | Files encoded at once within one book: `"auto"` (CPU cores) or a number, clamped to 1-32 |
| `max_concurrent_encodes` | `"auto"` | Caps how many books are processed at once. `"auto"` = CPU cores, or a number clamped to 1-16. It can lower `parallel_workers` but never raise it. Non-numeric values fall back to `"auto"` |
| `encoding_preset` | `"balanced"` | Currently has no effect |

```yaml
performance:
  enable_parallel_encoding: true
  max_concurrent_files_per_book: "8"
  max_concurrent_encodes: "auto"
```

### Processing

| Setting | Default | Description |
|---------|---------|-------------|
| `parallel_workers` | `2` | Audiobooks processed at once. Clamped to 1-16 (`--parallel` accepts 1-8) |
| `skip_existing` | `true` | Skip folders that already contain M4B files (folders whose M4B parts can be merged are still processed). `--force` disables it |
| `max_retries` | `2` | Retries after a failed attempt (total attempts = `max_retries` + 1) |
| `retry_delay` | `1` | Initial retry delay in seconds (doubles each attempt, max 30 s) |
| `keep_temp_files` | `false` | Keep temp files for debugging (not applied to M4B merges; use `--keep-temp` for those) |
| `force_reprocess` | `false` | Currently has no effect (use `--force`) |
| `normalize_existing` | `false` | Currently has no effect |

```yaml
processing:
  parallel_workers: 2
  skip_existing: true
  max_retries: 2
  retry_delay: 1
  keep_temp_files: false
```

### Quality

| Setting | Default | Options | Description |
|---------|---------|---------|-------------|
| `chapter_source` | `"auto"` | `auto`, `files`, `cue`, `none` | How to generate chapters (`id3` is accepted but behaves as `auto`) |
| `default_bitrate` | `"auto"` | | Currently has no effect (use `--quality`) |
| `default_sample_rate` | `"auto"` | | Currently has no effect (use `--quality`) |
| `prefer_stereo` | `true` | | Currently has no effect |

```yaml
quality:
  chapter_source: "auto"
```

> Output quality is chosen with `build --quality` (`low`, `medium`, `high`, `ultra`, `maximum`, `source`), which has no config equivalent. Higher bitrate does not improve quality beyond what exists in the source files.

### Metadata

| Setting | Default | Description |
|---------|---------|-------------|
| `cover_filenames` | `["cover.jpg", "folder.jpg", "cover.png", "folder.png"]` | Cover art filenames to search for |
| `auto_extract_cover` | `true` | Extract embedded cover art as fallback |
| `default_language` | `"es"` | Currently has no effect |
| `match_mode` | `disabled` | Currently has no effect (use the `match` command) |

```yaml
metadata:
  cover_filenames:
    - "cover.jpg"
    - "folder.jpg"
    - "cover.png"
    - "folder.png"
  auto_extract_cover: true
```

### Audible Integration

| Setting | Default | Description |
|---------|---------|-------------|
| `enabled` | `false` | Auto-fetch during build |
| `region` | `"us"` | Region: `us`, `ca`, `uk`, `au`, `fr`, `de`, `jp`, `it`, `in`, `es` |
| `auto_match` | `false` | Match by folder name (may have false positives) |
| `download_covers` | `true` | Download cover art from Audible |
| `fetch_chapters` | `false` | Fetch chapter data from the Audnex API during build |
| `cache_duration_hours` | `168` | Cache TTL in hours (0 = disabled) |
| `rate_limit_per_minute` | `100` | API rate limit (do not exceed 100) |
| `api_max_retries` | `3` | Retries for failed API requests (0 = no retry) |
| `api_retry_delay_secs` | `1` | Initial API retry delay in seconds |
| `api_max_retry_delay_secs` | `30` | Maximum API retry delay in seconds (exponential backoff) |

```yaml
metadata:
  audible:
    enabled: false
    region: "us"
    auto_match: false
    download_covers: true
    fetch_chapters: false
    cache_duration_hours: 168
    rate_limit_per_minute: 100
    api_max_retries: 3
    api_retry_delay_secs: 1
    api_max_retry_delay_secs: 30
```

See [Metadata: Audible Integration](Metadata#audible-integration) for full documentation.

### Organization

Folder names used by the `organize` command.

| Setting | Default | Description |
|---------|---------|-------------|
| `m4b_folder` | `"M4B"` | Folder for completed audiobooks |
| `convert_folder` | `"To_Convert"` | Folder for audiobooks queued for conversion |

```yaml
organization:
  m4b_folder: "M4B"
  convert_folder: "To_Convert"
```

### Logging

| Setting | Default | Description |
|---------|---------|-------------|
| `log_to_file` | `false` | Also write logs to a file (rotated daily) |
| `log_file` | `null` | Log file path. When `null`: `~/.audiobook-forge/logs/audiobook-forge.log` |
| `log_level` | `"INFO"` | `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`. `--verbose` forces debug output |

```yaml
logging:
  log_to_file: false
  log_file: null
  log_level: "INFO"
```

### Advanced

| Setting | Default | Options | Description |
|---------|---------|---------|-------------|
| `aac_encoder` | `"auto"` | `auto`, `aac_at`, `libfdk_aac`, `aac`, `aac_mf` | AAC encoder selection |
| `ffmpeg_path` | `null` | | Currently has no effect (FFmpeg is found in `PATH`) |
| `atomic_parsley_path` | `null` | | Currently has no effect (AtomicParsley is found in `PATH`) |
| `temp_directory` | `null` | | Currently has no effect (the system temp directory is used) |
| `use_apple_silicon_encoder` | unset | `true`, `false` | Deprecated. If set, it overrides `aac_encoder` (`true` = `aac_at`, `false` = `aac`). Remove it from old configs |

- `auto`: detect best available
- `aac_at`: Apple hardware-accelerated (macOS only)
- `libfdk_aac`: Fraunhofer FDK AAC (if compiled into FFmpeg)
- `aac`: FFmpeg built-in (fallback)
- `aac_mf`: Windows Media Foundation. Never picked by `auto` yet; select it explicitly. Supports only 44.1/48 kHz and 96 to 192 kbps.

An unknown encoder name logs a warning and falls back to `auto`.

```yaml
advanced:
  aac_encoder: "auto"
```

---

## CLI Overrides

Only these `build` flags override the config file:

```bash
audiobook-forge build --root /path                      # Override directories.source
audiobook-forge build --root /path --out /other/dir     # Override directories.output
audiobook-forge build --root /path --parallel 8         # Override parallel_workers (1-8)
audiobook-forge build --root /path --force              # Override skip_existing
audiobook-forge build --root /path --keep-temp          # Override keep_temp_files
audiobook-forge build --root /path --aac-encoder aac    # Override aac_encoder
audiobook-forge build --root /path --fetch-audible      # Override audible.enabled
audiobook-forge build --root /path --audible-region uk  # Override audible.region
audiobook-forge build --root /path --audible-auto-match # Override audible.auto_match
audiobook-forge --verbose build --root /path            # Override log_level (debug)
```

`--prefer-stereo`, `--chapter-source`, `--cover-names`, `--language`, `--skip-existing` and `--config` are accepted but currently have no effect. The config file is always read from the default location.

---

## Example Configurations

### Fast Processing

```yaml
performance:
  enable_parallel_encoding: true
  max_concurrent_encodes: "auto"

processing:
  parallel_workers: 8
  skip_existing: true
```

### CUE Chapters

```yaml
quality:
  chapter_source: "cue"
```

Pair it with `build --quality ultra` or `--quality maximum` for higher-bitrate output.

### Audible-Integrated

```yaml
metadata:
  audible:
    enabled: true
    region: "us"
    auto_match: false
    download_covers: true
```

---

## Reset Configuration

```bash
cp "$(audiobook-forge config path)" "$(audiobook-forge config path).backup"
audiobook-forge config init --force
```
