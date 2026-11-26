# Building Localwright

Run these commands from the repository root. Rustup selects the toolchain in `rust-toolchain.toml`.

```sh
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

On Linux, install the X11 and Wayland development libraries listed in the check workflow. To launch the independent native example from a graphical session, run:

```sh
cargo run --locked -p localwright-ui --example external_consumer
```

The check workflow covers the five supported build targets. Update the committed lockfile deliberately when dependencies change. Consumers should identify a reviewed Localwright revision by its full Git SHA.
