use std::io::{Read, Seek, Write};
use image::{Rgba, RgbaImage};
use png::{BlendOp, ColorType, BitDepth, DisposeOp};
use crate::core::frame::Frame;

pub fn decode_apng<R: Read + Seek>(reader: R) -> Result<Vec<Frame>, String> {
    let decoder = png::Decoder::new(reader);
    let mut reader = decoder.read_info().map_err(|e| format!("Failed to read PNG header: {e}"))?;

    let info = reader.info().clone();
    let canvas_w = info.width;
    let canvas_h = info.height;

    // Allocate buffer for reading each raw sub-frame
    let mut buf = vec![0u8; reader.output_buffer_size()];

    let mut frames = Vec::new();
    let mut canvas = RgbaImage::new(canvas_w, canvas_h);
    let mut prev_canvas = canvas.clone();

    // Check if image is animated
    let is_animated = info.animation_control.is_some();

    loop {
        let frame_info = match reader.next_frame(&mut buf) {
            Ok(info) => info,
            Err(png::DecodingError::IoError(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => {
                // If we already decoded at least one frame, we can stop; otherwise it's an error
                if !frames.is_empty() {
                    break;
                }
                return Err(format!("PNG decoding error: {e}"));
            }
        };

        // Delay in ms (default to 100ms if not specified)
        let delay_ms = if let Some(fc) = reader.info().frame_control {
            let num = fc.delay_num as u32;
            let den = if fc.delay_den == 0 { 100 } else { fc.delay_den as u32 };
            (num * 1000 / den).max(10)
        } else {
            100
        };

        let sub_w = frame_info.width;
        let sub_h = frame_info.height;
        let x_offset = if let Some(fc) = reader.info().frame_control { fc.x_offset } else { 0 };
        let y_offset = if let Some(fc) = reader.info().frame_control { fc.y_offset } else { 0 };
        let dispose_op = if let Some(fc) = reader.info().frame_control { fc.dispose_op } else { DisposeOp::None };
        let blend_op = if let Some(fc) = reader.info().frame_control { fc.blend_op } else { BlendOp::Source };

        // Save previous canvas if DisposeOp::Previous
        if dispose_op == DisposeOp::Previous {
            prev_canvas = canvas.clone();
        }

        // Convert the decoded buffer slice into RGBA sub-image
        let sub_bytes = &buf[..frame_info.buffer_size()];
        let sub_img = raw_to_rgba(sub_bytes, sub_w, sub_h, frame_info.color_type, frame_info.bit_depth)?;

        // Blend sub-frame onto main canvas
        for sy in 0..sub_h {
            for sx in 0..sub_w {
                let dx = x_offset + sx;
                let dy = y_offset + sy;
                if dx < canvas_w && dy < canvas_h {
                    let src_pixel = *sub_img.get_pixel(sx, sy);
                    let dst_pixel = canvas.get_pixel_mut(dx, dy);

                    match blend_op {
                        BlendOp::Source => {
                            *dst_pixel = src_pixel;
                        }
                        BlendOp::Over => {
                            let src_a = src_pixel[3] as f32 / 255.0;
                            let dst_a = dst_pixel[3] as f32 / 255.0;
                            let out_a = src_a + dst_a * (1.0 - src_a);

                            if out_a > 0.0 {
                                let blend_channel = |sc: u8, dc: u8| -> u8 {
                                    let s = sc as f32 / 255.0;
                                    let d = dc as f32 / 255.0;
                                    let out = (s * src_a + d * dst_a * (1.0 - src_a)) / out_a;
                                    (out * 255.0).round().clamp(0.0, 255.0) as u8
                                };
                                *dst_pixel = Rgba([
                                    blend_channel(src_pixel[0], dst_pixel[0]),
                                    blend_channel(src_pixel[1], dst_pixel[1]),
                                    blend_channel(src_pixel[2], dst_pixel[2]),
                                    (out_a * 255.0).round().clamp(0.0, 255.0) as u8,
                                ]);
                            }
                        }
                    }
                }
            }
        }

        // Snapshot current canvas state for this frame
        frames.push(Frame::new(canvas.clone(), delay_ms));

        // Apply dispose operation for next frame
        match dispose_op {
            DisposeOp::None => {}
            DisposeOp::Background => {
                for sy in 0..sub_h {
                    for sx in 0..sub_w {
                        let dx = x_offset + sx;
                        let dy = y_offset + sy;
                        if dx < canvas_w && dy < canvas_h {
                            canvas.put_pixel(dx, dy, Rgba([0, 0, 0, 0]));
                        }
                    }
                }
            }
            DisposeOp::Previous => {
                canvas = prev_canvas.clone();
            }
        }

        if !is_animated {
            break;
        }
    }

    if frames.is_empty() {
        return Err("No frames could be decoded from PNG".to_string());
    }

    Ok(frames)
}

fn raw_to_rgba(
    bytes: &[u8],
    w: u32,
    h: u32,
    color_type: ColorType,
    bit_depth: BitDepth,
) -> Result<RgbaImage, String> {
    if bit_depth != BitDepth::Eight {
        // Fallback for non-8-bit depth
        return Err(format!("Unsupported PNG bit depth: {bit_depth:?}"));
    }

    let mut img = RgbaImage::new(w, h);
    match color_type {
        ColorType::Rgba => {
            if bytes.len() < (w * h * 4) as usize {
                return Err("Insufficient RGBA buffer length".to_string());
            }
            img.copy_from_slice(bytes);
        }
        ColorType::Rgb => {
            if bytes.len() < (w * h * 3) as usize {
                return Err("Insufficient RGB buffer length".to_string());
            }
            let mut src_idx = 0;
            for y in 0..h {
                for x in 0..w {
                    let r = bytes[src_idx];
                    let g = bytes[src_idx + 1];
                    let b = bytes[src_idx + 2];
                    src_idx += 3;
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
        }
        ColorType::GrayscaleAlpha => {
            if bytes.len() < (w * h * 2) as usize {
                return Err("Insufficient GrayscaleAlpha buffer length".to_string());
            }
            let mut src_idx = 0;
            for y in 0..h {
                for x in 0..w {
                    let g = bytes[src_idx];
                    let a = bytes[src_idx + 1];
                    src_idx += 2;
                    img.put_pixel(x, y, Rgba([g, g, g, a]));
                }
            }
        }
        ColorType::Grayscale => {
            if bytes.len() < (w * h) as usize {
                return Err("Insufficient Grayscale buffer length".to_string());
            }
            let mut src_idx = 0;
            for y in 0..h {
                for x in 0..w {
                    let g = bytes[src_idx];
                    src_idx += 1;
                    img.put_pixel(x, y, Rgba([g, g, g, 255]));
                }
            }
        }
        _ => {
            return Err(format!("Unsupported PNG color type: {color_type:?}"));
        }
    }

    Ok(img)
}

pub fn encode_apng<W: Write>(
    writer: W,
    frames: &[Frame],
    repeat_count: u32, // 0 = infinite
) -> Result<(), String> {
    if frames.is_empty() {
        return Err("No frames to encode into APNG".to_string());
    }

    let (width, height) = frames[0].dimensions();

    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);

    // If multiple frames, configure APNG animated metadata
    if frames.len() > 1 {
        encoder
            .set_animated(frames.len() as u32, repeat_count)
            .map_err(|e| format!("Failed to set APNG animated info: {e}"))?;
        let first_delay = frames[0].delay_ms;
        encoder
            .set_frame_delay(first_delay as u16, 1000)
            .map_err(|e| format!("Failed to set APNG initial frame delay: {e}"))?;
    }

    let mut png_writer = encoder
        .write_header()
        .map_err(|e| format!("Failed to write PNG header: {e}"))?;

    for (i, frame) in frames.iter().enumerate() {
        // Resize frame if dimensions don't match first frame
        let rgba_img = if frame.dimensions() != (width, height) {
            image::imageops::resize(&frame.image, width, height, image::imageops::FilterType::Nearest)
        } else {
            frame.image.clone()
        };

        if frames.len() > 1 && i > 0 {
            png_writer
                .set_frame_delay(frame.delay_ms as u16, 1000)
                .map_err(|e| format!("Failed to set APNG frame delay: {e}"))?;
        }

        png_writer
            .write_image_data(rgba_img.as_raw())
            .map_err(|e| format!("Failed to write APNG frame data: {e}"))?;
    }

    png_writer
        .finish()
        .map_err(|e| format!("Failed to finish APNG stream: {e}"))?;

    Ok(())
}
