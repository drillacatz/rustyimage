use std::path::PathBuf;

use eframe::egui::{
    self, Align, CentralPanel, Color32, Context, Layout, RichText, Rounding, Stroke,
    TopBottomPanel, Vec2,
};

use crate::codec::static_codec::{load_any_image, TargetFormat};
use crate::converter::{AsyncConverter, ConvertEvent, ConvertItem, ConvertJob};
use crate::core::frame::Frame;
use crate::i18n::{Language, Loc};
use crate::registry::{is_context_menu_registered, register_context_menu, unregister_context_menu};
use crate::ui::DynamicArrow;

pub const GITHUB_REPO_URL: &str = "https://github.com/drillacatz/rustyimage";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseAction {
    Quit,
    Minimize,
}

#[derive(Clone)]
pub struct ImportedImage {
    pub path: PathBuf,
    pub frames: Vec<Frame>,
    pub texture: Option<egui::TextureHandle>,
    pub file_size: u64,
}

fn toggle_switch(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = Vec2::new(34.0, 18.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let bg_color = if *on {
            Color32::from_rgb(45, 110, 215)
        } else {
            Color32::from_rgb(46, 52, 62)
        };
        let border_color = if *on {
            Color32::from_rgb(60, 130, 230)
        } else {
            Color32::from_rgb(65, 72, 85)
        };
        ui.painter().rect(
            rect,
            Rounding::same(rect.height() / 2.0),
            bg_color,
            Stroke::new(1.0_f32, border_color),
        );

        let circle_x = egui::lerp(
            (rect.left() + rect.height() / 2.0)..=(rect.right() - rect.height() / 2.0),
            how_on,
        );
        let center = egui::pos2(circle_x, rect.center().y);
        let radius = 0.72 * rect.height() / 2.0;
        let knob_color = if *on {
            Color32::WHITE
        } else {
            Color32::from_rgb(140, 150, 165)
        };
        ui.painter().circle(center, radius, knob_color, Stroke::NONE);
    }
    response
}

pub struct RustImageApp {
    pub imported_images: Vec<ImportedImage>,
    pub target_format: TargetFormat,
    pub compression_enabled: bool,
    pub compression_quality: u8,
    pub merge_into_animation: bool,
    pub custom_delay_ms: Option<u32>,
    pub repeat_count: u32,
    pub destination_text: String,
    pub manual_destination: bool,
    pub converter: AsyncConverter,
    pub context_menu_active: bool,
    pub close_action: CloseAction,
    pub status_message: Option<(String, bool)>, // (text, is_error)
    pub output_files: Vec<PathBuf>,
    pub language: Language,
}

impl RustImageApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, initial_files: Vec<PathBuf>) -> Self {
        let mut app = Self {
            imported_images: Vec::new(),
            target_format: TargetFormat::Png,
            compression_enabled: false,
            compression_quality: 80,
            merge_into_animation: false,
            custom_delay_ms: None,
            repeat_count: 0,
            destination_text: String::new(),
            manual_destination: false,
            converter: AsyncConverter::new(),
            context_menu_active: is_context_menu_registered(),
            close_action: CloseAction::Quit,
            status_message: None,
            output_files: Vec::new(),
            language: Language::English,
        };

        if !initial_files.is_empty() {
            app.load_files_from_paths(&initial_files, &_cc.egui_ctx);
        }

