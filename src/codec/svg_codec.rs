use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::core::frame::Frame;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn load_svg(path: &Path) -> Result<Vec<Frame>, String> {
    let svg_data = std::fs::read(path).map_err(|e| format!("Failed to read SVG file: {e}"))?;
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&svg_data, &opt)
        .map_err(|e| format!("Failed to parse SVG: {e}"))?;

    let size = tree.size();
    let width = size.width().ceil() as u32;
    let height = size.height().ceil() as u32;

    if width == 0 || height == 0 {
        return Err("SVG has zero dimensions".to_string());
    }

    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "Failed to allocate pixmap for SVG rendering".to_string())?;

    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let rgba_image = image::RgbaImage::from_raw(width, height, pixmap.take())
        .ok_or_else(|| "Failed to create RGBA image from pixmap".to_string())?;

    Ok(vec![Frame::new(rgba_image, 100)])
}

pub fn export_svg(path: &Path, frame: &Frame) -> Result<(), String> {
    let count = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp_dir = std::env::temp_dir();
    let temp_input = temp_dir.join(format!(
        "rustyimage_vtracer_tmp_{}_{}.png",
        std::process::id(),
        count
    ));

    frame
        .image
        .save_with_format(&temp_input, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to prepare image for vectorization: {e}"))?;

    let config = vtracer::Config::default();
    let result = vtracer::convert_image_to_svg(&temp_input, path, config);
    let _ = std::fs::remove_file(temp_input);

    result.map_err(|e| format!("Vectorization failed: {e}"))
}
