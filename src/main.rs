#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Arc;
use eframe::egui;
use rustyimage::app::RustImageApp;
use rustyimage::registry::{register_context_menu, unregister_context_menu};

fn create_app_icon() -> Arc<egui::IconData> {
    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - 15.5;
            let dy = y as f32 - 15.5;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= 14.5 {
                if dist >= 13.0 {
                    rgba.extend_from_slice(&[30, 30, 35, 255]);
                } else {
                    let r = (245.0 - (y as f32 * 2.0)).clamp(180.0, 255.0) as u8;
                    let g = (100.0 + (x as f32 * 2.5)).clamp(80.0, 180.0) as u8;
                    let b = 35u8;
                    rgba.extend_from_slice(&[r, g, b, 255]);
                }
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    Arc::new(egui::IconData {
        rgba,
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut initial_files: Vec<PathBuf> = Vec::new();

    if args.len() > 1 {
        let arg = &args[1];
        if arg == "--register" || arg == "--register-context-menu" {
            match register_context_menu() {
                Ok(()) => {
                    println!("Successfully registered RustyImage Windows Explorer context menu.")
                }
                Err(e) => eprintln!("Failed to register context menu: {e}"),
            }
            return Ok(());
        } else if arg == "--unregister" || arg == "--unregister-context-menu" {
            match unregister_context_menu() {
                Ok(()) => {
                    println!("Successfully unregistered RustyImage Windows Explorer context menu.")
                }
                Err(e) => eprintln!("Failed to unregister context menu: {e}"),
            }
            return Ok(());
        }

        for arg in &args[1..] {
            let p = PathBuf::from(arg);
            if p.exists() {
                initial_files.push(p);
            }
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([534.0, 300.0])
            .with_min_inner_size([500.0, 280.0])
            .with_title("RustyImage")
            .with_resizable(true)
            .with_decorations(true)
            .with_transparent(false)
            .with_icon(create_app_icon())
            .with_app_id("rustyimage")
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "RustyImage",
        options,
        Box::new(|cc| {
            rustyimage::font::setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(RustImageApp::new(cc, initial_files)))
        }),
    )
}
