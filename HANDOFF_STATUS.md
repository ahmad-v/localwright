# Studio portfolio handoff

Scope follows [Master Specification v1.3](MASTER_SPECIFICATION.md). App IDs and publishing are deferred at the maintainer's direction. Keep later and conditional work in its specified phase. See [the requirements checklist](docs/V1_3_CHECKLIST.md) for the evidence boundary and current implementation gaps.

## Branch heads at the start of verification

| Repository | Branch | Commit | Scope at this revision |
|---|---|---|---|
| Localwright | `codex/crates-publication` | `5cec6ed` | Shared core, Iced UI, and registry preparation |
| File Studio | `codex/feature-completion` | `8f7a8fb` | Inspection and hashes, directory scan and usage, duplicates, ZIP listing |
| Data Studio | `codex/feature-completion` | `7f4a8e9` | Table routes, CSV/TSV and UTF-8 Parquet, sort, deduplication, and correlation |
| Document Studio | `codex/feature-completion` | `c5dbc4d` | DOCX generation, structural PDF routes, and optional private Linux OCR |
| Speech Studio | `codex/feature-completion` | `b85b150` | WAV preparation, subtitle routes, and optional CPU ASR with pinned model |

## Current verification

Linux x86_64 offline, locked, headless test runs passed at the starting feature revisions for File Studio (6), Data Studio (8), Document Studio (6), and Speech Studio (5). Logs are under `/workspace/work/*-v13-test.log`. These were Rust unit and integration tests. See each repository's status and verification docs for earlier runs.

The initial rebuild ran out of disk. Recent unneeded Rust build files were cleared, and a tmpfs target directory with debug symbols and incremental output disabled allowed the headless checks to run. Keep Cargo caches in `/workspace/work/toolchain/cargo` and Rustup state in `/workspace/work/toolchain/rustup`. Use a fresh writable target or a tmpfs target and run compilation and fixture execution in the same shell session. Protect the staged OCR pack and speech model inputs independently of disposable build outputs.

## Continuation

- Document Studio source changes belong in `src/compare.rs`, `src/lib.rs`, `tests/compare_text.rs`, and the operation status and capability docs. Text comparison is UTF-8 line comparison with bounded working memory and a no-overwrite report export. Structural and visual diff remain open.
- Continue clean-host and platform validation from Document Studio's `docs/OCR_ENGINE.md` and Speech Studio's `docs/WHISPER_MODEL_PACK.md`. Both features stay app-local. No shared engine dependency is required.
- For Data Studio continue with structural data operations and reproducible projects. For File Studio continue with verified safe archive operations and persistent cataloguing. No added dependencies are required for text comparison.
- Native device, installer, accessibility, licensing, SBOM, and dependency-security gates require separate evidence. Preserve local commits and use authorized integration branches for remote work.

## Latest tested commits

| Repository | Commit | Verified work |
|---|---|---|
| Localwright | `b18f530` | Added vector line charts along with the existing scatter API. Four core tests and a doctest pass. The prior workspace run passed four unit tests and two doctests. Studios remain pinned to their original Localwright revision. |
| File Studio | `73731d9` | ZIP integrity tests for stored and Deflate entries. Seven locked headless tests pass, including a corrupt fixture. CLI report matched a Python-generated ZIP. Enabled ZIP's Rust Deflate backend with one added lockfile edge to the existing flate2 version. |
| Data Studio | `1b8af24` | Nine locked headless tests pass. CLI regression coefficients matched an independent exact-rational calculation. TSV, sorting, deduplication and Pearson fixtures pass. |
| Document Studio | `de0963a` | Seven locked tests pass with OCR enabled. Comparison CLI produces the expected line report. Poppler independently confirmed PDF reorder. Staged private OCR produced the expected string with socket creation blocked. |
| Speech Studio | `7892c27` | Six locked tests pass with ASR enabled. App-relative pinned model produced a two-cue draft and SRT export. The trace contained no socket activity, and truncation triggered a SHA-256 mismatch. Added 44.1 kHz input to WAV preparation. |

RustSec checked the five lockfiles against the cached advisory snapshot `c9f6c91`. It reported maintenance warnings for transitive dependencies, including `instant`, `pin-utils`, `rustybuzz`, `ttf-parser`, and Data Studio's `chrono` and `paste`. Check the current upstream advisory snapshot and confirm package-level native and model dependency exposure before release. No new known vulnerability appeared in that snapshot. Registry metadata lookup was unavailable in the standalone audit invocation. No native security clearance or complete licensing approval follows from that run.

The grouped checklist has 50 active groups, of which 42 are partial and 8 are open. Two groups are deferred. Verification is tracked separately for each fixture. These counts measure the checklist groups, which represent different amounts of work.

## Next source and platform work

1. Expand archive creation and extraction using explicit path, resource and collision checks. Keep extraction separate from integrity testing.
2. Complete typed data, reproducible project pipelines, SQL and inferential/statistical methods with independent reference data.
3. Add PDF rendering and searchable PDF first. Verify a clean Office conversion runtime before adding layout-sensitive routes.
4. Expand media decoding, playback and local translation/TTS. Record engine and model provenance before adding a default pack.
5. Extend shared settings and accessibility while preserving the existing consumer pins. Validate clean installers and GUI behavior on each supported device and target.
