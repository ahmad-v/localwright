#!/usr/bin/env python3
"""Generate 8-bit Rust palette tables from the vendored Crameri RGB data."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "crates/localwright-core/data/palettes"
OUTPUT = ROOT / "crates/localwright-core/src/palette_data.rs"
SOURCE = json.loads((DATA / "source.json").read_text(encoding="utf-8"))
FILES = {"BATLOW": "batlow.txt", "VIK": "vik.txt", "BATLOW_S": "batlowS.txt"}

lines = ["// Generated 8-bit palette values. Regenerate with scripts/generate_scientific_palette_data.py.\n"]
for constant, filename in FILES.items():
    data = (DATA / filename).read_text(encoding="utf-8")
    expected = SOURCE["files"][filename]["sha256"]
    actual = hashlib.sha256(data.encode()).hexdigest()
    if actual != expected:
        raise SystemExit(f"{filename}: SHA-256 {actual} does not match {expected}")
    colors: list[str] = []
    for line_no, line in enumerate(data.splitlines(), 1):
        components = [float(value) for value in line.split()]
        if len(components) != 3 or any(not 0.0 <= value <= 1.0 for value in components):
            raise SystemExit(f"{filename}:{line_no}: expected three values in [0,1]")
        rgb = [round(value * 255) for value in components]
        colors.append("#" + "".join(f"{component:02X}" for component in rgb))
    if len(colors) != SOURCE["files"][filename]["entries"]:
        raise SystemExit(f"{filename}: unexpected palette length {len(colors)}")
    lines.append(f"pub(super) const {constant}: &[&str] = &[\n")
    lines.extend(f'    "{color}",\n' for color in colors)
    lines.append("];\n")
OUTPUT.write_text("".join(lines), encoding="utf-8")
print(f"generated {OUTPUT.relative_to(ROOT)} from {len(FILES)} pinned maps")
