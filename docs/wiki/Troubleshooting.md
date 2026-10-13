# Troubleshooting

## Dependencies

### FFmpeg / AtomicParsley Not Found

`audiobook-forge check` reports a missing tool as:

```
  ✗ FFmpeg (not found)
```

During a build it shows up as `Error: FFmpeg not found in PATH` (or `FFprobe not found in PATH`). FFprobe ships with FFmpeg but is not listed by `check`.

Install the missing tool for your platform:

```bash
# macOS
brew install ffmpeg atomicparsley

# Ubuntu/Debian
sudo apt install ffmpeg atomicparsley

# Fedora/RHEL
sudo dnf install ffmpeg atomicparsley
```

Verify with `audiobook-forge check`.

---

## File and Permission Issues

### Permission Denied

```
Error: Permission denied (os error 13)
```

```bash
chmod -R u+rw /path/to/audiobooks
```

### Disk Space

```
Error: No space left on device (os error 28)
```

```bash
# Use a different output directory
audiobook-forge build --root /path --out /path/to/larger/disk

# Clean up temp files left by failed or interrupted runs
rm -rf "${TMPDIR:-/tmp}"/audiobook-forge-*
```

---

## Processing Issues

### Out of Memory

```bash
# Reduce parallelism
audiobook-forge build --root /path --parallel 1
```

Or in config (fewer books at once, and fewer files encoded at once within each book):
```yaml
processing:
  parallel_workers: 1
performance:
  max_concurrent_files_per_book: "2"
```

### Processing Hangs

```bash
# Run with verbose logging to identify where it stops
audiobook-forge build --root /path --verbose

# Try single worker
audiobook-forge build --root /path --parallel 1
```

If the issue persists, disable parallel encoding:
```yaml
performance:
  enable_parallel_encoding: false
```

### High CPU Usage

This is expected during parallel encoding. To reduce:
```bash
audiobook-forge build --root /path --parallel 1
```

Or lower `max_concurrent_files_per_book` (default `"8"`) in config, or set `enable_parallel_encoding: false`. `max_concurrent_encodes` only caps how many books run at once.

---

## Build Failures

### Output Name Collision

```
✗ Book - Output name 'Book.m4b' is shared with another book in this run (/path/to/Book); rename one of the folders
```

All books in a run write to the same output directory, so two folders that would produce the same M4B name are failed up front instead of overwriting each other. Rename one of the folders, or build them in separate runs with different `--out` directories.

### Chapter Count Mismatch

```
/path/to/Book.m4b has 10 chapters but 12 were written
```

After writing chapters, the finished file is read back and its chapter count checked. A mismatch fails the book instead of producing a file with missing chapters. Run with `--verbose` and report it with the source files' format.

### FFmpeg Chapter Injection Failed

```
ffmpeg failed to inject chapters: <ffmpeg error>
```

Chapters are written with FFmpeg. The text after the colon is FFmpeg's own error; check that your FFmpeg is recent and that the output disk has space.

### Incompatible M4B Streams When Merging

```
M4B audio streams are incompatible for lossless concatenation (first: 44100 Hz/2 ch/aac, incompatible: 22050 Hz/1 ch/aac); normalize the source files first
```

Merging M4B parts is a lossless copy, so every part must share sample rate, channels and codec. Re-encode the odd part to match the others, then build again. A related error, `M4B stream codec '<codec>' cannot be concatenated losslessly`, means the parts are not AAC or ALAC.

### Unknown Encoder / aac_mf Limits

```
Unknown encoder 'xyz' in config, falling back to auto-detection
```

`aac_encoder` (or `--aac-encoder`) must be one of `auto`, `aac_at`, `libfdk_aac`, `aac`, `aac_mf`; any other value falls back to `auto`. `aac_mf` (Windows Media Foundation) is never chosen by `auto` and supports only 44.1/48 kHz and 96 to 192 kbps, so the `low` preset (64 kbps, 22.05 kHz) and high source bitrates fall outside what it supports. Use `aac` in that case.

---

## Quality Issues

### Output Sounds Worse Than Source

