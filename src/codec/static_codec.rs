use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use image::ImageFormat;
use crate::codec::apng_codec::{decode_apng, encode_apng};
use crate::codec::gif_codec::{decode_gif, encode_gif};
use crate::core::frame::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetFormat {
    Png,
    Jpg,
    Webp,
    Gif,
    Apng,
    Bmp,
    Ico,
    Tiff,
    Tga,
    Svg,
}

impl TargetFormat {
    pub const ALL: [TargetFormat; 10] = [
        TargetFormat::Png,
        TargetFormat::Jpg,
        TargetFormat::Webp,
        TargetFormat::Gif,
        TargetFormat::Apng,
        TargetFormat::Bmp,
        TargetFormat::Ico,
        TargetFormat::Tiff,
        TargetFormat::Tga,
        TargetFormat::Svg,
    ];

    pub fn extension(&self) -> &'static str {
        match self {
            TargetFormat::Png => "png",
            TargetFormat::Jpg => "jpg",
            TargetFormat::Webp => "webp",
            TargetFormat::Gif => "gif",
            TargetFormat::Apng => "apng",
            TargetFormat::Bmp => "bmp",
            TargetFormat::Ico => "ico",
            TargetFormat::Tiff => "tiff",
            TargetFormat::Tga => "tga",
            TargetFormat::Svg => "svg",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TargetFormat::Png => "PNG",
            TargetFormat::Jpg => "JPG",
            TargetFormat::Webp => "WEBP",
            TargetFormat::Gif => "GIF",
            TargetFormat::Apng => "APNG",
            TargetFormat::Bmp => "BMP",
            TargetFormat::Ico => "ICO",
            TargetFormat::Tiff => "TIFF",
            TargetFormat::Tga => "TGA",
            TargetFormat::Svg => "SVG",
        }
    }

    pub fn is_animated(&self) -> bool {
        matches!(self, TargetFormat::Gif | TargetFormat::Apng)
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(TargetFormat::Png),
            "jpg" | "jpeg" => Some(TargetFormat::Jpg),
            "webp" => Some(TargetFormat::Webp),
            "gif" => Some(TargetFormat::Gif),
            "apng" => Some(TargetFormat::Apng),
            "bmp" => Some(TargetFormat::Bmp),
            "ico" => Some(TargetFormat::Ico),
            "tiff" | "tif" => Some(TargetFormat::Tiff),
            "tga" => Some(TargetFormat::Tga),
            "svg" => Some(TargetFormat::Svg),
            _ => None,
        }
    }
}

pub fn load_any_image(path: &Path) -> Result<Vec<Frame>, String> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "svg" {
        return crate::codec::svg_codec::load_svg(path);
    }

    if ext == "gif" {
        let file = File::open(path).map_err(|e| format!("Failed to open GIF: {e}"))?;
        let reader = BufReader::new(file);
        return decode_gif(reader);
    }

    if ext == "png" || ext == "apng" {
        let file = File::open(path).map_err(|e| format!("Failed to open image: {e}"))?;
        let reader = BufReader::new(file);
        if let Ok(frames) = decode_apng(reader) {
            if !frames.is_empty() {
                return Ok(frames);
            }
        }
    }

    // Fallback: load using standard image crate (supports WebP, PNG, JPEG, BMP, etc.)
    let dyn_img = image::open(path).map_err(|e| format!("Failed to load image: {e}"))?;
    let rgba = dyn_img.to_rgba8();
    Ok(vec![Frame::new(rgba, 100)])
}

pub fn load_multiple_images(paths: &[PathBuf]) -> Result<Vec<Frame>, String> {
    let mut all_frames = Vec::new();
    for p in paths {
        let frames = load_any_image(p)?;
        all_frames.extend(frames);
    }
    if all_frames.is_empty() {
        return Err("No valid frames could be loaded".to_string());
    }
    Ok(all_frames)
}

