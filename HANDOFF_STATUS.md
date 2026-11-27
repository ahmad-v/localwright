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

## Current coordinated checkpoint | 2026-11-27

This section supersedes the earlier branch-head, consumer-pin, and “next source work” snapshots above. GitHub remains canonical; publishing and application IDs remain deferred.

| Repository | Current integration state | Evidence and remaining gate |
|---|---|---|
| Localwright | PR #31 head `e93519a845049a9cecfc044d66fb3de84e0d4099` adds loaded-preview filtering to the shared paginated table. | Quick, Integration, and five native targets passed. Maintainer review was requested; PR remains open pending review. Local Linux verification: 37 UI tests plus the example compile. |
| File Studio | PR #23 merged as `1c7114b617af8b98f0a7fe970adc459e41ed969a`. | Check and Integration passed. Manifests, lockfile, compatibility record, and license-source URLs pin the exact Localwright development SHA above. |
| Data Studio | PR #27 merged as `9e347cb3d52d66a2a7d2ccb8a7b4915182219650`; PR #28 merged as `8ceeb074c70228ed151c3521f4019a4f9b405d9f`. | PR #27 Check, Integration, and five native targets passed. PR #28 adds typed standalone and project column operations; local full headless suite and all seven typed-operation tests passed after integrating PR #27. Its hosted Check, Integration, and five-target run was still in progress at this checkpoint. |
| Document Studio | PR #18 merged as `fbf379ab70fdb25ec7de95008e7bafdaa6fc8718`. | Check and Linux GUI/headless/OCR Integration passed. The merge contains the latest PDF workspace baseline and pins the exact Localwright SHA. The PDF workspace code had prior five-target verification in PR #17. |
| Speech Studio | PR #18 remains open at `e8e289e08ad7b9f5c77632dca2b935528a705e70`. | Check passed. The latest five-target Whisper/ASR run had four targets pass; Windows ARM64 was still building after the targeted clang-cl/ggml FP16 correction, and Integration was running. No physical audio-device test or multilingual WER/CER evidence is claimed. |

All four studio manifests and lockfiles now resolve Localwright `e93519a845049a9cecfc044d66fb3de84e0d4099`; compatibility records and Localwright license-source URLs match. This is an immutable development Git revision, not a crates.io release. Localwright PR #31 is the reviewed source of that revision; the approval request is recorded on the PR.

Data Studio’s typed representation now powers ordinary typed filtering/sorting, selection and rename, typed multi-column rename/drop, typed constant/index addition, and the version 2 project pipeline with joins, grouped aggregates, expressions and null-aware operations. Its table view remains a bounded preview; full-dataset virtualization, typed editing/undo, and migration of all legacy text transforms remain open. File catalogue refresh still uses complete traversal when change journals are not reliably available. Offline model-pack distribution, cross-device GUI behavior, full accessibility, packaging, and release acceptance remain open across the studios.

The exact next continuation is to inspect the Windows ARM64 job on Speech Studio PR #18, correct only compiler errors evidenced by that job, and verify its final Check/Integration/native outcomes. Then reconcile the Data Studio PR #28 hosted outcomes against the already passing local full headless suite, and continue only after maintainer review of Localwright PR #31 where merge policy requires it. Do not claim the Speech Windows ARM64 optional ASR target until that job succeeds.

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

## Twenty-operation increment

Data Studio adds ten derived numeric column operations to direct dispatch and version 1 project replay: add, subtract, multiply, divide, percent change, absolute value, square root, base-10 logarithm, rounding from -12 through 12 decimal places, and clipping to finite inclusive bounds. Blank sources produce blanks. Division by zero and zero percent-change baselines fail; square root rejects negatives; log10 rejects nonpositive values; all reject malformed numeric inputs and non-finite results. Output column dimensions and 32 MiB text are checked before writing. No dependencies were added.

Speech Studio adds ten bounded 1-based cue-range text operations: uppercase, lowercase, title case, sentence case, trim, literal prefix, literal suffix, reverse cue lines, sort cue lines, and exact duplicate-line removal. Timestamps remain fixed; edits reject invalid ranges, blank cues, malformed SRT and output text above 2 MiB. No dependencies were added.

