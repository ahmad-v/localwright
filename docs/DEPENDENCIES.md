# Dependencies

Cargo manifests list the exact direct requirements. The committed lockfile records the resolved versions and registry checksums. Builds use Rust 1.88.0 from `rust-toolchain.toml`. The CI checkout action is pinned to its upstream v4.2.2 commit.

A preliminary RustSec review used advisory database revision `c9f6c91a44dbad9d71742329e9df061279f9c4be`. The workbench uses Iced’s Tiny Skia software renderer. This removes the WGPU path through `iced_glyphon` and the affected `lru 0.12.5` dependency from the locked graph (RUSTSEC-2026-0002 and RUSTSEC-2026-0253). Full license review, SBOM and native dependency review remain open.

The private engine runtime adds `command-group 5.0.1` for process-group/job-object lifecycle management and `sha2 0.10.8` for streaming executable verification. Both are exact-version Rust dependencies in the lockfile; the workflow and security review must continue to evaluate their transitive licenses and advisories. Engine descriptors carry a license reference, but this does not replace per-engine provenance, notice, and compatibility review.

### 27 November 2026 security review

`cargo audit --file Cargo.lock` scanned 311 locked packages against the current RustSec advisory database. It reported no vulnerability findings and six allowed unmaintained-package warnings: `instant 0.1.13`, `pin-utils 0.1.1`, `rustybuzz 0.14.1`, and `ttf-parser 0.20.0`, `0.21.1`, and `0.25.1`. The advisory snapshot reported no available fixes for those warnings. This check covers Rust advisories only; it does not audit system libraries or engine payloads.

Cargo metadata found license metadata for all external packages. The newly added direct dependencies declare `command-group 5.0.1` as `Apache-2.0 OR MIT` and `sha2 0.10.8` as `MIT OR Apache-2.0`. License metadata presence is not a compatibility or notice approval; a full dependency and native-component review remains open.

A local CycloneDX 1.6 generation check produced valid x86_64 Linux SBOMs for both crates (30 components for `localwright-core`, 195 for `localwright-ui`). The scheduled/manual GitHub artifact workflow still needs to run after it becomes dispatchable from the default branch.
