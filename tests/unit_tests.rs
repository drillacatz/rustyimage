use image::{Rgba, RgbaImage};
use rustyimage::codec::static_codec::{convert_image, TargetFormat};
use rustyimage::codec::{decode_apng, decode_gif, encode_apng, encode_gif};
use rustyimage::core::Frame;
use std::io::Cursor;

#[test]
fn test_gif_roundtrip() {
    let mut img1 = RgbaImage::new(16, 16);
    img1.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
    let mut img2 = RgbaImage::new(16, 16);
    img2.put_pixel(1, 1, Rgba([0, 255, 0, 255]));

    let frames = vec![Frame::new(img1, 100), Frame::new(img2, 120)];

    let mut buf = Vec::new();
    encode_gif(&mut buf, &frames, 0).expect("GIF encoding failed");
    assert!(!buf.is_empty());

    let decoded = decode_gif(Cursor::new(buf)).expect("GIF decoding failed");
    assert_eq!(decoded.len(), 2);
    assert_eq!(decoded[0].dimensions(), (16, 16));
}

#[test]
fn test_apng_roundtrip() {
    let mut img1 = RgbaImage::new(16, 16);
    img1.put_pixel(0, 0, Rgba([255, 0, 0, 128]));
    let mut img2 = RgbaImage::new(16, 16);
    img2.put_pixel(1, 1, Rgba([0, 255, 0, 255]));

    let frames = vec![Frame::new(img1, 100), Frame::new(img2, 150)];

    let mut buf = Vec::new();
    encode_apng(&mut buf, &frames, 0).expect("APNG encoding failed");
    assert!(!buf.is_empty());

    let decoded = decode_apng(Cursor::new(buf)).expect("APNG decoding failed");
    assert_eq!(decoded.len(), 2);
    assert_eq!(decoded[0].dimensions(), (16, 16));
    let p0 = decoded[0].image.get_pixel(0, 0);
    assert_eq!(p0[0], 255);
    assert_eq!(p0[3], 128);
}

#[test]
fn test_target_format_properties() {
    assert_eq!(TargetFormat::ALL.len(), 10);
    for fmt in TargetFormat::ALL {
        assert!(!fmt.extension().is_empty());
        assert!(!fmt.display_name().is_empty());
        assert_eq!(TargetFormat::from_extension(fmt.extension()), Some(fmt));
    }
    assert_eq!(TargetFormat::from_extension("jpeg"), Some(TargetFormat::Jpg));
    assert_eq!(TargetFormat::from_extension("tif"), Some(TargetFormat::Tiff));
    assert_eq!(TargetFormat::from_extension("svg"), Some(TargetFormat::Svg));
    assert_eq!(TargetFormat::from_extension("unknown"), None);
}

#[test]
fn test_convert_image_file_outputs() {
    let temp_dir = std::env::temp_dir().join("rustymedia_tests");
    let _ = std::fs::create_dir_all(&temp_dir);

    let mut img = RgbaImage::new(32, 32);
    for x in 0..32 {
        for y in 0..32 {
            img.put_pixel(x, y, Rgba([x as u8 * 8, y as u8 * 8, 120, 255]));
        }
    }
    let frames = vec![Frame::new(img, 100)];

    for fmt in [
        TargetFormat::Png,
        TargetFormat::Jpg,
        TargetFormat::Webp,
        TargetFormat::Bmp,
        TargetFormat::Gif,
        TargetFormat::Apng,
        TargetFormat::Ico,
        TargetFormat::Tiff,
        TargetFormat::Tga,
        TargetFormat::Svg,
    ] {
        let out_file = temp_dir.join(format!("test_out.{}", fmt.extension()));
        let res = convert_image(&frames, &out_file, fmt, 85, None, 0);
        assert!(res.is_ok(), "Failed to convert to {:?}: {:?}", fmt, res);
        assert!(out_file.exists());
        let _ = std::fs::remove_file(out_file);
    }
}

