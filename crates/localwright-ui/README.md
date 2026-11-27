# localwright-ui

An Iced workbench for offline desktop tools. Provide an operation list and a `localwright_core::Handler`. The workbench dispatches one job on a worker thread, shows its preview and warnings, and lets the user request cooperative cancellation. It includes native asynchronous input and save dialogs, a resizable controls/workspace split, Results and Draft tabs, 20 client-area themes, session scaling, persisted theme/scale/layout preferences, and configurable Run/Cancel shortcuts.

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

Requires Rust 1.88 and a graphical desktop. On Linux, the project CI installs `libxkbcommon-dev` and `libwayland-dev`; native dialogs use the XDG Desktop Portal and can fall back to `zenity` when the portal is unavailable. The basic `run` entry point keeps appearance and shortcut changes in the session. Applications that want them restored on restart can use `run_with_saved_appearance(title, app_key, operations, handler)` with a stable lowercase application key. This writes a small, validated preferences file in the user configuration directory. Existing version 1 preferences without shortcut entries load with defaults. The workbench exposes key selectors for Run and Cancel; bindings default to Ctrl/Command+Enter and Escape. The public `ShortcutRegistry` supports bounded JSON import/export and custom modifier combinations for embedding applications.

The native split is mouse-resizable and its ratio is saved when using `run_with_saved_appearance`. Existing version 1 preferences without a panel-width entry receive the 34% default. The Results and Draft tabs separate operation output from editable content. Full project tabs, detachable panels, and keyboard-resizable split controls remain future work.

See the MIT and Apache-2.0 license texts and third-party notices included in this package.
