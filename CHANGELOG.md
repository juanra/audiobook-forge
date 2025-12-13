# Changelog

All notable changes to audiobook-forge (Rust version) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
