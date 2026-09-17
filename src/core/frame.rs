use egui::ColorImage;
use image::RgbaImage;

#[derive(Debug, Clone)]
pub struct Frame {
    pub image: RgbaImage,
    pub delay_ms: u32,
}

impl Frame {
    pub fn new(image: RgbaImage, delay_ms: u32) -> Self {
        Self {
            image,
            delay_ms: delay_ms.max(10), // minimum 10ms
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn update_image(&mut self, new_image: RgbaImage) {
        self.image = new_image;
    }

    pub fn to_color_image(&self) -> ColorImage {
        let (w, h) = self.image.dimensions();
        let raw = self.image.as_raw();
        ColorImage::from_rgba_unmultiplied([w as usize, h as usize], raw)
    }
}
