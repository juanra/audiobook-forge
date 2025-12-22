# Audiobook Forge Documentation

Welcome to the Audiobook Forge documentation! This guide will help you get started and make the most of all features.

## Quick Links

### For New Users

Start here if you're new to Audiobook Forge:

1. **[Installation Guide](installation.md)** - Install Audiobook Forge and dependencies
2. **[Usage Guide](usage.md)** - Learn basic commands and workflows
3. **[Configuration](configuration.md)** - Customize settings for your needs

### For All Users

- **[Usage Guide](usage.md)** - Complete command reference and examples
- **[Metadata Guide](metadata.md)** - Metadata management, Audible integration, and interactive matching
- **[Configuration](configuration.md)** - All configuration options explained
- **[Troubleshooting](troubleshooting.md)** - Common issues and solutions

## Documentation Overview

### [Installation](installation.md)

Learn how to install Audiobook Forge and its required dependencies (FFmpeg, AtomicParsley, MP4Box).

**Topics covered:**
- Installing via Cargo
- Installing from source
- Platform-specific dependency installation (macOS, Linux, Windows)
- Verifying installation
- Updating and uninstalling

### [Usage](usage.md)

Complete guide to using Audiobook Forge commands with examples.

**Topics covered:**
- Quick start guide
- Command reference (`build`, `organize`, `match`, `metadata`, `config`, `check`)
- 6 detailed usage examples
- Common workflows
- Tips and best practices

### [Configuration](configuration.md)

Comprehensive configuration reference.

**Topics covered:**
- Configuration file location and setup
- Performance settings (parallel encoding, workers)
- Processing settings (retries, temp files)
- Quality settings (bitrate, sample rate, chapters)
- Metadata settings (extraction, cover art, Audible, matching)
- Advanced settings (encoder selection)
- CLI flag overrides
- Example configurations

### [Metadata](metadata.md)

Everything about metadata management.

**Topics covered:**
- Local metadata extraction (ID3 tags, M4A atoms, CUE sheets, filenames)
- Cover art management and auto-extraction (v2.8.0)
- Audible integration (ASIN detection, regions, caching, API)
- Interactive matching (fuzzy search, confidence scoring, batch processing)
- Configuration reference
- Metadata priority and precedence
- Troubleshooting metadata issues

### [Troubleshooting](troubleshooting.md)

Solutions for common issues.

**Topics covered:**
- Dependency installation issues
- File and permission problems
- Processing issues (memory, hangs, quality)
- Metadata problems (Audible API, matching, cover art)
- Configuration issues
- Performance optimization
- FAQ

## Additional Resources

### Project Files

- **[CHANGELOG](../CHANGELOG.md)** - Release history and version changes
- **[CONTRIBUTING](../CONTRIBUTING.md)** - How to contribute to the project
- **[LICENSE](../LICENSE)** - MIT License details

### External Links

- **[GitHub Repository](https://github.com/juanra/audiobook-forge)** - Source code
- **[GitHub Issues](https://github.com/juanra/audiobook-forge/issues)** - Report bugs
- **[GitHub Discussions](https://github.com/juanra/audiobook-forge/discussions)** - Ask questions

## Getting Started

### 1. Install Audiobook Forge

Follow the [Installation Guide](installation.md) to set up Audiobook Forge and its dependencies.

### 2. Verify Installation

```bash
audiobook-forge check
```

### 3. Run Your First Conversion

```bash
cd /path/to/audiobook
audiobook-forge build
```

### 4. Explore Features

- Try [Audible metadata integration](metadata.md#audible-integration)
- Use [interactive matching](metadata.md#interactive-matching) for existing M4B files
- Configure [performance settings](configuration.md#performance-settings) for faster processing

## Quick Reference

### Essential Commands

```bash
# Convert audiobook (auto-detect current directory)
audiobook-forge build

# Convert with explicit path
audiobook-forge build --root /path/to/audiobook

# Batch process with parallel workers
audiobook-forge build --root /audiobooks --parallel 4

# Fetch Audible metadata
audiobook-forge metadata fetch --asin B00B5HZGUG

# Interactive metadata matching
audiobook-forge match --file book.m4b

# Organize library
audiobook-forge organize --root /audiobooks

# Check dependencies
audiobook-forge check

# Show configuration
audiobook-forge config show
```

### Key Configuration Options

```yaml
performance:
  enable_parallel_encoding: true  # 3.8x faster encoding
  max_concurrent_encodes: "auto"

processing:
  parallel_workers: 4
  skip_existing: true

metadata:
  auto_extract_cover: true  # Auto-extract from audio files (v2.8.0)
  audible:
    enabled: false
    region: "us"
```

## Feature Highlights

### Performance (v2.1.0+)

- **Parallel File Encoding**: 3.8x faster processing (121s → 32s)
- **Multi-core Support**: Automatic CPU detection and optimization
- **Copy Mode**: Ultra-fast concatenation without re-encoding

### Metadata (v2.2.0+)

- **Audible Integration**: Comprehensive metadata from 10 regional stores
- **Auto-Extract Cover Art**: Embedded cover art as fallback (v2.8.0)
- **Interactive Matching**: BEETS-inspired fuzzy matching with confidence scores (v2.3.0+)

### Quality

- **Smart Detection**: Automatically preserves source audio quality
- **Multiple Presets**: From low (64kbps) to maximum (256kbps)
- **Copy Mode**: Lossless for compatible formats

## Version History

See [CHANGELOG](../CHANGELOG.md) for detailed release notes.

**Latest version**: v2.8.0
- Auto-extract embedded cover art
- Enhanced metadata extraction
- Bug fixes and improvements

## Contributing

We welcome contributions! See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

- 🐛 Report bugs via [GitHub Issues](https://github.com/juanra/audiobook-forge/issues)
- 💡 Suggest features via [GitHub Discussions](https://github.com/juanra/audiobook-forge/discussions)
- 🔧 Submit pull requests for improvements

## Support

Need help?

1. Check [Troubleshooting Guide](troubleshooting.md)
2. Search [GitHub Issues](https://github.com/juanra/audiobook-forge/issues)
3. Ask in [GitHub Discussions](https://github.com/juanra/audiobook-forge/discussions)
4. Run with `--verbose` for detailed logs

## License

Audiobook Forge is licensed under the MIT License. See [LICENSE](../LICENSE) for details.

---

**Happy audiobook converting! 🎧**
