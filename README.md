# Localwright

Localwright is a Rust toolkit for native, offline desktop tools. You can share a handler between a command-line interface and a native Iced workbench.

| Crate | Use it for |
|---|---|
| [localwright-core](crates/localwright-core/README.md) | Operation contracts, cooperative jobs, bounded file reads, no-overwrite writes, JSON settings, labeled multi-series SVG figures with legends, and eight licensed scientific palettes |
| [localwright-ui](crates/localwright-ui/README.md) | A native workbench with independent project tabs, explicit versioned session save/restore, typed paginated tables, twenty themes, and UI scaling |

The first crates.io release is being prepared as `0.1.0`. The four studios currently consume reviewed Git revisions independently of registry publication.

## Build and try it

Use Rust 1.88. On Linux see the [build notes](docs/BUILD.md) for graphical prerequisites.

```sh
cargo test --locked
cargo run --locked -p localwright-ui --example external_consumer
```

See the [architecture](docs/ARCHITECTURE.md), [verification status](docs/STATUS.md), [release process](docs/CRATES_RELEASE.md), and [master specification](MASTER_SPECIFICATION.md).

## License and contributions

The intended original-code license is MIT OR Apache-2.0. The [license files](LICENSE) and [third-party notices](THIRD_PARTY_NOTICES.md) describe the publication review. See [CONTRIBUTING](CONTRIBUTING.md) for development and contribution details.
