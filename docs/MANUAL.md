# Offline manual | Localwright

Development source only. Launch the GUI after compiling. Choose an operation, enter local input path and a new output path where needed, set the parameter as described under the operation selector, then Run. Status and errors are shown below. Cancel requests cooperative cancellation. The draft editor accepts transcript or plain text. Structured result reports with bounded string `headers` and row-array `preview` fields render in a scrollable table with row indices; other results remain plain text. Native asynchronous open/save dialogs, workspace tabs and persisted resizing are integrated; full accessibility remains pending.

- `settings`: JSON settings parse/save-new.
- `jobs`: Single background job and cooperative cancellation.
- `themes`: 20 client-area color presets.
- `figures`: Labeled multi-series SVG line/scatter exports, licensed scientific palettes and black-only marker/dash style helpers for print-safe figures.
- `external-consumer`: Independent character-count GUI example.

Limitations: This source increment has not yet been compiled in the current workspace. Only two crates; jobs are cooperative, not hard timeout or process isolation. Theme presets primarily customize client colors; system theme detection, persistent project tabs, icons, localization, complete accessibility and the scientific palette suite remain open. Private engine discovery and process lifecycle hygiene are available, but do not constitute a security sandbox.

No output overwrite is allowed. No accounts, ads, telemetry or runtime downloads are intended. Do not use this unverified snapshot with valuable or hostile input. Citation and donations are voluntary.
