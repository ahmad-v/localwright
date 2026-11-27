# localwright-core

Local application contracts, cooperative jobs, bounded file I/O, scientific SVG figures and a private engine runtime. This crate has no GUI dependency.

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

## Private engines

Place a versioned `manifest.json` in the application's `engines/` directory. Each entry names a relative executable, its SHA-256 digest, version, license reference, capabilities, time and pipe limits, and permitted environment names. `EngineRuntime::discover` validates the descriptor and executable; `EngineRuntime::run` starts only a registered executable, with argument-vector handling (no shell), a cleared/allowlisted environment, a private temporary working directory, bounded pipes, a deadline, and process-group cancellation. `health_check` uses the descriptor's health-check arguments. An explicitly trusted absolute path is supported through `resolve_trusted_path`; it is not found through `PATH`.

The descriptor is the trust root: the digest does not authenticate a manifest that an attacker can replace. Keep the application bundle and manifest protected by the installer or operating system. This is process lifecycle hygiene, not a security sandbox. Engines inherit the user's operating-system permissions and may access the network; this API does not impose hard CPU or memory quotas. Review engine provenance and licenses independently, and use OS-level isolation if an engine is untrusted.

```json
{
  "schema_version": 1,
  "engines": [{
    "id": "example-converter",
    "version": "1.2.0",
    "executable": "example-converter/bin/convert",
    "sha256": "<64 lowercase hexadecimal characters>",
    "license": "Apache-2.0",
    "capabilities": [],
    "timeout_ms": 30000,
    "max_stdin_bytes": 1048576,
    "max_stdout_bytes": 16777216,
    "max_stderr_bytes": 1048576,
    "environment_allowlist": [],
    "health_check_args": ["--version"]
  }]
}
```

Requires Rust 1.88. See the MIT and Apache-2.0 license texts and third-party notices included in this package.
