# Installation

This guide will help you install Audiobook Forge and its required dependencies.

## Prerequisites

Before installing Audiobook Forge, you need:

- **Rust 1.75+**: Install from [rustup.rs](https://rustup.rs/)
- **FFmpeg**: Required for audio processing
- **AtomicParsley**: Required for metadata embedding
- **MP4Box** (from GPAC): Required for M4B container creation

## Installing Audiobook Forge

### Via Cargo (Recommended)

The easiest way to install Audiobook Forge is using Cargo:

```bash
cargo install --git https://github.com/juanra/audiobook-forge
```

This will download, compile, and install the latest version from the repository.

### From Source

If you want to build from source or contribute to development:

```bash
# Clone the repository
git clone https://github.com/juanra/audiobook-forge
cd audiobook-forge

# Build and install
cargo build --release
cargo install --path .

# Or just build (binary will be at: target/release/audiobook-forge)
cargo build --release
```

## Installing Dependencies

Audiobook Forge requires three external tools to function properly.

### macOS

Use Homebrew to install all dependencies:

```bash
brew install ffmpeg atomicparsley gpac
```

### Ubuntu/Debian

Use apt to install the dependencies:

```bash
sudo apt update
sudo apt install ffmpeg atomicparsley gpac
```

### Fedora/RHEL

Use dnf to install the dependencies:

```bash
sudo dnf install ffmpeg atomicparsley gpac
```

### Windows

For Windows users:

1. **FFmpeg**: Download from [ffmpeg.org](https://ffmpeg.org/download.html) and add to PATH
2. **AtomicParsley**: Download from [AtomicParsley releases](https://github.com/wez/atomicparsley/releases)
3. **MP4Box/GPAC**: Download from [GPAC](https://gpac.wp.imt.fr/downloads/)

## Verifying Installation

After installing Audiobook Forge and its dependencies, verify everything is working:

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

If any dependencies are missing, you'll see an error message indicating which tool needs to be installed.

## Troubleshooting Installation

### Rust Not Installed

If you don't have Rust installed:

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then restart your terminal

# Verify installation
rustc --version
cargo --version
```

### FFmpeg Not Found

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
ffmpeg -version
```

### AtomicParsley Not Found

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

# Verify
AtomicParsley --version
```

### MP4Box Not Found

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

# Verify
MP4Box -version
```

### Permission Issues

If you encounter permission errors during installation:

```bash
# For cargo install issues, ensure you have write access to cargo's bin directory
# Usually located at: ~/.cargo/bin

# For system package managers (apt, dnf), use sudo:
sudo apt install ffmpeg atomicparsley gpac
```

## Updating Audiobook Forge

To update to the latest version:

```bash
# If installed via cargo install
cargo install --git https://github.com/juanra/audiobook-forge --force

# If built from source
cd audiobook-forge
git pull
cargo build --release
cargo install --path . --force
```

## Uninstalling

To uninstall Audiobook Forge:

```bash
cargo uninstall audiobook-forge
```

This will remove the binary from `~/.cargo/bin/`. The dependencies (FFmpeg, AtomicParsley, MP4Box) can be uninstalled separately if desired.

## Next Steps

Once installed, check out:

- [Usage Guide](usage.md) - Learn how to use Audiobook Forge
- [Configuration](configuration.md) - Customize settings
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
