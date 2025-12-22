# Audiobook Forge 🎧

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-77%20passing-brightgreen.svg)](tests/)

A blazing-fast CLI tool for converting audiobook directories to M4B format with chapters and metadata. Written in Rust for maximum performance and reliability.

## 📑 Table of Contents

- [Why Audiobook Forge?](#-why-audiobook-forge)
- [Features](#-features)
- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Documentation](#-documentation)
- [Performance](#-performance)
- [Contributing](#-contributing)
- [License](#-license)

---

## 🎯 Why Audiobook Forge?

When downloading audiobooks, they often come as **multiple separate MP3 files** — one for each chapter or section. While this works fine with audiobook players, managing your library becomes significantly easier when **each audiobook is consolidated into a single file**.

**Audiobook Forge** takes those scattered audio files and merges them into a single **M4B file** (MPEG-4 Audiobook format), which is the standard for audiobook applications.

### 📚 Benefits of Single M4B Files

✅ **Simplified Library Management** - One file per audiobook instead of dozens

✅ **Better Metadata & Chapter Support** - Embedded chapter markers and complete metadata

✅ **Improved Portability** - Transfer entire audiobooks with a single file copy

✅ **Enhanced Playback Experience** - Resume exactly where you left off across devices

✅ **Reduced Storage Overhead** - Efficient compression while preserving quality

✅ **Universal Compatibility** - Works with Apple Books, Audiobookshelf, Plex, and most players

---

## ✨ Features

- **📁 Auto-Detect Current Directory**: Run from inside audiobook folders without `--root` parameter
- **⚡ Parallel File Encoding**: Encode files concurrently for **3.8x faster** processing (121s → 32s)
- **🚀 Parallel Book Processing**: Convert multiple audiobooks simultaneously
- **🎯 Smart Quality Detection**: Automatically detects and preserves source audio quality
- **📖 Chapter Generation**: Multiple sources (files, CUE sheets, auto-detection)
- **🎨 Metadata Management**: Comprehensive metadata from multiple sources
- **🖼️ Auto-Extract Cover Art** (v2.8.0): Automatically extracts embedded cover art as fallback
- **🎭 Interactive Metadata Matching** (v2.3.0+): BEETS-inspired fuzzy matching with confidence scoring
- **🎧 Audible Integration** (v2.2.0): Fetch comprehensive metadata from Audible's catalog
- **🔄 Batch Operations**: Process entire libraries with a single command
- **⚡ Copy Mode**: Ultra-fast concatenation without re-encoding when possible
- **🔁 Error Recovery**: Automatic retry with configurable settings
- **📊 Progress Tracking**: Real-time progress with ETA calculation
- **⚙️ Configuration**: Comprehensive YAML-based configuration with CLI overrides

---

## 💖 Support This Project

If you find **audiobook-forge** useful, please consider supporting its development!

[![Sponsor](https://img.shields.io/badge/Sponsor-❤-ea4aaa?style=for-the-badge&logo=github-sponsors)](https://github.com/sponsors/juanra)

Your sponsorship helps:
- 🚀 **Active Development**: Keep the project maintained and add new features
- 🐛 **Bug Fixes**: Respond quickly to issues and edge cases
- 📚 **Documentation**: Maintain comprehensive guides and examples
- 🆓 **Free & Open Source**: Keep audiobook-forge free for everyone

Every contribution, no matter the size, is deeply appreciated! 🙏

---

## 📦 Installation

### Quick Install

```bash
cargo install --git https://github.com/juanra/audiobook-forge
```

### Dependencies

Install required tools:

**macOS:**
```bash
brew install ffmpeg atomicparsley gpac
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install ffmpeg atomicparsley gpac
```

**Fedora/RHEL:**
```bash
sudo dnf install ffmpeg atomicparsley gpac
```

### Verify Installation

```bash
audiobook-forge check
```

**📖 Detailed installation guide**: See [docs/installation.md](docs/installation.md)

---

## 🚀 Quick Start

### Convert a Single Audiobook

```bash
# Auto-detect: Run from inside an audiobook folder
cd "/path/to/My Audiobook"
audiobook-forge build

# Or specify the path explicitly
audiobook-forge build --root "/path/to/My Audiobook"
```

### Batch Process Multiple Audiobooks

```bash
audiobook-forge build --root "/path/to/audiobooks" --parallel 4
```

### With Audible Metadata

```bash
# Rename folders with ASINs: "Book Title [B00G3L6JMS]"
audiobook-forge build --root /audiobooks --fetch-audible
```

### Interactive Metadata Matching

```bash
# Match existing M4B files with Audible metadata
audiobook-forge match --file "Book.m4b"

# Batch match entire directory
audiobook-forge match --dir /path/to/m4b/files
```

**📖 Complete usage guide**: See [docs/usage.md](docs/usage.md)

---

## 📚 Documentation

Comprehensive documentation is available in the `/docs` directory:

- **[Installation Guide](docs/installation.md)** - Setup and dependencies
- **[Usage Guide](docs/usage.md)** - Commands, examples, and workflows
- **[Configuration Guide](docs/configuration.md)** - All configuration options
- **[Metadata Guide](docs/metadata.md)** - Metadata management and Audible integration
- **[Troubleshooting Guide](docs/troubleshooting.md)** - Common issues and solutions

### Quick Reference

**Essential Commands:**

```bash
audiobook-forge build                    # Convert audiobook (auto-detect)
audiobook-forge build --root /path       # Convert with explicit path
audiobook-forge organize --root /path    # Organize library
audiobook-forge match --file book.m4b    # Interactive matching
audiobook-forge metadata fetch --asin ID # Fetch Audible metadata
audiobook-forge config show              # Show configuration
audiobook-forge check                    # Verify dependencies
```

**Key Configuration:**

```yaml
performance:
  enable_parallel_encoding: true  # 3.8x faster encoding
  max_concurrent_encodes: "auto"

metadata:
  auto_extract_cover: true  # Auto-extract embedded cover art (v2.8.0)
  audible:
    enabled: false
    region: "us"
```

See [docs/configuration.md](docs/configuration.md) for complete options.

---

## 📊 Performance

### Benchmarks

**Parallel File Encoding (v2.1.0+):**

| Mode | Time | CPU Usage | Speedup |
|------|------|-----------|---------|
| Serial encoding | 121.5s | 13% | Baseline |
| Parallel encoding | 32.1s | 590% | **3.8x faster** 🚀 |

*Test: 10-file audiobook (~276MB) on 8-core CPU*

**Overall Performance vs Python:**

| Operation | Python | Rust v2.0 | Rust v2.1+ | Speedup |
|-----------|--------|-----------|------------|---------|
| Startup | ~500ms | ~10ms | ~10ms | **50x** |
| Single book (copy) | 45s | 12s | 12s | **3.8x** |
| Single book (transcode) | 180s | 65s | 17s | **10.6x** 🚀 |
| Batch (10 books) | 25m | 8m | 2.5m | **10x** 🚀 |
| Memory | ~250 MB | ~25 MB | ~25 MB | **10x less** |

### Performance Tips

1. **Enable parallel encoding** (default in v2.1.0+)
2. **Use parallel workers**: `--parallel 4` or more
3. **Enable copy mode**: Automatic for M4A/AAC files
4. **Use SSD storage**: Faster I/O for large libraries
5. **Apple Silicon**: Hardware acceleration with `aac_at` encoder

---

## 🎯 Supported Formats

### Input

- **MP3** (`.mp3`) - MPEG Audio Layer III
- **M4A** (`.m4a`) - MPEG-4 Audio
- **AAC** (`.aac`) - Advanced Audio Coding

### Output

- **M4B** (`.m4b`) - MPEG-4 Audiobook with embedded chapters and metadata

### Metadata Sources

- **Local**: ID3 tags, M4A atoms, CUE sheets, filenames, folder names
- **Auto-Extract**: Embedded cover art from MP3/M4A files (v2.8.0)
- **Audible**: Comprehensive metadata from 10 regional stores (v2.2.0)
- **Interactive**: Fuzzy matching with confidence scoring (v2.3.0+)

---

## ⚙️ Configuration

Configuration file: `~/.config/audiobook-forge/config.yaml`

```bash
# Initialize default configuration
audiobook-forge config init

# Show current configuration
audiobook-forge config show

# Edit configuration
audiobook-forge config edit
```

**Example configuration:**

```yaml
performance:
  enable_parallel_encoding: true
  max_concurrent_encodes: "auto"

processing:
  parallel_workers: 4
  skip_existing: true

metadata:
  auto_extract_cover: true
  audible:
    enabled: false
    region: "us"
    download_covers: true
```

**📖 Complete configuration reference**: See [docs/configuration.md](docs/configuration.md)

---

## 🔧 Common Issues

### FFmpeg not found

```bash
brew install ffmpeg                    # macOS
sudo apt install ffmpeg                # Ubuntu/Debian
```

### Permission denied

```bash
chmod -R u+rw /path/to/audiobooks
```

### Out of memory

```bash
audiobook-forge build --root /path --parallel 1
```

**📖 Complete troubleshooting guide**: See [docs/troubleshooting.md](docs/troubleshooting.md)

---

## 🤝 Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Start for Contributors

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Run linter: `cargo clippy`
6. Format code: `cargo fmt`
7. Commit: `git commit -m "feat: add my feature"`
8. Push and open a Pull Request

---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

**MIT License Summary:**

- ✅ Commercial use
- ✅ Modification
- ✅ Distribution
- ✅ Private use
- ⚠️ Liability and warranty disclaimer

---

## 🙏 Acknowledgments

- **Original Python version**: This Rust rewrite delivers 3-4x better performance
- **FFmpeg**: The backbone of audio processing
- **Rust community**: For excellent crates and tooling
- **Contributors**: Thanks to all who have contributed

### Built With

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Tokio](https://tokio.rs/) - Async runtime
- [Clap](https://github.com/clap-rs/clap) - CLI framework
- [FFmpeg](https://ffmpeg.org/) - Audio/video processing
- [AtomicParsley](https://github.com/wez/atomicparsley) - Metadata embedding
- [MP4Box/GPAC](https://github.com/gpac/gpac) - MP4 container tools

---

## 📞 Support

- **Documentation**: [docs/](docs/) folder
- **Issues**: [GitHub Issues](https://github.com/juanra/audiobook-forge/issues)
- **Discussions**: [GitHub Discussions](https://github.com/juanra/audiobook-forge/discussions)

---

Made with ❤️ and 🦀 (Rust)
