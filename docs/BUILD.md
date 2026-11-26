# Building the alpha snapshot

Extract all five source archives into one directory as siblings. Use Rust toolchain from rust-toolchain.toml. Run `cargo generate-lockfile`, review dependency resolution, then `cargo fmt --all`, `cargo check --locked --all-targets`, `cargo test --locked`, `cargo clippy --locked --all-targets`. Run `cargo run -p localwright-ui --example external_consumer` for the independent GUI example.

On Linux, development needs platform libraries for Iced (X11/Wayland, keyboard and graphics). CI installs candidate build dependencies. No system packages are installed by applications. 

## Integration

Commit and tag a reviewed Localwright revision first. Ensure all consumers use that exact revision.

The check workflow intentionally fails if Cargo.lock is absent. The manually dispatched bootstrap workflow produces a candidate lockfile artifact for review, never commits it. Commit reviewed lockfiles. The check gate includes formatting, so run rustfmt before first green check. No package workflow can publish a release. No default OS support versions have been validated.
