//! Generate an accessible, multi-series vector PDF figure.
//!
//! Run with: cargo run -p localwright-core --features figure-pdf --example scientific_pdf -- figure.pdf
use localwright_core::{
    figure::{export_line_series_pdf, FigureDimensions, FigureLabels, FigureSeries},
    palettes::palette,
    Error,
};

fn main() -> localwright_core::Result<()> {
    let output = std::env::args_os()
        .nth(1)
        .ok_or_else(|| Error::input("Provide a new output PDF path"))?;
    let colors = palette("batlowS").ok_or_else(|| Error::input("palette is not bundled"))?;
    let control = [(0.0, 1.2), (1.0, 2.0), (2.0, 2.5), (3.0, 3.1)];
    let treatment = [(0.0, 1.0), (1.0, 1.6), (2.0, 2.4), (3.0, 3.8)];
    let series = [
        FigureSeries::new(
            "Control",
            &control,
            colors.series_style(0, localwright_core::figure::Marker::Circle, false)?,
        ),
        FigureSeries::new(
            "Treatment",
            &treatment,
            colors.series_style(1, localwright_core::figure::Marker::Triangle, true)?,
        ),
    ];
    export_line_series_pdf(
        std::path::Path::new(&output),
        &series,
        FigureDimensions::new(180.0, 120.0),
        FigureLabels {
            title: "Response by dose",
            x_axis: "Dose (mg)",
            y_axis: "Response (a.u.)",
        },
    )
}
