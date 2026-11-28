//! Offline scientific palettes and provenance-aware figure styles.
//!
//! The included maps are selected from Fabio Crameri's perceptually uniform
//! Scientific Colour Maps. Values are quantized from the upstream normalized
//! RGB files to 8-bit sRGB by the checked-in generator.
mod data {
    include!("palette_data.rs");
}

use crate::{figure::SeriesStyle, Error, Result};

/// Scientific use for a palette. Do not use sequential hues for signed values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteKind {
    /// Ordered magnitude with monotonic lightness.
    Sequential,
    /// Values centered on a meaningful midpoint.
    Diverging,
    /// Distinct nominal categories; pair color with markers or line styles.
    Qualitative,
}

/// A bundled scientific color map, with 8-bit sRGB colors encoded as `#RRGGBB`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScientificPalette {
    name: &'static str,
    kind: PaletteKind,
    colors: &'static [&'static str],
}

impl ScientificPalette {
    /// Stable public name of the bundled map.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Intended value semantics for this map.
    pub const fn kind(&self) -> PaletteKind {
        self.kind
    }

    /// Number of available color samples.
    pub const fn len(&self) -> usize {
        self.colors.len()
    }

    /// Whether the palette has no colors. Bundled palettes are nonempty.
    pub const fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    /// Return one color by discrete sample index.
    pub fn color(&self, index: usize) -> Option<&'static str> {
        self.colors.get(index).copied()
    }

    /// Select the nearest palette sample for a normalized value in `[0, 1]`.
    ///
    /// Categorical palettes must be accessed with [`Self::color`] so their
    /// category order is explicit.
    pub fn sample(&self, position: f64) -> Result<&'static str> {
        if self.kind == PaletteKind::Qualitative {
            return Err(Error::input(
                "Use a discrete index for a qualitative palette",
            ));
        }
        if !position.is_finite() || !(0.0..=1.0).contains(&position) {
            return Err(Error::input(
                "Palette position must be finite and in [0, 1]",
            ));
        }
        let index = (position * (self.colors.len() - 1) as f64).round() as usize;
        Ok(self.colors[index])
    }

    /// Build a series style from a discrete palette sample.
    ///
    /// Markers and dashes add independent visual channels for monochrome and
    /// color-vision accessibility. See the crate's figure guidance before
    /// relying on hue alone to distinguish series.
    pub fn series_style(
        &self,
        index: usize,
        marker: crate::figure::Marker,
        dashed: bool,
    ) -> Result<SeriesStyle> {
        let color = self
            .color(index)
            .ok_or_else(|| Error::input("Palette sample index is out of range"))?;
        Ok(SeriesStyle {
            color,
            marker,
            dashed,
        })
    }

    /// Choose the `index`th qualitative palette sample with at least 3:1
    /// contrast against a white figure background, then add marker and dash
    /// channels. This threshold is for non-text plot marks, not small text.
    pub fn series_style_on_white(
        &self,
        index: usize,
        marker: crate::figure::Marker,
        dashed: bool,
    ) -> Result<SeriesStyle> {
        if self.kind != PaletteKind::Qualitative {
            return Err(Error::input(
                "White-background series styles require a qualitative palette",
            ));
        }
        let color = self
            .colors
            .iter()
            .copied()
            .filter(|color| contrast_on_white(color).is_some_and(|ratio| ratio >= 3.0))
            .nth(index)
            .ok_or_else(|| Error::input("No qualifying palette sample for this series index"))?;
        Ok(SeriesStyle {
            color,
            marker,
            dashed,
        })
    }
}

fn contrast_on_white(color: &str) -> Option<f64> {
    let bytes = color.as_bytes();
    if bytes.len() != 7 || bytes[0] != b'#' {
        return None;
    }
    let mut channels = [0.0; 3];
    for (channel, pair) in channels.iter_mut().zip(bytes[1..].chunks_exact(2)) {
        let component =
            u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()? as f64 / 255.0;
        *channel = if component <= 0.04045 {
            component / 12.92
        } else {
            ((component + 0.055) / 1.055).powf(2.4)
        };
    }
    let luminance = 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
    Some(1.05 / (luminance + 0.05))
}

