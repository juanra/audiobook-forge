# Development Documentation

Documentation for developers working on Audiobook Forge.

## Phase Documentation

The `phases/` directory contains detailed documentation of all 6 development phases:

1. **[Phase 1 - Foundation](phases/01-foundation.md)** (Complete ✅)
   - Project structure and setup
   - Core data models (BookFolder, Track, QualityProfile, Config)
   - CLI framework with clap
   - Configuration system
   - Dependency checking
   - **19 tests passing**

2. **[Phase 2 - Audio Operations](phases/02-audio-ops.md)** (Complete ✅)
   - FFmpeg wrapper for probing, concatenation, transcoding
   - Metadata extraction (ID3 for MP3, M4A tags)
   - Chapter generation
   - CUE file parsing
   - **Additional tests added**

3. **[Phase 3 - Parallel Processing](phases/03-parallel.md)** (Complete ✅)
   - Multi-core batch processing
   - Tokio async runtime
   - Semaphore-based resource management
   - Concurrent audiobook processing
   - **Performance improvements**

4. **[Phase 4 - Progress & Polish](phases/04-progress.md)** (Complete ✅)
   - Progress bars with indicatif
   - Real-time ETA calculation
   - Error recovery and retry logic
   - Smart error classification
   - **User experience enhancements**

5. **[Phase 5 - Organization](phases/05-organize.md)** (Complete ✅)
   - Library organization features
   - Auto-organize into M4B/To_Convert folders
   - Batch operations
   - Directory management
   - **Library management features**

6. **[Phase 6 - Final Polish](phases/06-polish.md)** (Complete ✅)
   - Performance optimization
   - Final testing (77 tests passing)
   - Documentation completion
   - Production readiness
   - **Project completion**

## Architecture

### Module Overview

```
src/
├── models/      # Core data structures
├── utils/       # Utilities (config, validation, sorting)
├── cli/         # Command-line interface
├── audio/       # Audio operations (FFmpeg, metadata, chapters)
└── core/        # Processing logic (processor, parallel, organizer)
```

See `../AGENTS.md` for detailed architecture documentation.

## Testing

All tests should pass before committing:
```bash
cargo test --lib     # Library tests only
cargo test --all     # All tests including integration
```

Current status: **77 tests passing**

## Contributing

When contributing:
1. Read `../AGENTS.md` for coding conventions
2. Review relevant phase documentation to understand implementation
3. Add tests for new features
4. Update documentation as needed
5. Follow commit message conventions (see `../AGENTS.md`)

## Future Enhancements

See `../specs/` directory for planned features and specifications.
