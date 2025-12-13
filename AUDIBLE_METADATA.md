# Audible Metadata Integration

Audiobook Forge now includes powerful Audible metadata integration, allowing you to automatically enrich your audiobooks with comprehensive metadata from Audible's catalog.

## Features

✅ **Complete Metadata** - Fetches title, subtitle, authors, narrators, publisher, year, description, cover art, series info, genres, rating, and more
✅ **10 Regional Stores** - Supports US, CA, UK, AU, FR, DE, JP, IT, IN, and ES Audible regions
✅ **ASIN Auto-Detection** - Automatically detects ASINs in folder names (e.g., `Book Title [B00G3L6JMS]`)
✅ **Smart Search** - Search by title/author when ASIN isn't available
✅ **Intelligent Caching** - Reduces API calls with 7-day filesystem cache
✅ **Rate Limiting** - Respects Audnexus API limits (100 req/min)
✅ **Cover Art** - Downloads and embeds high-quality cover artwork

## Quick Start

### Fetch Metadata by ASIN

```bash
audiobook-forge metadata fetch --asin B00B5HZGUG
```

### Search by Title/Author

```bash
audiobook-forge metadata fetch --title "The Martian" --author "Andy Weir" --region us
```

### Enrich Existing M4B File

```bash
# With explicit ASIN
audiobook-forge metadata enrich --file book.m4b --asin B00B5HZGUG

# Auto-detect ASIN from filename
audiobook-forge metadata enrich --file "The Martian [B00B5HZGUG].m4b" --auto-detect
```

### Auto-Fetch During Build

```bash
# Fetch metadata during conversion (requires folder names with ASINs)
audiobook-forge build --root /audiobooks --fetch-audible

# Auto-match by folder name (may have false positives)
audiobook-forge build --root /audiobooks --fetch-audible --audible-auto-match

# Specify region
audiobook-forge build --root /audiobooks --fetch-audible --audible-region uk
```

## Configuration

Add Audible settings to your `~/.config/audiobook-forge/config.yaml`:

```yaml
metadata:
  audible:
    # Enable automatic fetching during build
    enabled: false

    # Default region for metadata queries
    # Options: us, ca, uk, au, fr, de, jp, it, in, es
    region: "us"

    # Auto-match books by folder name
    # Warning: May produce false positives
    auto_match: false

    # Download and embed cover art from Audible
    download_covers: true

    # Cache metadata locally (in hours, 0 = no cache)
    cache_duration_hours: 168  # 7 days

    # API rate limit (requests per minute)
    # Do not exceed 100
    rate_limit_per_minute: 100
```

## ASIN Detection

The tool automatically detects Audible ASINs in folder names using pattern: `B[0-9A-Z]{9}`

**Supported formats:**
- `Book Title [B00G3L6JMS]`
- `B00G3L6JMS - Book Title`
- `The Martian - B00G3L6JMS - Andy Weir`
- `Book [B00G3L6JMS].m4b`

**ASIN Requirements:**
- Must start with the letter `B`
- Must be exactly 10 characters total
- Can only contain uppercase letters and numbers

## Metadata Fields

The following metadata is extracted from Audible:

### Core Fields
- **Title** - Book title
- **Subtitle** - Book subtitle (if available)
- **Authors** - List of authors with ASINs
- **Narrators** - Audiobook narrators
- **Publisher** - Publishing house
- **Published Year** - Publication year

### Content Information
- **Description** - Book synopsis/summary
- **Language** - Content language
- **Duration** - Runtime in minutes/hours
- **Is Abridged** - Whether content is abridged

### Organization
- **Series** - Series name and book sequence
- **Genres** - Primary genres
- **Tags** - Additional categorization tags

### Identifiers
- **ASIN** - Audible Standard Identification Number
- **ISBN** - International Standard Book Number (when available)

### Media
- **Cover URL** - High-resolution cover artwork URL
- **Rating** - Audible customer rating

## How It Works

### API Integration

