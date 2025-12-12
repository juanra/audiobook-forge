# AGENTS.md - Audiobook Forge (Rust)

## Project Overview

Audiobook Forge is a blazing-fast CLI tool for converting audiobook directories to M4B format with chapters and metadata. This is a complete Rust rewrite of the original Python version, delivering 3-4x faster performance with true multi-core utilization.

**Key Capabilities**:
- Parallel processing of multiple audiobooks
- Smart quality detection and preservation
- Chapter generation from multiple sources (files, CUE sheets, auto-detection)
- Metadata extraction and enhancement (ID3, M4A tags)
- Cover art detection and embedding
- Batch operations on entire libraries
- Error recovery with automatic retry
- Hardware acceleration support (Apple Silicon aac_at encoder)

**Project Status**: All 6 development phases complete. See `docs/development/phases/` for detailed implementation history.

---

## Build & Test Commands

### Building
```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Binary location
./target/debug/audiobook-forge    # debug
./target/release/audiobook-forge  # release
```

### Testing
```bash
# Run all tests
cargo test

# Run library tests only
cargo test --lib

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Running
```bash
# Check dependencies
cargo run -- check

# Process audiobooks
cargo run -- build --root /path/to/audiobooks --parallel 4

# Organize library
cargo run -- organize --root /path/to/audiobooks

# Config management
cargo run -- config init
cargo run -- config show
cargo run -- config validate
```

---

## Architecture

### Module Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library root
├── models/              # Core data structures
│   ├── book.rs          # BookFolder, BookCase (A/B/C/D classification)
│   ├── track.rs         # Audio track representation
│   ├── quality.rs       # QualityProfile (bitrate, codec, etc.)
│   ├── config.rs        # Configuration structures
│   └── result.rs        # ProcessingResult
├── utils/               # Utilities
│   ├── config.rs        # ConfigManager (load/save YAML)
│   ├── validation.rs    # DependencyChecker (FFmpeg, etc.)
│   └── sorting.rs       # Natural sorting (track1, track2, track10)
├── cli/                 # CLI interface
│   └── commands.rs      # Clap command definitions
├── audio/               # Audio operations
│   ├── ffmpeg.rs        # FFmpeg wrapper (probe, concat, transcode)
│   ├── metadata.rs      # Metadata extraction (ID3, M4A)
│   ├── chapters.rs      # Chapter generation
│   └── cue.rs           # CUE file parsing
└── core/                # Core processing logic
    ├── processor.rs     # Single book processing
    ├── parallel.rs      # Parallel batch processing
    ├── organizer.rs     # Library organization
    └── progress.rs      # Progress tracking (indicatif)
```

### Key Concepts

**BookCase Classification**:
- **Case A**: Already M4B (skip)
- **Case B**: MP3 files only (requires conversion)
- **Case C**: M4A files only (can use fast concat)
- **Case D**: Mixed MP3/M4A (requires transcode to AAC)

**Quality Profiles**: Automatically detect best quality across tracks and preserve it. Support compatibility checking for concatenation.

**Processing Modes**:
- Copy mode: Ultra-fast concatenation without re-encoding (when compatible)
- Transcode mode: Convert to AAC when necessary

---

## Code Style & Conventions

### Rust Patterns
- Use `anyhow::Result<T>` for application errors
- Use `thiserror` for custom error types
- Prefer `?` operator over explicit error handling
- Use `.context()` to add error context
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)

### Error Handling
```rust
// Good - provides context
let tracks = discover_tracks(&path)
    .context(format!("Failed to discover tracks in {}", path.display()))?;

// Avoid - no context
let tracks = discover_tracks(&path)?;
```

### Async/Await
- Use tokio runtime for async operations
- FFmpeg calls use `tokio::process::Command`
- Parallel processing uses `tokio::spawn` with semaphore for resource limits

### Testing
- Unit tests in same file as implementation (`#[cfg(test)] mod tests`)
- Integration tests in `tests/` directory
- Use `tempfile` for temp directories in tests
- Mock external dependencies when possible

---

## Testing Guidelines

### Running Tests
All tests should pass before committing:
```bash
cargo test --lib
cargo test --all
```

### Writing Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Arrange
        let input = "value";

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_function() {
        // For async tests
    }
}
```

### Test Coverage
- All public functions should have tests
- Edge cases should be covered (empty input, invalid data, etc.)
- Quality profile comparison logic is critical (heavily tested)

---

## Dependencies

### External Tools (Required)
The application requires these external tools to be installed:

```bash
# macOS
brew install ffmpeg atomicparsley gpac

