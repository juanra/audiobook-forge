# Audiobook Forge 🎧

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-77%20passing-brightgreen.svg)](tests/)

A fast, multi-process Rust CLI that orchestrates FFmpeg to convert audiobook directories into single M4B files with chapters and metadata.

## Table of Contents

- [Why Audiobook Forge?](#why-audiobook-forge)
- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Advanced Features](#advanced-features)
- [Performance](#performance)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [Support](#support)
- [License](#license)

---

## Why Audiobook Forge?

Audiobooks often come as dozens of separate MP3 files. While that works, managing a library is much easier when each audiobook is a single **M4B file** (MPEG-4 Audiobook) — the standard format for audiobook players.

Audiobook Forge takes those scattered files and produces one M4B with embedded chapters, metadata, and cover art.

- **One file per book** — simplified library management and transfers
- **Chapter markers** — jump between sections, resume where you left off
- **Full metadata** — title, author, narrator, cover art, all embedded
- **Universal playback** — Apple Books, Audiobookshelf, Plex, and most players

---

## Features

### Performance
- **Multi-process encoding** — encode files concurrently across all CPU cores (3.8x faster)
- **Parallel book processing** — convert multiple audiobooks simultaneously
- **Copy mode** — lossless concatenation without re-encoding when possible
- **M4B merge** — combine multiple M4B files without re-encoding (v2.9.1)

### Audio Processing
- **Smart quality detection** — automatically matches source audio quality
- **Chapter generation** — from files, CUE sheets, text files, EPUB, or Audnex API
- **Chapter updates** — replace generic names with meaningful titles (v2.9.0)
- **Cover art extraction** — pulls embedded artwork from source files (v2.8.0)

### Metadata
- **Full tag preservation** — artist, album artist, composer, comment, genre, year
- **Audible integration** — fetch metadata from Audible's catalog across 10 regions (v2.2.0)
- **Interactive matching** — BEETS-inspired fuzzy matching with confidence scoring (v2.3.0)

### Workflow
- **Auto-detect** — run from inside an audiobook folder, no flags needed
- **Batch operations** — process entire libraries with a single command
- **Error recovery** — automatic retry with configurable settings
- **Progress tracking** — real-time progress with ETA
- **YAML configuration** — with CLI overrides for everything

---

## Installation

### Prerequisites

**Rust 1.85 or later** is required. Distro-packaged Rust (e.g., Ubuntu 24.04 ships 1.75) is often too old — install via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install

```bash
cargo install audiobook-forge
```

### Runtime Dependencies

Audiobook Forge wraps these tools — install them for your platform:

**macOS:**
```bash
brew install ffmpeg atomicparsley gpac
```

**Ubuntu/Debian:**
```bash
sudo apt install ffmpeg atomicparsley gpac
```

**Fedora/RHEL:**
```bash
sudo dnf install ffmpeg atomicparsley gpac
```

### Verify

```bash
audiobook-forge check
```

See [docs/installation.md](docs/installation.md) for detailed setup instructions.

---

## Quick Start

### Convert a Single Audiobook

```bash
# Run from inside an audiobook folder
cd "/path/to/My Audiobook"
audiobook-forge build

# Or specify the path
audiobook-forge build --root "/path/to/My Audiobook"
```

### Batch Process

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
audiobook-forge match --file "Book.m4b"
audiobook-forge match --dir /path/to/m4b/files
```

See [docs/usage.md](docs/usage.md) for the complete command reference.

---

## Advanced Features

### Chapter Updates (v2.9.0)

Replace generic chapter names ("Chapter 1", "Chapter 2") with meaningful titles:

```bash
# From Audnex API (Audible chapter data)
audiobook-forge metadata enrich --file "Book.m4b" --chapters-asin B08V3XQ7LK

# From text file (simple, timestamped, or MP4Box format)
audiobook-forge metadata enrich --file "Book.m4b" \
  --chapters chapters.txt --merge-strategy keep-timestamps

# From EPUB table of contents
audiobook-forge metadata enrich --file "Book.m4b" --chapters book.epub
```

**Merge strategies:** `keep-timestamps` (default for text/EPUB), `replace-all`, `skip-on-mismatch`, `interactive` (default).

### M4B Merge (v2.9.1)

Combine multiple M4B files into one without re-encoding:

```bash
# Auto-detect sequential parts
audiobook-forge build --root /path/to/book

# Force merge
audiobook-forge build --root /path/to/book --merge-m4b
```

Recognizes naming patterns like `Part 1`, `Disc 1`, `CD1`, `Book 01.m4b`, etc. Chapters are merged with adjusted timestamps and metadata is preserved.

---

## Performance

### Encoding Benchmarks

| Mode | Time | CPU Usage | Speedup |
|------|------|-----------|---------|
| Serial encoding | 121.5s | 13% | Baseline |
| Parallel encoding | 32.1s | 590% | **3.8x** |

*10-file audiobook (~276MB) on 8-core CPU*

### vs Python (Original Version)

| Operation | Python | Rust (parallel) | Speedup |
|-----------|--------|-----------------|---------|
| Startup | ~500ms | ~10ms | **50x** |
| Single book (copy) | 45s | 12s | **3.8x** |
| Single book (transcode) | 180s | 17s | **10.6x** |
| Batch (10 books) | 25m | 2.5m | **10x** |
| Memory | ~250 MB | ~25 MB | **10x less** |

**Tips:** Enable parallel encoding (default), use `--parallel 4+` for batch jobs, use SSD storage, Apple Silicon benefits from hardware `aac_at` encoder.

---

## Documentation

- [Installation Guide](docs/installation.md) — setup and dependencies
- [Usage Guide](docs/usage.md) — commands, examples, and workflows
- [Configuration Guide](docs/configuration.md) — all YAML and CLI options
- [Metadata Guide](docs/metadata.md) — metadata management and Audible integration
- [Troubleshooting](docs/troubleshooting.md) — common issues and solutions

---

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Run linter: `cargo clippy`
6. Format code: `cargo fmt`
7. Commit: `git commit -m "feat: add my feature"`
8. Push and open a Pull Request

---

## Support

- **Issues**: [GitHub Issues](https://github.com/juanra/audiobook-forge/issues)
- **Discussions**: [GitHub Discussions](https://github.com/juanra/audiobook-forge/discussions)

If you find Audiobook Forge useful, consider sponsoring its development:

[![Sponsor](https://img.shields.io/badge/Sponsor-❤-ea4aaa?style=for-the-badge&logo=github-sponsors)](https://github.com/sponsors/juanra)

---

## License

MIT License — see [LICENSE](LICENSE) for details.

## Acknowledgments

Built with [Rust](https://www.rust-lang.org/), [Tokio](https://tokio.rs/), [Clap](https://github.com/clap-rs/clap), [FFmpeg](https://ffmpeg.org/), [AtomicParsley](https://github.com/wez/atomicparsley), and [MP4Box/GPAC](https://github.com/gpac/gpac).
