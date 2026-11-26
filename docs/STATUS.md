# Status | 0.1.0-alpha.1

M0 complete as source planning; partial M1 implementation, not verified. All five repos created locally as source trees only. No remote resources created.

| Operation | Scope | Status |
|---|---|---|
| settings | JSON settings parse/save-new | implemented-unverified |
| jobs | Single background job and cooperative cancellation | implemented-unverified |
| themes | 20 client-area color presets | implemented-unverified |
| figures | Monochrome vector scatter SVG | implemented-unverified |
| external-consumer | Independent character-count GUI example | implemented-unverified |

## Known gaps

No actual compiler or device test. Only two crates; jobs are cooperative, not hard timeout or process isolation. Settings schema is a library API, not automatically persisted by UI. Colors only, no system theme detection, full layouts, icons, shortcut registry, native dialogs, localization, full accessibility or scientific palette suite. Engine descriptor is a data type only; no runtime discovery or health checks.

Cargo and rustc absent; registry and distribution DNS failed. No lockfile, compilation, rustfmt, clippy, Rust tests, GUI/device tests, package builds or security audit performed. All five platform builds blocked pending CI. See MASTER_SPECIFICATION.md for authoritative acceptance gates.
