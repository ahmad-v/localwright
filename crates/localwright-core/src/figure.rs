//! Minimal print-oriented vector figure export, separate from UI theme.
use crate::{write_bytes, Error, Result};
use std::{fmt::Write, path::Path};

/// Physical dimensions for PDF export. SVG output uses CSS pixels at 96 DPI,
/// so the PDF page preserves the same figure layout at the requested size.
#[cfg(feature = "figure-pdf")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FigureDimensions {
    pub width_mm: f64,
    pub height_mm: f64,
}

#[cfg(feature = "figure-pdf")]
impl FigureDimensions {
    pub const fn new(width_mm: f64, height_mm: f64) -> Self {
        Self {
            width_mm,
            height_mm,
        }
    }

    fn css_pixels(self) -> Result<(u32, u32)> {
        fn dimension(mm: f64) -> Option<u32> {
            if !mm.is_finite() || mm <= 0.0 {
                return None;
            }
            let pixels = (mm * 96.0 / 25.4).round();
            if !(200.0..=4000.0).contains(&pixels) {
                return None;
            }
            Some(pixels as u32)
        }

        let width = dimension(self.width_mm).ok_or_else(|| {
            Error::input("Figure width must map to 200..=4000 CSS pixels at 96 DPI")
        })?;
        let height = dimension(self.height_mm).ok_or_else(|| {
            Error::input("Figure height must map to 200..=4000 CSS pixels at 96 DPI")
        })?;
        Ok((width, height))
    }
}

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

/// One named series in a shared-axis scientific line or scatter figure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FigureSeries<'a> {
    /// Human-readable series label shown in the figure legend.
    pub label: &'a str,
    /// Ordered data-coordinate points.
    pub points: &'a [(f64, f64)],
    /// Palette-selected color, marker and line-style channels.
    pub style: SeriesStyle,
}

impl<'a> FigureSeries<'a> {
    /// Construct a named series from points and a validated-at-render style.
    pub const fn new(label: &'a str, points: &'a [(f64, f64)], style: SeriesStyle) -> Self {
        Self {
            label,
            points,
            style,
        }
    }
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

    /// Select a black-only line style with a distinct marker and dash pattern.
    ///
    /// The first five series use solid lines and the next five use dashed
    /// lines. Each group cycles through five distinct markers, providing ten
    /// distinguishable combinations before the style set repeats. This is a
    /// useful starting point for print-safe figures; labels and legends remain
    /// necessary for interpretation.
    pub fn monochrome_line(series_index: usize) -> Result<Self> {
        const MARKERS: [Marker; 5] = [
            Marker::Circle,
            Marker::Square,
            Marker::Triangle,
            Marker::Diamond,
            Marker::Cross,
        ];
        if series_index >= MARKERS.len() * 2 {
            return Err(Error::input(
                "Monochrome line styles support at most ten distinct series",
            ));
        }
        Ok(Self {
            color: "#000000",
            marker: MARKERS[series_index % MARKERS.len()],
            dashed: series_index >= MARKERS.len(),
        })
    }