        app
    }

    pub fn is_batch_mode(&self) -> bool {
        self.imported_images.len() > 1
            && !(self.merge_into_animation && self.target_format.is_animated())
    }

    pub fn compute_default_destination(&self) -> String {
        if self.imported_images.is_empty() {
            return String::new();
        }

        if self.is_batch_mode() {
            if let Some(parent) = self.imported_images[0].path.parent() {
                let out_dir = parent.join("converted");
                return out_dir.to_string_lossy().to_string();
            }
        } else if let Some(parent) = self.imported_images[0].path.parent() {
            let stem = if self.merge_into_animation && self.target_format.is_animated() {
                "animation_merged".to_string()
            } else {
                let s = self.imported_images[0]
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                format!("{s}_converted")
            };
            let ext = self.target_format.extension();
            let out = parent.join(format!("{stem}.{ext}"));
            return out.to_string_lossy().to_string();
        }

        String::new()
    }

    pub fn sync_destination_if_needed(&mut self) {
        if !self.manual_destination || self.destination_text.trim().is_empty() {
            self.destination_text = self.compute_default_destination();
        } else if !self.is_batch_mode() {
            let path = PathBuf::from(&self.destination_text);
            let ext = self.target_format.extension();
            let new_path = path.with_extension(ext);
            self.destination_text = new_path.to_string_lossy().to_string();
        }
    }

    pub fn load_files_from_paths(&mut self, paths: &[PathBuf], ctx: &Context) {
        let is_first_batch = self.imported_images.is_empty();

        for path in paths {
            if !path.exists() {
                continue;
            }
            if self.imported_images.iter().any(|item| item.path == *path) {
                continue;
            }
            match load_any_image(path) {
                Ok(frames) => {
                    if frames.is_empty() {
                        continue;
                    }
                    let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                    let first_ci = frames[0].to_color_image();
                    let tex = ctx.load_texture(
                        format!("thumb_{}", path.display()),
                        first_ci,
                        egui::TextureOptions::LINEAR,
                    );
                    self.imported_images.push(ImportedImage {
                        path: path.clone(),
                        frames,
                        texture: Some(tex),
                        file_size,
                    });
                }
                Err(e) => {
                    self.status_message =
                        Some((format!("Error loading {}: {e}", path.display()), true));
                }
            }
        }

        if is_first_batch && !self.imported_images.is_empty() {
            if self.imported_images[0].frames.len() > 1 {
                self.custom_delay_ms = Some(self.imported_images[0].frames[0].delay_ms);
            } else {
                self.custom_delay_ms = None;
            }
        }

        self.manual_destination = false;
        self.destination_text = self.compute_default_destination();
        self.status_message = None;
    }

    pub fn clear_input(&mut self) {
        self.imported_images.clear();
        self.destination_text.clear();
        self.manual_destination = false;
        self.status_message = None;
    }

    pub fn trigger_conversion(&mut self) {
        if self.converter.is_converting || self.imported_images.is_empty() {
            return;
        }

        let quality = if self.compression_enabled {
            self.compression_quality
        } else {
            98
        };

        if self.is_batch_mode() {
            let out_dir = if !self.destination_text.trim().is_empty() {
                PathBuf::from(self.destination_text.trim())
            } else {
                PathBuf::from(self.compute_default_destination())
            };

            if let Err(e) = std::fs::create_dir_all(&out_dir) {
                self.status_message =
                    Some((format!("Failed to create destination directory: {e}"), true));
                return;
            }

            let mut items = Vec::new();
            for img in &self.imported_images {
                let stem = img
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("converted");
                let ext = self.target_format.extension();
                let mut target_file = out_dir.join(format!("{stem}.{ext}"));
                if target_file == img.path {
                    target_file = out_dir.join(format!("{stem}_converted.{ext}"));
                }
                items.push(ConvertItem {
                    frames: img.frames.clone(),
                    out_path: target_file,
                });
            }

            let job = ConvertJob {
                items,
                format: self.target_format,
                quality,
                custom_delay_ms: self.custom_delay_ms,
                repeat_count: self.repeat_count,
            };

            self.status_message = None;
            self.converter.start(job);
        } else {
            let out_path = if !self.destination_text.trim().is_empty() {
                PathBuf::from(self.destination_text.trim())
            } else {
                PathBuf::from(self.compute_default_destination())
            };

            if out_path.as_os_str().is_empty() {
                self.status_message =
                    Some(("Please specify a valid destination path".to_string(), true));
                return;
            }

            if let Some(parent) = out_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            let frames = if self.merge_into_animation && self.target_format.is_animated() {
                let mut combined = Vec::new();
                for img in &self.imported_images {
                    combined.extend(img.frames.clone());
                }
                combined
            } else {
                self.imported_images[0].frames.clone()
            };

            let job = ConvertJob::single(
                frames,
                out_path,
                self.target_format,
                quality,
                self.custom_delay_ms,
                self.repeat_count,
            );

            self.status_message = None;
            self.converter.start(job);
        }
    }

    fn handle_drag_and_drop(&mut self, ctx: &Context) {
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped_files.is_empty() {
            let paths: Vec<PathBuf> = dropped_files
                .into_iter()
                .filter_map(|df| df.path)
                .collect();
            if !paths.is_empty() {
                self.load_files_from_paths(&paths, ctx);
            }
        }
    }

    fn toggle_registry(&mut self) {
        if self.context_menu_active {
            match unregister_context_menu() {
                Ok(()) => {
                    self.context_menu_active = false;
                    self.status_message =
                        Some(("Right-click context menu removed".to_string(), false));
                }
                Err(err) => {
                    self.status_message = Some((format!("Failed to unregister: {err}"), true));
                }
            }
        } else {
            match register_context_menu() {
                Ok(()) => {
                    self.context_menu_active = true;
                    self.status_message =
                        Some(("Right-click context menu installed!".to_string(), false));
                }
                Err(err) => {
                    self.status_message = Some((format!("Failed to register: {err}"), true));
                }
            }
        }
    }

    fn open_github_repo(&self) {
        let _ = open::that(GITHUB_REPO_URL);
    }
}