# Ubuntu/Debian
apt install ffmpeg atomicparsley gpac
```

Validation: Use `audiobook-forge check` to verify installation.

### Rust Crates

**Core**:
- `clap 4.5` - CLI framework with derive API
- `tokio 1.35` - Async runtime
- `serde 1.0` + `serde_yaml 0.9` - Config serialization
- `anyhow 1.0` + `thiserror 1.0` - Error handling

**Audio/Metadata**:
- `id3 1.13` - MP3 metadata reading
- `mp4ameta 0.11` - M4B/M4A metadata reading

**Utilities**:
- `indicatif 0.17` - Progress bars
- `walkdir 2.4` - Directory traversal
- `natord 1.0` - Natural sorting
- `tracing 0.1` - Structured logging
- `which 6.0` - Executable lookup
- `dirs 5.0` - Config directory paths
- `chrono 0.4` - Date/time handling

**Testing**:
- `tempfile 3.8` - Temp files/directories
- `assert_cmd 2.0` - CLI testing
- `predicates 3.0` - Test assertions

---

## Phase History

The project was built in 6 phases. Full documentation available in `docs/development/phases/`:

1. **Phase 1 - Foundation** (`01-foundation.md`): Data models, CLI structure, config system
2. **Phase 2 - Audio Operations** (`02-audio-ops.md`): FFmpeg integration, metadata extraction
3. **Phase 3 - Parallel Processing** (`03-parallel.md`): Multi-core batch processing
4. **Phase 4 - Progress & Polish** (`04-progress.md`): Progress bars, error recovery
5. **Phase 5 - Organization** (`05-organize.md`): Library organization features
6. **Phase 6 - Final Polish** (`06-polish.md`): Performance optimization, final testing

**Current Status**: All phases complete, 77 tests passing.

---

## Common Tasks

### Adding a New CLI Command
1. Define command in `src/cli/commands.rs` using clap derive
2. Add handler function in `main.rs`
3. Implement logic in appropriate module
4. Add tests
5. Update help text

### Adding a New Audio Operation
1. Add function to `src/audio/ffmpeg.rs` or related module
2. Use `tokio::process::Command` for FFmpeg calls
3. Parse output with regex if needed
4. Add error handling with context
5. Write unit tests with temp files

### Modifying Configuration
1. Update structs in `src/models/config.rs`
2. Add serde defaults if needed
3. Update `templates/config.yaml` template
4. Update validation in `src/utils/config.rs`
5. Test serialization/deserialization

### Performance Optimization
- Profile with `cargo flamegraph` or similar tools
- Check parallel worker count (default: CPU cores / 2)
- Verify copy mode is used when possible (no transcoding)
- Monitor memory usage during batch operations

---

## Development Environment

### Recommended Tools
- `rust-analyzer` - LSP for IDE integration
- `cargo-watch` - Auto-rebuild on file changes
- `cargo-clippy` - Linting
- `cargo-fmt` - Code formatting

### Configuration Location
- Config file: `~/.config/audiobook-forge/config.yaml`
- Logs: stdout/stderr (structured with tracing)

### Platform Notes
- **macOS**: Apple Silicon encoder (`aac_at`) is automatically used when available
- **Linux**: Standard AAC encoder (`libfdk_aac` or `aac`)
- FFmpeg must be compiled with AAC support

---

## Commit & PR Guidelines

### Commit Messages
Follow conventional commits format:
```
<type>(<scope>): <description>

[optional body]
[optional footer]
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

Examples:
```
feat(audio): add CUE file parsing support
fix(metadata): handle missing album tag gracefully
docs(readme): update installation instructions
test(quality): add compatibility check tests
```

### Before Committing
1. Run tests: `cargo test --all`
2. Run clippy: `cargo clippy -- -D warnings`
3. Format code: `cargo fmt`
4. Update CHANGELOG.md if needed
5. Ensure binary builds: `cargo build --release`

---

## Security Considerations

- **Path Traversal**: Validate all user-provided paths
- **Command Injection**: Never interpolate user input into shell commands
  - Use `Command::arg()` instead of string interpolation
  - FFmpeg args are always passed as separate arguments
- **Resource Limits**: Semaphore limits concurrent FFmpeg processes
- **Temporary Files**: Always clean up temp files (use `tempfile` crate)

---

## Troubleshooting

### Common Issues

**FFmpeg not found**:
- Run `audiobook-forge check`
- Ensure FFmpeg is in PATH
- On macOS: `brew install ffmpeg`

**Tests failing**:
- Check external dependencies are installed
- Verify FFmpeg version is recent enough
- Check temp directory permissions

**Slow processing**:
- Check parallel worker count: `--parallel N`
- Verify copy mode is being used (check logs)
- Profile with release build, not debug

**Metadata extraction fails**:
- Ensure `atomicparsley` and `MP4Box` are installed
- Check file permissions
- Verify audio files are not corrupted

---

## Additional Resources

- Main documentation: `docs/`
- Phase implementation details: `docs/development/phases/`
- Specifications: `docs/specs/` (for future requirements)
- User guides: `docs/guides/`
- Changelog: `CHANGELOG.md` (in root)
