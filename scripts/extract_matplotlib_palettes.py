#!/usr/bin/env python3
"""Recreate pinned Matplotlib RGB tables from an explicitly supplied source file."""
from __future__ import annotations

import argparse
import ast
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "crates/localwright-core/data/palettes"
SOURCE = json.loads((DATA / "source.json").read_text(encoding="utf-8"))
MATPLOTLIB = SOURCE["secondary_sources"]["matplotlib"]


def extract(source_file: Path, output_dir: Path) -> None:
    raw = source_file.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    if digest != MATPLOTLIB["source_sha256"]:
        raise SystemExit(f"source SHA-256 {digest} does not match the pinned revision")

    module = ast.parse(raw, filename=str(source_file))
    assignments = {
        target.id: node.value
        for node in module.body
        if isinstance(node, ast.Assign)
        for target in node.targets
        if isinstance(target, ast.Name) and target.id in MATPLOTLIB["arrays"].values()
    }
    output_dir.mkdir(parents=True, exist_ok=True)
    for palette_name, array_name in MATPLOTLIB["arrays"].items():
        if array_name not in assignments:
            raise SystemExit(f"pinned source is missing {array_name}")
        colors = ast.literal_eval(assignments[array_name])
        if len(colors) != 256 or any(
            len(rgb) != 3
            or any(not isinstance(channel, (int, float)) or not 0.0 <= channel <= 1.0 for channel in rgb)
            for rgb in colors
        ):
            raise SystemExit(f"{array_name} must contain 256 normalized RGB triples")
        text = "".join(" ".join(f"{channel:.6f}" for channel in rgb) + "\n" for rgb in colors)
        filename = f"{palette_name}.txt"
        expected = SOURCE["files"][filename]["sha256"]
        actual = hashlib.sha256(text.encode("utf-8")).hexdigest()
        if actual != expected:
            raise SystemExit(f"{filename}: extracted SHA-256 {actual} does not match {expected}")
        with (output_dir / filename).open("w", encoding="utf-8", newline="\n") as output:
            output.write(text)
        print(f"verified and wrote {output_dir / filename}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source_file", type=Path, help="Matplotlib 3.10.8 _cm_listed.py at the pinned revision")
    parser.add_argument("--output-dir", type=Path, required=True, help="directory for verified RGB text tables")
    arguments = parser.parse_args()
    extract(arguments.source_file, arguments.output_dir)


if __name__ == "__main__":
    main()
