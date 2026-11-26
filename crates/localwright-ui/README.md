# localwright-ui

An Iced workbench for offline desktop tools. Provide an operation list and a `localwright_core::Handler`. The workbench dispatches one job on a worker thread, shows its preview and warnings, and lets the user request cooperative cancellation. It includes 20 client-area themes and session scaling.

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

Requires Rust 1.88 and a graphical desktop. On Linux, the project CI installs `libxkbcommon-dev` and `libwayland-dev`. The basic `run` entry point keeps appearance changes in the session. Applications that want theme and scale restored on restart can use `run_with_saved_appearance(title, app_key, operations, handler)` with a stable lowercase application key. This writes only a small appearance preferences file in the user configuration directory.

See the MIT and Apache-2.0 license texts and third-party notices included in this package.
