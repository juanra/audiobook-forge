<!-- Generated from the clap definitions by tests/docs_test.rs. Do not edit by hand. -->

# Command-Line Help for `audiobook-forge`

This document contains the help content for the `audiobook-forge` command-line program.

**Command Overview:**

* [`audiobook-forge`↴](#audiobook-forge)
* [`audiobook-forge build`↴](#audiobook-forge-build)
* [`audiobook-forge organize`↴](#audiobook-forge-organize)
* [`audiobook-forge config`↴](#audiobook-forge-config)
* [`audiobook-forge config init`↴](#audiobook-forge-config-init)
* [`audiobook-forge config show`↴](#audiobook-forge-config-show)
* [`audiobook-forge config validate`↴](#audiobook-forge-config-validate)
* [`audiobook-forge config path`↴](#audiobook-forge-config-path)
* [`audiobook-forge config edit`↴](#audiobook-forge-config-edit)
* [`audiobook-forge metadata`↴](#audiobook-forge-metadata)
* [`audiobook-forge metadata fetch`↴](#audiobook-forge-metadata-fetch)
* [`audiobook-forge metadata enrich`↴](#audiobook-forge-metadata-enrich)
* [`audiobook-forge match`↴](#audiobook-forge-match)
* [`audiobook-forge check`↴](#audiobook-forge-check)
* [`audiobook-forge version`↴](#audiobook-forge-version)

## `audiobook-forge`


Audiobook Forge is a CLI tool that converts audiobook directories containing
MP3 files into high-quality M4B audiobook files with proper chapters and metadata.

Features:
• Automatic quality detection and preservation
• Smart chapter generation from multiple sources
• Parallel batch processing
• Metadata extraction and enhancement
• Cover art embedding


**Usage:** `audiobook-forge [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `build` — Process audiobooks and convert to M4B
* `organize` — Organize audiobooks into M4B and To_Convert folders
* `config` — Manage configuration
* `metadata` — Fetch and manage Audible metadata
* `match` — Interactive metadata matching for M4B files
* `check` — Check system dependencies
* `version` — Show version information

###### **Options:**

* `-v`, `--verbose` — Enable verbose output



## `audiobook-forge build`

Process audiobooks and convert to M4B

**Usage:** `audiobook-forge build [OPTIONS]`

###### **Options:**

* `-r`, `--root <ROOT>` — Root directory containing audiobook folders
* `-o`, `--out <OUT>` — Output directory (defaults to same as root)
* `-j`, `--parallel <PARALLEL>` — Number of audiobooks to process at once (clamped to 1-16)
* `--skip-existing <SKIP_EXISTING>` — Skip folders with existing M4B files

  Possible values: `true`, `false`

* `--force` — Force reprocessing (overwrite existing)
* `--merge-m4b` — Merge multiple M4B files even without detected naming pattern
* `--normalize` — Normalize existing M4B files (fix metadata)
* `--dry-run` — Dry run (analyze without creating files)
* `--prefer-stereo <PREFER_STEREO>` — Prefer stereo over mono

  Possible values: `true`, `false`

* `--chapter-source <CHAPTER_SOURCE>` — Chapter source priority

  Possible values: `auto`, `files`, `cue`, `id3`, `none`

* `--cover-names <COVER_NAMES>` — Cover art filenames (comma-separated)
* `--language <LANGUAGE>` — Default language for metadata
* `--keep-temp` — Keep temporary files for debugging
* `--delete-originals` — Delete original files after conversion
* `--quality <QUALITY>` — Quality preset for output audio

  Possible values: `low`, `medium`, `high`, `ultra`, `maximum`, `source`

* `--aac-encoder <AAC_ENCODER>` — AAC encoder to use (auto, aac_at, libfdk_aac, aac, aac_mf)
* `--fetch-audible` — Fetch metadata from Audible during build
* `--audible-region <AUDIBLE_REGION>` — Audible region (us, uk, ca, au, fr, de, jp, it, in, es)
* `--audible-auto-match` — Auto-match books with Audible by folder name
* `--config <CONFIG>` — Configuration file path



## `audiobook-forge organize`

Organize audiobooks into M4B and To_Convert folders

**Usage:** `audiobook-forge organize [OPTIONS]`

###### **Options:**

* `-r`, `--root <ROOT>` — Root directory to organize
* `--dry-run` — Dry run (show what would be done)
* `--config <CONFIG>` — Configuration file path



## `audiobook-forge config`

Manage configuration

**Usage:** `audiobook-forge config <COMMAND>`

###### **Subcommands:**

* `init` — Initialize config file with defaults
* `show` — Show current configuration
* `validate` — Validate configuration file
* `path` — Show config file path
* `edit` — Edit config file in default editor



## `audiobook-forge config init`

Initialize config file with defaults

**Usage:** `audiobook-forge config init [OPTIONS]`

###### **Options:**

* `--force` — Overwrite existing config file



## `audiobook-forge config show`

Show current configuration

**Usage:** `audiobook-forge config show [OPTIONS]`

###### **Options:**

* `--config <CONFIG>` — Configuration file path



## `audiobook-forge config validate`

Validate configuration file

**Usage:** `audiobook-forge config validate [OPTIONS]`

###### **Options:**

* `--config <CONFIG>` — Configuration file path



## `audiobook-forge config path`

Show config file path

**Usage:** `audiobook-forge config path`



## `audiobook-forge config edit`

Edit config file in default editor

**Usage:** `audiobook-forge config edit`



## `audiobook-forge metadata`

Fetch and manage Audible metadata

**Usage:** `audiobook-forge metadata <COMMAND>`

###### **Subcommands:**

* `fetch` — Fetch metadata from Audible
* `enrich` — Enrich M4B file with Audible metadata



## `audiobook-forge metadata fetch`

Fetch metadata from Audible

**Usage:** `audiobook-forge metadata fetch [OPTIONS]`

###### **Options:**

* `--asin <ASIN>` — Audible ASIN (B002V5D7RU format)
* `--title <TITLE>` — Search by title
* `--author <AUTHOR>` — Search by author
* `--region <REGION>` — Audible region (us, uk, ca, au, fr, de, jp, it, in, es)

  Default value: `us`
* `--output <OUTPUT>` — Save metadata to JSON file



## `audiobook-forge metadata enrich`

Enrich M4B file with Audible metadata

**Usage:** `audiobook-forge metadata enrich [OPTIONS] --file <FILE>`

###### **Options:**

* `--file <FILE>` — M4B file to enrich
* `--asin <ASIN>` — Audible ASIN
* `--auto-detect` — Auto-detect ASIN from filename
* `--region <REGION>` — Audible region

  Default value: `us`
* `--chapters <CHAPTERS>` — Update chapters from file (text/EPUB)
* `--chapters-asin <CHAPTERS_ASIN>` — Fetch chapters from Audnex API by ASIN
* `--update-chapters-only` — Only update chapters, skip metadata enrichment
* `--merge-strategy <MERGE_STRATEGY>` — Chapter merge strategy (keep-timestamps, replace-all, skip-on-mismatch, interactive)

  Default value: `interactive`



## `audiobook-forge match`

Interactive metadata matching for M4B files

**Usage:** `audiobook-forge match [OPTIONS]`

###### **Options:**

* `-f`, `--file <FILE>` — M4B file to match
* `-d`, `--dir <DIR>` — Directory of M4B files
* `--title <TITLE>` — Manual title override
* `--author <AUTHOR>` — Manual author override
* `--auto` — Auto mode (non-interactive, select best match)
* `--region <REGION>` — Audible region

  Default value: `us`
* `--keep-cover` — Keep existing cover art instead of downloading
* `--dry-run` — Dry run (show matches but don't apply)



## `audiobook-forge check`

Check system dependencies

**Usage:** `audiobook-forge check`



## `audiobook-forge version`

Show version information

**Usage:** `audiobook-forge version`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
