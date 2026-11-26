# Publishing Localwright crates

The first registry release targets `0.1.0` on Rust 1.88. The core crate is independent. The UI crate depends on the compatible core release. The four studio applications have their own packaging and release process.

## Release review

1. Review and approve the exact source commit, the MIT OR Apache-2.0 license declaration, attribution and third-party obligations. The repository's current provisional license and citation text require maintainer review.
2. Review CI and package contents. Run formatting, locked tests, strict Clippy and rustdoc. On Linux, run `bash scripts/check-crate-packages.sh` to package both crates and compile an external consumer against their extracted sources.
3. Sign into [crates.io](https://crates.io/) with your GitHub account, verify your account email, and check that both crate names can be claimed. Publishing the first version claims the name for your account. A name lookup does not reserve it.
4. Create a short-lived API token with the scopes needed to publish new crates. Keep it in your local Cargo credential provider. Run `cargo login` and enter the token at the prompt. Keep tokens out of Git, chat, shell arguments and logs.

## Publish core, then UI

Use a clean checkout of the approved source commit. Uploads cannot be overwritten, so verify each dry run first.

```sh
cargo publish --locked --dry-run -p localwright-core
cargo publish --locked -p localwright-core
cargo info localwright-core@0.1.0 --registry crates-io
cargo publish --locked --dry-run -p localwright-ui
cargo publish --locked -p localwright-ui
cargo info localwright-ui@0.1.0 --registry crates-io
```

Wait for core to resolve from the registry before the UI dry run. Verify both crates resolve and compile in a clean third-party project, then create the release tag and verify docs.rs. Revoke the first-publish token after the release. Future automated publication can use crates.io trusted publishing after the maintainer configures the specific repository and workflow. A broken version can be yanked and superseded with a new version.

References: [Cargo publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html) .