    /// Select a black-only scatter style with a distinct point marker.
    ///
    /// Scatter series have no connecting stroke, so dash patterns cannot
    /// distinguish them. This method provides five marker styles and rejects
    /// larger indices instead of silently assigning a duplicate.
    pub fn monochrome_scatter(series_index: usize) -> Result<Self> {
        const MARKERS: [Marker; 5] = [
            Marker::Circle,
            Marker::Square,
            Marker::Triangle,
            Marker::Diamond,
            Marker::Cross,
        ];
        let marker = MARKERS.get(series_index).copied().ok_or_else(|| {
            Error::input("Monochrome scatter styles support at most five distinct series")
        })?;
        Ok(Self {
            color: "#000000",
            marker,
            dashed: false,
        })
    }

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

/// Render multiple series on shared axes with an accessible legend and redundant encodings.
///
/// Each entry in the legend shows the series marker and color. Line figures also
/// show the dash pattern and markers on the plotted line. The layout reserves
/// space for the legend and rejects figures too small to display it safely.
pub fn line_series_svg_labeled(
    series: &[FigureSeries<'_>],
    width: u32,
    height: u32,
    labels: FigureLabels<'_>,
) -> Result<String> {
    multi_series_svg(series, width, height, PlotKind::Line, labels.validate()?)
}

/// Render multiple scatter series on shared axes with an accessible legend.
pub fn scatter_series_svg_labeled(
    series: &[FigureSeries<'_>],
    width: u32,
    height: u32,
    labels: FigureLabels<'_>,
) -> Result<String> {
    multi_series_svg(series, width, height, PlotKind::Scatter, labels.validate()?)
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

/// Export labeled multi-series line SVG to a new path without overwriting a file.
pub fn export_line_series_svg_labeled(
    path: &Path,
    series: &[FigureSeries<'_>],
    width: u32,
    height: u32,
    labels: FigureLabels<'_>,
) -> Result<()> {
    write_bytes(
        path,
        line_series_svg_labeled(series, width, height, labels)?.as_bytes(),
    )
}

/// Export labeled multi-series scatter SVG to a new path without overwriting a file.
pub fn export_scatter_series_svg_labeled(
    path: &Path,
    series: &[FigureSeries<'_>],
    width: u32,
    height: u32,
    labels: FigureLabels<'_>,
) -> Result<()> {
    write_bytes(
        path,
        scatter_series_svg_labeled(series, width, height, labels)?.as_bytes(),
    )
}

/// Render a labeled scatter figure to a vector PDF buffer at physical page
/// dimensions. Its SVG input is produced by the same renderer as SVG export.
#[cfg(feature = "figure-pdf")]
pub fn scatter_pdf(
    points: &[(f64, f64)],
    dimensions: FigureDimensions,
    style: SeriesStyle,
    labels: FigureLabels<'_>,
) -> Result<Vec<u8>> {
    let (width, height) = dimensions.css_pixels()?;
    let svg = scatter_svg_labeled(points, width, height, style, labels)?;
    svg_to_pdf(&svg)
}

/// Render a labeled line figure to a vector PDF buffer at physical page
/// dimensions. Its SVG input is produced by the same renderer as SVG export.
#[cfg(feature = "figure-pdf")]
pub fn line_pdf(
    points: &[(f64, f64)],
    dimensions: FigureDimensions,
    style: SeriesStyle,
    labels: FigureLabels<'_>,
) -> Result<Vec<u8>> {
    let (width, height) = dimensions.css_pixels()?;
    let svg = line_svg_labeled(points, width, height, style, labels)?;
    svg_to_pdf(&svg)
}

/// Render a labeled multi-series line figure to a vector PDF buffer.
#[cfg(feature = "figure-pdf")]
pub fn line_series_pdf(
    series: &[FigureSeries<'_>],
    dimensions: FigureDimensions,
    labels: FigureLabels<'_>,
) -> Result<Vec<u8>> {
    let (width, height) = dimensions.css_pixels()?;
    let svg = line_series_svg_labeled(series, width, height, labels)?;
    svg_to_pdf(&svg)
}

/// Render a labeled multi-series scatter figure to a vector PDF buffer.
#[cfg(feature = "figure-pdf")]
pub fn scatter_series_pdf(
    series: &[FigureSeries<'_>],
    dimensions: FigureDimensions,
    labels: FigureLabels<'_>,
) -> Result<Vec<u8>> {
    let (width, height) = dimensions.css_pixels()?;
    let svg = scatter_series_svg_labeled(series, width, height, labels)?;
    svg_to_pdf(&svg)
}

/// Export a labeled multi-series line figure as a new vector PDF file.
#[cfg(feature = "figure-pdf")]
pub fn export_line_series_pdf(
    path: &Path,
    series: &[FigureSeries<'_>],
    dimensions: FigureDimensions,
    labels: FigureLabels<'_>,
) -> Result<()> {
    write_bytes(path, &line_series_pdf(series, dimensions, labels)?)
}

/// Export a labeled multi-series scatter figure as a new vector PDF file.
#[cfg(feature = "figure-pdf")]
pub fn export_scatter_series_pdf(
    path: &Path,
    series: &[FigureSeries<'_>],
    dimensions: FigureDimensions,
    labels: FigureLabels<'_>,
) -> Result<()> {
    write_bytes(path, &scatter_series_pdf(series, dimensions, labels)?)
}

#[cfg(feature = "figure-pdf")]
fn svg_to_pdf(svg: &str) -> Result<Vec<u8>> {
    let mut options = svg2pdf::usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    const FONT_PREFERENCE: &[&str] = &[
        "Arial",
        "Liberation Sans",
        "DejaVu Sans",
        "Noto Sans",
        "Segoe UI",
        "Helvetica",
    ];
    let faces = options.fontdb.faces().collect::<Vec<_>>();
    let font_family = FONT_PREFERENCE
        .iter()
        .find_map(|preferred| {
            faces.iter().find_map(|face| {
                face.families
                    .iter()
                    .find(|(family, _)| family.eq_ignore_ascii_case(preferred))
                    .map(|(family, _)| family.as_str())
            })
        })
        .or_else(|| {
            faces
                .iter()
                .find_map(|face| face.families.first().map(|(family, _)| family.as_str()))
        })
        .filter(|family| {
            !family.is_empty()
                && family.chars().count() <= 128
                && !family.chars().any(char::is_control)
        })
        .ok_or_else(|| Error::engine("PDF figure export requires an installed system font"))?;
    let escaped_family = escape_xml_attribute(font_family);
    let svg = svg.replace(
        "font-family=\"sans-serif\"",
        &format!("font-family=\"{escaped_family}\""),
    );
    let tree = svg2pdf::usvg::Tree::from_str(&svg, &options).map_err(|error| {
        Error::engine(format!(
            "Could not prepare the scientific figure for PDF export: {error}"
        ))
    })?;
    // Convert labels to vector outlines. The PDF remains font-independent and
    // does not embed host font files; labels may reflect the host's font metrics.
    svg2pdf::to_pdf(
        &tree,
        svg2pdf::ConversionOptions {
            embed_text: false,
            ..svg2pdf::ConversionOptions::default()
        },
        svg2pdf::PageOptions { dpi: 96.0 },
    )
    .map_err(|error| {
        Error::engine(format!(
            "Could not convert the scientific figure to PDF: {error}"
        ))
    })
}

#[cfg(feature = "figure-pdf")]
fn escape_xml_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

enum PlotKind {
    Scatter,
    Line,
}

const MULTI_LEFT: f64 = 72.0;
const MULTI_TOP: f64 = 40.0;
const MULTI_RIGHT: f64 = 222.0;
const MULTI_BOTTOM: f64 = 66.0;
const MULTI_LEGEND_ROW: f64 = 18.0;
const MAX_FIGURE_SERIES: usize = 12;
const MAX_SERIES_POINTS: usize = 10000;

fn multi_series_svg(
    series: &[FigureSeries<'_>],
    width: u32,
    height: u32,
    kind: PlotKind,
    labels: FigureLabels<'_>,
) -> Result<String> {
    if series.is_empty()
        || series.len() > MAX_FIGURE_SERIES
        || !(400..=4000).contains(&width)
        || !(200..=4000).contains(&height)
    {
        return Err(Error::input(
            "Invalid multi-series figure count or dimensions (series 1..=12, width 400..=4000, height 200..=4000)",
        ));
    }
    let mut point_count = 0usize;
    let (mut xmin, mut xmax, mut ymin, mut ymax) = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for (index, entry) in series.iter().enumerate() {
        if entry.label.is_empty()
            || entry.label.chars().count() > 80
            || entry.label.chars().any(char::is_control)
            || series[..index]
                .iter()
                .any(|prior| prior.label == entry.label)
            || entry.points.is_empty()
            || (matches!(kind, PlotKind::Line) && entry.points.len() < 2)
        {
            return Err(Error::input(
                "Each figure series needs a printable label and valid points",
            ));
        }
        entry.style.validate()?;
        point_count = point_count
            .checked_add(entry.points.len())
            .filter(|&count| count <= MAX_SERIES_POINTS)
            .ok_or_else(|| Error::input("Multi-series figure exceeds 10,000 total points"))?;
        for &(x, y) in entry.points {
            if !x.is_finite() || !y.is_finite() {
                return Err(Error::input("Figure points must be finite"));
            }
            xmin = xmin.min(x);
            xmax = xmax.max(x);
            ymin = ymin.min(y);
            ymax = ymax.max(y);
        }
    }
    if xmin == xmax
        || (ymin == ymax && matches!(kind, PlotKind::Scatter))
        || !(xmax - xmin).is_finite()
        || !(ymax - ymin).is_finite()
    {
        return Err(Error::input("Axes need nonconstant finite ranges"));
    }

    let (w, h) = (width as f64, height as f64);
    let plot_right = w - MULTI_RIGHT;
    let plot_bottom = h - MULTI_BOTTOM;
    let plot_width = plot_right - MULTI_LEFT;
    let plot_height = plot_bottom - MULTI_TOP;
    if plot_width < 100.0 || plot_height < series.len() as f64 * MULTI_LEGEND_ROW {
        return Err(Error::input(
            "Figure is too small for its axis labels and series legend",
        ));
    }

    let title = escape_xml(labels.title);
    let x_axis = escape_xml(labels.x_axis);
    let y_axis = escape_xml(labels.y_axis);
    let series_description = series
        .iter()
        .map(|entry| escape_xml(entry.label))
        .collect::<Vec<_>>()
        .join(", ");
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img"><title>{title}</title><desc>{title}; X axis: {x_axis}; Y axis: {y_axis}; Series: {series_description}</desc><rect width="100%" height="100%" fill="white"/><g stroke="black" fill="none"><path d="M {MULTI_LEFT} {MULTI_TOP} V {plot_bottom} H {plot_right}"/></g>"#
    );
    let _ = write!(
        svg,
        r#"<g fill="black" font-family="sans-serif"><text x="{}" y="22" text-anchor="middle" font-size="14">{title}</text>"#,
        (MULTI_LEFT + plot_right) / 2.0
    );

    for entry in series {
        let coordinates = entry.points.iter().map(|&(x, y)| {
            let px = MULTI_LEFT + (x - xmin) / (xmax - xmin) * plot_width;
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
                    write_marker(&mut svg, entry.style.marker, px, py, 3.0, entry.style.color);
                }
            }
            PlotKind::Line => {
                let dash = if entry.style.dashed {
                    " stroke-dasharray=\"7 4\""
                } else {
                    ""
                };
                let _ = write!(
                    svg,
                    r#"<polyline fill="none" stroke="{}" stroke-width="2"{} points=""#,
                    entry.style.color, dash
                );
                for (px, py) in coordinates {
                    let _ = write!(svg, " {px:.3},{py:.3}");
                }
                svg.push_str(r#""/>"#);
                for &(x, y) in entry.points {
                    let px = MULTI_LEFT + (x - xmin) / (xmax - xmin) * plot_width;
                    let y_fraction = if ymin == ymax {
                        0.5
                    } else {
                        (y - ymin) / (ymax - ymin)
                    };
                    let py = plot_bottom - y_fraction * plot_height;
                    write_marker(&mut svg, entry.style.marker, px, py, 2.6, entry.style.color);
                }
            }
        }
    }

    let x_tick_y = h - 44.0;
    let _ = write!(
        svg,
        r#"<text x="{MULTI_LEFT}" y="{x_tick_y}">{xmin:.4}</text><text x="{plot_right}" y="{x_tick_y}" text-anchor="end">{xmax:.4}</text>"#
    );
    if ymin == ymax {
        let _ = write!(
            svg,
            r#"<text x="{}" y="{}" text-anchor="end">{ymin:.4}</text>"#,
            MULTI_LEFT - 8.0,
            (MULTI_TOP + plot_bottom) / 2.0
        );
    } else {
        let _ = write!(
            svg,
            r#"<text x="{}" y="{}" text-anchor="end">{ymax:.4}</text><text x="{}" y="{}" text-anchor="end">{ymin:.4}</text>"#,
            MULTI_LEFT - 8.0,
            MULTI_TOP + 4.0,
            MULTI_LEFT - 8.0,
            plot_bottom
        );
    }
    let _ = write!(
        svg,
        r#"<text x="{}" y="{}" text-anchor="middle" font-size="12">{x_axis}</text><text x="16" y="{}" transform="rotate(-90 16 {})" text-anchor="middle" font-size="12">{y_axis}</text></g>"#,
        (MULTI_LEFT + plot_right) / 2.0,
        h - 10.0,
        (MULTI_TOP + plot_bottom) / 2.0,
        (MULTI_TOP + plot_bottom) / 2.0
    );

    let legend_x = plot_right + 14.0;
    svg.push_str(r#"<g role="group" aria-label="Legend" fill="black" font-family="sans-serif" font-size="11">"#);
    for (index, entry) in series.iter().enumerate() {
        let y = MULTI_TOP + 13.0 + index as f64 * MULTI_LEGEND_ROW;
        match kind {
            PlotKind::Scatter => {
                write_marker(
                    &mut svg,
                    entry.style.marker,
                    legend_x + 8.0,
                    y - 4.0,
                    3.0,
                    entry.style.color,
                );
            }
            PlotKind::Line => {
                let dash = if entry.style.dashed {
                    " stroke-dasharray=\"7 4\""
                } else {
                    ""
                };
                let _ = write!(
                    svg,
                    r#"<path d="M {legend_x} {y_minus_4} H {line_end}" fill="none" stroke="{}" stroke-width="2"{}/>"#,
                    entry.style.color,
                    dash,
                    y_minus_4 = y - 4.0,
                    line_end = legend_x + 16.0,
                );
                write_marker(
                    &mut svg,
                    entry.style.marker,
                    legend_x + 8.0,
                    y - 4.0,
                    2.6,
                    entry.style.color,
                );
            }
        }
        let label = escape_xml(entry.label);
        let _ = write!(
            svg,
            r#"<text x="{}" y="{y}">{label}</text>"#,
            legend_x + 24.0
        );
    }
    svg.push_str("</g></svg>");
    Ok(svg)
}

fn write_marker(svg: &mut String, marker: Marker, x: f64, y: f64, size: f64, color: &str) {
    match marker {
        Marker::Circle => {
            let _ = write!(
                svg,
                r#"<circle cx="{x:.3}" cy="{y:.3}" r="{size:.3}" fill="{color}"/>"#
            );
        }
        Marker::Square => {
            let _ = write!(
                svg,
                r#"<rect x="{:.3}" y="{:.3}" width="{:.3}" height="{:.3}" fill="{color}"/>"#,
                x - size,
                y - size,
                size * 2.0,
                size * 2.0
            );
        }
        Marker::Triangle => {
            let _ = write!(
                svg,
                r#"<path d="M {x:.3} {:.3} L {:.3} {:.3} L {:.3} {:.3} Z" fill="{color}"/>"#,
                y - size - 1.0,
                x + size + 1.0,
                y + size,
                x - size - 1.0,
                y + size
            );
        }
        Marker::Diamond => {
            let _ = write!(
                svg,
                r#"<path d="M {x:.3} {:.3} L {:.3} {y:.3} L {x:.3} {:.3} L {:.3} {y:.3} Z" fill="{color}"/>"#,
                y - size - 1.0,
                x + size + 1.0,
                y + size + 1.0,
                x - size - 1.0
            );
        }
        Marker::Cross => {
            let _ = write!(
                svg,
                r#"<path d="M {:.3} {y:.3} H {:.3} M {x:.3} {:.3} V {:.3}" stroke="{color}" stroke-width="2"/>"#,
                x - size,
                x + size,
                y - size,
                y + size
            );
        }
    }
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
            r#"<text x="{}" y="22" text-anchor="middle" fill="black" font-size="14" font-family="sans-serif">{title}</text>"#,
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
            r#"<text x="{}" y="{}" text-anchor="middle" fill="black" font-size="12" font-family="sans-serif">{x_axis}</text><text x="16" y="{}" transform="rotate(-90 16 {})" text-anchor="middle" fill="black" font-size="12" font-family="sans-serif">{y_axis}</text>"#,
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
    use crate::palettes::palette;

    #[test]
    fn monochrome_styles_are_distinct_and_bounded_for_plot_kind() {
        let lines = (0..10)
            .map(SeriesStyle::monochrome_line)
            .collect::<Result<Vec<_>>>()
            .unwrap();
        assert!(lines.iter().all(|style| style.color == "#000000"));
        for left in 0..lines.len() {
            for right in left + 1..lines.len() {
                assert_ne!(lines[left], lines[right]);
            }
        }
        assert!(SeriesStyle::monochrome_line(10).is_err());

        let scatter = (0..5)
            .map(SeriesStyle::monochrome_scatter)
            .collect::<Result<Vec<_>>>()
            .unwrap();
        assert!(scatter.iter().all(|style| style.color == "#000000"));
        assert!(scatter.iter().all(|style| !style.dashed));
        assert!(SeriesStyle::monochrome_scatter(5).is_err());

        let labels = [
            "Series 1",
            "Series 2",
            "Series 3",
            "Series 4",
            "Series 5",
            "Series 6",
            "Series 7",
            "Series 8",
            "Series 9",
            "Series 10",
        ];
        let points = [(0.0, 0.0), (1.0, 1.0)];
        let series = labels
            .iter()
            .enumerate()
            .map(|(index, label)| {
                Ok(FigureSeries::new(
                    label,
                    &points,
                    SeriesStyle::monochrome_line(index)?,
                ))
            })
            .collect::<Result<Vec<_>>>()
            .unwrap();
        let chart = line_series_svg_labeled(
            &series,
            1000,
            900,
            FigureLabels {
                title: "Print test",
                x_axis: "X",
                y_axis: "Y",
            },
        )
        .unwrap();
        assert_eq!(chart.matches("<polyline ").count(), 10);
        assert_eq!(chart.matches("stroke-dasharray=\"7 4\"").count(), 10);
        assert!(chart.contains("stroke=\"#000000\""));
        assert!(chart.contains("<rect "));
        assert!(chart.contains("aria-label=\"Legend\""));
    }

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
        assert!(!line.contains("textLength="));
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

    static FIRST_POINTS: [(f64, f64); 3] = [(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)];
    static SECOND_POINTS: [(f64, f64); 3] = [(0.0, 2.0), (1.0, 1.0), (2.0, 4.0)];

    fn fixture_series() -> [FigureSeries<'static>; 2] {
        let categorical = palette("batlowS").unwrap();
        [
            FigureSeries::new(
                "Control",
                &FIRST_POINTS,
                categorical.series_style(0, Marker::Circle, false).unwrap(),
            ),
            FigureSeries::new(
                "Treatment <A>",
                &SECOND_POINTS,
                categorical.series_style(1, Marker::Triangle, true).unwrap(),
            ),
        ]
    }

    fn fixture_labels() -> FigureLabels<'static> {
        FigureLabels {
            title: "Response comparison",
            x_axis: "Dose (mg)",
            y_axis: "Response <units>",
        }
    }

    #[test]
    fn multi_series_line_and_scatter_share_axes_and_render_accessible_legends() {
        let series = fixture_series();
        let line = line_series_svg_labeled(&series, 800, 600, fixture_labels()).unwrap();
        assert!(line.contains("role=\"group\" aria-label=\"Legend\""));
        assert!(line.contains("Control</text>"));
        assert!(line.contains("Treatment &lt;A&gt;</text>"));
        assert!(line.contains(
            "<desc>Response comparison; X axis: Dose (mg); Y axis: Response &lt;units&gt;; Series: Control, Treatment &lt;A&gt;</desc>"
        ));
        assert!(line.contains("stroke-dasharray=\"7 4\""));
        assert!(line.contains("<circle"));
        assert!(line.matches("<polyline").count() == 2);
        assert!(line.contains("Response &lt;units&gt;</text>"));

        let scatter = scatter_series_svg_labeled(&series, 800, 600, fixture_labels()).unwrap();
        assert!(scatter.contains("<circle"));
        assert!(scatter.contains("Z\" fill=\"#"));
        assert!(scatter.contains("Treatment &lt;A&gt;</text>"));
        assert!(!scatter.contains("<polyline"));
    }

    #[test]
    fn multi_series_export_preserves_existing_files_and_rejects_unfittable_layouts() {
        let series = fixture_series();
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("series.svg");
        export_line_series_svg_labeled(&output, &series, 800, 600, fixture_labels()).unwrap();
        let original = std::fs::read_to_string(&output).unwrap();
        assert_eq!(
            original,
            include_str!("../test-fixtures/multiseries-line.svg")
        );
        assert!(
            export_scatter_series_svg_labeled(&output, &series, 800, 600, fixture_labels())
                .is_err()
        );
        assert_eq!(std::fs::read_to_string(&output).unwrap(), original);
        assert!(line_series_svg_labeled(&[], 800, 600, fixture_labels()).is_err());
        assert!(line_series_svg_labeled(&series, 320, 600, fixture_labels()).is_err());
        let crowded_legend = [series[0]; 6];
        assert!(line_series_svg_labeled(&crowded_legend, 800, 200, fixture_labels()).is_err());
        let bad_label = [FigureSeries::new(
            "invalid\nlegend",
            series[0].points,
            series[0].style,
        )];
        assert!(line_series_svg_labeled(&bad_label, 800, 600, fixture_labels()).is_err());
        let duplicate_label = [series[0], series[0]];
        assert!(line_series_svg_labeled(&duplicate_label, 800, 600, fixture_labels()).is_err());
    }

    #[cfg(feature = "figure-pdf")]
    #[test]
    fn pdf_export_is_vector_sized_and_does_not_overwrite() {
        let series = fixture_series();
        let dimensions = FigureDimensions::new(210.0, 148.0);
        let pdf = line_series_pdf(&series, dimensions, fixture_labels()).unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
        let content = String::from_utf8_lossy(&pdf);
        assert!(content.contains("/MediaBox"));
        assert!(!content.contains("/Subtype /Image"));

        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("figure.pdf");
        export_line_series_pdf(&output, &series, dimensions, fixture_labels()).unwrap();
        assert_eq!(std::fs::read(&output).unwrap(), pdf);
        assert!(export_scatter_series_pdf(&output, &series, dimensions, fixture_labels()).is_err());
        assert_eq!(std::fs::read(&output).unwrap(), pdf);
    }

    #[cfg(feature = "figure-pdf")]
    #[test]
    fn pdf_export_rejects_nonfinite_and_unbounded_physical_dimensions() {
        for dimensions in [
            FigureDimensions::new(f64::NAN, 100.0),
            FigureDimensions::new(100.0, f64::INFINITY),
            FigureDimensions::new(1.0, 100.0),
            FigureDimensions::new(100.0, 2000.0),
        ] {
            assert!(dimensions.css_pixels().is_err());
        }
    }
}
