# Third-party notices

## Scientific Colour Maps

The bundled `batlow`, `vik`, and `batlowS` RGB palette data is from Fabio Crameri's Scientific Colour Maps 8.0.0, through the `cmcrameri` data files at upstream commit `87197e141a2c451691ec53fadc6da4d017f79a4c`. `batlowS` is an upstream categorical palette. Source URLs and SHA-256 values are recorded in `data/palettes/source.json`; normalized RGB values are quantized to 8-bit sRGB by `scripts/generate_scientific_palette_data.py`.

The palette data is licensed under MIT. Copyright (c) 2020 Fabio Crameri. The full upstream license text is included at `data/palettes/LICENSE.txt`. Scientific use should cite Fabio Crameri's Scientific Colour Maps (2018) and Crameri, Shephard, and Heron (2020), *The misuse of colour in science communication*, Nature Communications 11, 5444, DOI `10.1038/s41467-020-19160-7`.

## Matplotlib sequential maps

The bundled `viridis`, `cividis`, `plasma`, `inferno`, and `magma` RGB sample arrays are extracted from Matplotlib 3.10.8's `lib/matplotlib/_cm_listed.py` at immutable commit `1392cbe3c79cdb93f9282747841d648770f60249`. The original source file SHA-256, array names, source URLs, and hashes of the extracted normalized RGB tables are recorded in `data/palettes/source.json`.

These palette data are redistributed under the Matplotlib License Agreement for versions 1.3.0 and later. The full agreement is included as `data/palettes/MATPLOTLIB-LICENSE.txt`; retain its notice: `Copyright (c) 2012- Matplotlib Development Team; All Rights Reserved`. The source arrays were extracted to standalone six-decimal normalized RGB text tables. The generator verifies each table hash and converts the encoded sRGB values to 8-bit hexadecimal stops. No Matplotlib code or Python runtime is included.

The Matplotlib source defines these as listed colormaps. Localwright records their sequential use and adds component-wise encoded-sRGB interpolation between adjacent published stops for continuous sampling. Use indexed `color` access when exact source stops are required.

Okabe-Ito is not bundled. Its official source and captured SHA-256 evidence are recorded as an unbundled gap in `data/palettes/source.json`; the official materials identify their copyright holders but no explicit software redistribution license was located. Obtain maintainer permission or a suitable license before adding it to a distributable package.

No third-party engine binaries, models, icons or fonts are redistributed in this source snapshot. Cargo dependencies are declared in manifests. Their full transitive inventory, licenses, notices, advisories and redistribution obligations are unresolved until dependency resolution and review. Do not distribute binaries before completing these checks. See `docs/DEPENDENCIES.md`.

The opt-in `figure-pdf` feature uses `svg2pdf` 0.13.0 and its `usvg` parser to convert Localwright-generated SVG figures to PDF. `svg2pdf` and `usvg` are licensed MIT OR Apache-2.0; their source and license texts are available from their crates.io packages and upstream repositories. The feature loads host-installed fonts and converts glyphs to outlines without embedding font files. Host font licensing terms still apply to output created with those fonts. Review the resolved dependency graph before distributing binaries with this feature.