Linux verification passed Data Studio's 61 locked no-default-feature tests (25 unit and 36 integration/fixture tests), strict Clippy, default-feature all-target compilation, formatting, capability JSON parsing and diff check. Speech Studio passed 24 locked no-default-feature tests (20 unit and 4 integration), strict Clippy, GUI-enabled all-target compilation, formatting, capability JSON parsing and diff check. Data commit `00aa89c` merged as `690bc88` in [PR #14](https://github.com/ahmad-v/data-studio/pull/14), with hosted matrix [46011125638](https://github.com/ahmad-v/data-studio/actions/runs/46011125638). Speech commit `0f44fc5` merged as `59119c8` in [PR #10](https://github.com/ahmad-v/speech-studio/pull/10), with hosted matrix [46011122838](https://github.com/ahmad-v/speech-studio/actions/runs/46011122838). Both matrices were running on four targets and queued on macOS when checked.

These twenty operations expand the existing Data transformation and Speech transcript editing groups. They do not close either broad group; regex/arbitrary formulas, project authoring/undo, audio playback, waveform interaction and timeline workflows remain incomplete. Totals remain 48 Partial, 2 Open and 2 Deferred; Section 14 remains 1 of 12 verified gates. Offline aligned translation and narration remain Open pending actual engines and license-cleared model/voice assets.

## Architecture acceleration batch — 27 November 2026

Current work is on Localwright branch `codex/private-engine-runtime` based on `origin/main` `8eab6f2`. Studio CI changes are being developed independently in their own repositories; do not cherry-pick or merge a branch until its stable quick check and any applicable acceptance workflows have finished.

### Localwright implementation in this batch

- Added `localwright_core::engine`: strict schema-v1 descriptors, app-relative executable discovery, explicit absolute trusted-path API, SHA-256 checks at discovery and launch, capability registration, health-check entry point, structured shell-free arguments, environment clearing/allowlist, per-job private temporary working directory, bounded stdin/stdout/stderr, deadline, cancellation polling, and process-group/job-object termination.
- Added exact dependencies `command-group 5.0.1` and `sha2 0.10.8`; lockfile updated. Engine descriptor licenses are recorded, but this is not an approval of bundled engine licenses.
- This is process lifecycle hygiene, not sandbox isolation: engine processes retain user permissions and network access; hard CPU/memory limits are not implemented.
- Reworked ordinary Localwright checks from the full native matrix to one Linux x64 quick check, preserving `Check / Rust (ubuntu-24.04)`. Added separate labeled/manual integration and native workflows, manual crate archive rehearsal, and scheduled/manual dependency/license metadata inventory workflow. Do not run distribution on ordinary changes.
- Updated Localwright architecture, dependency, status and requirement checklist documentation.

### Verification

On Linux x86_64 with Rust 1.88.0: `cargo test --locked -p localwright-core` passed 8 unit tests and 1 doctest; `cargo clippy --locked -p localwright-core --all-targets -- -D warnings` passed; `cargo check --locked --workspace --all-targets` passed (default Iced UI included). Formatting, locked metadata, dependency tree, capability JSON parsing, all five workflow YAML parses and `git diff --check` passed. Native platform, engine distribution, and installed consumer verification remain pending. Earlier first test attempts failed because shared target caches filled the 7.9 GB workspace; only generated `target/` caches in File, Data and Speech repositories were removed to recover space. No source from those repos was changed by cleanup.

### Cross-studio CI status

- File Studio PR #11, branch `codex/ci-architecture`, commit `9b56e75`; quick, integration and five-target native runs pending at last update. Local manifest/format/all-target check passed, test compilation hit the disk limit before removal of its generated target.
- Data Studio PR #16, branch `codex/ci-architecture-data`, commit `828e397`; hosted Linux quick check substantive steps (format, locked all-target compile, tests, Clippy) passed; GitHub still showed post-job cleanup in progress. Local target cache was removed only after disk exhaustion.
- Document Studio PR #9, branch `codex/ci-architecture-document`; YAML/Python/JSON/diff checks passed; hosted quick check pending; local Cargo unavailable for its agent.
- Speech Studio CI workflows were drafted and YAML/metadata/capability/diff validated; branch/PR confirmation pending.
- GitHub branch protection inspection returned 403 in each studio. Preserve `Check / Rust (ubuntu-24.04)` and do not assume an inaccessible rule is satisfied. CI labels may be required for milestone workflows.

### Next exact continuation

