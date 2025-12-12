# Changelog

All notable changes to audiobook-forge (Rust version) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

- **2.0.0** - Rust rewrite (2025-12-12) ← Current
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
