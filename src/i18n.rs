#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    pub fn label(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Chinese => "简体中文",
        }
    }
}

pub struct Loc;

impl Loc {
    pub fn app_name() -> &'static str {
        "RustyImage"
    }

    pub fn input_title(lang: Language) -> &'static str {
        match lang {
            Language::English => "INPUT",
            Language::Chinese => "输入源",
        }
    }

    pub fn clear(lang: Language) -> &'static str {
        match lang {
            Language::English => "Clear",
            Language::Chinese => "清空",
        }
    }

    pub fn drop_prompt(lang: Language) -> &'static str {
        match lang {
            Language::English => "Drop image file here",
            Language::Chinese => "拖放图片或动图至此",
        }
    }

    pub fn drop_subtext() -> &'static str {
        "PNG, APNG, GIF, WEBP, JPG, BMP"
    }

    pub fn choose_file(lang: Language) -> &'static str {
        match lang {
            Language::English => "Choose File...",
            Language::Chinese => "浏览文件...",
        }
    }

    pub fn static_image(lang: Language) -> &'static str {
        match lang {
            Language::English => "Static Image (1 frame)",
            Language::Chinese => "静态图片 (1 帧)",
        }
    }

    pub fn anim_info(lang: Language, frames: usize, delay_ms: u32) -> String {
        match lang {
            Language::English => format!("{frames} frames • {delay_ms}ms/f (Animated)"),
            Language::Chinese => format!("{frames} 帧 • {delay_ms}ms/帧 (动图)"),
        }
    }

    pub fn convert_btn(lang: Language, is_converting: bool) -> &'static str {
        match (lang, is_converting) {
            (Language::English, false) => "CONVERT",
            (Language::English, true) => "Converting...",
            (Language::Chinese, false) => "开始转换",
            (Language::Chinese, true) => "转换中...",
        }
    }

    pub fn target_title(lang: Language) -> &'static str {
        match lang {
            Language::English => "OUTPUT",
            Language::Chinese => "输出目标",
        }
    }

    pub fn quality(lang: Language) -> &'static str {
        match lang {
            Language::English => "Quality:",
            Language::Chinese => "质量:",
        }
    }

    pub fn frame_delay(lang: Language) -> &'static str {
        match lang {
            Language::English => "Delay:",
            Language::Chinese => "帧延迟:",
        }
    }

    pub fn destination(lang: Language) -> &'static str {
        match lang {
            Language::English => "Destination:",
            Language::Chinese => "保存路径:",
        }
    }

    pub fn change_folder(lang: Language) -> &'static str {
        match lang {
            Language::English => "Browse...",
            Language::Chinese => "更改...",
        }
    }

    pub fn select_input_hint(lang: Language) -> &'static str {
        match lang {
            Language::English => "(Select input file)",
            Language::Chinese => "(请选择输入图片)",
        }
    }

    pub fn conversion_success(lang: Language) -> &'static str {
        match lang {
            Language::English => "Converted successfully!",
            Language::Chinese => "转换成功！",
        }
    }

    pub fn open_file(lang: Language) -> &'static str {
        match lang {
            Language::English => "Open File",
            Language::Chinese => "打开文件",
        }
    }

    pub fn open_folder(lang: Language) -> &'static str {
        match lang {
            Language::English => "Open Folder",
            Language::Chinese => "定位文件",
        }
    }

    pub fn settings_title(lang: Language) -> &'static str {
        match lang {
            Language::English => "Settings",
            Language::Chinese => "系统设置",
        }
    }

    pub fn language_label(lang: Language) -> &'static str {
        match lang {
            Language::English => "Language / 语言:",
            Language::Chinese => "界面语言:",
        }
    }

    pub fn explorer_menu_label(lang: Language) -> &'static str {
        match lang {
            Language::English => "Right-Click Context Menu:",
            Language::Chinese => "右键快捷菜单:",
        }
    }

    pub fn close_behavior_label(lang: Language) -> &'static str {
        match lang {
            Language::English => "When closing window:",
            Language::Chinese => "关闭窗口时:",
        }
    }

    pub fn close_quit(lang: Language) -> &'static str {
        match lang {
            Language::English => "Quit completely",
            Language::Chinese => "彻底退出程序",
        }
    }

    pub fn close_minimize(lang: Language) -> &'static str {
        match lang {
            Language::English => "Minimize to background",
            Language::Chinese => "最小化到后台",
        }
    }

    pub fn info_title(lang: Language) -> &'static str {
        match lang {
            Language::English => "About RustyImage",
            Language::Chinese => "关于 RustyImage",
        }
    }

    pub fn app_description(lang: Language) -> &'static str {
        match lang {
            Language::English => "Minimalist Pure-Rust Image & Animation Converter",
            Language::Chinese => "纯 Rust 编写的极简图片与动画转换器",
        }
    }

    pub fn app_menu() -> &'static str {
        "RustyImage"
    }

    pub fn settings_menu(lang: Language) -> &'static str {
        match lang {
            Language::English => "Settings",
            Language::Chinese => "设置",
        }
    }

    pub fn without_compression(lang: Language) -> &'static str {
        match lang {
            Language::English => "Without Compression",
            Language::Chinese => "无损/原画质",
        }
    }

    pub fn with_compression(lang: Language) -> &'static str {
        match lang {
            Language::English => "With Compression",
            Language::Chinese => "开启压缩",
        }
    }

    pub fn compression_mode(lang: Language) -> &'static str {
        match lang {
            Language::English => "Quality Mode:",
            Language::Chinese => "品质模式:",
        }
    }

    pub fn visit_github_tooltip(lang: Language) -> &'static str {
        match lang {
            Language::English => "Click to visit GitHub repository",
            Language::Chinese => "点击访问 GitHub 开源仓库",
        }
    }

    pub fn add_more_files(lang: Language) -> &'static str {
        match lang {
            Language::English => "+ Add More Media",
            Language::Chinese => "+ 添加更多图片",
        }
    }

    pub fn clear_all(lang: Language) -> &'static str {
        match lang {
            Language::English => "🗑 Clear All",
            Language::Chinese => "🗑 全部清空",
        }
    }

    pub fn compression_switch(lang: Language) -> &'static str {
        match lang {
            Language::English => "Compression",
            Language::Chinese => "启用压缩",
        }
    }

    pub fn merge_animation_toggle(lang: Language) -> &'static str {
        match lang {
            Language::English => "Merge as single animation",
            Language::Chinese => "合并为单一动图",
        }
    }

    pub fn output_files_title(lang: Language) -> &'static str {
        match lang {
            Language::English => "Output Files:",
            Language::Chinese => "已输出文件:",
        }
    }

    pub fn batch_converting_progress(lang: Language, current: usize, total: usize) -> String {
        match lang {
            Language::English => format!("Converting {current}/{total}..."),
            Language::Chinese => format!("正在转换 {current}/{total}..."),
        }
    }

    pub fn batch_completed(lang: Language, count: usize) -> String {
        match lang {
            Language::English => format!("Converted {count} files successfully!"),
            Language::Chinese => format!("成功转换 {count} 个文件！"),
        }
    }

    pub fn select_input_folder_hint(lang: Language) -> &'static str {
        match lang {
            Language::English => "(Select output folder)",
            Language::Chinese => "(选择输出文件夹)",
        }
    }
}

