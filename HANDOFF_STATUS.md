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

RustSec checked the five lockfiles against the cached advisory snapshot `c9f6c91`. It reported maintenance warnings for transitive dependencies, including `instant`, `pin-utils`, `rustybuzz`, `ttf-parser`, and Data Studio's `chrono` and `paste`. Check the current upstream advisory snapshot and confirm package-level native and model dependency exposure before release. No vulnerability finding was emitted for the scanned lockfiles in that snapshot. Registry metadata lookup was unavailable in the standalone audit invocation. No native security clearance or complete licensing approval follows from that run.

The grouped checklist has 50 active groups, of which 45 are partial and 5 are open. Two groups are deferred. Verification is tracked separately for each fixture. These counts measure the checklist groups, which represent different amounts of work.

## Next source and platform work

1. Expand the verified single-file ZIP routes into safe multi-file archive creation and extraction, then cover additional formats and persistent catalogues.
2. Complete typed data, reproducible project pipelines, SQL and inferential/statistical methods with independent reference data.
3. Add native PDF rendering and expand searchable PDF beyond the verified single PNG route. Verify a clean Office conversion runtime before adding layout-sensitive routes.
4. Expand media decoding, playback and local translation/TTS. Record engine and model provenance before adding a default pack.
5. Extend shared settings and accessibility while preserving the existing consumer pins. Validate clean installers and GUI behavior on each supported device and target.

## Coordinated integration branch

The development branch on each GitHub repository is `codex/v1.3-implementation`. The four studios initially pinned Localwright `b18f5303ca5e7684881ccce9951cbee8b25dea82` and now pin `3091d10892df79b9959f49a073db34cf81c9a15e`, crate version 0.1.0, in their manifests, lockfiles, integration records and license source records. This is a Git revision pin. Registry publication remains deferred.

