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

/// Descriptive labels rendered into a scientific figure's visible title and axes.
/// Text is bounded and escaped before it is inserted into SVG markup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FigureLabels<'a> {
    pub title: &'a str,
    pub x_axis: &'a str,
    pub y_axis: &'a str,
}

impl FigureLabels<'_> {
    fn validate(self) -> Result<Self> {
        for label in [self.title, self.x_axis, self.y_axis] {
            if label.chars().count() > 256 || label.chars().any(char::is_control) {
                return Err(Error::input(
                    "Figure labels must contain at most 256 printable characters",
                ));
            }
        }
        Ok(self)
    }
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
        None,
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
        None,
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
    plot_svg(points, width, height, PlotKind::Scatter, style, None)
}

/// Render a line plot with color and dash pattern as independent series channels.
pub fn line_svg_styled(
    points: &[(f64, f64)],
    width: u32,
    height: u32,
    style: SeriesStyle,
) -> Result<String> {
    let style = style.validate()?;
    plot_svg(points, width, height, PlotKind::Line, style, None)
}

/// Render a scatterplot with visible title and scientific axis labels.
pub fn scatter_svg_labeled(
    points: &[(f64, f64)],
    width: u32,
    height: u32,
    style: SeriesStyle,
    labels: FigureLabels<'_>,
) -> Result<String> {
    plot_svg(
        points,
        width,
        height,
        PlotKind::Scatter,
        style.validate()?,
        Some(labels.validate()?),
    )
}

/// Render a line chart with visible title and scientific axis labels.
pub fn line_svg_labeled(
    points: &[(f64, f64)],
    width: u32,
    height: u32,
    style: SeriesStyle,
    labels: FigureLabels<'_>,
) -> Result<String> {
    if points.len() < 2 {
        return Err(Error::input("A line chart requires at least two points"));
    }
    plot_svg(
        points,
        width,
        height,
        PlotKind::Line,
        style.validate()?,
        Some(labels.validate()?),
    )
}

/// Export a labeled scatter figure to a new SVG path without overwriting an existing file.
pub fn export_scatter_svg_labeled(
    path: &Path,
    points: &[(f64, f64)],
    width: u32,
    height: u32,
    style: SeriesStyle,
    labels: FigureLabels<'_>,
) -> Result<()> {
    write_bytes(
        path,
        scatter_svg_labeled(points, width, height, style, labels)?.as_bytes(),
    )
}

/// Export a labeled line figure to a new SVG path without overwriting an existing file.
pub fn export_line_svg_labeled(
    path: &Path,
    points: &[(f64, f64)],
    width: u32,
    height: u32,
    style: SeriesStyle,
    labels: FigureLabels<'_>,
) -> Result<()> {
    write_bytes(
        path,
        line_svg_labeled(points, width, height, style, labels)?.as_bytes(),
    )
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
    labels: Option<FigureLabels<'_>>,
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
    let default_title = match kind {
        PlotKind::Scatter => "Scatter plot",
        PlotKind::Line => "Line chart",
    };
    let title = labels.map_or(default_title, |labels| labels.title);
    let title = escape_xml(title);
    let (left, top, right, bottom) = if labels.is_some() {
        (72.0, 40.0, 24.0, 66.0)
    } else {
        (60.0, 20.0, 20.0, 50.0)
    };
    let plot_bottom = h - bottom;
    let plot_width = w - left - right;
    let plot_height = plot_bottom - top;
    let mut s = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img"><title>{title}</title><rect width="100%" height="100%" fill="white"/><g stroke="black" fill="none"><path d="M {left} {top} V {plot_bottom} H {}"/>"#,
        w - right
    );
    if labels.is_some() {
        let _ = write!(
            s,
            r#"<text x="{}" y="22" text-anchor="middle" fill="black" font-size="14" font-family="sans-serif" textLength="{plot_width}" lengthAdjust="spacingAndGlyphs">{title}</text>"#,
            (left + w - right) / 2.0
        );
    }
    let coordinates = points.iter().map(|&(x, y)| {
        let px = left + (x - xmin) / (xmax - xmin) * plot_width;
        let y_fraction = if ymin == ymax {
            0.5
        } else {
            (y - ymin) / (ymax - ymin)
        };
        let py = plot_bottom - y_fraction * plot_height;
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
        r#"</g><g fill="black" font-size="12" font-family="sans-serif"><text x="{left}" y="{}">{xmin:.4}</text><text x="{}" y="{}" text-anchor="end">{xmax:.4}</text>"#,
        if labels.is_some() { h - 44.0 } else { h - 32.0 },
        w - right,
        if labels.is_some() { h - 44.0 } else { h - 32.0 }
    );
    if ymin == ymax {
        let _ = write!(
            s,
            r#"<text x="{}" y="{}" text-anchor="end">{ymin:.4}</text>"#,
            left - 8.0,
            (top + plot_bottom) / 2.0
        );
    } else {
        let _ = write!(
            s,
            r#"<text x="{}" y="{}" text-anchor="end">{ymax:.4}</text><text x="{}" y="{}" text-anchor="end">{ymin:.4}</text>"#,
            left - 8.0,
            top + 4.0,
            left - 8.0,
            plot_bottom
        );
    }
    if let Some(labels) = labels {
        let x_axis = escape_xml(labels.x_axis);
        let y_axis = escape_xml(labels.y_axis);
        let _ = write!(
            s,
            r#"<text x="{}" y="{}" text-anchor="middle" fill="black" font-size="12" font-family="sans-serif" textLength="{plot_width}" lengthAdjust="spacingAndGlyphs">{x_axis}</text><text x="16" y="{}" transform="rotate(-90 16 {})" text-anchor="middle" fill="black" font-size="12" font-family="sans-serif" textLength="{plot_height}" lengthAdjust="spacingAndGlyphs">{y_axis}</text>"#,
            (left + w - right) / 2.0,
            h - 10.0,
            (top + plot_bottom) / 2.0,
            (top + plot_bottom) / 2.0
        );
    }
    s.push_str("</g></svg>");
    Ok(s)
}

