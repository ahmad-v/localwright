# localwright-ui

An Iced workbench for offline desktop tools. Provide an operation list and a `localwright_core::Handler`. The workbench dispatches one job at a time on a worker thread, shows its preview and warnings, and lets the user request cooperative cancellation. Structured reports with bounded `headers` and `preview` arrays render as a horizontally and vertically scrollable table with row indices; other results remain plain text. It includes native asynchronous input and save dialogs, up to 32 project tabs with independent operation, path, parameter, draft and result state, a resizable controls/workspace split, Results and Draft tabs, a shared `COMMAND_REGISTRY` used by its menus and toolbar, 20 client-area themes, session scaling, persisted theme/scale/layout, toolbar visibility and project-tab metadata, and configurable shortcuts for project management, running/cancelling operations and resizing the controls pane.

```toml
[dependencies]
localwright-core = "0.1"
localwright-ui = "0.1"
```

```rust,no_run
use localwright_core::{Job, Operation, Outcome, Request};

fn count(request: Request, job: Job) -> localwright_core::Result<Outcome> {
    job.check()?;
    Ok(Outcome {
        preview: format!("{} characters", request.parameter.chars().count()),
        ..Outcome::default()
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    localwright_ui::run("Text counter", &[Operation {
        id: "count",
        label: "Count",
        hint: "Enter text in the parameter field.",
    }], count)?;
    Ok(())
}
```

The bundled `external_consumer` example is available in the [source repository](https://github.com/ahmad-v/localwright).

Requires Rust 1.88 and a graphical desktop. On Linux, the project CI installs `libxkbcommon-dev` and `libwayland-dev`; native dialogs use the XDG Desktop Portal and can fall back to `zenity` when the portal is unavailable. The basic `run` entry point keeps appearance and workspace preference changes in the session. Applications that want them restored on restart can use `run_with_saved_appearance(title, app_key, operations, handler)` with a stable lowercase application key. This writes a small, validated preferences file in the user configuration directory. Version 1 preferences migrate missing shortcuts and tab metadata to defaults. The public `ShortcutRegistry` supports bounded JSON import/export and custom modifier combinations for embedding applications.

The common action registry is exposed as `COMMAND_REGISTRY` with `WorkbenchCommand`, `CommandMenu`, and `CommandDescriptor`. Menu, toolbar, browse/save buttons, and keyboard shortcuts route through the same action dispatch. The View menu action and sidebar preference control can hide or restore the toolbar. This choice is persisted when using `run_with_saved_appearance`; older version 1 preferences default to a visible toolbar. The native split is mouse-resizable and its ratio is saved when using `run_with_saved_appearance`; Ctrl/Command+Alt+\[ and \] narrow or widen it in five-point increments within the 20%–70% range. Existing version 1 preferences without a panel-width entry receive the 34% default. Each project tab owns its input/output paths, operation selection, parameter, editable draft and result while the application is running. Use the visible tab buttons, Ctrl/Command+T to open a tab, Ctrl/Command+W to close the active tab, or Ctrl+Tab / Ctrl+Shift+Tab on all targets to switch projects. These shortcuts can be customized in the workbench. Closing a project cancels its active operation cooperatively; only one operation runs at a time. With `run_with_saved_appearance`, tab order, titles and active selection are restored on restart; inputs, drafts and results intentionally remain blank until a user restores an explicit session file. Without saved preferences, tab metadata remains session-local. The File menu provides **Save workspace session…** and **Restore workspace session…**. A version-1 JSON session file (suggested name `.lw-session.json`) stores ordered tabs, the active tab, the controls-pane ratio, toolbar visibility, operation ids, input/output paths, parameters, drafts and Results/Draft view choice. Restoring appends tabs to the current workbench, refuses unknown application operation ids and does not run anything automatically. Result payloads and loaded table data are excluded. Session files are unencrypted and may contain sensitive paths, parameters and draft text; save them only to a location appropriate for that content. Files are bounded to 8 MiB and 32 tabs, and an existing destination is replaced only when it is already a valid Localwright session. Selected file paths must be valid UTF-8 in the portable session format. Tab buttons expose the title, selection state and position in their visible labels, while keyboard shortcuts provide a keyboard route for tab selection and management. Iced 0.13's button widget does not expose programmatic focus controls, so the workbench does not claim focus restoration when a tab closes. Detachable panels remain future work.

## Paginated typed table

`TableModel` and `paginated_table` provide a native Iced table component for typed rows. It validates consistent non-null column types, keeps null values distinct from empty text, caps the schema at 128 columns, renders at most 100 data rows per page, truncates displayed cell text to 256 characters, supports typed stable sorting with nulls last, and emits accessible row-selection and page actions to the embedding application. A bounded case-insensitive substring filter searches all cell values in the already loaded model; it does not query source files. This bounds widget construction and visible text; the caller still owns the full in-memory dataset and must use its own streaming or storage layer for very large inputs.

Run the standalone example with `cargo run -p localwright-ui --example paginated_table`. It demonstrates integer, categorical, floating-point, Boolean and null values, page navigation, header sorting, row selection and filtering over loaded rows.

Applications that want to show a typed table in the standard workbench can use `run_with_saved_appearance_and_table_workspace`. The application supplies a `TableWorkspaceProvider`, which converts its result payload into `TableWorkspace` metadata plus a validated `TableModel`. The workbench adds a per-project Data table tab, schema summary, paging, row selection, typed sorting and loaded-preview filtering while retaining the existing Results and Draft tabs. Providers should keep payloads bounded and return `Ok(None)` for ordinary operation results.

See the MIT and Apache-2.0 license texts and third-party notices included in this package.