1. Re-run Localwright formatting, core tests, strict Clippy, workspace all-target check and YAML validation after final workflow edits; then commit and push `codex/private-engine-runtime`, open a PR, and wait for its quick check.
2. Review all four studio workflow branches, identify incorrect paths/label conditions and ensure no routine release build remains in `check.yml`; keep PRs unmerged until checks report a terminal result. Apply `integration` and `platform-validation` labels only when appropriate.
3. At the next architecture milestone, implement shared native workspace infrastructure (native dialogs, command/shortcut registry, persistent layout, tabs/panels/accessibility) or typed Data Studio architecture as a separate substantive batch. Studios still use Localwright rev `3091d10`; update those immutable pins together only after the shared change is reviewed and validated across consumers.
4. The user asked to keep IDs and publishing deferred. Do not publish GitHub releases or crates.io artifacts.

### Verification update — architecture CI and private engine batch

Latest local implementation branch is `codex/private-engine-runtime` at `30e5e2d` (ahead of this update). Localwright PR [#18](https://github.com/ahmad-v/localwright/pull/18) is open. The five studio CI branches/PRs currently point to File `dc8103a`, Data `b30e95c`, Document `6b5bb90`, and Speech `1462ddb`.

The Localwright private engine implementation passed Linux Rust 1.88 verification: 8 core unit tests plus 3 UI unit tests and 2 doctests passed in the workspace; strict core Clippy and workspace all-target compile passed; Rustdoc with warnings denied passed; the external crate packaging consumer smoke passed. The verified x86_64 Linux SBOMs are CycloneDX 1.6 (core: 30 components, UI: 195). `cargo audit --file Cargo.lock` found zero vulnerabilities and six no-fix unmaintained warnings (`instant 0.1.13`, `pin-utils 0.1.1`, `rustybuzz 0.14.1`, `ttf-parser 0.20.0`, `0.21.1`, `0.25.1`). Cargo metadata reports no external package missing declared license metadata. Full license compatibility and native library review remain open. The descriptor itself is the trust root and is not signed; protect the installed bundle.

At this update, all five PRs' current quick checks passed. The integration checks passed for all four studios and Localwright. Four of five native targets passed for each branch; macOS Apple Silicon remains queued. Older cancelled/failing runs belong to prior workflow commits; the current Data branch's rerun is green. None of these PRs has been merged yet. Branch protection settings remain inaccessible (GitHub API 403), so continue to respect GitHub's merge gates and stable `Rust (ubuntu-24.04)` check.

The workflow corrections that followed the first check include adding the XKB X11 runtime package for Xvfb on File and Data, adding GUI startup smoke tests for Document and Speech, and fixing `cargo audit` to use its supported `--file Cargo.lock` argument. Localwright's manual security workflow emits two per-crate SBOM files from their actual package directories and validates both. GitHub cannot dispatch a new workflow from this branch while the workflow is absent from default `main`; run the manual inventory after the CI PR is merged.

Next continuation: finish the queued macOS native jobs. Merge only PRs whose current commit has green quick, integration, and native gates. After the Localwright PR lands, coordinate one immutable merged Localwright revision update across all four studio manifests/lockfiles and verify all consumers together. Then begin the typed Data Studio architecture batch; do not claim v1.3 completion from this CI/runtime milestone. Native workspace/dialog/layout systems and the complete File, Data, Document, and Speech studio workflows remain outstanding. IDs, releases, and crates.io publishing remain deferred.

### Localwright configurable shortcut follow-on — 27 November 2026

On the same `codex/private-engine-runtime` branch, `localwright-ui` now provides a validated `ShortcutRegistry` for Run and Cancel with platform-aware key matching, collision checks, bounded JSON customization and a native Iced subscription that leaves key events already consumed by focused editors alone. The shared workbench presents key selectors for both actions and persists selected keys alongside theme and scale in per-app preferences. Existing schema-v1 preferences without shortcut fields continue to load with the defaults. The public registry allows embedding applications to use custom modifier combinations; the built-in selectors currently choose the key while retaining each action's default modifiers.

Linux x86_64 Rust 1.88 verification on the combined engine and shortcut branch passed 15 unit tests (8 core, 7 UI) and 2 doctests, strict workspace all-target Clippy, default GUI workspace all-target compilation, formatting and `git diff --check`. Tests cover default platform bindings, custom character resolution, key import/export bounds, collision rejection and schema-v1 settings migration. The full workspace was not rerun on the five native hosted targets after this shortcut follow-on; platform coverage is for the previously pushed engine/CI commit only until the new head's workflows complete.

This advances the Section 08 customization/input work but does not close either broad Section 08 group. Native file dialogs, modifier editing in the built-in selector, toolbar/menu/command registry, tabs, resizable and persistent layouts, icons, system appearance detection, screen-reader/focus verification and localization remain. No current v1.3 requirement group is marked fully verified by this increment. Localwright PR [#18](https://github.com/ahmad-v/localwright/pull/18) remains open; the four studio CI PRs also remain open, with their latest macOS native jobs awaiting hosted runner completion. After pushing the updated Localwright commit, refresh all PR heads/checks and continue with native dialog/workspace foundations, then coordinate the immutable Localwright pin update across studios only after the shared PR is merged.

### Localwright native dialog follow-on — 27 November 2026

The generic Iced workbench now has asynchronous native input-file and output-save pickers while retaining editable path fields. Linux uses RFD's XDG Desktop Portal backend with its documented `zenity` fallback. The dependency is isolated to `localwright-ui`; `localwright-core` remains independent. Cancellation leaves the current typed path intact. Native selections preserve the original `PathBuf`, including non-Unicode Unix filenames, while the text field uses a safe lossy display string. The built-in dialog buttons do not claim studio-specific format filters, folder selection, or package-installed portal acceptance.

The UI dependency is pinned to `rfd 0.15.4` (MIT); Cargo.lock resolves 383 packages. With the dialog dependency present, the Linux Rust 1.88 workspace passed formatting, all-target compilation, 17 unit tests (8 core and 9 UI), 2 doctests, strict all-target Clippy, rustdoc with warnings denied, capability JSON parsing and diff checks. `cargo audit --file Cargo.lock` found no vulnerability findings and nine no-fix unmaintained warnings (six previously documented UI/runtime warnings plus `zbus_names 4.3.5`, `zvariant 5.15.1`, and `zvariant_derive 5.15.1`). Locked Cargo metadata found no external packages missing license metadata. Local CycloneDX 1.6 generation and JSON validation completed for both current crate graphs (30 core and 274 UI components). The packaged external consumer smoke passed on the expanded graph. Native picker interaction and current-head hosted native matrix remain pending.

This is additional Section 08 implementation, not a completed requirement group. Other remaining shared UI work includes menu/command and toolbar registration, tabbed projects, resizable and persistent layouts, status/task UI, system appearance, icons, full keyboard focus, screen-reader verification and localization. The five studio CI PRs are separate and unmerged; the next Localwright action is to push this dialog increment, wait for the new-head quick/integration/native checks, then continue into workspace layout or typed Data architecture without treating command counts as coverage.

### Updated portfolio checkpoint — 27 November 2026

Current development branches and commits:

| Repository | Branch / PR | Current commit | Current status |
|---|---|---|---|
| Localwright | `codex/private-engine-runtime`, [PR #18](https://github.com/ahmad-v/localwright/pull/18) | `f2aea84` | Private engine, Run/Cancel shortcut registry, async native file dialogs. Quick and Integration passed; Linux x64/ARM64 and Windows x64/ARM64 native jobs passed; macOS Apple Silicon queued for 15 minutes and cancelled before any step. |
| File Studio | `codex/ci-architecture`, [PR #11](https://github.com/ahmad-v/file-studio/pull/11) | `ef14b7b` | Integrated bounded inspector and versioned SQLite catalogue. Local Rust 1.89 default/headless suites each passed 26 tests; strict headless Clippy, GUI all-target check, CLI route listing, formatting and registry checks passed. Current hosted Quick and Integration are running; current five-target Native run is queued. Prior head passed four native targets; macOS queued/cancelled without steps. |
| Data Studio | `codex/ci-architecture-data`, [PR #16](https://github.com/ahmad-v/data-studio/pull/16) | `4ad45fe` | Typed schema inference and Arrow Parquet roundtrip for supported scalar types. Quick and Integration passed; Linux x64/ARM64 and Windows x64/ARM64 passed; macOS queued for 15 minutes and cancelled without steps. |
| Document Studio | `codex/ci-architecture-document`, [PR #9](https://github.com/ahmad-v/document-studio/pull/9) | `6b5bb90` | CI split and workflow verification. Quick and Integration passed; four non-macOS native jobs passed; macOS queued and cancelled without steps. |
| Speech Studio | `codex/speech-ci-architecture`, [PR #11](https://github.com/ahmad-v/speech-studio/pull/11) | `1462ddb` | CI split with optional ASR integration and preview/security stages. Quick and Integration passed; four non-macOS native jobs passed; macOS queued and cancelled without steps. |

After pushing this checkpoint, fresh manually dispatched native runs were started for these exact heads: Localwright `46031832722`, File `46031835836`, Data `46031839233`, Document `46031842090`, and Speech `46031845452`. They were queued at the last status refresh; see each linked run for completion. File's PR-triggered runs for `ef14b7b` were also active (`46031481291` Quick, `46031481301` Integration, `46031481290` Native) at that point.

File Studio's SQLite catalogue stores up to 10,000 records per scan and at most 64 MiB, uses schema/application IDs, checks page size and catalog path, publishes without overwrite, records append-only full scans, and permits disconnected search/history. It does not yet identify physical volumes or perform incremental scans. `rusqlite 0.43.0` requires Rust 1.89; the minimum, toolchain and CI cache keys are updated accordingly. `cargo audit --file Cargo.lock` found zero vulnerability advisories and eight unmaintained dependency warnings; license compatibility and native asset review remain open.

Data Studio's typed Parquet interchange does not yet replace the legacy string-backed transformation model, persist type metadata in projects, or deliver virtualized typed editing. Document Studio still lacks native PDF rendering; Speech Studio still lacks native playback/recording and bundled multilingual model distribution. These are material v1.3 requirements and remain open work. All changes are on unmerged development PRs. GitHub branch protection inspection has returned 403; do not infer merge readiness from that inaccessible setting. The repeated macOS outcome is a runner-queue cancellation (no job steps executed), not a successful target verification. Retry at a later milestone without holding other implementation work idle.

The checklist still contains 48 Partial, 2 Open and 2 Deferred active groups; those groups are not equal in effort. Section 14 remains 1 of 12 fully verified acceptance gates. Application IDs, releases and crates.io publication remain deferred.

### Exact next implementation batches

1. Continue with the largest coherent Document/Speech native workflow foundation while File's current hosted run proceeds: inspect current code for any existing rendering/audio components and implement an actual preview/playback route with fixtures rather than adding more command-only operations.
2. Retry the five native matrices at an integration milestone; count only completed jobs that execute all required steps. Current code verification is Linux plus Windows/Linux hosted targets, with Apple Silicon blocked by hosted runner queue cancellations.
3. Once Localwright's shared PR is merged by the normal review gate, update all four immutable consumer pins together, run every consumer's locked tests/builds, then resume reusable workspace layout and typed Data work.
4. Reconcile docs/checklist at each coherent batch, preserve open requirements and verification boundaries, and do not claim full v1.3 compliance from these partial architecture foundations.

### Localwright resizable workspace increment — 27 November 2026

Extended the Iced workbench itself with a native `pane_grid` split between operation controls and the main workspace. The divider is mouse-resizable; the workspace has Results and Draft/transcript tabs. Added the public built-in `COMMAND_REGISTRY`, and wired menu, toolbar, browse/save controls, and Run/Cancel shortcuts through its common command dispatch. The layout ratio is stored with opt-in appearance preferences, validated to 20–70%, and older schema-v1 settings default to 34%. The independent `external_consumer` example uses the same public `run` entry point and therefore opens this functional workspace. This is a reusable integrated slice, not multiple project tabs or detachable panels.

Rust 1.88 Linux verification: `cargo check --locked -p localwright-ui`, `cargo test --locked -p localwright-ui` (12 unit tests and 1 doctest), strict UI all-target Clippy, workspace all-target check and `git diff --check` passed. Tests cover command-registry uniqueness/menu coverage, preference migration, panel-ratio bounds and finite-value fallback. `cargo fmt --all -- --check` passed after the final code changes. A first Clippy attempt found `field_reassign_with_default` in the new test; the initializer was corrected and the strict rerun passed. A live display server is unavailable here, so this UI has not had a local interaction smoke test; hosted native validation is still pending. Screen-reader and physical-device tests remain unverified.

This updates the shared UI implementation checklist but does not close Section 08. The resizable-pane slice was pushed in `7939602`; the common command-registry extension is currently being verified for a follow-up on the same PR #18. Consumer pins remain on immutable `3091d10892df79b9959f49a073db34cf81c9a15e` until the shared PR is merged through normal review.

### Major functional architecture batch — 27 November 2026

All changes remain on the existing open PRs and are unmerged. GitHub is canonical; application IDs and publishing remain deferred.

| Repository / PR | Current commit | Implementation and verification |
|---|---|---|
| Localwright [#18](https://github.com/ahmad-v/localwright/pull/18) | `13a4b52` | The Iced workbench now has a resizable pane grid, Results/Draft tabs, persistent sidebar ratio, and a common command registry wired into menu, toolbar, file actions and Run/Cancel. On Rust 1.88, UI tests (12), doctest, strict Clippy and workspace checks passed. Hosted Check and Integration passed. Windows x64/ARM64 and Linux x64/ARM64 passed; the macOS job was cancelled before steps ran. |
| File Studio [#11](https://github.com/ahmad-v/file-studio/pull/11) | `bf56fd7` | The SQLite catalogue has a native paginated search/history/refresh workspace with native file and folder selection, background jobs, cancellation and scan-progress reporting. The prior `6135e98` head passed its 20-test library suite, Check and Integration, including Xvfb startup; four non-macOS native targets passed. `bf56fd7` corrects the lockfile for the new progress UI; its refreshed hosted checks are running. |
| Data Studio [#16](https://github.com/ahmad-v/data-studio/pull/16) | `bd899c0` | Typed project runs now cover schema inference/overrides, Parquet preservation, typed filtering/sorting/selection, numeric expressions/comparisons, column rename, typed inner join and typed group sum/mean. Legacy projects stay on their existing text behavior; typed steps require explicit schema opt-in. Rust 1.95 formatting and all 45 library tests passed locally. Hosted Check and Integration passed; the current native matrix was queued at last inspection. |
| Document Studio [#9](https://github.com/ahmad-v/document-studio/pull/9) | `31a935b` | Hayro 0.9.0 (Apache-2.0 OR MIT; Rust 1.92 MSRV) provides an in-process bounded PDF raster preview and separate native Iced page viewer. A synthetic two-page PDF rendered at 200×300 with expected pixel checks; invalid page numbers were rejected. The viewer catches panics and caps input and raster dimensions. Rust 1.95 default GUI check and strict Clippy passed locally; hosted Check and Integration passed. Four native targets passed; macOS was cancelled before execution. Rendering is not OS-sandboxed and unsupported PDF features may differ. |
| Speech Studio [#11](https://github.com/ahmad-v/speech-studio/pull/11) | `1fdaf4e` | Bounded PCM16 WAV playback now has a dedicated Iced workspace with a file dialog, play/pause/resume/stop, seek slider, elapsed/duration and a 240-bin canvas waveform with moving playhead. Worker-thread loading and audio output keep the UI responsive. Formatting, locked metadata, registry and workflow checks passed; GUI tests and the new headless transport tests await hosted CI. Playback decoding passed on the earlier `96f4b22` Integration run; actual audio hardware output remains unverified. |

### Integration, dependency and remaining-work boundaries

- The four studios still pin Localwright revision `3091d10892df79b9959f49a073db34cf81c9a15e`. No consumer has yet adopted the new private-engine runtime or updated shared revision; coordinate those immutable pins only after Localwright's PR passes review and is merged.
- Document Studio adds Hayro 0.9.0 and pins Rust 1.95.0 because the renderer declares Rust 1.92. Speech adds Rodio 0.20.1, optional Iced canvas/async UI and the dialog dependency to the audio-workspace graph. File's new progress UI needed a lockfile refresh. Data adds no heavyweight processing backend.
- The Data architecture remains partial: the ordinary table/transformation path is still string-backed, the typed pipeline is opt-in, and typed join/aggregation cover only the new inner-join and single-key sum/mean routes. A virtualized typed table, schema editor, typed outer joins, richer pipelines and undo/redo remain.
- File still needs stable volume identity/incremental scanning and integrated duplicate, archive, privacy and integrity workflows. Document still needs broader format routing, visual page selection/zoom, OCR and document-structure workflows; current PDF preview is not a sandbox. Speech still needs broader decoding, transcript synchronization, recording, offline model packaging, multilingual/reference validation, WER/CER and device testing. Localwright still needs fuller accessibility, system appearance, workspace and reusable table coverage.
- The current changes do not complete or verify all v1.3 groups. No release, crate publication or merge was performed. The five current PR heads and workflow states should be refreshed before the next integration decision; queued/cancelled macOS jobs do not count as Apple Silicon verification.

## Coordinated shared-workspace integration — 27 November 2026

### Merged, verified implementation

| Repository | Main revision | Integration result |
|---|---|---|
| Localwright | `eccef00f6011207e7e67dd86bb4741cdcca1bf0a` (PR #21) | Added bounded native tabular rendering for structured operation previews, with byte/row/column/cell limits, safe fallback to text, and malformed/oversized input tests. Quick, Linux GUI/headless Integration, and Linux x64, Linux ARM64, Windows x64, Windows ARM64, and Apple Silicon checks passed. Multiple project tabs (PR #19) and keyboard panel resizing (PR #20) are also in main. |
| Data Studio | `d3e491ddc24eefced5d7db3a30df43fe46dcb943` (PR #18) | Common text predicates, comparison filters, row windows, and exact collision-checked deduplication now use typed columns on ordinary CSV/TSV/Parquet routes. Quick, Integration, and all five native targets passed on the tested feature head. This is a migration increment; the full processing/statistics/UI path is not yet typed. |
| File Studio | `c20fbee2e984347fa79a67b4140e66243640b1c3` (PR #13) | Native catalogue creation is integrated with save/root pickers, scan progress, cancellation, and history. Quick and Integration passed on the tested head. |
| Speech Studio | `6d148ff9b2558d9aa15262b46ccf0086a2b480af` (PR #12) | Consumer dependency pin to the merged Localwright runtime/workspace baseline. Quick and Integration passed; existing device playback remains hardware-dependent. |

### Current review branches

- Localwright PR #21 is merged; all four studio dependency manifests, lockfiles, compatibility records, and license provenance records now target exact Localwright revision `eccef00f6011207e7e67dd86bb4741cdcca1bf0a`.
- File Studio PR #14: shared dependency pin; PR #15: Windows volume serial + root directory file-index identity. Windows/API verification is required for #15. Both are open.
- Data Studio PR #19: shared dependency pin. Its Quick check passed; integration/native compatibility checks are running.
- Document Studio PR #10: native PDF opening/recent paths, bounded Hayro preview, page selection, and safe selected-page extraction to a new path. The `items_after_test_module` Clippy failure on the earlier head was fixed by moving the viewer runner before tests. Quick and Integration pass on `d12ce08`; native target checks were still in progress at checkpoint.
- Speech Studio PR #13: shared pin plus clickable waveform seek with bounded click mapping and focused unit test. The model test passed locally with `--no-default-features`; strict headless Clippy and formatting passed. A playback-feature compile could not initialize `alsa-sys` because the local image lacks the ALSA development package. Physical device output is unverified; hosted default GUI/Integration/native checks are required.

### Remaining v1.3 boundaries

Do not describe the ecosystem as complete. Data Studio still needs typed processing to become the default across imports, transformations, statistics and the editable large-data workspace. File Studio still needs broader archive/privacy/integrity workflows and robust filesystem edge-case coverage. Document Studio still needs broader conversion/EPUB/structured editing/OCR workflows and representative PDF corpus verification. Speech Studio still needs broader decoding, recording, ASR distribution and multilingual reference validation. Localwright still needs deeper accessibility, appearance detection and scientific figure/palette work. App IDs, publication and releases remain explicitly deferred. Cancelled or queued checks do not count as platform passes.

## Shared integration checkpoint — 27 November 2026 (20:38 UTC)

This is a continuation of the v1.3 implementation, not a completion declaration. IDs and publication remain deferred. The detailed section checklist remains in `docs/V1_3_CHECKLIST.md`; cancelled/skipped CI jobs are not evidence of a pass.

### Merged functional increments

| Repository | Main revision | Functional changes and evidence |
|---|---|---|
| Localwright | `e57e8395c566ad0b289196e771cbe43f4bd6c463` (PR #23) | Persistent toolbar preference, View-menu toggle, sidebar control, settings migration, persisted preference tests and updated independent consumer example. Check, Integration and all five target checks succeeded. Earlier merged PRs provide multiple project tabs, keyboard panel resizing and bounded result tables. |
| File Studio | `9ac13e89d142d5c6c49723d5f61d8e4cfea418e7` (PR #16) | SQLite catalogue snapshots can be compared from the native Catalogue workspace: added, removed, size-changed and same-size paths; reports stable source-identity match. Builds on the merged Windows stable volume/directory identity fix in PR #15 (`b9af509`). Focused snapshot fixture passed locally; PR #16 Check, Integration and five-target native validation passed. |
| Data Studio | `994b6b4ee1d83d48c4fe6175c0ca06fa726e9e1f` (PR #20) | Added type-preserving `fill-missing`, `fill-forward` and `fill-backward` to ordinary CSV/TSV/Parquet routes and versioned typed projects. Single-column quantiles, standard error, Student-t interval and deterministic bootstrap routes now read typed scalars, exclude nulls and reject integers beyond exact `f64` conversion. Focused fill and statistics tests passed; strict headless Clippy passed; hosted Check, Integration and all five platform targets passed. This is a migration increment; typed processing and the editable table workspace remain incomplete. |
| Document Studio | `afdf00374103fea6773d2db5f13fa7759be1aa0a` (PR #12) | Native PDF viewer rotates a selected page clockwise/counterclockwise into a new PDF, preserves the original, validates output path and supports cancellation. Focused rotation test and GUI check passed locally; Check, Integration and all five platform targets passed. |

### Shared immutable dependency baseline

Localwright main is `e57e8395c566ad0b289196e771cbe43f4bd6c463`. Data Studio PR #21 and Document Studio PR #13 updated manifests, lockfiles, compatibility records and license provenance to this exact revision; both are merged after Check, Integration and all five targets passed. File Studio PR #17 and Speech Studio PR #15 are updating the same immutable pins; verify their current heads and checks before merging. Do not publish Localwright to crates.io.

### Current open feature work and verification boundaries

- File Studio PR #17 (`de70b5d5b30ac27b4740bb825c3cead3f94e4f6e`): exact Localwright pin to `e57e839`. Quick Check passed. Integration and the five-target native workflow are running after applying both repository-specific workflow labels. Earlier canceled/skipped runs do not count.
- Speech Studio PR #14 latest head `a7843425778eafd96e651f15a24fc6434eab6362`: multi-format native decoder for WAV, FLAC, MP3, OGG/Vorbis, AAC/M4A and AIFF; bounded actual decoded samples; offline path selection; synthetic decoder fixtures. Earlier heads failed for missing `Duration`/`rodio::Source` imports, borrowed decoder buffer lifetime, Ogg demuxer feature omission, and missing Ogg duration metadata. Those causes have code fixes in the current head: owned `Arc<[u8]>`, direct Symphonia `ogg` feature, optional duration handling and waveform rebinning. Current Check, Integration and five-target jobs are running. No decoder fixture pass or device playback claim is made until the current runs succeed.
- Speech Studio PR #15 pins Localwright `e57e839`; current Quick/Integration/native verification is pending/in progress. Physical playback and recording remain hardware-unverified.

The earlier Speech Studio Check failure run `46040626137` was a Clippy dead-code warning for the payload in `Message::Tick`; that issue was fixed in the earlier waveform increment. It is distinct from the current multi-format decoder failures described above.

### Local verification from this batch

- Data Studio: `typed_missing_fills_keep_scalar_types_and_distinct_null_semantics`, all four `typed_operations` integration tests, and `typed_statistics_exclude_nulls_and_reject_lossy_integer_conversion` passed. Strict `cargo clippy --locked --no-default-features --lib -- -D warnings` passed. Formatting, capability JSON and diff checks passed.
- File Studio: stable identity test passed locally; saved-snapshot comparison fixture passed in its worktree. Broader GUI checks were stopped due shared Cargo target disk pressure; hosted PR #16 checks passed.
- Document Studio: selected-page rotation unit test and `cargo check --locked --features gui --bin document-studio` passed locally; hosted PR #12 checks passed.
- Speech Studio: formatting, locked metadata, feature tree and fixture file-format checks passed. Decoder tests initially could not compile locally because ALSA development metadata is absent. Use the hosted current-head results; do not infer codec or device verification from source presence.

### Remaining v1.3 work

The ecosystem is not complete. Data Studio still needs ordinary typed CSV/TSV/XLSX transformation migration, typed joins/aggregation across supported routes, and a native large-data editable table with filtering, schema inspection and undo/redo. File Studio still needs stable-volume reconciliation refinements, scan interruption/resume, duplicate inspection and integrated archive/privacy/integrity workspaces. Document Studio still needs thumbnails/zoom and broader visual page workflows, OCR integration, conversion routing, EPUB and structure-preservation/corpus validation. Speech Studio still needs this decoder matrix to pass, transcript timeline synchronization, recording/device workflows, multilingual offline ASR distribution/provenance and reference evaluation. Scientific palette/figure work, accessibility, system appearance, security review, SBOM/license closure and the 60 package/architecture distribution gate remain incomplete. Do not start v1.4 DoE or publish packages.

### Exact continuation point

1. Inspect the current head and terminal results for Speech PR #14; fix evidence-backed failures and rerun only the affected validation. Merge only after Quick, Integration and applicable five-target gates pass.
2. Finish File PR #17 and Speech PR #15 against Localwright `e57e839`, then confirm all four consumer manifests/lockfiles refer to a reviewed immutable shared revision.
3. Continue Data Studio’s default typed pipeline migration and a real paginated/virtualized typed table workspace; preserve legacy projects and verify XLSX bounds.
4. Continue native File/Document/Speech workflows and scientific figures as separate coherent, tested batches. Refresh this section with exact commits and current workflow identifiers after the next integration milestone.