| Repository | Changes after the initial Linux pass | Hosted validation |
|---|---|---|
| Localwright | Shared SVG line charts, tests, this handoff and its checklist | [Run](https://github.com/ahmad-v/localwright/actions/runs/45875717973) |
| File Studio | Hex preview and one-file ZIP creation and extraction. Expanded checksum fixtures. | [Run](https://github.com/ahmad-v/file-studio/actions/runs/45876638409) |
| Data Studio | Vector line chart through the shared renderer, fixed column masking, and versioned project replay with no-clobber output | [Run](https://github.com/ahmad-v/data-studio/actions/runs/45877790674) |
| Document Studio | Searchable PNG-to-PDF OCR, WordprocessingML body-text extraction and matching Localwright pin | [Run](https://github.com/ahmad-v/document-studio/actions/runs/45877148067) |
| Speech Studio | Matching Localwright pin, private ASR, 44.1 kHz preparation and JSON transcript exchange | [Run](https://github.com/ahmad-v/speech-studio/actions/runs/45877154966) |

On Linux x86_64 the default-feature suites passed across all five repositories. The final File Studio archive route is tested separately with the headless fixture and a Python-generated ZIP. Data Studio's last suite passed 12 unit and integration tests, including masking, line output and declarative project replay. The latest searchable PDF smoke uses the Tesseract pack hashes in Document Studio's `docs/OCR_ENGINE.md`. Poppler confirmed the expected text and one page and provided the embedded image for an RGB byte-for-byte comparison. A second write refused the existing output.

### Reproduce the offline engine fixtures

- OCR: Use the staged archive in `work/tesseract-native/tesseract-5.5.0-linux-x86_64.tar.gz` or reconstruct the pack from Document Studio's engine notes. Build with `--features ocr` and the three documented SHA-256 pins. Stage `engines/tesseract` beside the built binary. Run `--cli ocr-png input.png output.txt` and `--cli ocr-png-pdf input.png output.pdf`. In the build workspace the synthetic input path is recorded under `work/ocr-engine-stage`. Use Poppler to check the text and image independently. The trace used `strace -f -e trace=socket,connect -e inject=socket:error=EACCES`.
- ASR: Use `work/asr-verification/whisper-tiny-en.zip` and the short local `reference.wav`, or rebuild the model pack from the pinned source as in Speech Studio's notes. Build with `--features asr`, extract the pack beside the binary, and run `--cli transcribe reference.wav` with no model parameter. Read the SRT draft from the response JSON and export it through `--cli export-srt draft.srt result.srt`. A truncated model copy must fail its SHA-256 check.
- Keep model files, recorded audio and native binaries outside Git. Retain exact upstream sources, hash pins and notices. Review release licensing and acceptance separately.

The current studio commits on the integration branches are `9ba335f` (File), `6527339` (Data), `8b0af0a` (Document), and `51a9c65` (Speech). The new Document Studio DOCX CLI output matched a fixture generated and read by Python DOCX. The new Speech Studio JSON CLI round trip preserved Unicode and integer millisecond times. The File Studio headless suite passed 10 tests, Document Studio passed 8 headless tests, and Speech Studio passed 7 headless tests. All four default desktop executables opened their named native windows on the Linux X11 test display. This verifies launch on that display.

Document Studio added `quick-xml 0.42.0` (MIT) for namespace-aware parsing. Data and Speech Studio made the existing pinned `serde 1.0.219` a direct dependency. After both changes, repeat RustSec JSON audits of all five lockfiles exited zero with no vulnerability findings and with unmaintained-dependency warnings. This does not inspect packaged native libraries or language/model assets.

### Acceptance checkpoint

Clean GitHub clones of the five feature branches resolved the locked trees and contained matching master specifications. The Linux x86_64 all-features tests and strict Clippy passed across all five repositories. Test totals were 7 in Localwright including doctests, 10 in File Studio, 12 in Data Studio, 8 in Document Studio and 7 in Speech Studio. The corresponding logs are `work/*-v13-all-features.log`.

The hosted runs listed above passed on Ubuntu x64 and ARM64 and Windows x64 and ARM64. Their macOS 26 jobs were cancelled before any steps ran. The Localwright workflow retained the ARM64 assertion and switched to the GitHub-hosted macOS 15 runner for a recovery attempt. Installer lifespan, clean-device and comprehensive accessibility testing still need execution.

## Continued feature work

- Localwright `3091d10` added the opt-in saved-appearance API. Preferences contain only theme and scale. Invalid, unrecognized and symlinked settings are refused for reading and replacement. The four studios use their own stable key and pin this immutable revision in manifests, lockfiles and license records. The stateless shared entry point remains usable.
- File Studio `f43a1aa` added PNG metadata inspection and selective removal of the four text/EXIF chunk types. It checks structure and CRCs and writes a new image. Pillow independently confirmed identical pixels and confirmed removal. Filtering preserves other metadata and does not anonymize images.
- Data Studio `b7c49c6` added exact nonblank inner joins, an explicit second input, and a 32 MiB joined-text budget in addition to the table limits. Unit tests and mixed CSV/TSV CLI output passed.
- Document Studio `3e8f603` added the Linux OCR preview assembler. The extracted bundle passed SHA-256, engine and license staging, reference text recognition and searchable PDF verification with Poppler. The assembler rejects symlinked and architecture-mismatched packs.
- Speech Studio `d11e111` recorded successful checksum, offline transcription and SRT export from the extracted Linux x64 ASR preview. Commit `7557ab9` added validated find/replace with timestamps preserved.
- Using `macos-15` with an ARM64 assertion resolved the hosted macOS 26 queue cancellation for Localwright. GitHub reported a full successful Localwright matrix at run [45901441847](https://github.com/ahmad-v/localwright/actions/runs/45901441847). Studio workflows have the same runner change.

The 12 acceptance gates in section 14 remain the denominator for verified release readiness. Source integrity has evidence. Device, installer, accessibility, performance, broader formats, engines and application scope remain open.

## Current source and CI references

| Repository | Current code revision | Latest matrix |
|---|---|---|
| Localwright | `3091d10` | [Run 45902306178](https://github.com/ahmad-v/localwright/actions/runs/45902306178) |
| File Studio | `b68e9b5` | [Run 45902946519](https://github.com/ahmad-v/file-studio/actions/runs/45902946519) |
| Data Studio | `f9a197d` | [Run 45904248028](https://github.com/ahmad-v/data-studio/actions/runs/45904248028) |
| Document Studio | `fff517e` | [Run 45902462259](https://github.com/ahmad-v/document-studio/actions/runs/45902462259) |
| Speech Studio | `cac25fb` | [Run 45904109929](https://github.com/ahmad-v/speech-studio/actions/runs/45904109929) |

File Studio's 12-test locked headless suite passed. The newest fixture moves the source directory away and finds the saved Unicode record from the catalog. File Studio made the already-locked `crc32fast 1.5.6` and `serde 1.0.219` direct dependencies. Data Studio's 14-test locked suite passed. Python statistics independently checked its new group summaries, including missing-value and sample SD behavior. These fixture sequences are logged in `work/file-v13-catalog-verification.log`, `work/file-v13-png-verification.log`, `work/data-v13-group-verification.log` and `work/data-v13-join-verification.log`.

The mandatory next release checks include clean-device installs and removal, full accessibility testing, native engine compatibility and licensing review, and verified answers for the open functionality in the checklist. The snapshot does not satisfy the complete feature set. Application IDs and publication remain deferred.

### Latest Linux verification and native matrix boundary

All four studios opened their Linux X11 windows with the stored dark appearance. The shared settings tests also verified theme and scale persistence, updates and preservation of unrecognized settings. Both the extracted Linux OCR preview and speech preview passed checksum and functional checks. The Speech Studio packager now requires an ASR binary and its model together. Local checks accepted the matching variants and refused an ASR binary without its model.

GitHub reported five successful targets per repository at the tested baseline revisions. The prior Data Studio run [45902962963](https://github.com/ahmad-v/data-studio/actions/runs/45902962963) and Speech Studio run [45902470471](https://github.com/ahmad-v/speech-studio/actions/runs/45902470471) both passed all five jobs before their later SQL and packaging changes. Those changes have Linux tests and their new hosted runs are pending. In particular, Data Studio's newer SQLite route uses Rust 1.95.0 and bundled SQLite 3.54.1 to avoid the older SQLite release affected by upstream SQL-parser advisories. Keep the SQLite version tied to the bundled source in the lockfile. Release dependency review includes native advisories independently from RustSec.

Data Studio commit `f9a197d` passed 15 locked headless tests, formatting and strict Clippy on Rust 1.95.0. The SQL CLI matched an independent Python SQLite query. The SQL authorization fixture rejected writes, database attachment, extension loading and multiple statements. Source and test output are in Data Studio's documentation and `work/data-v13-sql-final.log`. Build Data Studio using its own pinned Rust 1.95.0 toolchain, and use the other projects' declared toolchains for their verification.

At this checkpoint, 1 of the original 12 acceptance gates in section 14 of the master specification is fully verified. The exact gate is source integrity. The checklist also records 45 partially covered and 5 open active requirement groups. Group counts describe documented scope, while the 1-of-12 release gate count measures full acceptance evidence. A fresh RustSec audit of all five final lockfiles reported zero vulnerability findings, with unmaintained-dependency warnings. Native and model review, package lifecycle, cross-platform accessibility and the open feature requirements remain.

## Editing increment

- Speech Studio `ffd61bb` adds signed SRT draft shifts and mono PCM16 WAV peak SVG export. Ten Linux headless tests, strict Clippy, and two CLI checks passed. Interactive waveform and playback remain open.
- Document Studio `59b6c27` adds relative clockwise rotation of a selected PDF page range. Nine Linux headless tests and strict Clippy passed. Poppler confirmed that the CLI rotated only the requested page.
- The earlier Data Studio SQL and Speech Studio portable packaging matrices have both completed with five successful targets.
