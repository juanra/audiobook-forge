# Usage

## Quick Start

```bash
# Run from inside an audiobook folder (audio files directly in it,
# and no --root or directories.source configured)
cd "/path/to/My Audiobook"
audiobook-forge build

# Or specify the path
audiobook-forge build --root "/path/to/My Audiobook"

# Batch process with parallel workers
audiobook-forge build --root "/path/to/audiobooks" --out "/path/to/output" --parallel 4
```

---

## Command Reference

### `build` — Convert Audiobooks to M4B

```bash
audiobook-forge build [OPTIONS]
```

| Option | Description | Default |
|--------|-------------|---------|
| `-r, --root <PATH>` | Root directory containing audiobook folders. If it holds audio files directly, it is treated as a single book | `directories.source`, else the current folder (single book) |
| `-o, --out <PATH>` | Output directory. All M4B files are written flat into it | The `--root` folder (or `directories.output`) |
| `-j, --parallel <N>` | Number of audiobooks processed at once (1-8 on the command line; the config value is clamped to 1-16) | `2` (`processing.parallel_workers`) |
| `--force` | Reprocess folders that already contain M4B files | `false` |
| `--dry-run` | Analyze and list the books without creating files | `false` |
| `--quality <PRESET>` | Quality preset (see below) | `source` |
| `--aac-encoder <ENCODER>` | `auto`, `aac_at`, `libfdk_aac`, `aac`, `aac_mf` (see below) | `auto` (`advanced.aac_encoder`) |
| `--keep-temp` | Keep temporary files for debugging | `false` |
| `--fetch-audible` | Download the Audible cover during build (cover only, see below) | `false` |
| `--audible-region <REGION>` | Audible region code (an invalid value falls back to `us`) | `us` (`metadata.audible.region`) |
| `--audible-auto-match` | For folders without an ASIN, use the first Audible search result for the folder name (no confidence check) | `false` |
| `--merge-m4b` | Merge multiple M4B files even without a detected naming pattern (needs `--force`) | `false` |
| `-v, --verbose` | Verbose logging | `false` |

Folders that already contain an M4B file are skipped unless you pass `--force` (or set `processing.skip_existing: false` in the config).

**Quality presets:**

| Preset | Bitrate | Sample Rate | Channels |
|--------|---------|-------------|----------|
| `low` | 64kbps | 22050Hz | Mono |
| `medium` | 96kbps | 44100Hz | Stereo |
| `high` | 128kbps | 48000Hz | Stereo |
| `ultra` | 192kbps | 48000Hz | Stereo |
| `maximum` | 256kbps | 48000Hz | Stereo |
| `source` | Auto-detect from source files | Auto | Auto |

With `source` (or no preset), lossy sources keep their bitrate, lossless sources (FLAC, etc.) are capped at 192 kbps stereo or 128 kbps mono, and surround sources are downmixed to stereo.

**AAC encoders:** `auto` picks `aac_at` (Apple Silicon) when available, then `libfdk_aac`, then FFmpeg's native `aac`. `aac_mf` (Windows Media Foundation) is never auto-selected; use it only by passing it explicitly.

