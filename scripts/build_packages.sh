#!/usr/bin/env bash
# Local packaging helper script for Linux (.deb) and macOS (.dmg)
set -e

OS="$(uname -s)"
echo "Detected OS: $OS"

if [ "$OS" = "Linux" ]; then
    echo "=== Building Linux release binary & .deb package ==="
    cargo build --release
    if command -v cargo-deb &> /dev/null; then
        cargo deb --no-build
        echo "Debian package created in target/debian/"
    else
        echo "cargo-deb not installed. Install with: cargo install cargo-deb"
    fi
elif [ "$OS" = "Darwin" ]; then
    echo "=== Building macOS Universal binary & .dmg ==="
    rustup target add x86_64-apple-darwin aarch64-apple-darwin
    cargo build --release --target x86_64-apple-darwin
    cargo build --release --target aarch64-apple-darwin

    mkdir -p bin-universal
    lipo -create -output bin-universal/rustyimage target/x86_64-apple-darwin/release/rustyimage target/aarch64-apple-darwin/release/rustyimage

    mkdir -p RustyImage.app/Contents/MacOS
    mkdir -p RustyImage.app/Contents/Resources
    cp bin-universal/rustyimage RustyImage.app/Contents/MacOS/rustyimage
    chmod +x RustyImage.app/Contents/MacOS/rustyimage

    cat << 'EOF' > RustyImage.app/Contents/Info.plist
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>rustyimage</string>
    <key>CFBundleIdentifier</key>
    <string>com.rustyimage.app</string>
    <key>CFBundleName</key>
    <string>RustyImage</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.2.0</string>
    <key>CFBundleVersion</key>
    <string>0.2.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

    hdiutil create -volname "RustyImage" -srcfolder RustyImage.app -ov -format UDZO RustyImage-macOS-Universal.dmg
    echo "macOS DMG created: RustyImage-macOS-Universal.dmg"
else
    echo "Unsupported OS for this script: $OS. On Windows, use 'cargo build --release'."
fi