/// All palettes bundled in this version of Localwright.
pub const SCIENTIFIC_PALETTES: &[ScientificPalette] = &[
    ScientificPalette {
        name: "batlow",
        kind: PaletteKind::Sequential,
        colors: data::BATLOW,
    },
    ScientificPalette {
        name: "vik",
        kind: PaletteKind::Diverging,
        colors: data::VIK,
    },
    ScientificPalette {
        name: "batlowS",
        kind: PaletteKind::Qualitative,
        colors: data::BATLOW_S,
    },
];

/// Return the palettes bundled in this crate version.
pub const fn scientific_palettes() -> &'static [ScientificPalette] {
    SCIENTIFIC_PALETTES
}

/// Find an included map by its stable name.
pub fn palette(name: &str) -> Option<&'static ScientificPalette> {
    SCIENTIFIC_PALETTES
        .iter()
        .find(|palette| palette.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::figure::{export_line_svg_styled, line_svg_styled, Marker};

    #[test]
    fn registry_exposes_sequential_diverging_and_categorical_maps() {
        assert_eq!(palette("batlow").unwrap().kind(), PaletteKind::Sequential);
        assert_eq!(palette("vik").unwrap().kind(), PaletteKind::Diverging);
        assert_eq!(palette("batlowS").unwrap().kind(), PaletteKind::Qualitative);
        assert_eq!(palette("batlow").unwrap().len(), 256);
        assert_eq!(palette("vik").unwrap().len(), 256);
        assert_eq!(palette("batlowS").unwrap().len(), 100);
        let categorical = palette("batlowS").unwrap();
        let unique = (0..categorical.len())
            .map(|index| categorical.color(index).unwrap())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(unique.len(), categorical.len());
        assert_eq!(palette("batlow").unwrap().color(0), Some("#011959"));
        assert_eq!(palette("vik").unwrap().sample(0.5).unwrap(), "#ECE5E0");
        assert!(palette("not-present").is_none());
    }

    #[test]
    fn palettes_produce_styles_accepted_by_vector_figure_renderers() {
        let colors = palette("batlowS").unwrap();
        let first = colors.series_style(0, Marker::Circle, false).unwrap();
        let second = colors.series_style(1, Marker::Square, true).unwrap();
        assert_ne!(first.color, second.color);
        let figure = line_svg_styled(&[(0.0, 1.0), (1.0, 2.0)], 640, 480, first).unwrap();
        assert!(figure.contains(&format!("stroke=\"{}\"", first.color)));
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("series.svg");
        export_line_svg_styled(&output, &[(0.0, 1.0), (1.0, 2.0)], 640, 480, second).unwrap();
        assert!(std::fs::read_to_string(&output)
            .unwrap()
            .contains(&format!("stroke=\"{}\"", second.color)));
        assert!(
            export_line_svg_styled(&output, &[(0.0, 1.0), (1.0, 2.0)], 640, 480, second).is_err()
        );
        assert!(colors.series_style(100, Marker::Circle, false).is_err());
        assert!(colors.sample(0.5).is_err());
        assert!(palette("vik").unwrap().sample(f64::NAN).is_err());
        assert!(palette("vik").unwrap().sample(1.1).is_err());
    }

    #[test]
    fn white_background_series_styles_preserve_source_colors_with_graphic_contrast() {
        let categorical = palette("batlowS").unwrap();
        let styles = (0..12)
            .map(|index| categorical.series_style_on_white(index, Marker::Circle, index % 2 == 1))
            .collect::<Result<Vec<_>>>()
            .unwrap();
        assert!(styles
            .iter()
            .all(|style| { contrast_on_white(style.color).is_some_and(|ratio| ratio >= 3.0) }));
        assert_eq!(
            styles
                .iter()
                .map(|style| style.color)
                .collect::<std::collections::HashSet<_>>()
                .len(),
            styles.len()
        );
        assert!(palette("batlow")
            .unwrap()
            .series_style_on_white(0, Marker::Circle, false)
            .is_err());
        assert!(categorical
            .series_style_on_white(62, Marker::Circle, false)
            .is_err());
    }
}
