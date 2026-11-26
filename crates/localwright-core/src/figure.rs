//! Minimal print-oriented vector figure export, separate from UI theme.
use crate::{write_bytes, Error, Result};
use std::path::Path;
/// Render a monochrome scatterplot with explicit axes and no raster content.
/// Points are in data units. The plot uses fixed margins and labels numeric bounds.
pub fn scatter_svg(points: &[(f64, f64)], width: u32, height: u32) -> Result<String> {
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
    if xmin == xmax || ymin == ymax || !(xmax - xmin).is_finite() || !(ymax - ymin).is_finite() {
        return Err(Error::input("Axes need nonconstant finite ranges"));
    }
    let (w, h) = (width as f64, height as f64);
    let mut s = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img"><title>Scatter plot</title><rect width="100%" height="100%" fill="white"/><g stroke="black" fill="none"><path d="M 60 20 V {} H {}"/>"#,
        h - 50.0,
        w - 20.0
    );
    for &(x, y) in points {
        let px = 60.0 + (x - xmin) / (xmax - xmin) * (w - 80.0);
        let py = h - 50.0 - (y - ymin) / (ymax - ymin) * (h - 70.0);
        s.push_str(&format!(r#"<circle cx="{px:.3}" cy="{py:.3}" r="3"/>"#));
    }
    s.push_str(&format!(r#"</g><g fill="black" font-size="12" font-family="sans-serif"><text x="60" y="{}">{xmin:.4}</text><text x="{}" y="{}" text-anchor="end">{xmax:.4}</text><text x="5" y="20">{ymax:.4}</text><text x="5" y="{}">{ymin:.4}</text></g></svg>"#,h-32.0,w-20.0,h-32.0,h-50.0));
    Ok(s)
}
pub fn export_scatter(path: &Path, points: &[(f64, f64)], width: u32, height: u32) -> Result<()> {
    write_bytes(path, scatter_svg(points, width, height)?.as_bytes())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vector() {
        let s = scatter_svg(&[(0., 0.), (1., 2.)], 640, 480).unwrap();
        assert_eq!(s.matches("<circle").count(), 2);
        assert!(!s.contains("<image"));
        assert!(scatter_svg(&[(1., 2.)], 640, 480).is_err());
    }
}
