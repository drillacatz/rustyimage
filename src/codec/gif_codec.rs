use std::io::{BufRead, Seek, Write};
use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::{AnimationDecoder, Delay, Frame as ImageFrame, RgbaImage};
use crate::core::frame::Frame;

pub fn decode_gif<R: BufRead + Seek>(reader: R) -> Result<Vec<Frame>, String> {
    let decoder = GifDecoder::new(reader).map_err(|e| format!("Failed to read GIF: {e}"))?;
    let image_frames = decoder
        .into_frames()
        .collect_frames()
        .map_err(|e| format!("Failed to decode GIF frames: {e}"))?;

    let mut frames = Vec::new();
    for img_frame in image_frames {
        let (numer, denom) = img_frame.delay().numer_denom_ms();
        let delay_ms = numer.checked_div(denom).unwrap_or(100).max(10);
        let buffer: RgbaImage = img_frame.into_buffer();
        frames.push(Frame::new(buffer, delay_ms));
    }

    if frames.is_empty() {
        return Err("GIF has no frames".to_string());
    }

    Ok(frames)
}

pub fn encode_gif<W: Write>(
    writer: &mut W,
    frames: &[Frame],
    repeat_count: u16, // 0 = infinite
) -> Result<(), String> {
    if frames.is_empty() {
        return Err("No frames to encode into GIF".to_string());
    }

    let mut encoder = GifEncoder::new(writer);
    let repeat = if repeat_count == 0 {
        Repeat::Infinite
    } else {
        Repeat::Finite(repeat_count)
    };
    encoder
        .set_repeat(repeat)
        .map_err(|e| format!("Failed to set GIF repeat: {e}"))?;

    for frame in frames {
        let delay = Delay::from_numer_denom_ms(frame.delay_ms, 1);
        let img_frame = ImageFrame::from_parts(frame.image.clone(), 0, 0, delay);
        encoder
            .encode_frame(img_frame)
            .map_err(|e| format!("Failed to encode GIF frame: {e}"))?;
    }

    Ok(())
}
