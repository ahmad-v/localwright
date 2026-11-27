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

At this checkpoint, 1 of the original 12 acceptance gates in section 14 of the master specification is fully verified. The exact gate is source integrity. The checklist currently records 46 partially covered and 4 open active requirement groups. Group counts describe documented scope, while the 1-of-12 release gate count measures full acceptance evidence. A fresh RustSec audit of the five prior lockfiles reported zero vulnerability findings, with unmaintained-dependency warnings. Speech Studio's new language-analysis dependencies have not yet received a refreshed RustSec audit. Native and model review, package lifecycle, cross-platform accessibility and the open feature requirements remain.

## Editing increment

- Speech Studio `ffd61bb` adds signed SRT draft shifts and mono PCM16 WAV peak SVG export. Ten Linux headless tests, strict Clippy, and two CLI checks passed. Interactive waveform and playback remain open.
- Document Studio `59b6c27` adds relative clockwise rotation of a selected PDF page range. Nine Linux headless tests and strict Clippy passed. Poppler confirmed that the CLI rotated only the requested page.
- The earlier Data Studio SQL and Speech Studio portable packaging matrices have both completed with five successful targets.

## Ten-operation increment

- Speech Studio `13fdf0c` adds cue scaling, time-range trimming, splitting, merging and deletion with optional SRT export. Eleven Linux headless tests passed.
- File Studio `ffd842b` adds saved inventory comparison and filesystem timestamps. Fourteen Linux headless tests passed.
- Data Studio `964628c` adds drop-missing, fill-missing and rename-column to the direct workbench and versioned project runner. Sixteen Linux headless tests passed.
- Strict Clippy passed in all three repositories. Hosted checks were dispatched for the three commits.

## Follow-up ten-operation increment

The `codex/next-ten` branches are based on the merged `origin/main` heads. This increment adds ten operation routes in four studios:

1. File Studio versioned SHA-256 manifest creation.
2. File Studio manifest verification with matching, changed and missing paths.
3. File Studio recursive ZIP creation from regular files.
4. File Studio bounded whole-archive ZIP extraction with path and entry checks.
5. Data Studio left outer join.
6. Data Studio right outer join.
7. Data Studio full outer join.
8. Document Studio selected PDF page deletion.
9. Speech Studio LRC lyric import.
10. Speech Studio LRC lyric export.

