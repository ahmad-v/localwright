# Offline manual | Localwright

Development source only. Launch the GUI after compiling. Choose an operation, enter local input path and a new output path where needed, set the parameter as described under the operation selector, then Run. Status and errors are shown below. Cancel requests cooperative cancellation. The draft editor accepts transcript or plain text. Theme selection changes client colors only. Paths are manually typed in this increment; native dialogs and full accessibility remain pending.

- `settings`: JSON settings parse/save-new.
- `jobs`: Single background job and cooperative cancellation.
- `themes`: 20 client-area color presets.
- `figures`: Monochrome vector scatter SVG.
- `external-consumer`: Independent character-count GUI example.

Limitations: No actual compiler or device test. Only two crates; jobs are cooperative, not hard timeout or process isolation. Settings schema is a library API, not automatically persisted by UI. Colors only, no system theme detection, full layouts, icons, shortcut registry, native dialogs, localization, full accessibility or scientific palette suite. Engine descriptor is a data type only; no runtime discovery or health checks.

No output overwrite is allowed. No accounts, ads, telemetry or runtime downloads are intended. Do not use this unverified snapshot with valuable or hostile input. Citation and donations are voluntary.
