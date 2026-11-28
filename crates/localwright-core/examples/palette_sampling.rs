//! Demonstrate continuous and discrete palette sampling without a GUI or
//! external plotting runtime.
//!
//! Run with: cargo run -p localwright-core --example palette_sampling

use localwright_core::{palettes::palette, Error};

fn main() -> localwright_core::Result<()> {
    let sequential = palette("cividis").ok_or_else(|| Error::input("palette is not bundled"))?;
    for position in [0.0, 0.25, 0.5, 0.75, 1.0] {
        println!(
            "cividis {position:.2}: {:?}",
            sequential.sample_continuous_rgb(position)?
        );
    }

    let categorical = palette("batlowS").ok_or_else(|| Error::input("palette is not bundled"))?;
    for index in 0..3 {
        let color = categorical
            .color(index)
            .ok_or_else(|| Error::input("palette sample is not available"))?;
        println!("batlowS category {index}: {color}");
    }
    Ok(())
}
