# Metadata

Audiobook Forge reads metadata from the source audio during `build`, and can overwrite it with Audible metadata afterwards using `match` or `metadata enrich`.

## Sources

### Local Sources

**ID3 Tags (MP3 files):**

| Tag | ID3 Frame | Description |
|-----|-----------|-------------|
| Title | TIT2 | Track/book title |
| Artist | TPE1 | Author |
| Album | TALB | Album/book title |
| Album Artist | TPE2 | Album artist |
| Composer | TCOM | Composer/narrator |
| Year | TDRC/TYER | Publication year |
| Genre | TCON | Genre |
| Comment | COMM | Comment |
| Track Number | TRCK | Track number |
| Cover Art | APIC | Embedded artwork |

**M4A/M4B Atoms:**

| Tag | Atom | Description |
|-----|------|-------------|
| Title | ©nam | Track/book title |
| Artist | ©ART | Author |
| Album | ©alb | Album/book title |
| Album Artist | aART | Album artist |
| Composer | ©wrt | Composer/narrator |
| Year | ©day | Publication year |
| Genre | ©gen | Genre |
| Comment | ©cmt | Comment |
| Track | trkn | Track number |
| Artwork | covr | Embedded cover |

**CUE Sheets:** Chapter markers and titles (used for single-file books).

**Filename Parsing:** Used by `match` only. Fills in a title or author missing from the M4B tags from an `Author - Title.m4b` or `Author_-_Title.m4b` file name; otherwise the whole file name is the title.

**Folder Name:** Fallback title during `build`, used verbatim (an `[ASIN]` in the name is kept). It is also the output file name.

### External Sources

- **Audible**: book details and chapters via the Audnexus API, title/author search via Audible's own catalog API, 10 regional stores
- **Interactive matching** — fuzzy search with confidence scoring

---

## Metadata Priority

### Tag Priority

During `build`:

1. **Embedded audio tags**: ID3/M4A/FLAC tags, first track that has each field
2. **Folder name**: fallback for the title

Audible tags are not written during `build`. `match` and `metadata enrich` overwrite the tags of an existing M4B with Audible metadata.

### Cover Art Priority

1. **Standalone files**: `cover.jpg`, `folder.jpg`, etc. (configurable)
2. **Embedded art**: auto-extracted from the first audio file
3. **Audible cover**: downloaded with `build --fetch-audible`
4. **None**: if no source provides artwork

---

## Cover Art

### File Detection

Recognized file names (exact name, case-insensitive): `cover.jpg`, `folder.jpg`, `cover.png`, `folder.png`. The list is not a priority order: the first matching file found in the folder is used.