impl eframe::App for RustImageApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested())
            && self.close_action == CloseAction::Minimize
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }

        self.handle_drag_and_drop(ctx);

        // Check background converter events
        if let Some(event) = self.converter.poll() {
            match event {
                ConvertEvent::ItemSuccess {
                    out_path,
                    current,
                    total,
                } => {
                    if !self.output_files.contains(&out_path) {
                        self.output_files.push(out_path);
                    }
                    self.status_message = Some((
                        Loc::batch_converting_progress(self.language, current, total),
                        false,
                    ));
                }
                ConvertEvent::ItemError { error, .. } => {
                    self.status_message = Some((error, true));
                }
                ConvertEvent::AllFinished {
                    successful,
                    errors,
                    ..
                } => {
                    for p in successful {
                        if !self.output_files.contains(&p) {
                            self.output_files.push(p);
                        }
                    }
                    if errors.is_empty() {
                        self.status_message = Some((
                            Loc::batch_completed(self.language, self.output_files.len()),
                            false,
                        ));
                    } else {
                        self.status_message =
                            Some((format!("Completed with {} error(s)", errors.len()), true));
                    }
                }
            }
        }

        let lang = self.language;

        // ================= 1. WINDOWS NATIVE-STYLE TOP MENU BAR =================
        TopBottomPanel::top("top_menu_bar")
            .resizable(false)
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 4.0);

                    // Dropdown 1: RustyImage (Version & GitHub Repo)
                    ui.menu_button(
                        RichText::new(Loc::app_menu())
                            .strong()
                            .size(12.5)
                            .color(Color32::from_rgb(225, 235, 250)),
                        |ui| {
                            ui.set_min_width(170.0);
                            ui.label(
                                RichText::new(format!("Version: v{}", APP_VERSION))
                                    .size(11.5)
                                    .color(Color32::from_rgb(190, 210, 235)),
                            );

                            ui.separator();

                            if ui.button("🔗 GitHub Repository").clicked() {
                                self.open_github_repo();
                                ui.close_menu();
                            }
                        },
                    );

                    ui.separator();

                    // Dropdown 2: Settings (Language, Explorer Context Menu & Close Behavior)
                    ui.menu_button(RichText::new(Loc::settings_menu(lang)).size(12.0), |ui| {
                        ui.set_min_width(230.0);

                        // Single-row Language setting with ComboBox on the right
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(Loc::language_label(lang)).size(11.5));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                egui::ComboBox::from_id_salt("settings_lang_combo")
                                    .selected_text(self.language.label())
                                    .width(85.0)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.language,
                                            Language::English,
                                            "English",
                                        );
                                        ui.selectable_value(
                                            &mut self.language,
                                            Language::Chinese,
                                            "简体中文",
                                        );
                                    });
                            });
                        });

                        #[cfg(windows)]
                        {
                            ui.separator();

                            // Windows native On/Off switch for Explorer Context Menu (no icons, no hint text)
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(Loc::explorer_menu_label(lang)).size(11.5));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let mut active = self.context_menu_active;
                                    if toggle_switch(ui, &mut active).changed() {
                                        self.toggle_registry();
                                    }
                                });
                            });
                        }

                        ui.separator();

                        // Close button radio behavior setting
                        ui.label(
                            RichText::new(Loc::close_behavior_label(lang))
                                .size(11.0)
                                .color(Color32::from_rgb(180, 195, 215)),
                        );
                        ui.radio_value(
                            &mut self.close_action,
                            CloseAction::Quit,
                            RichText::new(Loc::close_quit(lang)).size(11.0),
                        );
                        ui.radio_value(
                            &mut self.close_action,
                            CloseAction::Minimize,
                            RichText::new(Loc::close_minimize(lang)).size(11.0),
                        );
                    });
                });
            });

        // ================= 2. FLAT BORDERLESS MAIN BODY =================
        CentralPanel::default().show(ctx, |ui| {
            let total_avail_w = ui.available_width();
            let total_avail_h = ui.available_height();
            let side_pad = 10.0_f32;
            let bottom_pad = 10.0_f32;
            let body_h = (total_avail_h - bottom_pad).max(180.0_f32);
            let mid_w = 76.0_f32;
            let col_w = ((total_avail_w - (side_pad * 2.0_f32) - mid_w - 16.0_f32) / 2.0_f32)
                .max(170.0_f32);

            ui.horizontal(|ui| {
                // 10px left padding for input zone
                ui.add_space(side_pad);

                // ---------- COLUMN 1: LEFT INPUT (FULL-HEIGHT, MULTI-FILE & CONTINUOUS IMPORT) ----------
                ui.allocate_ui_with_layout(
                    Vec2::new(col_w, body_h),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.set_min_height(body_h);

                        // Header row with title and action buttons
                        ui.horizontal(|ui| {
                            let title_text = if self.imported_images.is_empty() {
                                Loc::input_title(lang).to_string()
                            } else {
                                format!(
                                    "{} ({})",
                                    Loc::input_title(lang),
                                    self.imported_images.len()
                                )
                            };
                            ui.label(
                                RichText::new(title_text)
                                    .size(15.0)
                                    .strong()
                                    .color(Color32::from_rgb(210, 220, 235)),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if !self.imported_images.is_empty() {
                                    let clear_btn = egui::Button::new(
                                        RichText::new(Loc::clear_all(lang))
                                            .size(11.0)
                                            .color(Color32::from_rgb(240, 100, 100)),
                                    )
                                    .fill(Color32::from_rgb(45, 20, 20))
                                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(90, 35, 35)))
                                    .rounding(Rounding::same(4.0));

                                    if ui.add(clear_btn).clicked() {
                                        self.clear_input();
                                    }
                                }
                            });
                        });

                        ui.add_space(4.0);

                        if self.imported_images.is_empty() {
                            // Empty Drag & Drop Zone filling available height
                            let drop_h = (body_h - 40.0_f32).max(110.0);
                            egui::Frame::none()
                                .fill(Color32::from_rgb(22, 25, 31))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 50, 60)))
                                .rounding(Rounding::same(6.0))
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_min_height(drop_h);
                                    ui.vertical_centered(|ui| {
                                        let pad = ((drop_h - 70.0) / 2.0).max(4.0);
                                        ui.add_space(pad);
                                        ui.label(
                                            RichText::new(Loc::drop_prompt(lang))
                                                .size(12.0)
                                                .color(Color32::from_rgb(195, 205, 220)),
                                        );
                                        ui.label(
                                            RichText::new(Loc::drop_subtext())
                                                .size(9.5)
                                                .color(Color32::from_rgb(115, 125, 140)),
                                        );
                                        ui.add_space(6.0);
                                        if ui
                                            .button(
                                                RichText::new(Loc::choose_file(lang)).size(11.0),
                                            )
                                            .clicked()
                                        {
                                            if let Some(picked) = rfd::FileDialog::new()
                                                .add_filter(
                                                    "Images & Animations",
                                                    &[
                                                        "png", "apng", "gif", "webp", "jpg",
                                                        "jpeg", "bmp", "ico", "tiff", "tif",
                                                        "tga", "svg",
                                                    ],
                                                )
                                                .pick_files()
                                            {
                                                self.load_files_from_paths(&picked, ctx);
                                            }
                                        }
                                    });
                                });
                        } else {
                            // Scrollable list of loaded files with per-item remove button
                            let add_btn_h = 26.0_f32;
                            let list_h = (body_h - 38.0_f32 - add_btn_h).max(70.0);
                            egui::ScrollArea::vertical()
                                .max_height(list_h)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    let mut remove_idx = None;
                                    for (idx, img) in self.imported_images.iter().enumerate() {
                                        ui.horizontal(|ui| {
                                            if let Some(ref tex) = img.texture {
                                                ui.image((tex.id(), Vec2::new(36.0, 36.0)));
                                            }
                                            ui.vertical(|ui| {
                                                let fname = img
                                                    .path
                                                    .file_name()
                                                    .and_then(|s| s.to_str())
                                                    .unwrap_or("image");
                                                ui.label(
                                                    RichText::new(fname)
                                                        .strong()
                                                        .size(11.0)
                                                        .color(Color32::WHITE),
                                                );

                                                let (w, h) = if !img.frames.is_empty() {
                                                    img.frames[0].dimensions()
                                                } else {
                                                    (0, 0)
                                                };
                                                let size_kb = img.file_size as f32 / 1024.0;
                                                let size_str = if size_kb > 1024.0 {
                                                    format!("{:.1} MB", size_kb / 1024.0)
                                                } else {
                                                    format!("{:.0} KB", size_kb)
                                                };
                                                ui.label(
                                                    RichText::new(format!("{w}×{h} • {size_str}"))
                                                        .size(9.5)
                                                        .color(Color32::from_rgb(160, 170, 185)),
                                                );
                                            });

                                            ui.with_layout(
                                                Layout::right_to_left(Align::Center),
                                                |ui| {
                                                    if ui
                                                        .small_button(
                                                            RichText::new("×")
                                                                .size(11.0)
                                                                .color(Color32::from_rgb(
                                                                    220, 110, 110,
                                                                )),
                                                        )
                                                        .clicked()
                                                    {
                                                        remove_idx = Some(idx);
                                                    }
                                                },
                                            );
                                        });
                                        ui.add_space(2.0);
                                    }

                                    if let Some(idx) = remove_idx {
                                        self.imported_images.remove(idx);
                                        self.sync_destination_if_needed();
                                    }
                                });

                            ui.add_space(4.0);

                            // Pinned Add More Media button at bottom of input area
                            let add_btn = egui::Button::new(
                                RichText::new(Loc::add_more_files(lang))
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::from_rgb(215, 230, 255)),
                            )
                            .min_size(Vec2::new(ui.available_width(), add_btn_h))
                            .fill(Color32::from_rgb(34, 42, 54))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(52, 65, 85)))
                            .rounding(Rounding::same(4.0));

                            if ui.add(add_btn).clicked() {
                                if let Some(picked) = rfd::FileDialog::new()
                                    .add_filter(
                                        "Images & Animations",
                                        &[
                                            "png", "apng", "gif", "webp", "jpg", "jpeg",
                                            "bmp", "ico", "tiff", "tif", "tga", "svg",
                                        ],
                                    )
                                    .pick_files()
                                {
                                    self.load_files_from_paths(&picked, ctx);
                                }
                            }
                        }
                    },
                );

                ui.add_space(2.0);

                // ---------- COLUMN 2: MIDDLE (VERTICALLY & HORIZONTALLY CENTERED) ----------
                ui.allocate_ui_with_layout(
                    Vec2::new(mid_w, body_h),
                    Layout::top_down(Align::Center),
                    |ui| {
                        ui.set_min_height(body_h);

                        let content_h = 84.0_f32;
                        let pad_y = ((body_h - content_h) / 2.0_f32).max(0.0);
                        ui.add_space(pad_y);

                        // Horizontal animated process arrow
                        DynamicArrow::with_size(self.converter.is_converting, 54.0, 36.0).show(ui);

                        // Progress indicator during batch
                        if let Some((curr, tot)) = self.converter.progress {
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new(format!("{curr}/{tot}"))
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::from_rgb(120, 180, 245)),
                            );
                        }

                        ui.add_space(6.0);

                        // Convert Action Button
                        let can_convert =
                            !self.imported_images.is_empty() && !self.converter.is_converting;
                        let btn_text = Loc::convert_btn(lang, self.converter.is_converting);

                        let btn = egui::Button::new(
                            RichText::new(btn_text)
                                .size(11.5)
                                .strong()
                                .color(if can_convert {
                                    Color32::WHITE
                                } else {
                                    Color32::from_rgb(130, 140, 155)
                                }),
                        )
                        .min_size(Vec2::new(68.0, 32.0))
                        .fill(if can_convert {
                            Color32::from_rgb(45, 110, 215)
                        } else {
                            Color32::from_rgb(38, 44, 54)
                        })
                        .rounding(Rounding::same(6.0));

                        if ui.add_enabled(can_convert, btn).clicked() {
                            self.trigger_conversion();
                        }
                    },
                );

                ui.add_space(2.0);

                // ---------- COLUMN 3: RIGHT OUTPUT ----------
                ui.allocate_ui_with_layout(
                    Vec2::new(col_w, body_h),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.set_min_height(body_h);

                        // Output Title
                        ui.label(
                            RichText::new(Loc::target_title(lang))
                                .size(15.0)
                                .strong()
                                .color(Color32::from_rgb(210, 220, 235)),
                        );
                        ui.add_space(4.0);

                        // Format pill buttons
                        ui.horizontal_wrapped(|ui| {
                            for fmt in TargetFormat::ALL {
                                let is_selected = self.target_format == fmt;
                                let btn = egui::Button::new(
                                    RichText::new(fmt.display_name()).size(12.5).strong(),
                                )
                                .fill(if is_selected {
                                    Color32::from_rgb(45, 110, 210)
                                } else {
                                    Color32::from_rgb(32, 36, 44)
                                })
                                .rounding(Rounding::same(4.0));

                                if ui.add(btn).clicked() {
                                    self.target_format = fmt;
                                    self.sync_destination_if_needed();
                                }
                            }
                        });

                        // Sizable spacing between format section and compression section
                        ui.add_space(10.0);

                        // Compression Toggle Switch (default OFF & greyed out)
                        ui.horizontal(|ui| {
                            toggle_switch(ui, &mut self.compression_enabled);
                            let label_color = if self.compression_enabled {
                                Color32::WHITE
                            } else {
                                Color32::from_rgb(140, 150, 165)
                            };
                            ui.label(
                                RichText::new(Loc::compression_switch(lang))
                                    .size(11.5)
                                    .strong()
                                    .color(label_color),
                            );
                        });

                        // When compression toggle is turned ON: reveal active quality slider
                        if self.compression_enabled {
                            ui.add_space(3.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(Loc::quality(lang))
                                        .size(11.0)
                                        .color(Color32::from_rgb(190, 205, 225)),
                                );
                                ui.add(
                                    egui::Slider::new(&mut self.compression_quality, 1..=100)
                                        .suffix("%")
                                        .show_value(true),
                                );
                            });
                        }

                        // Animation merge option (if multiple files and animated target format)
                        if self.imported_images.len() > 1 && self.target_format.is_animated() {
                            ui.add_space(3.0);
                            if ui
                                .checkbox(
                                    &mut self.merge_into_animation,
                                    RichText::new(Loc::merge_animation_toggle(lang)).size(11.0),
                                )
                                .changed()
                            {
                                self.sync_destination_if_needed();
                            }
                        }

                        // Delay slider for animations
                        if self.target_format.is_animated()
                            || self.imported_images.iter().any(|img| img.frames.len() > 1)
                        {
                            ui.add_space(3.0);
                            if let Some(ref mut delay) = self.custom_delay_ms {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(Loc::frame_delay(lang)).size(11.5));
                                    ui.add(
                                        egui::Slider::new(delay, 10..=1000)
                                            .suffix("ms")
                                            .show_value(true),
                                    );
                                });
                            }
                        }

                        ui.add_space(6.0);

                        // Destination label and Browse button
                        ui.horizontal(|ui| {
                            let dest_label = if self.is_batch_mode() {
                                "Folder:"
                            } else {
                                Loc::destination(lang)
                            };
                            ui.label(
                                RichText::new(dest_label)
                                    .size(11.5)
                                    .strong()
                                    .color(Color32::from_rgb(170, 185, 205)),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui
                                    .small_button(
                                        RichText::new(Loc::change_folder(lang)).size(11.0),
                                    )
                                    .clicked()
                                {
                                    if self.is_batch_mode() {
                                        if let Some(picked) = rfd::FileDialog::new().pick_folder() {
                                            self.destination_text =
                                                picked.to_string_lossy().to_string();
                                            self.manual_destination = true;
                                        }
                                    } else if let Some(picked) = rfd::FileDialog::new()
                                        .set_file_name(format!(
                                            "output.{}",
                                            self.target_format.extension()
                                        ))
                                        .save_file()
                                    {
                                        self.destination_text =
                                            picked.to_string_lossy().to_string();
                                        self.manual_destination = true;
                                    }
                                }
                            });
                        });

                        ui.add_space(2.0);

                        // Editable Destination Textfield with auto-wrapping
                        let hint = if self.is_batch_mode() {
                            Loc::select_input_folder_hint(lang)
                        } else {
                            Loc::select_input_hint(lang)
                        };
                        let text_edit = ui.add(
                            egui::TextEdit::multiline(&mut self.destination_text)
                                .desired_width(ui.available_width())
                                .desired_rows(2)
                                .hint_text(hint)
                                .font(egui::TextStyle::Monospace),
                        );
                        if text_edit.changed() {
                            self.manual_destination = true;
                        }

                        // Outputted files list below Destination section
                        if !self.output_files.is_empty() {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(Loc::output_files_title(lang))
                                        .size(11.5)
                                        .strong()
                                        .color(Color32::from_rgb(190, 210, 235)),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if let Some(last_file) = self.output_files.last() {
                                        if let Some(parent) = last_file.parent() {
                                            let parent_buf = parent.to_path_buf();
                                            if ui
                                                .small_button(
                                                    RichText::new(Loc::open_folder(lang))
                                                        .size(10.5),
                                                )
                                                .clicked()
                                            {
                                                let _ = open::that(&parent_buf);
                                            }
                                        }
                                    }
                                });
                            });

                            ui.add_space(2.0);

                            egui::ScrollArea::vertical()
                                .max_height(60.0)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    for out_path in self.output_files.iter().rev() {
                                        ui.horizontal(|ui| {
                                            let fname = out_path
                                                .file_name()
                                                .and_then(|s| s.to_str())
                                                .unwrap_or("output");
                                            ui.label(
                                                RichText::new(fname)
                                                    .size(10.5)
                                                    .color(Color32::from_rgb(130, 220, 160)),
                                            );
                                            ui.with_layout(
                                                Layout::right_to_left(Align::Center),
                                                |ui| {
                                                    let path_copy = out_path.clone();
                                                    if ui
                                                        .small_button(
                                                            RichText::new(Loc::open_file(lang))
                                                                .size(10.0),
                                                        )
                                                        .clicked()
                                                    {
                                                        let _ = open::that(&path_copy);
                                                    }
                                                },
                                            );
                                        });
                                    }
                                });
                        } else if let Some((ref msg, is_error)) = self.status_message {
                            ui.add_space(4.0);
                            let bg = if is_error {
                                Color32::from_rgb(55, 25, 25)
                            } else {
                                Color32::from_rgb(22, 50, 32)
                            };
                            let fg = if is_error {
                                Color32::from_rgb(255, 130, 130)
                            } else {
                                Color32::from_rgb(130, 240, 160)
                            };

                            egui::Frame::none()
                                .fill(bg)
                                .rounding(Rounding::same(4.0))
                                .inner_margin(4.0)
                                .show(ui, |ui| {
                                    ui.label(RichText::new(msg).size(11.0).color(fg));
                                });
                        }
                    },
                );

                // 10px right padding for output zone
                ui.add_space(side_pad);
            });
        });
    }
}