Audiobook Forge uses the **Audnexus API** (https://api.audnex.us), a community-maintained wrapper around Audible data. This provides:

- **No authentication required** - Public API access
- **Comprehensive data** - Full metadata from Audible's catalog
- **Multi-regional support** - Access to all Audible stores
- **Free tier** - No API keys needed

### Caching Strategy

Metadata is cached locally to minimize API calls:

- **Location**: `~/.cache/audiobook-forge/audible/`
- **Format**: JSON files named by ASIN (e.g., `B00G3L6JMS.json`)
- **Default TTL**: 7 days (configurable)
- **Cache validation**: Automatic expiry based on file modification time

### Rate Limiting

To respect the Audnexus API:

- **Default limit**: 100 requests per minute
- **Implementation**: Token bucket algorithm via `governor` crate
- **Behavior**: Automatic wait when limit reached
- **Configurable**: Adjust via config (don't exceed 100)

### Metadata Priority

When enriching audiobooks:

1. **Audible metadata always wins** - Overrides existing ID3/M4A tags
2. **Cover art replacement** - Audible covers replace local artwork
3. **Comprehensive fields** - Narrator added as composer tag
4. **Description preservation** - Full synopsis stored in comment field

## Regional Support

### Available Regions

| Region | Code | Audible Store |
|--------|------|---------------|
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

### Specifying Region

```bash
# Via CLI flag
audiobook-forge metadata fetch --asin B00B5HZGUG --region uk

# Via config
metadata:
  audible:
    region: "uk"
```

## Troubleshooting

### No Metadata Found

**Problem**: `No results found for search query`

**Solutions**:
1. Try different search terms (partial titles often work better)
2. Verify the region matches where the book is available
3. Use ASIN directly if known (more reliable than search)
4. Check spelling of author/title

### API Rate Limiting

**Problem**: Too many requests in short time

**Solutions**:
1. Caching prevents most rate limiting issues
2. Process books in smaller batches
3. Clear cache only when necessary
4. Wait a few minutes before retrying

### ASIN Not Detected

**Problem**: Folder has ASIN but not detected

**Solutions**:
1. Ensure ASIN format is correct: `B` + 9 alphanumeric characters
2. Check ASIN is clearly separated (brackets or dashes help)
3. Use explicit `--asin` flag instead of auto-detection
4. Rename folder to include ASIN in supported format

### Cache Issues

**Problem**: Stale or corrupted cache data

**Solutions**:
```bash
# Clear cache for specific ASIN (not implemented yet, manual deletion)
rm ~/.cache/audiobook-forge/audible/B00G3L6JMS.json

# Clear entire cache
rm -rf ~/.cache/audiobook-forge/audible/

# Disable caching temporarily
# Set cache_duration_hours: 0 in config
```

## Examples

### Example 1: Single Book Fetch

```bash
$ audiobook-forge metadata fetch --asin B00B5HZGUG

→ Fetching Audible metadata...
  → Looking up ASIN: B00B5HZGUG

============================================================
Title: The Martian
Author(s): Andy Weir
Narrator(s): R. C. Bray
Publisher: Podium Publishing
Published: 2013
Duration: 10h 53m
Language: english
Genres: Science Fiction & Fantasy
ASIN: B00B5HZGUG
============================================================
```

### Example 2: Search and Save

```bash
$ audiobook-forge metadata fetch \
    --title "Project Hail Mary" \
    --author "Andy Weir" \
    --region us \
    --output metadata.json

→ Fetching Audible metadata...
  → Searching: title="Project Hail Mary", author="Andy Weir"

✓ Found 1 result(s):
  1. Project Hail Mary by Andy Weir

→ Fetching details for first result...
[Metadata display...]

✓ Saved metadata to: metadata.json
```

### Example 3: Batch Processing

```bash
# Directory structure:
# /audiobooks/
#   The Martian [B00B5HZGUG]/
#   Project Hail Mary [B00G3L6JMS]/
#   Artemis [B072RNS5J3]/

$ audiobook-forge build \
    --root /audiobooks \
    --fetch-audible \
    --audible-region us

→ Scanning audiobooks in: /audiobooks
✓ Found 3 audiobook(s)

→ Analyzing tracks...
✓ Analysis complete

→ Fetching Audible metadata...
  ✓ The Martian (ASIN: B00B5HZGUG)
  ✓ Project Hail Mary (ASIN: B00G3L6JMS)
  ✓ Artemis (ASIN: B072RNS5J3)
✓ Fetched metadata for 3/3 books

→ Processing 3 audiobook(s)...
[Build continues with enriched metadata...]
```

## Technical Details

### Dependencies

- **reqwest** - HTTP client for API calls
- **governor** - Rate limiting implementation
- **serde_json** - Metadata serialization
- **lazy_static** - Regex compilation
- **tokio** - Async runtime

### Files Modified

**New files:**
- `src/models/audible.rs` - Data structures
- `src/audio/audible.rs` - API client
- `src/utils/cache.rs` - Caching layer
- `tests/audible_integration.rs` - Integration tests

**Modified files:**
- `src/models/config.rs` - Added AudibleConfig
- `src/models/book.rs` - Added audible_metadata field
- `src/cli/commands.rs` - Added Metadata command
- `src/cli/handlers.rs` - Added handlers and build integration
- `src/audio/metadata.rs` - Added inject_audible_metadata
- `templates/config.yaml` - Added audible config section

### API Reference

**Audnexus Endpoints:**
- GET `/books/{ASIN}?region={region}` - Fetch by ASIN
- GET `/books?title={title}&author={author}&region={region}` - Search

**Response Format:**
```json
{
  "asin": "B00B5HZGUG",
  "title": "The Martian",
  "authors": [{"asin": "...", "name": "Andy Weir"}],
  "narrators": [{"name": "R. C. Bray"}],
  "publisherName": "Podium Publishing",
  "releaseDate": "2013-03-22T00:00:00.000Z",
  "runtimeLengthMin": 653,
  "image": "https://...",
  "summary": "...",
  "genres": [{"name": "...", "type": "genre"}],
  ...
}
```

## Future Enhancements

Potential improvements for future versions:

- [ ] Interactive search result selection
- [ ] Batch metadata update for existing M4B files
- [ ] Custom metadata field mapping
- [ ] Series-based folder organization
- [ ] Metadata export formats (CSV, XLSX)
- [ ] Integration with other metadata sources
- [ ] Automatic ASIN lookup by ISBN
- [ ] Metadata comparison and conflict resolution

## Credits

- **Audnexus API** - https://github.com/laxamentumtech/audnexus
- **Audible** - Original metadata source
- **audiobook-forge contributors** - Implementation and testing

## License

This feature is part of audiobook-forge and follows the same MIT license.
