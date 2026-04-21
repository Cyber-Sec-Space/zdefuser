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
echo "✅ Build Process Completed!"
echo "Outputs are available in: src-tauri/target/<target>/release/bundle/"
echo "============================================="
