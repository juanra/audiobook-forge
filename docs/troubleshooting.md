# Troubleshooting

This guide covers common issues and their solutions.

## Common Issues

### Dependencies

#### FFmpeg Not Found

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

# Fedora/RHEL
sudo dnf install ffmpeg

# Verify installation
ffmpeg -version
audiobook-forge check
```

---

#### AtomicParsley Not Found

**Error:**
```
✗ AtomicParsley not found in PATH
```

**Solution:**

```bash
# macOS
brew install atomicparsley

# Ubuntu/Debian
sudo apt install atomicparsley

# Verify installation
AtomicParsley --version
audiobook-forge check
```

---

#### MP4Box Not Found

**Error:**
```
✗ MP4Box not found in PATH
```

**Solution:**

```bash
# macOS
brew install gpac

# Ubuntu/Debian
sudo apt install gpac

# Verify installation
MP4Box -version
audiobook-forge check
```

---

### File and Permission Issues

#### Permission Denied

**Error:**
```
Error: Permission denied (os error 13)
```

**Solution:**

```bash
# Check file permissions
ls -la /path/to/audiobooks

# Fix permissions on the audiobook directory
chmod -R u+rw /path/to/audiobooks

# If still failing, check parent directory permissions
chmod u+w /path/to
```

---

#### Disk Space Issues

**Error:**
```
Error: No space left on device (os error 28)
```

**Solution:**

```bash
# Check available disk space
df -h

# Free up space or use a different output directory
audiobook-forge build --root /path --output /path/to/larger/disk

# Clean up temp files if interrupted
rm -rf /tmp/audiobook-forge-*
```

---

### Processing Issues

#### Out of Memory (Large Libraries)

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

# Adjust in config
```

**Config adjustment:**

```yaml
processing:
  parallel_workers: 1

performance:
  max_concurrent_encodes: 2
```

---

#### Processing Hangs or Freezes

**Symptoms:**
- Process stops responding
- No progress for extended period
- CPU usage drops to 0%

**Solution:**

```bash
# Run with verbose logging to see where it stops
audiobook-forge build --root /path --verbose

# Try with single worker
audiobook-forge build --root /path --parallel 1

# Disable parallel encoding if issue persists
```

**Config:**

```yaml
performance:
  enable_parallel_encoding: false
```

---

### Quality Issues

#### Quality Worse Than Original

**Issue:** Output file sounds compressed

**Solution:**

```bash
# First, check your source file quality
ffprobe /path/to/source.mp3

# The default preserves source quality
audiobook-forge build --root /path --quality source

# Verify you're not using a lower quality preset
audiobook-forge build --root /path  # Uses 'source' by default
```

**Important:** The tool preserves whatever quality exists in your source files. If your source is already compressed (e.g., 64kbps), encoding at a higher bitrate won't improve quality - it will just create a larger file with the same audio quality.

---

#### MP3 to M4B Conversion Fails

**Previous Error (Fixed in v2.1.0):**
```
✗ Failed to concatenate audio files
Could not find tag for codec mp3 in stream #0
```

**Solution:**

This issue was fixed in v2.1.0! The tool now automatically:
- Forces AAC transcoding for MP3 files (MP3 codec cannot be copied into M4B container)
- Skips video streams (embedded cover art) with `-vn` flag
- Uses parallel encoding for faster MP3 to M4B conversion

**If you're still on v2.0.0**, upgrade:

```bash
cargo install audiobook-forge --force
audiobook-forge --version  # Verify >= v2.1.0
```

---

### Metadata Issues

#### Match Command Fails with 404 Errors

**Error (Fixed in v2.4.0):**
```
✗ Error: Search API returned status: 404 Not Found
```

**Solution:**

This was a critical bug in v2.3.0 where the match command used a non-existent API endpoint.

**Upgrade to v2.4.1:**

```bash
cargo install audiobook-forge --force
audiobook-forge --version  # Verify >= v2.4.1
```

After upgrading, the match command will work correctly:

```bash
audiobook-forge match --file "Book.m4b"
# or
audiobook-forge match --dir ~/Downloads/m4b/ --auto
```

---

#### Poor Match Results with Underscore Filenames

**Issue (Fixed in v2.4.1):**

Files named like `Author_-_Title.m4b` return irrelevant search results.

**Example:**
- File: `Adam_Phillips_-_On_Giving_Up.m4b`
- Before v2.4.1: Only title "On Giving Up", missing author → Poor results
- After v2.4.1: title="On Giving Up" + author="Adam Phillips" → Good results

