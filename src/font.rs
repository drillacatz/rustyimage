use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};

pub fn setup_custom_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    let font_paths = [
        // Windows
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\msyh.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
        r"C:\Windows\Fonts\simhei.ttf",
        // macOS
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
        // Linux (Debian / Ubuntu / Arch / Fedora)
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
    ];

    for path in font_paths {
        if let Ok(font_bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "cjk_font".to_owned(),
                FontData::from_owned(font_bytes),
            );

            if let Some(prop) = fonts.families.get_mut(&FontFamily::Proportional) {
                prop.push("cjk_font".to_owned());
            }
            if let Some(mono) = fonts.families.get_mut(&FontFamily::Monospace) {
                mono.push("cjk_font".to_owned());
            }
            break;
        }
    }

    ctx.set_fonts(fonts);
}
