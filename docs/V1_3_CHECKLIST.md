# Master Specification v1.3 implementation checklist

Rows track explicit requirements and target capability groups in the [master specification](../MASTER_SPECIFICATION.md). A group is verified only when all of its in-scope acceptance evidence exists. Partial covers a narrower implementation, tests on a subset of platforms, or pending evidence. Open covers work with no substantiated completed slice. Deferred rows are excluded from the denominator. The handbook's format lists are directional targets and candidate backend names are implementation choices.

| Section | Requirement group | State | Current evidence or continuation |
|---|---|---|---|
| 01 | Open-source, private, offline operation and free access | Partial | Source and license files exist. Packaged no-network and no-download checks are open. |
| 01 | Rust and Iced native UI with Rust-first application architecture | Partial | Core and UI exist and apps use them. Required target checks remain open. |
| 02 | Four independently installable studios and independent shared toolkit | Partial | Five source repositories exist. Independent installer and uninstall checks remain open. |
| 02 | End-to-end initial slice in each repository | Partial | Local fixtures cover subsets. Full packaged slice remains open. |
| 03 | Reusable standalone framework, public API and external example | Partial | Core/UI crates and external example. Full integration and target tests remain. |
| 03 | Settings, jobs, errors, capability and engine contracts | Partial | Core interfaces exist. Jobs are cooperative and private runtime support is incomplete. |
| 04 | Read-only signature inspection, MIME, timestamps, encoding, hex and extraction | Partial | A limited signature and size report exists. Rich inspection remains. |
| 04 | Metadata inspection, export, editing and copy-on-write sanitation | Open | Implement format-specific parsing and verify residual metadata. |
| 04 | Catalogue, disconnected volume search, scan history and QR labels | Partial | Directory inventory exists. Persistent index and volume handling remain. |
| 04 | File hashes, manifest handling and PAR2 | Partial | SHA-256, SHA-512 and BLAKE3 are checked. Manifest and recovery workflows remain. |
| 04 | Duplicate detection, usage and previewable organization | Partial | Duplicate and usage fixtures pass. Rename preview and undo remain. |
| 04 | Safe archive listing, testing, extraction and creation | Partial | ZIP listing and integrity checks have Linux fixtures. Safe archive writes and extraction remain. |
| 04 | Authenticated encryption and signature workflows | Open | Select and review existing libraries. |
| 04 | Directional file format targets and reports | Partial | Limited format detection and JSON/CSV. Broader operation fixtures remain. |
| 05 | Data import, native preview and schema handling | Partial | CSV, TSV, XLSX and a narrow Parquet path. Types and virtualization remain. |
| 05 | Data transformation | Partial | Filter, select, sort and deduplicate fixtures. Join, aggregate, pivot and other transformations remain. |
| 05 | Statistical methods and reproducible analysis | Partial | Numeric summaries, Pearson correlation and simple least-squares regression. Inferential tests, intervals and embedded SQL remain. |
| 05 | Native charts and scientific export | Partial | Vector scatter exists. Plot variety and accessible raster and vector output remain. |
| 05 | Projects, provenance, undo and batch pipelines | Open | Design and implement declarative versioned project execution. |
| 05 | Data masking and pseudonymization | Open | Add explicit rules, preview and limits. |
| 05 | Directional data formats and reports | Partial | Narrow table formats exist. Scientific and other later targets follow the specification. |
| 06 | Document conversion, layout and legacy filters | Partial | Basic generated DOCX. Office fidelity and structured conversion remain. |
| 06 | PDF structural operations and native rendering | Partial | Text extraction, split and reorder have fixture coverage. Rendering and remaining edits are open. |
| 06 | Private OCR and independently verified searchable PDF | Partial | The Linux app-relative Tesseract CLI route passed an offline fixture. Searchable PDF and all required platform packs remain. |
| 06 | Publishing, citations and safe e-book workflows | Open | Typst or equivalent, bibliography and EPUB require fixture-backed routes. |
| 06 | Document comparison, indexing and metadata | Partial | Bounded UTF-8 line comparison passed integration and CLI tests. Structural/visual comparison and indexing remain. |
| 06 | Directional document formats and independent reader checks | Partial | Basic text, DOCX and PDF. Validate preservation and conversions independently. |
| 07 | Local ASR, recording, media import, timestamps and batch | Partial | The Linux app-relative Whisper CLI passed an offline WAV-to-draft and SRT export fixture. Default local model, media coverage, recording and batch remain. |
| 07 | Transcript editing, waveform, playback and projects | Partial | Text draft and subtitle routes exist. Timeline, playback and projects remain. |
| 07 | VAD, linguistic analysis and truthful model estimates | Open | Add verified analysis capabilities and measurable quality fixtures. |
| 07 | Offline aligned translation | Open | Review a local backend and model licenses. |
| 07 | Offline narration and chaptered exports | Open | Review TTS engines, voices and applicable licenses. |
| 07 | Audio preparation and directional media formats | Partial | WAV preparation at four input rates and narrow subtitles have Linux fixtures. Wider codecs and independent playback verification remain. |
| 08 | Twenty themes, appearance settings and customization | Partial | Twenty presets exist. Persistence, system theme behavior and full customization remain. |
| 08 | Accessibility, localization and desktop input | Partial | Scale and contrast coverage exist. Screen readers, dialogs, full focus and keyboard checks remain. |
| 09 | Scientific palettes and accessible figures | Partial | Shared monochrome SVG scatter and line charts. Verified palette metadata and publication controls remain. |
| 10 | Directional capability registry and native import/export routing | Partial | Development registries and route selectors exist. Platform, engine and fixture binding remains. |
| 10 | App-local isolated runtimes and signed or hashed offline packs | Partial | Private OCR and model files have pins. Standard package integration and clean-host checks remain. |
| 11 | Input and engine isolation, limits, preservation, error recovery | Partial | Bounds and no-overwrite handling exist. Platform isolation and adversarial coverage remain. |
| 11 | Dependency security, license inventory, SBOM and pinned CI | Partial | Lockfiles and notice tooling exist. Full current audits and package evidence remain. |
| 12 | Five native targets, minimum versions and all package combinations | Partial | Packaging work exists. Native launch, upgrade and uninstall evidence remains incomplete. |
| 12 | Application IDs and public distribution or signatures requiring release | Deferred | Maintainer explicitly deferred IDs and publishing. |
| 13 | Free CI checks, platform tests, packaging, security and benchmarks | Partial | Workflows exist. Full native matrix and repeatable benchmark evidence remain. |
| 13 | Coordinated pinned five-repository integration | Partial | Studios pin an immutable Localwright revision. Feature branches need coordinated integration verification. |
| 13 | GitLab mirrors after initial verified release and approval | Deferred | Handbook sequences this after the release and separate approval. |
| 14 | Full source, native build and functional evidence | Partial | Linux headless checks exist. Native device evidence and complete vertical slice verification remain. |
| 14 | Offline, isolation and safety acceptance evidence | Partial | Engine smoke fixtures exist. Complete package-level evidence remains. |
| 14 | UI, scientific, performance and distribution acceptance | Partial | Limited tests exist. Required platform and reference evidence remains. |
| 14 | Documentation, support and reproducible release records | Partial | Manuals and dependency docs exist. Fully reconciled release evidence remains. |
| 15 | Licensing, attribution, citations, academic archive and voluntary funding | Partial | Source notices and citation files exist. Final dependency approvals and archives remain. |
| 16 | Community files, verifiable milestones and change history | Partial | Repositories have foundation docs. Coordinated acceptance milestones remain. |
| 17 | Repository instructions and safe contribution practices | Partial | Canonical instructions exist. Ongoing review and reconciliation remain. |

Count each non-deferred row once. Verified groups alone count toward the verified percentage. This conservative denominator treats broad groups as incomplete until their constituent features and acceptance evidence are present. It does not estimate implementation effort.