#[test]
fn test_svg_vector_export_and_import_roundtrip() {
    use rustyimage::codec::load_any_image;

    let temp_dir = std::env::temp_dir().join("rustymedia_svg_tests");
    let _ = std::fs::create_dir_all(&temp_dir);

    // Create a 64x64 simple high-contrast icon (red box in white background)
    let mut img = RgbaImage::new(64, 64);
    for x in 0..64 {
        for y in 0..64 {
            if (16..48).contains(&x) && (16..48).contains(&y) {
                img.put_pixel(x, y, Rgba([220, 40, 40, 255]));
            } else {
                img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
    }
    let frames = vec![Frame::new(img, 100)];
    let out_file = temp_dir.join("test_vector.svg");

    // 1. Export raster to vector SVG (vtracer Bézier path tracing)
    let res = convert_image(&frames, &out_file, TargetFormat::Svg, 90, None, 0);
    assert!(res.is_ok(), "SVG export failed: {:?}", res);
    assert!(out_file.exists());

    // Verify SVG file content has valid SVG markup and paths
    let svg_content = std::fs::read_to_string(&out_file).expect("Failed to read generated SVG");
    assert!(svg_content.contains("<svg"));
    assert!(svg_content.contains("<path"));

    // 2. Import vector SVG (resvg rasterization)
    let loaded_frames = load_any_image(&out_file).expect("Failed to load/rasterize generated SVG");
    assert!(!loaded_frames.is_empty());
    assert_eq!(loaded_frames[0].dimensions(), (64, 64));

    let _ = std::fs::remove_file(out_file);
}

#[test]
fn test_ico_large_image_auto_downscaling() {
    let temp_dir = std::env::temp_dir().join("rustymedia_ico_tests");
    let _ = std::fs::create_dir_all(&temp_dir);

    // Create a large 1024x512 image (exceeding Windows ICO 256x256 limit)
    let mut large_img = RgbaImage::new(1024, 512);
    for x in 0..1024 {
        for y in 0..512 {
            large_img.put_pixel(x, y, Rgba([(x % 256) as u8, (y % 256) as u8, 128, 255]));
        }
    }
    let frames = vec![Frame::new(large_img, 100)];
    let out_file = temp_dir.join("test_large.ico");

    let res = convert_image(&frames, &out_file, TargetFormat::Ico, 90, None, 0);
    assert!(res.is_ok(), "Failed to auto-downscale and export large ICO: {:?}", res);
    assert!(out_file.exists());

    // Verify the saved ICO can be decoded and dimensions are within 256x256
    let decoded = image::open(&out_file).expect("Failed to decode saved ICO");
    assert!(decoded.width() <= 256, "ICO width {} exceeds 256", decoded.width());
    assert!(decoded.height() <= 256, "ICO height {} exceeds 256", decoded.height());
    assert_eq!(decoded.width(), 256);
    assert_eq!(decoded.height(), 128); // Proportionally scaled from 1024x512

    let _ = std::fs::remove_file(out_file);
}

#[test]
fn test_batch_converter_multi_item() {
    use rustyimage::converter::{AsyncConverter, ConvertEvent, ConvertItem, ConvertJob};
    use std::time::Duration;

    let temp_dir = std::env::temp_dir().join("rustymedia_batch_tests");
    let _ = std::fs::create_dir_all(&temp_dir);

    let mut img1 = RgbaImage::new(16, 16);
    img1.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
    let item1 = ConvertItem {
        frames: vec![Frame::new(img1, 100)],
        out_path: temp_dir.join("batch_out1.png"),
    };

    let mut img2 = RgbaImage::new(16, 16);
    img2.put_pixel(0, 0, Rgba([0, 255, 0, 255]));
    let item2 = ConvertItem {
        frames: vec![Frame::new(img2, 100)],
        out_path: temp_dir.join("batch_out2.png"),
    };

    let job = ConvertJob {
        items: vec![item1, item2],
        format: TargetFormat::Png,
        quality: 90,
        custom_delay_ms: None,
        repeat_count: 0,
    };

    let mut converter = AsyncConverter::new();
    converter.start(job);

    let mut finished = false;
    let mut item_count = 0;
    for _ in 0..100 {
        std::thread::sleep(Duration::from_millis(20));
        while let Some(event) = converter.poll() {
            match event {
                ConvertEvent::ItemSuccess { .. } => {
                    item_count += 1;
                }
                ConvertEvent::AllFinished { successful, errors, .. } => {
                    assert_eq!(successful.len(), 2);
                    assert!(errors.is_empty());
                    finished = true;
                }
                _ => {}
            }
        }
        if finished {
            break;
        }
    }

    assert!(finished, "Batch converter should have finished");
    assert_eq!(item_count, 2, "Expected 2 item success events");
    assert!(temp_dir.join("batch_out1.png").exists());
    assert!(temp_dir.join("batch_out2.png").exists());

    let _ = std::fs::remove_file(temp_dir.join("batch_out1.png"));
    let _ = std::fs::remove_file(temp_dir.join("batch_out2.png"));
}

