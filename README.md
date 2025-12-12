# Audiobook Forge 🎧

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-77%20passing-brightgreen.svg)](tests/)

A blazing-fast CLI tool for converting audiobook directories to M4B format with chapters and metadata. Written in Rust for maximum performance and reliability.

## 📑 Table of Contents

- [Why Audiobook Forge?](#-why-audiobook-forge)
- [Features](#-features)
- [Installation](#-installation)
- [Dependencies](#-dependencies)
- [Usage](#-usage)
  - [Quick Start](#quick-start)
  - [Command Reference](#command-reference)
  - [Usage Examples](#usage-examples)
- [Configuration](#-configuration)
- [Input Directory Structure](#-input-directory-structure)
- [Supported Formats](#-supported-formats)
- [Performance](#-performance)
- [Troubleshooting](#-troubleshooting)
- [Contributing](#-contributing)
- [License](#-license)
- [Acknowledgments](#-acknowledgments)

---

## 🎯 Why Audiobook Forge?

When downloading audiobooks, they often come as **multiple separate MP3 files** — one for each chapter or section. While this works perfectly fine with various audiobook players like AudiobookShelf and others, managing your audiobook library becomes significantly easier when **each audiobook is consolidated into a single file**.

**Audiobook Forge** solves this problem by taking those scattered audio files and merging them into a single **M4B file** (MPEG-4 Audiobook format), which has become the standard for audiobook applications and offers superior portability and management.

### 📚 Single File vs. Multiple Files: The Benefits

**With a single M4B file, you get:**

✅ **Simplified Library Management**
- One file per audiobook instead of dozens or hundreds of individual files
- Easier to organize, rename, move, and backup
- Cleaner directory structure

✅ **Better Metadata & Chapter Support**
- Embedded chapter markers for easy navigation
- Complete metadata (title, author, narrator, cover art) in one place
- Chapter information persists across devices and players

✅ **Improved Portability**
- Transfer entire audiobooks with a single file copy
- Sync seamlessly across devices (phone, tablet, computer)
- No risk of missing files or broken chapter sequences

✅ **Enhanced Playback Experience**
- Resume exactly where you left off, even across different apps
- Navigate chapters with built-in chapter markers
- Cover art displays properly in all compatible players

✅ **Reduced Storage Overhead**
- Eliminates file system overhead from multiple small files
- Efficient compression while preserving audio quality
- Optional copy mode for instant merging without re-encoding

✅ **Universal Compatibility**
- M4B format supported by Apple Books, Audiobookshelf, Plex, and most modern audiobook players
- Works across iOS, Android, macOS, Windows, and Linux
- Industry-standard format for audiobook distribution

---

## ✨ Features

- **🚀 Parallel Processing**: Convert multiple audiobooks simultaneously with intelligent resource management
- **🎯 Smart Quality Detection**: Automatically detects and preserves the best audio quality
- **📖 Chapter Generation**: Multiple sources (files, CUE sheets, auto-detection)
- **🎨 Metadata Management**: Extracts and enhances metadata from ID3 and M4A tags
- **🖼️ Cover Art**: Automatically detects and embeds cover images
- **🔄 Batch Operations**: Process entire libraries with a single command
- **⚡ Copy Mode**: Ultra-fast concatenation without re-encoding when possible
- **🍎 Hardware Acceleration**: Apple Silicon encoder support (aac_at)
- **🔁 Error Recovery**: Automatic retry with smart error classification
- **📊 Progress Tracking**: Real-time progress with ETA calculation
- **🗂️ Auto-Organization**: Organize books into M4B and To_Convert folders
- **⚙️ Configuration**: YAML-based configuration with CLI overrides

---

## 📦 Installation

### Prerequisites

- **Rust 1.75+**: Install from [rustup.rs](https://rustup.rs/)
- **FFmpeg**: Required for audio processing
- **AtomicParsley**: Required for metadata embedding
- **MP4Box** (from GPAC): Required for M4B container creation

### Via Cargo (Recommended)

```bash
cargo install --git https://github.com/juanra/audiobook-forge
```

### From Source

```bash
# Clone the repository
git clone https://github.com/juanra/audiobook-forge
cd audiobook-forge

# Build and install
cargo build --release
cargo install --path .

# Or just build (binary at: target/release/audiobook-forge)
cargo build --release
```

---

## 🔧 Dependencies

Install the required external tools:

### macOS

```bash
brew install ffmpeg atomicparsley gpac
```

### Ubuntu/Debian

```bash
sudo apt update
sudo apt install ffmpeg atomicparsley gpac
```

### Fedora/RHEL

```bash
sudo dnf install ffmpeg atomicparsley gpac
```

### Verify Installation

```bash
audiobook-forge check
```

**Expected output:**

```
→ Checking system dependencies...

  ✓ FFmpeg
  ✓ AtomicParsley
  ✓ MP4Box

✓ All dependencies found
```

---

## 🚀 Usage

### Quick Start

```bash
# Process a single audiobook directory
audiobook-forge build --root "/path/to/My Audiobook"

# Process multiple audiobooks in parallel
audiobook-forge build --root "/path/to/audiobooks" --parallel 4

# Organize library (move M4B files to M4B/ folder)
audiobook-forge organize --root "/path/to/audiobooks"
```

### Command Reference

#### `build` - Convert audiobooks to M4B

```bash
audiobook-forge build [OPTIONS] --root <PATH>
```

**Options:**
- `--root <PATH>` - Root directory containing audiobook(s) (required)
- `--parallel <N>` - Number of parallel workers (default: CPU cores / 2)
- `--skip-existing` - Skip audiobooks that already have M4B files (default: true)
- `--quality <PRESET>` - Quality preset: low, medium, high, source (default: source)
- `--output <PATH>` - Output directory (default: same as source)
- `-v, --verbose` - Verbose logging

#### `organize` - Organize audiobook library

```bash
audiobook-forge organize [OPTIONS] --root <PATH>
```

**Options:**
- `--root <PATH>` - Root directory to organize (required)
- `--dry-run` - Show what would be done without making changes
- `-v, --verbose` - Verbose logging

#### `config` - Manage configuration

```bash
audiobook-forge config <SUBCOMMAND>
```

**Subcommands:**
- `init` - Create default config file
- `show` - Display current configuration
- `validate` - Validate config file
- `path` - Show config file location
- `edit` - Open config in default editor

#### `check` - Verify dependencies

```bash
audiobook-forge check
```

Checks for FFmpeg, AtomicParsley, and MP4Box installation.

#### `--version` - Show version

```bash
audiobook-forge --version
```

---

### Usage Examples

#### Example 1: Convert a Single Audiobook

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
Audiobook Forge v2.0.0

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

**Result:**

```
/audiobooks/The Great Gatsby/
├── 01 - Chapter 1.mp3
├── 02 - Chapter 2.mp3
├── 03 - Chapter 3.mp3
├── cover.jpg
├── info.txt
└── The Great Gatsby.m4b  ← New file
```

---

#### Example 2: Batch Convert Multiple Audiobooks

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
Audiobook Forge v2.0.0

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

#### Example 3: Organize Library

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

#### Example 4: Configuration Management

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
directories:
  source: null
  output: same_as_source

processing:
  parallel_workers: 4
  skip_existing: true
  quality_preset: source
  use_copy_mode: true

metadata:
  extract_from_files: true
  prefer_embedded: true
  fallback_to_folder_name: true

chapters:
  generate_from_files: true
  parse_cue_files: true
  auto_chapters: false

output:
  format: m4b
  naming_pattern: "{title}"
  include_metadata: true

performance:
  ffmpeg_threads: 0
  buffer_size: 8192
```

---

## ⚙️ Configuration

Configuration file location: `~/.config/audiobook-forge/config.yaml`

### Key Settings

```yaml
processing:
  parallel_workers: 4        # Concurrent audiobooks to process
  skip_existing: true        # Skip if M4B already exists
  quality_preset: source     # Quality: low, medium, high, source
  use_copy_mode: true        # Use fast copy mode when possible

metadata:
  extract_from_files: true   # Extract metadata from audio files
  prefer_embedded: true      # Prefer embedded tags over filenames

chapters:
  generate_from_files: true  # Create chapters from individual files
  parse_cue_files: true      # Parse .cue files for chapters
  auto_chapters: false       # Auto-detect chapter markers

output:
  naming_pattern: "{title}"  # Options: {title}, {author}, {year}
  include_metadata: true     # Embed metadata in output
```

**Override config with CLI flags:**

```bash
# Override parallel workers
audiobook-forge build --root /path --parallel 8

# Override quality
audiobook-forge build --root /path --quality high
```

---

## 📁 Input Directory Structure

Audiobook Forge automatically detects and processes various directory structures:

### Supported Structures

✅ **Flat structure** (all files in one folder)
```
My Audiobook/
├── 01.mp3
├── 02.mp3
├── 03.mp3
└── cover.jpg
```

✅ **Nested chapters** (subfolders)
```
My Audiobook/
├── Part 1/
│   ├── Chapter 01.mp3
│   └── Chapter 02.mp3
└── Part 2/
    ├── Chapter 03.mp3
    └── Chapter 04.mp3
```

✅ **With CUE file** (single audio + chapters)
```
My Audiobook/
├── audiobook.mp3
├── audiobook.cue
└── cover.png
```

✅ **Mixed formats** (MP3 + M4A)
```
My Audiobook/
├── intro.mp3
├── chapter1.m4a
├── chapter2.m4a
└── outro.mp3
```

### Required Files

- **Audio files**: At least one `.mp3` or `.m4a` file
- **Cover art** (optional): `cover.jpg`, `cover.png`, `folder.jpg`, etc.
- **CUE file** (optional): `*.cue` for chapter information

---

## 🎵 Supported Formats

### Input Audio Formats

- **MP3** (`.mp3`) - MPEG Audio Layer III
- **M4A** (`.m4a`) - MPEG-4 Audio
- **AAC** (`.aac`) - Advanced Audio Coding

### Output Format

- **M4B** (`.m4b`) - MPEG-4 Audiobook with embedded chapters and metadata

### Metadata Sources

- **ID3 tags** (MP3 files)
- **M4A atoms** (M4A/M4B files)
- **CUE sheets** (`.cue` files)
- **Filenames** (fallback)
- **Folder names** (fallback)

---

## 📊 Performance

### Benchmarks

| Operation | Python Version | Rust Version | Speedup |
|-----------|---------------|--------------|---------|
| Startup time | ~500ms | ~10ms | **50x faster** |
| Single book (copy mode) | 45s | 12s | **3.8x faster** |
| Single book (transcode) | 180s | 65s | **2.8x faster** |
| Batch (10 books, parallel) | 25m | 8m | **3.1x faster** |
| Memory usage | ~250 MB | ~25 MB | **10x less** |

### Performance Tips

1. **Use parallel processing**: `--parallel 4` (or more based on CPU cores)
2. **Enable copy mode**: Automatic when input is already AAC/M4A
3. **Use SSD storage**: Significantly faster I/O for large libraries
4. **Apple Silicon**: Automatic hardware acceleration with `aac_at` encoder
5. **Skip existing**: Use `--skip-existing` for incremental processing

---

## 🔧 Troubleshooting

### Common Issues

#### FFmpeg not found

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

# Verify
audiobook-forge check
```

---

#### Permission denied

**Error:**
```
Error: Permission denied (os error 13)
```

**Solution:**
```bash
# Check file permissions
ls -la /path/to/audiobooks

# Fix permissions
chmod -R u+rw /path/to/audiobooks
```

---

#### Out of memory (large libraries)

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
```

---

#### Quality is worse than original

**Issue:** Output file sounds compressed

**Solution:**
```bash
# Use source quality (default)
audiobook-forge build --root /path --quality source

# Check original quality
ffmpeg -i input.mp3
```

---

### Getting Help

- **Check logs**: Run with `--verbose` flag for detailed output
- **Verify dependencies**: `audiobook-forge check`
- **Report issues**: [GitHub Issues](https://github.com/juanra/audiobook-forge/issues)
- **Documentation**: See `docs/` folder for detailed guides

---

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Quick Start for Contributors

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Run linter: `cargo clippy`
6. Format code: `cargo fmt`
7. Commit: `git commit -m "feat: add my feature"`
8. Push: `git push origin feature/my-feature`
9. Open a Pull Request

---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

### MIT License Summary

- ✅ Commercial use
- ✅ Modification
- ✅ Distribution
- ✅ Private use
- ⚠️ Liability and warranty disclaimer

---

## 🙏 Acknowledgments

- **Original Python version**: This Rust rewrite is based on the original Python implementation, delivering 3-4x better performance
- **FFmpeg**: The backbone of audio processing
- **Rust community**: For excellent crates and tooling
- **Contributors**: Thanks to all who have contributed to this project

### Built With

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Tokio](https://tokio.rs/) - Async runtime
- [Clap](https://github.com/clap-rs/clap) - CLI framework
- [FFmpeg](https://ffmpeg.org/) - Audio/video processing
- [AtomicParsley](https://github.com/wez/atomicparsley) - Metadata embedding
- [MP4Box/GPAC](https://github.com/gpac/gpac) - MP4 container tools

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/juanra/audiobook-forge/issues)
- **Discussions**: [GitHub Discussions](https://github.com/juanra/audiobook-forge/discussions)
- **Documentation**: [`docs/`](docs/) folder

---

Made with ❤️ and 🦀 (Rust)