**Solution:**

Upgrade to v2.4.1:

```bash
cargo install audiobook-forge --force
```

The fix automatically handles common patterns:
- `Author_-_Title.m4b` ✅
- `Author - Title.m4b` ✅
- `Author_ -_Title.m4b` ✅
- Mixed patterns ✅

Match accuracy improved from 60-70% → 85-95%

---

#### No Audible Metadata Found

**Error:**
```
No results found for search query
```

**Solutions:**

```bash
# 1. Try partial titles (often work better)
audiobook-forge metadata fetch --title "Hail Mary" --author "Weir"

# 2. Use ASIN directly (most reliable)
audiobook-forge metadata fetch --asin B00G3L6JMS

# 3. Try different region
audiobook-forge metadata fetch --title "Book" --region uk

# 4. Check spelling of author/title
audiobook-forge metadata fetch --title "The Martian" --author "Andy Weir"
```

---

#### ASIN Not Detected

**Issue:** Folder has ASIN but not auto-detected

**Correct ASIN formats:**
- `Book Title [B00G3L6JMS]` ✅
- `B00G3L6JMS - Book Title` ✅
- `Book - B00G3L6JMS - Author` ✅

**Incorrect formats:**
- `book-B00G3L6JMS` ✗ (missing brackets/separation)
- `[B00G3L6JMS]Book` ✗ (no space)

**Solution:**

```bash
# Rename folder with proper format
mv "Book" "Book [B00G3L6JMS]"

# Or use explicit ASIN
audiobook-forge metadata fetch --asin B00G3L6JMS

# Or auto-detect from filename
audiobook-forge metadata enrich --file "Book [B00G3L6JMS].m4b" --auto-detect
```

**ASIN Requirements:**
- Must start with letter `B`
- Exactly 10 characters total
- Only uppercase letters and numbers

---

#### API Rate Limiting

**Issue:** Too many requests to Audible API

**Error:**
```
Rate limit exceeded
```

**Solution:**

```yaml
# In config.yaml, reduce rate limit
metadata:
  audible:
    rate_limit_per_minute: 50  # Reduce from 100

# Or process in smaller batches
# Wait a few minutes between batches
```

Alternatively, process fewer audiobooks at once:

```bash
audiobook-forge build --root /path/batch1 --fetch-audible
# Wait 2-3 minutes
audiobook-forge build --root /path/batch2 --fetch-audible
```

---

#### Cache Issues

**Issue:** Stale or corrupted cache data

**Symptoms:**
- Outdated metadata returned
- Errors when reading cached files
- Incorrect information displayed

**Solution:**

```bash
# Clear cache for specific ASIN
rm ~/.cache/audiobook-forge/audible/B00G3L6JMS.json

# Clear entire cache
rm -rf ~/.cache/audiobook-forge/audible/

# Verify cache location
ls ~/.cache/audiobook-forge/audible/
```

**Temporarily disable caching:**

```yaml
# In config.yaml
metadata:
  audible:
    cache_duration_hours: 0  # Disable cache
```

---

### Cover Art Issues

#### No Cover Art Embedded

**Issue:** M4B file created without cover art

**Possible causes and solutions:**

**1. No cover file found:**

```bash
# Check if cover file exists
ls /path/to/audiobook/cover.jpg

# Add cover file with supported name
cp cover_image.jpg /path/to/audiobook/cover.jpg
```

**2. Auto-extract disabled:**

```yaml
# Enable auto-extraction in config
metadata:
  auto_extract_cover: true
```

**3. No embedded cover in audio files:**

```bash
# Check if audio file has embedded cover
ffprobe -v error -show_entries stream=codec_name,codec_type file.mp3

# Look for: codec_type=video (embedded cover)
```

**4. Custom cover filename not recognized:**

```yaml
# Add your cover filename to config
metadata:
  cover_filenames:
    - "cover.jpg"
    - "folder.jpg"
    - "your_custom_name.jpg"
```

---

#### Extracted Cover Not Cleaned Up

**Issue:** `.extracted_cover.jpg` remains after processing

**This should not happen**, but if it does:

```bash
# Manually remove
rm /path/to/audiobook/.extracted_cover.jpg

# Report as a bug
```

---

### Configuration Issues

#### Invalid Configuration File

**Error:**
```
Error parsing config file
```

**Solution:**

