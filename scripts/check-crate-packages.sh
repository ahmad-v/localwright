#!/usr/bin/env bash
# Rehearse first publication without uploading to crates.io.
set -euo pipefail
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"$root/target"}
target=$(mkdir -p "$CARGO_TARGET_DIR" && cd "$CARGO_TARGET_DIR" && pwd)
export CARGO_TARGET_DIR="$target"

stage=$(mktemp -d)
trap 'rm -rf -- "$stage"' EXIT
cp "$root"/Cargo.{toml,lock} "$stage/"
cp -R "$root/crates" "$stage/"
cd "$stage"

version=$(python3 -c 'import tomllib; print(tomllib.load(open("crates/localwright-core/Cargo.toml", "rb"))["package"]["version"])')
core="$target/package/localwright-core-$version"
ui="$target/package/localwright-ui-$version"

cargo package --locked -p localwright-core
# The scratch checkout may regenerate its lockfile to use the verified core
# tarball in place of the still-unpublished registry dependency.
core_patch=$(python3 -c 'import json,sys; print("patch.crates-io.localwright-core.path="+json.dumps(sys.argv[1]))' "$core")
cargo --config "$core_patch" package -p localwright-ui

mkdir -p consumer/src
python3 - "$core" "$ui" "$version" <<'PY'
import json
import pathlib
import sys

core, ui, version = map(json.dumps, sys.argv[1:])
pathlib.Path("consumer/Cargo.toml").write_text(f"""
[package]
name = "localwright-package-smoke"
edition = "2021"
version = "0.0.0"
publish = false
[workspace]

[dependencies]
localwright-core = {version}
localwright-ui = {version}

[patch.crates-io]
localwright-core.path = {core}
localwright-ui.path = {ui}
""")
example = pathlib.Path(sys.argv[2], "README.md").read_text().split('```rust,no_run\n', 1)[1]
pathlib.Path("consumer/src/main.rs").write_text(example.split('```', 1)[0])
PY
cargo check --manifest-path consumer/Cargo.toml