**`--fetch-audible`:** for each folder with an ASIN in its name (or a match from `--audible-auto-match`), the Audible cover is downloaded and embedded when the folder has no cover of its own. Audible tags are **not** written during build; use [`match`](#match--interactive-metadata-matching) or [`metadata enrich`](#metadata-enrich) on the finished M4B for that.

**Examples:**

```bash
# Auto-detect current directory
audiobook-forge build

# Preview what would be processed
audiobook-forge build --root /audiobooks --dry-run

# Batch into a separate output folder
audiobook-forge build --root /audiobooks --out /audiobooks-m4b --parallel 4

# Force high quality
audiobook-forge build --root /audiobooks --quality high
```

### How books are found and named

- Each folder under `--root` with audio files is one book. The output file is named after the folder (`<folder name>.m4b`), exactly as written, including any `[ASIN]` in the name.
- A book folder with no audio of its own whose only audio subfolder has a generic name (`MP3`, `M4A`, `M4B`, `AAC`, `FLAC`, `OGG`, `Audio`, `Audiobook`, `Files`, `Tracks`, or a lone `CD 1`/`Disc 1`) is named after the book folder, not the subfolder.
- Disc folders are folded into one book when the parent has no audio of its own and every subfolder is a disc/part folder of the same shape: `CD1`, `Disc 2`, `Disk 3`, `Part 4`, `Pt. 5`, the same wording behind a prefix shared by all siblings (`Title - Disc 01`), or bare numbers (`01`, `02`). The numbers must form a complete `1..n` set. Otherwise each subfolder stays a separate book (e.g. `Author/Book One`, `Author/Book Two`).
- All books in a run write into the same output directory. Books whose output names collide (case-insensitive) all fail up front with a message asking you to rename one of the folders; nothing is overwritten.
- Chapters are generated from the files (one chapter per track), or from a CUE sheet for single-file books, and injected with FFmpeg. The chapter count is verified in the finished file.

> **Tip:** use `--out` with a folder outside `--root`. With the default output, the M4B files land in the root itself, and a later `build --root` on that root sees audio directly in it and treats the whole library as a single book.

---

### `match` — Interactive Metadata Matching

Search Audible and interactively select the best metadata match for M4B files, using fuzzy scoring and confidence levels.

```bash
audiobook-forge match [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-f, --file <PATH>` | Match single M4B file |
| `-d, --dir <PATH>` | Match all M4B files directly in a directory (not recursive) |
| `--title <TITLE>` | Manual title override |
| `--author <AUTHOR>` | Manual author override |
| `--auto` | Apply the top-ranked match (non-interactive, regardless of confidence) |
| `--region <REGION>` | Audible region (default: `us`; an invalid value is an error) |
| `--keep-cover` | Keep existing cover art |
| `--dry-run` | Show matches without applying |

`--file` and `--dir` cannot be combined.

**Examples:**

```bash
audiobook-forge match --file "Book.m4b"
audiobook-forge match --dir /path/to/audiobooks --auto
audiobook-forge match --file "Book.m4b" --title "Title" --author "Author"
```

See [Metadata — Interactive Matching](Metadata#interactive-matching) for details on scoring and confidence levels.

---

### `metadata` — Fetch and Manage Metadata

```bash
audiobook-forge metadata <SUBCOMMAND>
```

#### `metadata fetch`

Fetch metadata from Audible by ASIN, or search by title and/or author (the top search result is used).

```bash
audiobook-forge metadata fetch --asin B00B5HZGUG
audiobook-forge metadata fetch --title "The Martian" --author "Andy Weir" --region us
audiobook-forge metadata fetch --asin B00B5HZGUG --output metadata.json
```

#### `metadata enrich`

Enrich an existing M4B file with Audible metadata and/or update chapters.

```bash
# Enrich with Audible metadata
audiobook-forge metadata enrich --file "Book.m4b" --asin B00B5HZGUG

# Auto-detect ASIN from the file path (file or folder name)
audiobook-forge metadata enrich --file "Book [B00B5HZGUG].m4b" --auto-detect

# Update chapters from Audnex API
audiobook-forge metadata enrich --file "Book.m4b" --chapters-asin B08V3XQ7LK --update-chapters-only

# Update chapters from text file
audiobook-forge metadata enrich --file "Book.m4b" \
  --chapters chapters.txt --merge-strategy keep-timestamps --update-chapters-only

# Update chapters from EPUB table of contents
audiobook-forge metadata enrich --file "Book.m4b" --chapters book.epub --update-chapters-only

# Update chapters and metadata in one run
audiobook-forge metadata enrich --file "Book.m4b" \
  --chapters chapters.txt --asin B00B5HZGUG
```

Without `--update-chapters-only`, `enrich` also needs `--asin` or `--auto-detect`; otherwise the chapters are written and the command then exits with an error.

**Chapter sources:** Audnex API (`--chapters-asin`), text files (simple/timestamped/`CHAPTER1=` format), EPUB files (`.epub` extension) via `--chapters`. `--chapters` and `--chapters-asin` cannot be combined.

**Merge strategies:** `interactive` (default; currently behaves like `keep-timestamps`, no prompt), `keep-timestamps` (keep existing timings, take the new names), `replace-all` (use the new timings and names), `skip-on-mismatch` (like `keep-timestamps`, but abort if the chapter counts differ).

See [Metadata](Metadata) for full documentation.

---

### `organize` — Organize Library

Move book folders that contain an M4B into an `M4B/` folder and folders with unconverted audio into `To_Convert/`. Loose files in the root are not moved. Folder names come from `organization.m4b_folder` and `organization.convert_folder`.

```bash
audiobook-forge organize -r <PATH> [--dry-run] [--verbose]
```

```bash
audiobook-forge organize --root /audiobooks
audiobook-forge organize --root /audiobooks --dry-run  # Preview only
```

Note that `build` writes M4B files to the output directory, not into the source folders, so after a build the source folders still count as unconverted and `organize` would move them to `To_Convert/`.

---

### `config` — Manage Configuration

```bash
audiobook-forge config init       # Create default config
audiobook-forge config show       # Display current settings
audiobook-forge config validate   # Check for errors
audiobook-forge config path       # Show config file location
audiobook-forge config edit       # Open in default editor
```

See [Configuration](Configuration) for all options.

---

### `check` — Verify Dependencies

Checks for the two required tools, FFmpeg and AtomicParsley, and lists the available AAC encoders.

```bash
audiobook-forge check
```

---

### `version`: Show Version

```bash
audiobook-forge version
audiobook-forge --version
```

---

### M4B Merge

Combine multiple M4B files into one without re-encoding. Triggered automatically when the build command detects sequential M4B files.

```bash
# Auto-detect sequential parts
audiobook-forge build --root /path/to/book

# Force merge of M4B files without a recognized pattern
audiobook-forge build --root /path/to/book --merge-m4b --force
```

**Recognized naming patterns:** `Part 1`, `Pt 1`, `Pt. 1`, `Disc 1`, `Disk 1`, `CD1`, or a space and 1-2 digits (`Book 01.m4b`). All files must share the same base name and the numbers must form a complete run starting at 1 (or 0).

Chapters are merged with adjusted timestamps; a part without chapters becomes one chapter. Metadata is preserved from the first source file. Parts must share codec (AAC or ALAC), sample rate and channel count; otherwise the merge fails and asks you to normalize the source files first.

---

## Supported Formats

**Input:** MP3 (`.mp3`), M4A (`.m4a`), FLAC (`.flac`); M4B (`.m4b`) parts for merging

**Output:** M4B (`.m4b`): MPEG-4 Audiobook with chapters and metadata

**Required tools:** FFmpeg and AtomicParsley.

---

## Examples

### Single Audiobook

```
~/Downloads/My Audiobook/
├── 01 - Introduction.mp3
├── 02 - Chapter One.mp3
├── 03 - Chapter Two.mp3
└── cover.jpg
```

```bash
cd ~/Downloads/My\ Audiobook
audiobook-forge build
```

Result: `My Audiobook.m4b` created alongside the source files (in batch mode it goes to the output directory instead).

### Batch with Audible

```bash
# Name folders with ASINs (the output file keeps the name)
mv "The Martian" "The Martian [B00B5HZGUG]"

# Build into a separate folder (--fetch-audible embeds the Audible cover)
audiobook-forge build --root /audiobooks --out /audiobooks-m4b --fetch-audible --parallel 4

# Apply Audible tags using the ASIN in the file name
audiobook-forge metadata enrich --file "/audiobooks-m4b/The Martian [B00B5HZGUG].m4b" --auto-detect

# Or match every file by its tags/filename
audiobook-forge match --dir /audiobooks-m4b --auto
```

### Tips

- Source files are never deleted — you can always reprocess
- Use `--verbose` to debug issues
- Use `--dry-run` with `build` or `organize` to preview
- Set up `config.yaml` once to avoid repetitive flags
- `--auto` with `match` enables non-interactive scripting