pub fn export_single_frame(path: &Path, frame: &Frame, quality: u8) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" => {
            let rgb = image::DynamicImage::ImageRgba8(frame.image.clone()).to_rgb8();
            let file = File::create(path).map_err(|e| format!("Failed to create file: {e}"))?;
            let mut writer = BufWriter::new(file);
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality.clamp(1, 100));
            encoder
                .encode(rgb.as_raw(), rgb.width(), rgb.height(), image::ExtendedColorType::Rgb8)
                .map_err(|e| format!("JPEG encode failed: {e}"))?;
        }
        "webp" => {
            frame
                .image
                .save_with_format(path, ImageFormat::WebP)
                .map_err(|e| format!("Failed to save WebP: {e}"))?;
        }
        "bmp" => {
            frame
                .image
                .save_with_format(path, ImageFormat::Bmp)
                .map_err(|e| format!("Failed to save BMP: {e}"))?;
        }
        "ico" => {
            let (w, h) = frame.dimensions();
            let final_image = if w > 256 || h > 256 {
                let ratio = (w as f32) / (h as f32);
                let (new_w, new_h) = if w >= h {
                    let nw = 256;
                    let nh = ((256.0 / ratio).round() as u32).clamp(1, 256);
                    (nw, nh)
                } else {
                    let nh = 256;
                    let nw = ((256.0 * ratio).round() as u32).clamp(1, 256);
                    (nw, nh)
                };
                image::imageops::resize(
                    &frame.image,
                    new_w,
                    new_h,
                    image::imageops::FilterType::Lanczos3,
                )
            } else {
                frame.image.clone()
            };
            final_image
                .save_with_format(path, ImageFormat::Ico)
                .map_err(|e| format!("Failed to save ICO: {e}"))?;
        }
        "tiff" | "tif" => {
            frame
                .image
                .save_with_format(path, ImageFormat::Tiff)
                .map_err(|e| format!("Failed to save TIFF: {e}"))?;
        }
        "tga" => {
            frame
                .image
                .save_with_format(path, ImageFormat::Tga)
                .map_err(|e| format!("Failed to save TGA: {e}"))?;
        }
        "svg" => {
            crate::codec::svg_codec::export_svg(path, frame)?;
        }
        _ => {
            frame
                .image
                .save_with_format(path, ImageFormat::Png)
                .map_err(|e| format!("Failed to save PNG: {e}"))?;
        }
    }
    Ok(())
}

pub fn export_animated_file(
    path: &Path,
    frames: &[Frame],
    repeat_count: u32,
) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_lowercase();

    let file = File::create(path).map_err(|e| format!("Failed to create file {:?}: {e}", path))?;
    let mut writer = BufWriter::new(file);

    if ext == "gif" {
        encode_gif(&mut writer, frames, repeat_count as u16)
    } else {
        encode_apng(writer, frames, repeat_count)
    }
}

pub fn convert_image(
    frames: &[Frame],
    out_path: &Path,
    format: TargetFormat,
    quality: u8,
    custom_delay_ms: Option<u32>,
    repeat_count: u32,
) -> Result<(), String> {
    if frames.is_empty() {
        return Err("No frames to convert".to_string());
    }

    let prepared_frames: Vec<Frame> = if let Some(delay) = custom_delay_ms {
        frames.iter().map(|f| Frame::new(f.image.clone(), delay)).collect()
    } else {
        frames.to_vec()
    };

    match format {
        TargetFormat::Gif => {
            let file = File::create(out_path).map_err(|e| format!("Failed to create output file: {e}"))?;
            let mut writer = BufWriter::new(file);
            encode_gif(&mut writer, &prepared_frames, repeat_count as u16)
        }
        TargetFormat::Apng => {
            let file = File::create(out_path).map_err(|e| format!("Failed to create output file: {e}"))?;
            let writer = BufWriter::new(file);
            encode_apng(writer, &prepared_frames, repeat_count)
        }
        _ => {
            // Static format: export first frame
            export_single_frame(out_path, &prepared_frames[0], quality)
        }
    }
}