Customize in [config](Configuration#metadata):
```yaml
metadata:
  cover_filenames:
    - "cover.jpg"
    - "folder.jpg"
    - "front.jpg"
```

### Embedded Cover Extraction

When no standalone cover file is found, Audiobook Forge extracts embedded artwork from the first audio file:

1. Checks for standalone cover files first
2. If none found, extracts from the first audio file (MP3 APIC frame, M4A/M4B artwork atom)
3. Saves it as `.extracted_cover.jpg` inside the book folder
4. Embeds into final M4B
5. Deletes `.extracted_cover.jpg` after a conversion (an M4B merge currently leaves it in place)

Disable with:
```yaml
metadata:
  auto_extract_cover: false
```

---

## Audible Integration

Fetches book details and chapters from the [Audnexus API](https://api.audnex.us) and searches Audible's catalog API (no authentication required).

### What's Fetched

- Title, subtitle, description
- Authors (with ASINs), narrators
- Publisher, published year
- Duration, language, abridgement status
- Series (name and sequence), genres, tags
- ASIN, ISBN
- Cover art URL, customer rating

### ASIN Detection

Detects ASINs in folder names (`build`) and in the file path (`metadata enrich --auto-detect`, which also sees the parent folder). Pattern: `B[0-9A-Z]{9}` as a whole word (starts with an uppercase `B`, 10 characters total).

**Supported formats:**
- `Book Title [B00G3L6JMS]`
- `B00G3L6JMS - Book Title`
- `The Martian - B00G3L6JMS - Andy Weir`

### Usage

```bash
# Fetch by ASIN
audiobook-forge metadata fetch --asin B00B5HZGUG

# Search by title/author
audiobook-forge metadata fetch --title "The Martian" --author "Andy Weir"

# Enrich existing M4B
audiobook-forge metadata enrich --file book.m4b --asin B00B5HZGUG

# During build: embeds the Audible cover only (ASIN in folder names,
# or --audible-auto-match); tags still come from the source files
audiobook-forge build --root /audiobooks --fetch-audible
```

### Regional Support

| Region | Code | Store |
|--------|------|-------|
| United States | `us` | audible.com |
| Canada | `ca` | audible.ca |
| United Kingdom | `uk` | audible.co.uk |
| Australia | `au` | audible.com.au |
| France | `fr` | audible.fr |
| Germany | `de` | audible.de |
| Japan | `jp` | audible.co.jp |
| Italy | `it` | audible.it |
| India | `in` | audible.in |
| Spain | `es` | audible.es |

```bash
audiobook-forge metadata fetch --asin B00B5HZGUG --region uk
```

### Caching

Metadata is cached as JSON files named by ASIN in the OS cache directory: `~/Library/Caches/audiobook-forge/audible/` on macOS, `~/.cache/audiobook-forge/audible/` on Linux. Default TTL: 7 days (`metadata.audible.cache_duration_hours: 168`; `0` disables the cache).

```bash
# Clear cache for one ASIN (Linux path; use ~/Library/Caches on macOS)
rm ~/.cache/audiobook-forge/audible/B00G3L6JMS.json

# Clear all cache
rm -rf ~/.cache/audiobook-forge/audible/
```

### Rate Limiting

Default: 100 requests/minute (token bucket algorithm), set with `metadata.audible.rate_limit_per_minute`.

### How Audible Metadata Maps to Tags

When Audible metadata is applied (`match` and `metadata enrich`):
- **Title** → `--title` (with subtitle appended if present) and `--album` (title only)
- **Primary author** → `--artist` and `--albumArtist`
- **Primary narrator** → `--composer` (audiobook convention)
- **Subtitle** → `--description`
- **Description** → `--longdesc` and `--comment` (truncated to 4000 chars)
- **Published year** → `--year`; **first genre** → `--genre`
- **Publisher** → custom iTunes atom
- **ASIN** → custom Audible atom (for Audiobookshelf compatibility)

See [Configuration — Audible](Configuration#audible-integration) for all settings.

---

## Interactive Matching

BEETS-inspired fuzzy matching system for M4B files.

### Scoring

| Factor | Weight | Description |
|--------|--------|-------------|
| Title | 40% | Most important identifier |
| Author | 30% | Secondary identifier |
| Duration | 20% | Validates correct edition (not scored yet: the file's duration is not read) |
| Year | 10% | Distinguishes versions |

### Confidence Levels

- **Green (>96%)** — excellent match
- **Yellow (88–96%)** — good match, review recommended
- **Red (80–88%)** — possible match, verify
- **Below 80%**: shown dimmed

A field missing on either side (for example no year in the tags) adds no penalty, so percentages can read high. `--auto` applies the top-ranked candidate whatever its confidence.

### Usage

```bash
# Interactive single file
audiobook-forge match --file "Book.m4b"

# Batch directory
audiobook-forge match --dir /path/to/audiobooks

# Auto mode (scripting)
audiobook-forge match --dir /path --auto

# Manual override
audiobook-forge match --file "Book.m4b" --title "Title" --author "Author"

# Preview without applying
audiobook-forge match --dir /path --dry-run
```
