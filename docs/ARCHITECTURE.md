# Architecture

Localwright: A modular Rust toolkit for native, cross-platform and offline desktop applications.

Localwright core contains serde-based domain contracts, request/response, capability and engine descriptors, job cancellation, bounded reads, no-clobber output and settings. It has no Iced dependency. Localwright UI is an optional dependency of the studios and provides a generic native operation workbench. It dispatches a single job to a background thread and receives the structured outcome through a channel. The same handler runs through the CLI. Studio processing lives in its own crate. No local server, system engine lookup or runtime network client is intentionally implemented.

Source contracts version 0.1.0-alpha.1; pre-1.0 API is unstable. Use exact shared revision after import. New breaking changes require migration notes and all four consumer checks. Candidate Rust minimum 1.85 (untested), toolchain 1.88.0 (untested). No promise of ABI stability. Current settings schema 1. Request and outcome are internal alpha interchange, not stable project file formats.

Limitations: No actual compiler or device test. Only two crates; jobs are cooperative, not hard timeout or process isolation. Settings schema is a library API, not automatically persisted by UI. Colors only, no system theme detection, full layouts, icons, shortcut registry, native dialogs, localization, full accessibility or scientific palette suite. Engine descriptor is a data type only; no runtime discovery or health checks.

## Private engine runtime

`localwright_core::engine` discovers versioned JSON descriptors beside the application, accepts only application-relative regular executables, and hashes each engine at discovery and again immediately before execution. Trusted absolute paths are available as an explicit API and are never searched through `PATH`. Invocations use structured arguments (no shell), a private temporary working directory, a cleared and allowlisted environment, bounded stdin/stdout/stderr, wall-clock deadlines, and child-process group cleanup on timeout, output overflow, or cooperative job cancellation. A descriptor may declare health-check arguments and operation capabilities.

This runtime provides process lifecycle hygiene, not an operating-system security sandbox: an engine retains the launching user's OS permissions, may access the network, and has no hard CPU or memory quota. Bundle only reviewed, licensed engines and do not use this API as a substitute for OS isolation of hostile code. Version 1 descriptors are intentionally strict and reject unknown fields.
