# localwright-core

Local application contracts, cooperative jobs, bounded file I/O and SVG scatter plots. This crate has no GUI dependency.

```toml
[dependencies]
localwright-core = "0.1"
```

Use a new output path to preserve existing files:

```rust,no_run
use localwright_core::{read_bounded, write_bytes};

fn main() -> localwright_core::Result<()> {
    let data = read_bounded(std::path::Path::new("input.txt"), 64 * 1024)?;
    write_bytes(std::path::Path::new("output.txt"), &data)?;
    Ok(())
}
```

Share the same `Handler` between a CLI and the optional `localwright-ui` workbench. Check `Job::check` during a long operation so cancellation can take effect. The file helpers are for trusted directories. They do not isolate a process or eliminate pathname races.

Requires Rust 1.88. See the MIT and Apache-2.0 license texts and third-party notices included in this package.
