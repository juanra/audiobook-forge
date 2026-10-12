# ADR 0001: Inject chapters with ffmpeg instead of MP4Box

- Status: accepted
- Date: 2026-10-12
- Issue: #35

## Context

Chapters were written with `MP4Box -chap`, which stores them only in the Nero
`chpl` atom. That atom holds the chapter count in a single byte. With more than
255 chapters the count wrapped (300 tracks became 44 chapters) and the last
surviving chapter covered the rest of the book. Every tool exited 0, so the run
reported success over an unusable file.

While investigating we also found that Apple's AVFoundation, which Apple Books
uses, does not read `chpl` at all: a file chaptered by MP4Box showed **0
chapters** there, regardless of count.

## Options considered

1. **MP4Box up to 255 chapters, ffmpeg above.** Smallest behavioural change for
   most books, but two code paths to keep correct and test, and Apple players
   still see no chapters on the common path.
2. **MP4Box with a QuickTime chapter track** (`-add file:chap`). Fixes Apple,
   but keeps MP4Box's `chpl` wrap and a tool we would otherwise not need.
3. **ffmpeg for all chapter injection** (`-f ffmetadata`, `-map_chapters`,
   stream copy).

## Decision

Option 3. ffmpeg writes both a QuickTime chapter text track carrying every
chapter and a `chpl` atom it caps at 255 rather than wrapping. Verified on
ffmpeg 9.0.2 with 300 chapters:

| Writer | ffprobe | AVFoundation | `chpl` count byte |
|---|---|---|---|
| `MP4Box -chap` | 44 | 0 | 44 (wrapped) |
| ffmpeg | 300 | 300 | 255 (capped) |

Both survive the later AtomicParsley metadata and cover pass.

After injection, and again once the finished file is written, the chapter
count is read back with ffprobe and the book fails if it differs from what was
generated. A silent loss can no longer be reported as success.

## Consequences

- Players that read only `chpl` see at most 255 chapters, instead of a wrapped
  count with the tail collapsed into the last chapter.
- Chapter injection remuxes the file once (stream copy, no re-encode), written
  to a temp file beside the target and renamed into place.
- MP4Box (gpac) is no longer a dependency: it is gone from `check`, the config
  (`advanced.mp4box_path`, now ignored if present) and the install instructions.
