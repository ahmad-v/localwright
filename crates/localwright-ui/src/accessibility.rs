//! Small, dependency-free helpers for legible focus indicators.

/// Select black or white for a focus outline against an sRGB background.
///
/// The returned color has the greater WCAG relative-luminance contrast. It is
/// suitable for non-text focus decoration; it does not imply that the rest of
/// a UI meets a contrast or accessibility standard.
pub fn high_contrast_focus_color(background: [u8; 3]) -> [u8; 3] {
    if contrast_ratio([0, 0, 0], background) >= contrast_ratio([255, 255, 255], background) {
        [0, 0, 0]
    } else {
        [255, 255, 255]
    }
}

/// Calculate the WCAG contrast ratio between two opaque sRGB colors.
pub fn contrast_ratio(first: [u8; 3], second: [u8; 3]) -> f64 {
    let first = relative_luminance(first);
    let second = relative_luminance(second);
    (first.max(second) + 0.05) / (first.min(second) + 0.05)
}

fn relative_luminance(color: [u8; 3]) -> f64 {
    color
        .into_iter()
        .map(|channel| {
            let value = f64::from(channel) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        })
        .zip([0.2126, 0.7152, 0.0722])
        .map(|(channel, weight)| channel * weight)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_outline_selects_the_higher_contrast_extreme() {
        for background in [[0, 0, 0], [255, 255, 255], [127, 127, 127], [250, 250, 250]] {
            let focus = high_contrast_focus_color(background);
            let other = if focus == [0, 0, 0] {
                [255, 255, 255]
            } else {
                [0, 0, 0]
            };
            assert!(contrast_ratio(focus, background) >= contrast_ratio(other, background));
            assert!(contrast_ratio(focus, background) >= 4.5);
        }
    }

    #[test]
    fn focus_outline_covers_every_builtin_theme_with_accessible_contrast() {
        for (_, background, _, _) in crate::THEMES {
            assert!(contrast_ratio(high_contrast_focus_color(*background), *background) >= 4.5);
        }
    }
}
