# RustyImage 🦀🖼️

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-blue.svg)](https://github.com/drillacatz/rustyimage)

**RustyImage** is a minimalist, ultra-lightweight, high-performance image, animation, and vector converter written in pure Rust with eframe/egui. It provides fast local conversion between formats, batch processing, true vectorization & SVG rendering, seamless Windows Explorer right-click integration, and cross-platform native typography.

一款极简、超轻量、纯 Rust 编写的高性能本地图片、动图与矢量图格式转换器。基于 eframe/egui 打造，启动秒开、无复杂依赖。支持多图批量转换、SVG 矢量渲染与位图转贝塞尔矢量路径、Windows 资源管理器右键快捷菜单直达、无损/压缩品质切换、动图帧率调节与序列合并，并内嵌跨平台中文字体支持，全面兼容 Windows、Linux 与 macOS。

---

## 🎯 Supported Formats / 支持格式

| Format | Extension | Static | Animated | Transparency | Description / Features |
| :--- | :--- | :---: | :---: | :---: | :--- |
| **PNG** | `.png` | ✅ | — | ✅ 32-bit RGBA | Default format, lossless & universal |
| **JPEG** | `.jpg`, `.jpeg` | ✅ | — | — | Photographic lossy compression |
| **WebP** | `.webp` | ✅ | — | ✅ Lossless / Lossy | Ultra-efficient web image format |
| **GIF** | `.gif` | ✅ | ✅ | ✅ 1-bit Transparency | Classic web animation & static |
| **APNG** | `.apng` | ✅ | ✅ | ✅ Full 8-bit Alpha | High-fidelity animated PNG |
| **BMP** | `.bmp` | ✅ | — | — | Uncompressed Windows bitmap |
| **ICO** | `.ico` | ✅ | — | ✅ 32-bit RGBA | Windows icon (auto-scaled to ≤256×256) |
| **TIFF** | `.tiff`, `.tif` | ✅ | — | ✅ 32-bit RGBA | High-depth scanning & print imaging |
| **TGA** | `.tga` | ✅ | — | ✅ 32-bit RGBA | Truevision Targa game/texture asset |
| **SVG** | `.svg` | ✅ | — | ✅ Scalable Vector | Pure-Rust Bézier curve tracing & rasterization |

---

## 🛠️ Installation & Packaging / 安装与打包

Pre-compiled packages for Windows (`.exe` / `.zip`), Linux (`.deb` / `.tar.gz`), and macOS (Universal `.dmg` / `.app`) are automatically built and published via GitHub Actions Releases on every release tag.

### Build from Source

#### 1. Clone repository:
```bash
git clone <repository-url>
cd rustyimage
```

#### 2. Compile release binary:
```bash
cargo build --release
```

#### 3. Package for Linux (.deb):
```bash
# Installs cargo-deb if not already installed
cargo install cargo-deb
cargo deb --no-build
# Output: target/debian/rustyimage_*.deb
```

#### 4. Package for macOS (.dmg & Universal .app):
```bash
# Runs the packaging script on macOS
chmod +x scripts/build_packages.sh
./scripts/build_packages.sh
# Output: RustyImage-macOS-Universal.dmg
```

#### 5. Windows Context Menu (Optional):
```bash
target\release\rustyimage.exe --register
# To unregister anytime:
target\release\rustyimage.exe --unregister
```

---

## 📖 Usage / 使用说明

### GUI Mode
1. Double-click `rustyimage.exe` (or `rustyimage` on Linux/macOS) or launch from terminal.
2. Drag and drop image/vector files into the left Input area, or click **[ Choose Files... ]**.
3. Select your desired output format (**PNG** by default, JPG, WEBP, GIF, APNG, BMP, ICO, TIFF, TGA, SVG).
4. Toggle **Compression** if you wish to adjust output quality.
5. Click **[ CONVERT ]** in the center!
6. Click **[ Open File ]** beside any converted item in the output list to inspect your image immediately.

### Settings Menu
Click **Settings** on the top menu bar to:
- Switch language between **English** and **简体中文**.
- Toggle Windows **Right-Click Context Menu** On/Off.
- Choose close window behavior: **Quit completely** or **Minimize to background**.

---

## 🧪 Testing / 单元测试

RustyImage includes end-to-end codec tests covering APNG, GIF roundtrips, format properties, file generation, and batch concurrency:
```bash
cargo test
```

---

## 📄 License / 开源许可

Distributed under the **MIT License**. See [LICENSE](LICENSE) for more information.

Copyright (c) 2026 RustyImage Contributors.
