# Installation

## 1. Install Runtime Dependencies

Audiobook Forge wraps these tools — install them first.

**macOS:**
```bash
brew install ffmpeg atomicparsley
```

**Ubuntu/Debian:**
```bash
sudo apt install ffmpeg atomicparsley
```

**Fedora/RHEL:**
```bash
sudo dnf install ffmpeg atomicparsley
```

**Windows:**
1. **FFmpeg** — download from [ffmpeg.org](https://ffmpeg.org/download.html), add to PATH
2. **AtomicParsley** — download from [releases](https://github.com/wez/atomicparsley/releases)

## 2. Install Rust

**Rust 1.88+** is required. Distro-packaged Rust (e.g., Ubuntu 24.04 ships 1.75) is often too old; install via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 3. Install Audiobook Forge

### From crates.io (recommended)

```bash
cargo install --locked audiobook-forge
```

`--locked` builds with the dependency versions the release was tested with. Without it, Cargo may pick newer dependencies that need a newer Rust.

### Latest development version

```bash
cargo install --git https://github.com/juanra/audiobook-forge
```

### From source

```bash
git clone https://github.com/juanra/audiobook-forge
cd audiobook-forge
cargo build --release
cargo install --path .
```

## 4. Verify

```bash
audiobook-forge check
```

Expected output:

```
→ Checking system dependencies...

  ✓ FFmpeg
    AAC Encoders: aac_at (selected), aac
  ✓ AtomicParsley

✓ All dependencies found
```

The encoder list depends on your FFmpeg build and platform (`aac_at` appears only on macOS). An INFO log line naming the detected encoder may also be printed.

## Updating

```bash
cargo install --locked audiobook-forge --force
```

## Uninstalling

```bash
cargo uninstall audiobook-forge
```

If you run into issues, see [Troubleshooting](Troubleshooting).