Without `--quality` (same as `--quality source`), the source bitrate is preserved (lossless sources are encoded to a sane AAC bitrate). If your source is already low quality (e.g., 64kbps), encoding at a higher bitrate won't improve it; it just creates a larger file.

```bash
# Check source quality
ffprobe /path/to/source.mp3

# Use the source preset explicitly (same as the default)
audiobook-forge build --root /path --quality source
```

---

## Metadata Issues

### No Audible Results

```bash
# Try partial titles
audiobook-forge metadata fetch --title "Hail Mary" --author "Weir"

# Use ASIN directly (most reliable)
audiobook-forge metadata fetch --asin B00G3L6JMS

# Try a different region
audiobook-forge metadata fetch --title "Book" --region uk
```

### ASIN Not Detected

Ensure the format is correct:
- `Book Title [B00G3L6JMS]` ✓
- `B00G3L6JMS - Book Title` ✓
- `book-B00G3L6JMS` ✓ (a hyphen counts as a separator)
- `book_B00G3L6JMS` ✗ (underscore is not a separator)
- `bookB00G3L6JMS` ✗ (no separation)

### API Rate Limiting

Reduce rate in config or process in smaller batches:
```yaml
metadata:
  audible:
    rate_limit_per_minute: 50
```

### Cache Issues

The cache lives in your OS cache directory:

| OS | Path |
|----|------|
| Linux | `~/.cache/audiobook-forge/audible/` |
| macOS | `~/Library/Caches/audiobook-forge/audible/` |
| Windows | `%LOCALAPPDATA%\audiobook-forge\audible\` |

```bash
# Clear cache for one ASIN (Linux path shown)
rm ~/.cache/audiobook-forge/audible/B00G3L6JMS.json

# Clear all
rm -rf ~/.cache/audiobook-forge/audible/
```

---

## Cover Art Issues

### No Cover Embedded

Check in order:
1. Does a cover file exist? (`cover.jpg`, `folder.jpg`, etc.)
2. Is auto-extraction enabled? (`auto_extract_cover: true` in config)
3. Does the source audio have embedded art? (`ffprobe -v error -show_entries stream=codec_name,codec_type file.mp3`)
4. Is your cover filename in the config list? Add it if custom.

---

## Configuration Issues

### Invalid Config File

```bash
audiobook-forge config validate
```

This only checks that the file parses. It does not check value ranges, and unknown (misspelled) keys are silently ignored. Common YAML errors: tabs instead of spaces, missing colons, unquoted special characters.

### Old Config Mentions mp4box_path

MP4Box is no longer used (chapters are written with FFmpeg). Configs that still contain `advanced.mp4box_path` load fine; the key is ignored and can be deleted.

### Reset to Defaults

```bash
cp "$(audiobook-forge config path)" "$(audiobook-forge config path).backup"
audiobook-forge config init --force
```

---

## FAQ

**Can I keep the original files?**
Yes, source files are never deleted. (`--delete-originals` is accepted but currently does nothing.)

**Copy mode vs transcode mode?**
Copy mode does lossless concatenation (fast). It is used when every track is AAC or ALAC with a matching format and the audio is mono or stereo. Everything else (MP3, FLAC, surround audio) is transcoded to AAC, preserving source quality; surround is downmixed to stereo.

**Can I cancel safely?**
Yes, press Ctrl+C. Leftover temp directories (`audiobook-forge-*` in your system temp directory) can be deleted.

**Windows support?**
Yes, but dependencies must be installed manually. See [Installation](Installation).

**Where are temp files?**
`audiobook-forge-<book-name>-<random>` in the system temp directory: `/tmp` on Linux, `$TMPDIR` (under `/var/folders/`) on macOS, `%TEMP%` on Windows. Removed after a book succeeds unless `--keep-temp` is used. Left behind when a book fails or the run is interrupted.

**Scripting/automation?**
All commands support non-interactive mode. Use `--auto` with the `match` command.

---

## Getting Help

1. Run with `--verbose` for detailed logs
2. Check [GitHub Issues](https://github.com/juanra/audiobook-forge/issues) for similar problems

**When reporting a bug, include:**
- `audiobook-forge --version`
- OS and version
- Full error message
- Steps to reproduce
