#!/usr/bin/env bash

# ==============================================================================
# ZDefuser Multi-Platform Release Builder
# 
# Note: Cross-compiling GUI applications (Tauri/Rust) from macOS to Linux/Windows 
# locally requires specific toolchains (e.g., cargo-xwin, osxcross, etc).
# For pure automated cross-compilation, using CI/CD (GitHub Actions) is recommended.
# ==============================================================================

set -e
export PATH="/Users/ashodesu/.nvm/versions/node/v24.15.0/bin:$PATH"

echo "============================================="
echo "🛡 ZDefuser: Multi-Platform Release Builder "
echo "============================================="

# 1. Build WASM Sandbox securely
echo "📦 [1/4] Building WASM Sandbox (wasm32-wasip1)..."
cd wasm-sandbox
cargo build --target wasm32-wasip1 --release
cd ..

# 2. Build Mac (Host)
echo "🍎 [2/4] Building macOS Release (.app, .dmg)..."
npm run tauri build

# 3. Build Windows (requires target installed)
echo "🪟 [3/4] Building Windows Release (.exe, .msi)..."
echo "ℹ️ Note: This requires 'x86_64-pc-windows-msvc' target and linker installed."
if rustup target list | grep -q "x86_64-pc-windows-msvc (installed)"; then
    npm run tauri build -- --target x86_64-pc-windows-msvc || echo "⚠️ Windows build encountered errors."
else
    echo "⚠️ Windows target not found. Skip or run: rustup target add x86_64-pc-windows-msvc"
fi

# 4. Build Linux (requires cross-compilation linker like cargo-zigbuild or docker)
echo "🐧 [4/4] Building Linux Release (.AppImage, .deb)..."
echo "ℹ️ Note: Building for Linux on macOS usually requires Docker or 'x86_64-unknown-linux-gnu' toolchain."
if rustup target list | grep -q "x86_64-unknown-linux-gnu (installed)"; then
    npm run tauri build -- --target x86_64-unknown-linux-gnu || echo "⚠️ Linux build encountered errors."
else
    echo "⚠️ Linux target not found. Skip or run: rustup target add x86_64-unknown-linux-gnu"
fi

echo "============================================="
echo "📦 Packaging Artifacts for GitHub Releases"
echo "============================================="

# Get version from package.json for naming
VERSION=$(node -p "require('./package.json').version")
ARTIFACTS_DIR="release-artifacts"

# Refresh artifacts directory
rm -rf "$ARTIFACTS_DIR"
mkdir -p "$ARTIFACTS_DIR"

# Mac
if [ -d "target/release/bundle/macos" ]; then
    echo "打包 macOS .app 為 tar.gz..."
    tar -czf "$ARTIFACTS_DIR/ZDefuser-v$VERSION-macos.app.tar.gz" -C target/release/bundle/macos ZDefuser.app
fi

if [ -d "target/release/bundle/dmg" ]; then
    echo "複製 macOS DMG..."
    cp target/release/bundle/dmg/*.dmg "$ARTIFACTS_DIR/ZDefuser-v$VERSION-macos.dmg" 2>/dev/null || true
fi

# Windows
if [ -d "target/x86_64-pc-windows-msvc/release/bundle/msi" ]; then
    echo "複製 Windows MSI..."
    cp target/x86_64-pc-windows-msvc/release/bundle/msi/*.msi "$ARTIFACTS_DIR/ZDefuser-v$VERSION-windows.msi" 2>/dev/null || true
fi
if [ -d "target/x86_64-pc-windows-msvc/release/bundle/nsis" ]; then
    echo "複製 Windows EXE setup..."
    cp target/x86_64-pc-windows-msvc/release/bundle/nsis/*.exe "$ARTIFACTS_DIR/ZDefuser-v$VERSION-windows-setup.exe" 2>/dev/null || true
fi

# Linux
if [ -d "target/x86_64-unknown-linux-gnu/release/bundle/appimage" ]; then
    echo "複製 Linux AppImage..."
    cp target/x86_64-unknown-linux-gnu/release/bundle/appimage/*.AppImage "$ARTIFACTS_DIR/ZDefuser-v$VERSION-linux.AppImage" 2>/dev/null || true
fi
if [ -d "target/x86_64-unknown-linux-gnu/release/bundle/deb" ]; then
    echo "複製 Linux DEB..."
    cp target/x86_64-unknown-linux-gnu/release/bundle/deb/*.deb "$ARTIFACTS_DIR/ZDefuser-v$VERSION-linux.deb" 2>/dev/null || true
fi

echo "============================================="
echo "✅ Build & Packaging Completed!"
echo "All release files are ready in: ./$ARTIFACTS_DIR/"
echo "You can now select these files and drop them into a GitHub Release."
echo "============================================="