```bash
# Validate configuration
audiobook-forge config validate

# Check YAML syntax
cat ~/.config/audiobook-forge/config.yaml

# Reinitialize with defaults
cp ~/.config/audiobook-forge/config.yaml ~/.config/audiobook-forge/config.yaml.backup
audiobook-forge config init --force
```

Common YAML errors:
- Incorrect indentation (must use spaces, not tabs)
- Missing colons after keys
- Unquoted strings with special characters

---

#### Config Not Being Used

**Issue:** CLI uses defaults despite config file existing

**Solution:**

```bash
# Verify config file location
audiobook-forge config path

# Show current effective config
audiobook-forge config show

# Ensure config file exists
ls -la ~/.config/audiobook-forge/config.yaml

# Check file permissions
chmod 644 ~/.config/audiobook-forge/config.yaml
```

---

## Performance Issues

### Slow Processing

**Issue:** Processing takes longer than expected

**Solutions:**

**1. Enable parallel encoding (should be default in v2.1.0+):**

```yaml
performance:
  enable_parallel_encoding: true
  max_concurrent_encodes: "auto"
```

**2. Increase parallel workers:**

```bash
audiobook-forge build --root /path --parallel 8
```

**3. Use copy mode when possible:**

For M4A/M4B input files, copy mode is automatic and much faster.

**4. Use SSD storage:**

Processing from/to SSD is significantly faster than HDD.

**5. Check system resources:**

```bash
# Monitor CPU usage
top

# Check if system is under load
uptime
```

---

### High CPU Usage

**Issue:** 100% CPU usage, system slow

**This is normal** during parallel encoding. To reduce:

```yaml
performance:
  max_concurrent_encodes: 2  # Reduce from auto

processing:
  parallel_workers: 1
```

Or use CLI:

```bash
audiobook-forge build --root /path --parallel 1
```

---

### High Memory Usage

**Issue:** Excessive RAM usage

**Solution:**

```yaml
processing:
  parallel_workers: 1

performance:
  max_concurrent_encodes: 1
```

Process one audiobook at a time:

```bash
audiobook-forge build --root "/path/Book" --parallel 1
```

---

## Getting Help

If your issue isn't covered here:

### 1. Check Logs

Run with verbose logging:

```bash
audiobook-forge build --root /path --verbose
```

Look for error messages and stack traces.

### 2. Verify Installation

```bash
# Check all dependencies
audiobook-forge check

# Verify version
audiobook-forge --version

# Show configuration
audiobook-forge config show
```

### 3. Search Existing Issues

Check [GitHub Issues](https://github.com/juanra/audiobook-forge/issues) for similar problems.

### 4. Report a Bug

If you've found a bug, open an issue with:
- Audiobook Forge version (`audiobook-forge --version`)
- Operating system and version
- Full error message
- Steps to reproduce
- Verbose logs (if applicable)

### 5. Ask for Help

- [GitHub Discussions](https://github.com/juanra/audiobook-forge/discussions) - Ask questions
- [GitHub Issues](https://github.com/juanra/audiobook-forge/issues) - Report bugs

---

## FAQ

### Q: Can I process files while keeping the originals?

**A:** Yes, Audiobook Forge never deletes source files. The M4B is created alongside the original files.

### Q: What's the difference between copy mode and transcode mode?

**A:**
- **Copy mode**: Direct concatenation without re-encoding (fast, no quality loss). Used for M4A/AAC files.
- **Transcode mode**: Re-encodes audio to AAC (slower, necessary for MP3 files). Quality preserved from source.

### Q: Can I cancel processing safely?

**A:** Yes, press Ctrl+C. Temporary files will be left in `/tmp/audiobook-forge-*` but can be safely deleted.

### Q: Does it work on Windows?

**A:** Yes, but requires manual installation of dependencies (FFmpeg, AtomicParsley, MP4Box). See [Installation Guide](installation.md).

### Q: How do I update to the latest version?

**A:**
```bash
cargo install audiobook-forge --force
```

### Q: Where are temp files stored?

**A:** In system temp directory: `/tmp/audiobook-forge-<book-name>` (Linux/macOS) or `%TEMP%\audiobook-forge-<book-name>` (Windows).

Temp files are automatically cleaned up unless you use `--keep-temp`.

### Q: Can I use this in scripts/automation?

**A:** Yes! All commands support non-interactive mode. Use `--auto` with match command for automation.

---

## Next Steps

- [Installation Guide](installation.md) - Setup and dependencies
- [Configuration Guide](configuration.md) - Customize settings
- [Usage Guide](usage.md) - Command reference
- [Metadata Guide](metadata.md) - Metadata features
