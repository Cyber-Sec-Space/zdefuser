# Changelog

All notable changes to this project will be documented in this file.

## [main] - 2026-04-21

### Added
- **RAR Archive Support**: Fully integrated native WebAssembly isolation support for `.rar` files by vendoring and patching a pure-Rust `rar` crate.
- **RAR Password Encryption**: Added structural interface bindings to allow `.rar` password decryption entirely within the WASI physical boundary.
- **Glassmorphic Password Input UI**: Added a slick dropzone password field with visual indicators for encrypted archive decryption (ZIP/RAR).

### Changed
- Refactored `wasm-sandbox/src/decompress.rs` to smartly route between ZIP, TAR, and now RAR extractors.
- Modified App UI to remove full-page scrolling and implemented scaling flexbox constraints so the ZDefuser brand text auto-hides processing logs gracefully.
- Moved the "About & Legal" button from the main UI header to the native OS application menu for a cleaner aesthetic.
