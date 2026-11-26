# Status | 0.1.0 publication candidate

Alpha source is on GitHub. The Linux x86_64 build and unit tests passed during integration. The independent example opened a native window on a virtual X11 display.

| Operation | Scope | Status |
|---|---|---|
| settings | JSON settings parse/save-new | Linux unit-tested |
| jobs | Single background job and cooperative cancellation | Core cancellation unit-tested on Linux |
| themes | 20 client-area presets, widget borders, and 80–300% UI scale | Linux contrast-tested |
| figures | Monochrome vector scatter SVG | Linux unit-tested |
| external-consumer | Independent character-count GUI example | Linux virtual-display launch checked |

## Known gaps

Only two crates; jobs are cooperative, not hard timeout or process isolation. Settings schema is a library API, not automatically persisted by UI. Theme and scale selections reset on restart. No system theme detection, full layouts, icons, shortcut registry, native dialogs, localization, full accessibility or scientific palette suite. Engine descriptor is a data type only; no runtime discovery or health checks.

Hosted matrix results and physical-device tests are pending. See [verification evidence](VERIFICATION.md) and [dependency review](DEPENDENCIES.md).
