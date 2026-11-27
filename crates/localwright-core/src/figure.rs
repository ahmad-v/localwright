//! Minimal print-oriented vector figure export, separate from UI theme.
use crate::{write_bytes, Error, Result};
use std::{fmt::Write, path::Path};

/// Marker geometry for monochrome-safe and color-vision-accessible figures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    Circle,
    Square,
    Triangle,
    Diamond,
    Cross,
}

/// A reusable print-oriented style for a scientific series.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeriesStyle {
    /// CSS color; only validated `#RRGGBB` values are accepted.
    pub color: &'static str,
    pub marker: Marker,
    /// Use a dashed line to distinguish series without relying on color.
    pub dashed: bool,
}

impl SeriesStyle {
    /// A black, solid circle style suitable for monochrome export.
    pub const MONOCHROME: Self = Self {
        color: "#000000",
        marker: Marker::Circle,
        dashed: false,
    };

    fn validate(self) -> Result<Self> {
        let bytes = self.color.as_bytes();
        if bytes.len() != 7 || bytes[0] != b'#' || !bytes[1..].iter().all(u8::is_ascii_hexdigit) {
            return Err(Error::input("Series color must be a #RRGGBB value"));
        }
        Ok(self)
    }
}
/// Render a monochrome scatterplot with explicit axes and no raster content.
/// Points are in data units. The plot uses fixed margins and labels numeric bounds.
pub fn scatter_svg(points: &[(f64, f64)], width: u32, height: u32) -> Result<String> {
    plot_svg(
        points,
        width,
        height,
        PlotKind::Scatter,
        SeriesStyle::MONOCHROME,
    )
}

/// Render a monochrome line chart connecting points in supplied order.
pub fn line_svg(points: &[(f64, f64)], width: u32, height: u32) -> Result<String> {
    if points.len() < 2 {
        return Err(Error::input("A line chart requires at least two points"));
    }
    plot_svg(
        points,
        width,
        height,
        PlotKind::Line,
        SeriesStyle::MONOCHROME,
    )
}

/// Render a scatter plot with a validated color and a non-color marker channel.
pub fn scatter_svg_styled(
    points: &[(f64, f64)],
    width: u32,
    height: u32,
    style: SeriesStyle,
) -> Result<String> {
    let style = style.validate()?;
    plot_svg(points, width, height, PlotKind::Scatter, style)
}

/// Render a line plot with color and dash pattern as independent series channels.
pub fn line_svg_styled(
    points: &[(f64, f64)],
    width: u32,
    height: u32,
    style: SeriesStyle,
) -> Result<String> {
    let style = style.validate()?;
    plot_svg(points, width, height, PlotKind::Line, style)
}

enum PlotKind {
    Scatter,
    Line,
}

