use eframe::egui::{self, Color32, Pos2, Stroke, Ui, Vec2};

pub struct DynamicArrow {
    pub is_processing: bool,
    pub width: f32,
    pub height: f32,
}

impl DynamicArrow {
    pub fn new(is_processing: bool) -> Self {
        Self {
            is_processing,
            width: 54.0,
            height: 38.0,
        }
    }

    pub fn with_size(is_processing: bool, width: f32, height: f32) -> Self {
        Self {
            is_processing,
            width,
            height,
        }
    }

    pub fn show(&self, ui: &mut Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(self.width, self.height),
            egui::Sense::hover(),
        );

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center_y = rect.center().y;
            let left_x = rect.min.x + 4.0_f32;
            let right_x = rect.max.x - 4.0_f32;
            let width_span = right_x - left_x;

            if self.is_processing {
                ui.ctx().request_repaint();
                let time = ui.input(|i| i.time);

                // Ambient glowing energy aura
                let pulse = ((time * 4.0).sin() * 0.5 + 0.5) as f32;
                let glow_radius = 16.0_f32 + pulse * 8.0_f32;
                let glow_alpha = (35.0_f32 + pulse * 45.0_f32) as u8;
                let glow_color = Color32::from_rgba_unmultiplied(60, 160, 255, glow_alpha);
                painter.circle_filled(rect.center(), glow_radius, glow_color);

                // 3 rightward-flowing energy chevrons (pointing ->)
                for i in 0..3 {
                    let phase = ((time * 2.0 + (i as f64) * 0.33) % 1.0) as f32;
                    let x = left_x + phase * (width_span - 12.0_f32);
                    let alpha = ((1.0_f32 - (phase - 0.5_f32).abs() * 1.8_f32).clamp(0.15_f32, 1.0_f32) * 255.0_f32) as u8;
                    let stroke_color = Color32::from_rgba_unmultiplied(
                        (80.0_f32 + phase * 160.0_f32) as u8,
                        (190.0_f32 + (1.0_f32 - phase) * 65.0_f32) as u8,
                        255,
                        alpha,
                    );
                    let stroke = Stroke::new(3.0_f32, stroke_color);

                    let wing_y = 10.0_f32;
                    let p1 = Pos2::new(x, center_y - wing_y);
                    let p2 = Pos2::new(x + 10.0_f32, center_y);
                    let p3 = Pos2::new(x, center_y + wing_y);

                    painter.line_segment([p1, p2], stroke);
                    painter.line_segment([p2, p3], stroke);
                }

                // Core traveling energetic beacon dot
                let dot_phase = ((time * 2.8) % 1.0) as f32;
                let dot_x = left_x + dot_phase * width_span;
                painter.circle_filled(
                    Pos2::new(dot_x, center_y),
                    3.5_f32,
                    Color32::from_rgb(240, 250, 255),
                );
            } else {
                // Elegant idle arrow pointing horizontally rightwards (->)
                let shaft_color = Color32::from_rgb(85, 100, 120);
                let head_color = Color32::from_rgb(130, 155, 185);

                // Horizontal shaft line
                painter.line_segment(
                    [Pos2::new(left_x + 2.0_f32, center_y), Pos2::new(right_x - 6.0_f32, center_y)],
                    Stroke::new(2.0_f32, shaft_color),
                );

                // Rightward chevron head
                let head_x = right_x - 4.0_f32;
                let wing_y = 7.0_f32;
                painter.line_segment(
                    [Pos2::new(head_x - 8.0_f32, center_y - wing_y), Pos2::new(head_x, center_y)],
                    Stroke::new(2.5_f32, head_color),
                );
                painter.line_segment(
                    [Pos2::new(head_x - 8.0_f32, center_y + wing_y), Pos2::new(head_x, center_y)],
                    Stroke::new(2.5_f32, head_color),
                );
            }
        }

        response
    }
}
