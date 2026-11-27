# localwright-core

Local application contracts, cooperative jobs, bounded file I/O, scientific SVG figures and a private engine runtime. Figure exports support monochrome-safe and marker/dash series styles, plus optional visible title and axis labels that are bounded and XML-escaped. The core includes sequential `batlow`, diverging `vik` and qualitative `batlowS` scientific palettes with their source notice. Okabe-Ito, a v1.3 target, is not bundled pending redistribution-license clarification. This crate has no GUI dependency.

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

Choose a discrete palette color and combine hue with a marker or dash pattern:

```rust,no_run
use localwright_core::{figure::{line_svg_styled, Marker}, palettes::palette};

fn main() -> localwright_core::Result<()> {
    let colors = palette("batlowS").ok_or_else(|| {
        localwright_core::Error::input("palette is not bundled")
    })?;
    let style = colors.series_style(0, Marker::Circle, false)?;
    let svg = line_svg_styled(&[(0.0, 1.0), (1.0, 2.0)], 640, 480, style)?;
    assert!(svg.contains(style.color));
    Ok(())
}
```

Render and export multiple labeled series with shared axes and a legend. Line figures include marker and dash examples in the legend; scatter figures show each series marker.

```rust,no_run
use localwright_core::{
    figure::{export_line_series_svg_labeled, FigureLabels, FigureSeries, Marker},
    palettes::palette,
};

fn main() -> localwright_core::Result<()> {
    let palette = palette("batlowS")
        .ok_or_else(|| localwright_core::Error::input("palette is not bundled"))?;
    let control = [(0.0, 1.0), (1.0, 2.0)];
    let treatment = [(0.0, 1.5), (1.0, 2.5)];
    let series = [
        FigureSeries::new(
            "Control",
            &control,
            palette.series_style(0, Marker::Circle, false)?,
        ),
        FigureSeries::new(
            "Treatment",
            &treatment,
            palette.series_style(1, Marker::Triangle, true)?,
        ),
    ];
    export_line_series_svg_labeled(
        std::path::Path::new("comparison.svg"),
        &series,
        900,
        600,
        FigureLabels {
            title: "Response by dose",
            x_axis: "Dose (mg)",
            y_axis: "Response (a.u.)",
        },
    )
}
```

`batlow` is for ordered nonnegative quantities, `vik` for values with a meaningful center, and `batlowS` for nominal categories. The quantitative maps return the nearest 8-bit sample for a normalized position. Do not rely on hue alone for categories: pair colors with marker or line styles and inspect print output. Palette data provenance and terms are in `THIRD_PARTY_NOTICES.md` and `data/palettes/source.json`.

Create a labeled SVG directly or export it to a new path:

```rust,no_run
use localwright_core::figure::{
    export_scatter_svg_labeled, FigureLabels, SeriesStyle,
};

fn main() -> localwright_core::Result<()> {
    let points = [(0.0, 1.2), (1.0, 1.8), (2.0, 2.6)];
    let labels = FigureLabels {
        title: "Measurement response",
        x_axis: "Concentration (mg/L)",
        y_axis: "Response (a.u.)",
    };
    export_scatter_svg_labeled(
        std::path::Path::new("figure.svg"),
        &points,
        900,
        600,
        SeriesStyle::MONOCHROME,
        labels,
    )
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