Commits and review requests: File Studio `755f955`, PR [#8](https://github.com/ahmad-v/file-studio/pull/8); Data Studio `b9eb888` and right-order follow-up `199a286`, PR [#8](https://github.com/ahmad-v/data-studio/pull/8); Document Studio `e5e47f6`, PR [#7](https://github.com/ahmad-v/document-studio/pull/7); Speech Studio `bc24eee`, PR [#7](https://github.com/ahmad-v/speech-studio/pull/7); Localwright documentation `283ce5e` and `ac12157`, PR [#6](https://github.com/ahmad-v/localwright/pull/6). All five PRs are merged. Merge commits: File `ada0385`, Data `d32124a`, Document `77df8be`, Speech `c10c34a`, Localwright `e048474`.

Linux locked headless tests passed for File Studio (16 tests), Data Studio (18), Document Studio (10), and Speech Studio (13). Strict Clippy passed for all four. The Data Studio run used its declared Rust 1.95.0 toolchain installed only in temporary storage; other local checks used Rust 1.88.0. Registry JSON parses and `git diff --check` passes. File, Speech and Document test runs used `--no-default-features`; native GUI and optional OCR/ASR configurations were not reverified in this increment. All five hosted target checks passed on each studio PR, and the Localwright documentation PR checks passed on five targets. The corresponding hosted runs were File `45950229144`, Data `45950654688`, Document `45950221837`, Speech `45950235973`, and Localwright `45950859051`. Main branch checks passed for File `45950930117`, Data `45951646256`, Document `45950917687`, Speech `45950908814`, and the pre-handoff Localwright baseline `45951283058`. The latest Localwright documentation merge run `45976194451` was queued at this update.

The ten operations have Linux fixture evidence, but they do not close ten broad v1.3 checklist rows. Their parent requirement groups remain partial because each includes additional features and platform/install acceptance evidence. The current totals are 46 partial, 4 open, and 2 deferred; the section 14 source-integrity gate remains the only fully verified acceptance gate.

## Speech analysis increment

Speech Studio commit `e99ae15`, PR [#8](https://github.com/ahmad-v/speech-studio/pull/8), adds two operations for the previously open VAD, linguistic analysis and model-estimates group:

- `detect-voice-activity` produces a versioned JSON report of RMS-energy intervals from bounded mono 16 kHz PCM16 WAV. It uses 20 ms frames, adaptive thresholding, hysteresis and short-gap bridging. It is explicitly an energy heuristic, not neural VAD or diarization.
- `analyze-transcript` identifies likely text language with `whatlang`, counts Unicode words, characters and graphemes, punctuation-based sentence endings, non-empty text lines and unique-word ratio, and estimates words per minute against the union of subtitle cue times. The JSON keeps language-classifier confidence separate and leaves `asr_confidence` null because the current decoder supplies no such value.
- Exact new direct dependencies: `whatlang 0.18.0` (MIT) and `unicode-segmentation 1.13.3` (MIT OR Apache-2.0). Lockfile and consumer scope are Speech Studio only.
- Rust 1.88 Linux headless verification passed 20 unit and 1 integration tests, strict Clippy, formatting, locked metadata and registry JSON validation. Default-feature all-target compilation passed on Linux. A CLI smoke processed the existing 11-second JFK WAV and produced six time-bounded activity intervals; SRT analysis identified English and kept ASR confidence absent.
- The fixtures are synthetic and smoke-level, not labeled speech/noise or held-out language quality measurements. CI run `45976498111` was queued when this handoff was prepared. Non-Linux, ASR-enabled and packaged GUI verification remain pending.

The requirement group moves from Open to Partial, not Verified. To finish its acceptance work, add labeled multilingual speech/noise fixtures, measure detector precision/recall and language quality, report real model confidence only if a future adapter provides it, and complete hosted and package checks.

## Statistical inference increment

Data Studio adds ten direct analysis routes: R7 quantiles/IQR, standard error, Student-t mean interval, deterministic bootstrap mean and median intervals, Welch two-sample t test, paired t test, one-way ANOVA, Spearman rank correlation with average ties, and a Fisher-z Pearson correlation interval. Results are available in the preview and optional no-clobber JSON report. The bootstrap is bounded to 5,000 observations and uses deterministic sampling. No dependency was added. The capability registry, manual and operation status table document the behavior and limitations.

The statistical methods parent group remains Partial: the increment does not provide multiplicity correction, broader diagnostics, a native guided analysis flow or full cross-platform acceptance. The project totals remain 46 partial, 4 open and 2 deferred; section 14 still has only its source-integrity gate fully verified. Rust 1.95.0 Linux x86_64 verification passed 25 unit tests and 5 integration tests, formatting, strict Clippy and default-feature all-target compilation. The tests exercise every new operation and JSON no-clobber output; the compile check does not replace GUI interaction, accessibility or non-Linux testing. Data Studio PR [#9](https://github.com/ahmad-v/data-studio/pull/9) merged as `a4b42a5`; this Localwright handoff update PR [#9](https://github.com/ahmad-v/localwright/pull/9) merged as `791feb1`. Hosted five-target runs Data Studio `45983849460` and Localwright `45983851598` both completed successfully.

## Data transformation increment

Data Studio adds ten transformations as direct routes and version 1 declarative project steps: exact value recoding; finite number, signed integer and boolean text conversion; literal column splitting and merging; long-to-wide pivot; wide-to-long unpivot; grouped sum; and grouped mean. Parameters are bounded JSON; transformed text is capped at 32 MiB. Pivots reject blank categories and duplicate identifier/category pairs instead of silently aggregating or dropping data. No dependency was added.

Linux x86_64 Rust 1.95.0 headless verification for this increment passed 25 unit tests and 12 integration tests, formatting and strict Clippy. Fixtures cover every route, a single ten-step project replay, group-mean stability on extreme finite values, overflow rejection, new-file export and no-clobber/original-preservation behavior. The data-transform checklist group remains Partial because regex/formula transforms, duplicate-aggregating and multi-key pivots, broader transformations, project editing and cross-platform acceptance remain open. Default-feature compilation and all five hosted target checks passed in Data Studio run `45991983743`. Data Studio PR [#10](https://github.com/ahmad-v/data-studio/pull/10) merged as `2b49d28`; the Localwright handoff PR [#10](https://github.com/ahmad-v/localwright/pull/10) merged as `3fec5ab`.

## Authenticated encryption increment

File Studio adds `age-keygen`, `age-encrypt` and `age-decrypt` using the exact `age 0.11.5` dependency with default features disabled. Key files are saved with owner-only permissions on Unix; Windows confines private identity and decrypted output paths to the user profile and relies on inherited directory ACLs. Encryption and authenticated decryption stream up to 4 GiB through temporary sibling outputs and never overwrite. Failed key checks, stream authentication, cancellation or output collision do not publish output. Source inputs are preserved and checked for basic size/time changes. The app warns that age recipient encryption does not identify a sender and generated identity files are not passphrase-protected.

Linux x86_64 Rust 1.88 headless verification passed 22 tests, formatting and strict all-target Clippy. Fixtures cover binary and empty-file round-trip, public/private key separation, owner-only Unix key mode, wrong-key rejection, modified ciphertext rejection without plaintext publication, collision preservation, cancellation and symlink refusal. File Studio hosted run `45998435551` passed on macOS 15, Ubuntu 24.04 x64/ARM64, Windows 2025 x64 and Windows 11 ARM64, including portable previews. File Studio PR [#9](https://github.com/ahmad-v/file-studio/pull/9) merged as `9ee49f4`; this Localwright handoff PR [#11](https://github.com/ahmad-v/localwright/pull/11) merged as `f68dbd5` after five-target checks in run `45998465667`. The group remains Partial, not Verified: digital signature workflows, key passphrase protection, Windows ACL auditing, full platform/security audits and package acceptance remain. The current checklist totals are 47 partial, 3 open and 2 deferred groups. Section 14 still has only source integrity fully verified. File Studio branch: `codex/age-encryption`.

## Citation interchange increment

Document Studio adds `citation-validate` and `citation-convert`. Validation accepts a bounded CSL JSON object or array (maximum 10,000 references) and checks required record types, IDs, titles and author shapes. Conversion supports CSL JSON to RIS, RIS to CSL JSON, and CSL JSON or RIS to BibTeX. The RIS parser accepts complete tagged records and refuses continuation lines; unsupported format fields may be dropped. BibTeX output escapes TeX-special text. This is format interchange only: it does not render APA, Harvard, Vancouver or IEEE styles, and it is not a full bibliography manager. No dependency or lockfile changes were made.

Rust 1.88 Linux verification passed 14 tests (10 unit, 1 text-comparison integration and 3 PDF fixture integration), strict all-target Clippy without default features, formatting, default-feature all-target compilation, capability JSON parsing and `git diff --check`. Citation tests cover CSL JSON/RIS round-trip fields, BibTeX escaping with Unicode, malformed inputs, route validation and refusing to overwrite existing output.

Document Studio branch commit `1621522` merged to `main` in PR [#8](https://github.com/ahmad-v/document-studio/pull/8), merge commit `a688d84972a12d97d3073177d511ac40e9a97897`. The five-target PR run [46002062569](https://github.com/ahmad-v/document-studio/actions/runs/46002062569) and the post-merge `main` run [46002190304](https://github.com/ahmad-v/document-studio/actions/runs/46002190304) were still running/queued at this update; cross-platform verification is therefore pending. File Studio verification notes PR [#10](https://github.com/ahmad-v/file-studio/pull/10) merged as `47db2f0` and Localwright handoff PR [#12](https://github.com/ahmad-v/localwright/pull/12) merged as `811c39b`; their post-merge runs `46002161529` and `46002175290` were queued.

The publishing/citation group moves from Open to Partial. Citation interchange has fixture evidence, but Typst-style reports, validated citation formatting, fuller bibliography management, EPUB parsing/validation and hosted acceptance remain. Current checklist counts: 48 Partial, 2 Open and 2 Deferred (50 active groups; none of the broad requirement groups are fully verified). Section 14 still has only source integrity fully verified, 1 of 12 gates. Application IDs and publishing remain deferred.

## Ten Data Studio transformations

Data Studio added ten operations to the direct workbench and version 1 reproducible project steps: case-sensitive literal substring include/exclude, first/last N row slicing, first-occurrence deduplication by selected exact-text keys, a sequential signed 64-bit row-index column, a literal constant column, Unicode trim/lower/upper/whitespace collapse, atomic multi-column renaming, and selected-column removal that retains at least one column. All preserve source row order where applicable, honor the existing 500,000-cell and 32 MiB text bounds, and add no dependencies.

Linux x86_64 Rust 1.95 verification passed 42 tests (25 unit, 2 fixture round-trip, 1 missing-values, 2 outer-join and 12 transformation integration tests), strict all-target Clippy without default features, default-feature all-target compilation, formatting, registry JSON parsing and `git diff --check`. Transformation fixtures cover each new operation and replay all ten in a single project. Data Studio source commit `eeefabc`, PR [#11](https://github.com/ahmad-v/data-studio/pull/11), merged as `5eb8dde0e540c3afd183ef345a74f55046308faa`. Hosted run [46005348949](https://github.com/ahmad-v/data-studio/actions/runs/46005348949) had four target jobs running and macOS queued at this update; cross-platform verification remains pending.

This increment expands the existing Partial data-transformation group; it does not close that broad group. The current totals remain 48 Partial, 2 Open and 2 Deferred (50 active groups); Section 14 has 1 of 12 gates verified.

## Twenty-operation increment

Data Studio adds ten transformations to direct operation dispatch and version 1 project replay: filter unequal values, case-insensitive contains, starts-with, ends-with, bounded literal replacement, dropping rows with any missing cell, forward fill, backward fill, removal of empty columns, and row reversal. Bounds and cancellation checks are retained; no dependency was added.

Speech Studio adds ten subtitle cue operations: replace cue text, insert a cue, set cue timing, shift a cue range, remove exact duplicate cues, normalize cue whitespace, wrap cue text at word boundaries, join cue lines, swap cue text while preserving times, and bounded literal replacement across a cue range. Final drafts are validated as SRT; no dependency was added.

Local Linux x86_64 verification passed Data Studio's locked no-default-feature suite (50 tests), strict Clippy, default-feature all-target compilation, formatting, registry JSON parsing and `git diff --check`. Speech Studio passed its locked no-default-feature suite (22 tests), strict Clippy, GUI-enabled all-target compilation, formatting, registry JSON parsing and `git diff --check`. Data source commit `cc74ab0` merged as `b53d7e6` in [PR #12](https://github.com/ahmad-v/data-studio/pull/12). Speech source commit `edcf570` merged as `08b628a` in [PR #9](https://github.com/ahmad-v/speech-studio/pull/9). Their five-target workflows were still running when checked; platform matrix completion is pending. These operations extend the existing Partial data-transformation and transcript-editing groups, so the checklist totals remain 48 Partial, 2 Open and 2 Deferred (50 active groups). The section 14 count remains 1 of 12 gates verified.

The two Open groups remain offline aligned translation and offline narration. The current repositories do not include the actual offline engines and license-cleared model/voice assets needed to implement and validate those workflows. Application IDs and publishing remain Deferred.

## Five numeric-filter increment

Data Studio adds `filter-number-greater`, `filter-number-greater-equal`, `filter-number-less`, `filter-number-less-equal`, and inclusive `filter-number-between`. They are exposed through operation dispatch and version 1 declarative project replay. All thresholds and bounds must be finite, range bounds must be ordered, blank cells are excluded, and malformed observed numeric values fail the operation. The change adds no dependency.

Linux x86_64 Rust 1.95 verification passed 56 tests (25 unit, 2 fixture, 1 missing-values, 2 outer-join, 26 transformation integration), strict Clippy, default-feature all-target compilation, formatting, capability registry JSON parsing and diff checks. Fixtures cover comparison edges, blank/malformed input, invalid bounds, and replay of all five operations in a single project. Data Studio commit `d93ba9c` merged as `1f8f22f` in [PR #13](https://github.com/ahmad-v/data-studio/pull/13). Hosted five-target workflow [46009605586](https://github.com/ahmad-v/data-studio/actions/runs/46009605586) was still running when this note was written. The checklist's Data transformation row remains Partial because regex/formula operations, duplicate aggregation in pivots, project authoring/undo, and other scope remain incomplete. Group totals stay at 48 Partial, 2 Open and 2 Deferred; Section 14 remains 1 of 12 verified gates.