fn plot_svg(
    points: &[(f64, f64)],
    width: u32,
    height: u32,
    kind: PlotKind,
    style: SeriesStyle,
) -> Result<String> {
    if points.is_empty()
        || points.len() > 10000
        || !(200..=4000).contains(&width)
        || !(200..=4000).contains(&height)
        || points.iter().any(|(x, y)| !x.is_finite() || !y.is_finite())
    {
        return Err(Error::input("Invalid points or figure dimensions"));
    }
    let (mut xmin, mut xmax, mut ymin, mut ymax) = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for &(x, y) in points {
        xmin = xmin.min(x);
        xmax = xmax.max(x);
        ymin = ymin.min(y);
        ymax = ymax.max(y);
    }
    if xmin == xmax
        || (ymin == ymax && matches!(kind, PlotKind::Scatter))
        || !(xmax - xmin).is_finite()
        || !(ymax - ymin).is_finite()
    {
        return Err(Error::input("Axes need nonconstant finite ranges"));
    }
    let (w, h) = (width as f64, height as f64);
    let title = match kind {
        PlotKind::Scatter => "Scatter plot",
        PlotKind::Line => "Line chart",
    };
    let mut s = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img"><title>{title}</title><rect width="100%" height="100%" fill="white"/><g stroke="black" fill="none"><path d="M 60 20 V {} H {}"/>"#,
        h - 50.0,
        w - 20.0
    );
    let coordinates = points.iter().map(|&(x, y)| {
        let px = 60.0 + (x - xmin) / (xmax - xmin) * (w - 80.0);
        let y_fraction = if ymin == ymax {
            0.5
        } else {
            (y - ymin) / (ymax - ymin)
        };
        let py = h - 50.0 - y_fraction * (h - 70.0);
        (px, py)
    });
    match kind {
        PlotKind::Scatter => {
            for (px, py) in coordinates {
                match style.marker {
                    Marker::Circle => {
                        let _ = write!(
                            s,
                            r#"<circle cx="{px:.3}" cy="{py:.3}" r="3" fill="{}"/>"#,
                            style.color
                        );
                    }
                    Marker::Square => {
                        let _ = write!(
                            s,
                            r#"<rect x="{:.3}" y="{:.3}" width="6" height="6" fill="{}"/>"#,
                            px - 3.0,
                            py - 3.0,
                            style.color
                        );
                    }
                    Marker::Triangle => {
                        let _ = write!(
                            s,
                            r#"<path d="M {px:.3} {:.3} L {:.3} {:.3} L {:.3} {:.3} Z" fill="{}"/>"#,
                            py - 4.0,
                            px + 4.0,
                            py + 3.0,
                            px - 4.0,
                            py + 3.0,
                            style.color
                        );
                    }
                    Marker::Diamond => {
                        let _ = write!(
                            s,
                            r#"<path d="M {px:.3} {:.3} L {:.3} {py:.3} L {px:.3} {:.3} L {:.3} {py:.3} Z" fill="{}"/>"#,
                            py - 4.0,
                            px + 4.0,
                            py + 4.0,
                            px - 4.0,
                            style.color
                        );
                    }
                    Marker::Cross => {
                        let _ = write!(
                            s,
                            r#"<path d="M {:.3} {py:.3} H {:.3} M {px:.3} {:.3} V {:.3}" stroke="{}"/>"#,
                            px - 3.0,
                            px + 3.0,
                            py - 3.0,
                            py + 3.0,
                            style.color
                        );
                    }
                }
            }
        }
        PlotKind::Line => {
            let dash = if style.dashed {
                " stroke-dasharray=\"7 4\""
            } else {
                ""
            };
            let _ = write!(
                s,
                r#"<polyline stroke="{}" stroke-width="2"{} points=""#,
                style.color, dash
            );
            for (px, py) in coordinates {
                let _ = write!(s, " {px:.3},{py:.3}");
            }
            s.push_str(r#""/>"#);
        }
    }
    let _ = write!(
        s,
        r#"</g><g fill="black" font-size="12" font-family="sans-serif"><text x="60" y="{}">{xmin:.4}</text><text x="{}" y="{}" text-anchor="end">{xmax:.4}</text>"#,
        h - 32.0,
        w - 20.0,
        h - 32.0
    );
    if ymin == ymax {
        let _ = write!(
            s,
            r#"<text x="5" y="{}">{ymin:.4}</text>"#,
            (h - 30.0) / 2.0
        );
    } else {
        let _ = write!(
            s,
            r#"<text x="5" y="20">{ymax:.4}</text><text x="5" y="{}">{ymin:.4}</text>"#,
            h - 50.0
        );
    }
    s.push_str("</g></svg>");
    Ok(s)
}
pub fn export_scatter(path: &Path, points: &[(f64, f64)], width: u32, height: u32) -> Result<()> {
    write_bytes(path, scatter_svg(points, width, height)?.as_bytes())
}
/// Write a vector line chart to a new path, preserving any existing file.
pub fn export_line(path: &Path, points: &[(f64, f64)], width: u32, height: u32) -> Result<()> {
    write_bytes(path, line_svg(points, width, height)?.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn line_chart_preserves_order_and_exports_once() {
        let points = [(3.0, 0.0), (1.0, 5.0), (2.0, 2.5)];
        let chart = line_svg(&points, 640, 480).unwrap();
        assert!(chart.contains("<title>Line chart</title>"));
        assert!(chart.contains(r#"points=" 620.000,430.000 60.000,20.000 340.000,225.000""#));
        assert!(!chart.contains("<image"));
        assert!(line_svg(&points[..1], 640, 480).is_err());
        let flat = line_svg(&[(1.0, 2.5), (3.0, 2.5)], 640, 480).unwrap();
        assert!(flat.contains(r#"points=" 60.000,225.000 620.000,225.000""#));
        assert_eq!(flat.matches(">2.5000</text>").count(), 1);
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("line.svg");
        export_line(&output, &points, 640, 480).unwrap();
        assert!(export_line(&output, &points, 640, 480).is_err());
        assert_eq!(std::fs::read_to_string(output).unwrap(), chart);
    }

    #[test]
    fn vector() {
        let s = scatter_svg(&[(0., 0.), (1., 2.)], 640, 480).unwrap();
        assert_eq!(s.matches("<circle").count(), 2);
        assert!(!s.contains("<image"));
        assert!(scatter_svg(&[(1., 2.)], 640, 480).is_err());
    }

    #[test]
    fn accessible_series_style_uses_marker_and_dash_channels() {
        let style = SeriesStyle {
            color: "#336699",
            marker: Marker::Diamond,
            dashed: true,
        };
        let scatter = scatter_svg_styled(&[(0., 0.), (1., 2.)], 640, 480, style).unwrap();
        assert!(scatter.contains("<path d=\"M "));
        assert!(scatter.contains("fill=\"#336699\""));
        let line = line_svg_styled(&[(0., 0.), (1., 2.)], 640, 480, style).unwrap();
        assert!(line.contains("stroke-dasharray=\"7 4\""));
        assert!(line.contains("stroke=\"#336699\""));
        assert!(scatter_svg_styled(
            &[(0., 0.), (1., 2.)],
            640,
            480,
            SeriesStyle {
                color: "red\" onload=\"x",
                ..style
            }
        )
        .is_err());
    }
}
