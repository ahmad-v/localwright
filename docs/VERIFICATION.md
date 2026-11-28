# Verification evidence

The Linux x86_64 integration run used Rust 1.88.0. Formatting, locked all-target checking, all three unit tests, and Clippy with warnings denied passed. The independent consumer opened its native window on a virtual X11 display.

## Merged native baseline

Localwright PR #18 was merged as `8e42f3feec829278fc4acc317da16ab909d10290`. The tested PR head `08629885492f81f8e760bad9723cbcc3d2592969` passed Quick, Integration, and all five hosted native jobs in run `46045388296` / `46045388232`:

- [Linux x64](https://github.com/ahmad-v/localwright/actions/runs/46045388296/job/140663359382)
- [Linux ARM64](https://github.com/ahmad-v/localwright/actions/runs/46045388296/job/140663359287)
- [Windows x64](https://github.com/ahmad-v/localwright/actions/runs/46045388296/job/140663359329)
- [Windows ARM64](https://github.com/ahmad-v/localwright/actions/runs/46045388296/job/140663358871)
- [macOS Apple Silicon](https://github.com/ahmad-v/localwright/actions/runs/46045388296/job/140663359127)
- [Quick Check](https://github.com/ahmad-v/localwright/actions/runs/46045388242/job/140663239615)
- [Integration](https://github.com/ahmad-v/localwright/actions/runs/46045388232/job/140663255598)

These are hosted build and automated integration results for the tested commit. They do not verify physical audio devices, screen-reader behavior, or all consumer workflows. The multiple-project-tabs feature is on a subsequent branch and is not included in these results.

The GitHub check workflow defines five platform jobs. Record hosted results after they run. Physical-device operation, accessibility, runtime isolation and release verification remain open.

# Local verification additions

- `cargo test --locked -p localwright-core figure::tests --lib`: 8 figure tests passed, including renderer output for all ten distinct monochrome line styles and the bounded five-marker scatter selector.
- `cargo clippy --locked -p localwright-core --all-targets -- -D warnings`: passed.
- Hosted workflow verification for this feature commit is pending; no platform-specific GUI or printed output testing is claimed.
### Scientific vector PDF export

`cargo test --locked -p localwright-core --features figure-pdf`: 18 unit tests and 6 doctests passed. `cargo clippy --locked -p localwright-core --features figure-pdf --all-targets -- -D warnings`, `cargo check --locked --workspace`, format check and `git diff --check` passed. The standalone `scientific_pdf` example generated a PDF inspected with Poppler `pdfinfo`: one unencrypted page, 510 × 340.5 points (180 × 120 mm), PDF 1.7. `pdftoppm` rasterized the PDF for visual inspection; title, axes, legend, lines and markers rendered. `pdffonts` found no embedded fonts because labels are vector outlines. The unit test verifies no image XObject marker and no overwrite. This is local Linux visual validation, not cross-platform typography equivalence.