fn escape_xml(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(character),
        }
    }
    escaped
}
pub fn export_scatter(path: &Path, points: &[(f64, f64)], width: u32, height: u32) -> Result<()> {
    write_bytes(path, scatter_svg(points, width, height)?.as_bytes())
}
/// Export a styled scatter plot to a new path without overwriting an existing file.
pub fn export_scatter_svg_styled(
    path: &Path,
    points: &[(f64, f64)],
    width: u32,
    height: u32,
    style: SeriesStyle,
) -> Result<()> {
    write_bytes(
        path,
        scatter_svg_styled(points, width, height, style)?.as_bytes(),
    )
}
/// Write a vector line chart to a new path, preserving any existing file.
pub fn export_line(path: &Path, points: &[(f64, f64)], width: u32, height: u32) -> Result<()> {
    write_bytes(path, line_svg(points, width, height)?.as_bytes())
}
/// Export a styled line chart to a new path without overwriting an existing file.
pub fn export_line_svg_styled(
    path: &Path,
    points: &[(f64, f64)],
    width: u32,
    height: u32,
    style: SeriesStyle,
) -> Result<()> {
    write_bytes(
        path,
        line_svg_styled(points, width, height, style)?.as_bytes(),
    )
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

    #[test]
    fn labeled_figure_escapes_metadata_and_reserves_axis_margins() {
        let labels = FigureLabels {
            title: "Dose < response",
            x_axis: "Dose & time (mg·h)",
            y_axis: "Δ response %",
        };
        let figure = scatter_svg_labeled(
            &[(0.0, 1.0), (1.0, 3.0)],
            640,
            480,
            SeriesStyle::MONOCHROME,
            labels,
        )
        .unwrap();

        assert!(figure.contains("<title>Dose &lt; response</title>"));
        assert!(figure.contains(">Dose &lt; response</text>"));
        assert!(figure.contains("Dose &amp; time (mg·h)</text>"));
        assert!(figure.contains("rotate(-90 16 227)"));
        assert!(figure.contains("M 72 40 V 414 H 616"));
        assert!(!figure.contains("<image"));

        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("dose-response.svg");
        export_scatter_svg_labeled(
            &output,
            &[(0.0, 1.0), (1.0, 3.0)],
            640,
            480,
            SeriesStyle::MONOCHROME,
            labels,
        )
        .unwrap();
        assert_eq!(std::fs::read_to_string(&output).unwrap(), figure);
        assert!(export_scatter_svg_labeled(
            &output,
            &[(0.0, 1.0), (1.0, 3.0)],
            640,
            480,
            SeriesStyle::MONOCHROME,
            labels,
        )
        .is_err());
    }

    #[test]
    fn labeled_line_chart_rejects_control_and_unbounded_labels() {
        let points = [(0.0, 1.0), (1.0, 2.0)];
        let style = SeriesStyle::MONOCHROME;
        assert!(line_svg_labeled(
            &points,
            640,
            480,
            style,
            FigureLabels {
                title: "bad\nlabel",
                x_axis: "x",
                y_axis: "y",
            }
        )
        .is_err());
        let long_label = "x".repeat(257);
        assert!(line_svg_labeled(
            &points,
            640,
            480,
            style,
            FigureLabels {
                title: &long_label,
                x_axis: "x",
                y_axis: "y",
            }
        )
        .is_err());
    }
}
