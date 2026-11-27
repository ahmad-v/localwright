# Status | 0.1.0 publication candidate

Alpha source is on GitHub. The Linux x86_64 build and unit tests passed during integration. The independent example opened a native window on a virtual X11 display.

| Operation | Scope | Status |
|---|---|---|
| settings | JSON settings parse/save-new | Linux unit-tested |
| jobs | Single background job and cooperative cancellation | Core cancellation unit-tested on Linux |
| themes | 20 client-area presets, widget borders, and 80–300% UI scale | Linux contrast-tested |
| figures | Monochrome vector scatter and line SVG | Linux unit-tested |
| external-consumer | Independent character-count GUI example | Linux virtual-display launch checked |

## Known gaps

Only two crates; settings schema is a library API, not automatically persisted by UI. Applications can opt into persistent theme and scale. The basic `run` entry point remains session-only. No system theme detection, full layouts, icons, shortcut registry, native dialogs, localization, full accessibility or scientific palette suite. The new engine runtime does not provide OS sandboxing or hard resource quotas, and has pending native target verification.

Hosted matrix results and physical-device tests are pending. See [verification evidence](VERIFICATION.md) and [dependency review](DEPENDENCIES.md).

## Private engine runtime increment

`localwright-core::engine` now implements version-1 private engine manifest parsing, application-relative discovery, explicit absolute trusted-path resolution, SHA-256 verification, capability registration, health checks, structured argument execution, an allowlisted cleared environment, private temporary working directories, bounded input/output, timeouts, and process-group cancellation/cleanup. The runtime deliberately does not claim an OS sandbox or hard CPU/memory quota. Linux Rust 1.88 verification: core unit tests (8) and doctest (1) passed; strict Clippy passed; locked all-target workspace check passed, including the default Iced UI; formatting, metadata, capability JSON and dependency-tree validation passed. Native platform verification remains pending.

The existing workflow previously ran the full native matrix on every ordinary check. The Localwright quick check is now a single Linux x64 workflow, preserving the `Check / Rust (ubuntu-24.04)` context. The native matrix and separate integration, distribution and security workflows are the next workflow changes in this batch; platform jobs must remain meaningful acceptance gates, not ordinary release builds.
