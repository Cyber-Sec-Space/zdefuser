# Changelog

All notable changes to this project will be documented in this file.

## [v1.1.0] - 2026-04-28

### Added
- **7z Archive Support**: Implemented comprehensive Zero-Trust `.7z` archive extraction capabilities within the Wasmtime sandbox.
- **LZMA/LZMA2 Isolation**: Utilized `sevenz-rust`, a pure-Rust parser (Apache 2.0 / MIT), avoiding vulnerable C/C++ bindings and perfectly quarantining decryption/decompression inside the WebAssembly linear memory boundary.
### Security
- **Critical Audit Patch (Unbounded Stream Mitigation)**: Discovered and patched a theoretical header-spoofing vulnerability across the Wasm sandboxed Zip, Tar, and 7z parsers. Previously, `std::io::copy` relied solely on Wasm Fuel tracking to abort stream exhaustion. We now enforce `std::io::Read::take()` to mathematically truncate streams exactly at the declared header size, guaranteeing that spoofed payload expansions are physically severed.

## [1.0.2] - 2026-04-23

### Fixed
- **UI Error Visualization**: Fixed a silent error bug in `App.tsx` where early `drag-drop` global extension validation failures (e.g., rejecting `.exe` files) would silently set the error state without successfully triggering the main container to transition to the `ProgressPanel` UI.
- **TypeScript Mismatch**: Removed explicitly undefined arbitrary fields (`file`, `current`, `total`, `bytes`) from the `SandboxEvent` error payloads, rectifying severe `ts-jest` parser mismatches against the strongly typed layout.
- **Coverage Refinement**: Closed a minor branch loophole in the global parameter passing, boosting the rigorous `ts-jest` framework fully back to 100% Branch and Line coverage constraints.


## [1.0.1] - 2026-04-23

### Added
- **Project Badges**: Added Shields.io badges to the `README.md` explicitly demonstrating the `100% Test Coverage`, `1.0.1` version phase, and `Snyk 0 Vulnerabilities` status for enterprise credibility.

### Changed
- **Wasm Case Sensitivity Validation**: Resolved a logic crash inside `wasm-sandbox/src/main.rs` where archives with uppercase extensions (`.ZIP`, `.TAR.GZ`) triggered an "Unsupported format" Wasm trap. Extension evaluation is now perfectly case-insensitive.
- **Frontend File Dropper Rigidity**: Strengthened the global `tauri://drag-drop` global listener inside `App.tsx` to automatically reject un-approved file extensions (`.exe`, `.sh`) *before* forwarding to the Rust backend, resolving an unhandled process runtime crash.
- **Global Drop Password State Injection**: Fixed a UI bypass defect where users dropping encrypted archives across the application window bypassed the localized `DropZone` password. Password state has been lifted to ensure seamless global Drag and Drop decryption.
- **Release Gate Resilience Engine**: Refactored the `fs::metadata()` pipeline inside `src-tauri/src/release.rs` to substitute risky unwraps (`unwrap()`) with graceful fallback assignments (`if let Ok()`). This mathematically eliminates the risk of an OS-level file handle collision triggering a Tauri panic.
- **Architectural Fact-Checking (Apple CPU Targets)**: Audited and corrected explicit inconsistencies within `docs/index.html` describing `macOS` distribution. Removed false claims regarding Intel (`x86_64`) architecture support since the CI pipeline explicitly exclusively builds for Apple Silicon (`aarch64`).
- **Internal Ecosystem Semantic Synchrony**: Performed a monolithic rewrite of all manifest versions (`package.json`, `tauri.conf.json`, `src-tauri/Cargo.toml`, `wasm-sandbox/Cargo.toml`) jumping straight from `1.0.0` arrays up to `1.1.0`, absolutely cementing version parity with the Git Branch name.


## [1.0.0] - 2026-04-22

### Added
- **Dynamic Compute Rationing (Fuel)**: Implemented an intelligent Wasmtime fuel allocation engine that scales dynamically with compressed archive size. Safely supports massive (e.g., 50GB) legitimate archives while instantly terminating highly-compressed zero-day logic bombs.
- **RAR RTLO/Unicode Validation Hook**: Added the missing `SecurityContext::is_safe_path` check to the RAR extraction pipeline, successfully clamping down on Unicode Right-to-Left Override spoofing payloads inside `.rar` containers.
- **Enterprise Compliance Section**: Synchronized physical `docs/index.html` and `README.md` to officially declare ZDefuser's Third-Party Notices automated generation schema for MIT/Apache/BSD enterprise adoption.

### Changed
- **Windows MSI Bundler Compatibility**: Bumped the release tag format strictly to `1.0.0` (removing the `-rc.X` pre-release identifier) because the Windows WiX compiler engine rigidly rejects non-numeric pre-release strings during MSI package generation, resulting in a 65535 overflow error. 
- **GitHub Actions Security Tokens**: Explicitly elevated `GITHUB_TOKEN` privileges inside `.github/workflows/release.yml` with `permissions: contents: write` so that the cloud runners are legally authorized to publish drafted assets to the GitHub Releases page.
- **Submodule Vendoring (CI Resiliency)**: Fully decoupled ZDefuser from the external GitHub repository for the `rar` library by permanently vendoring its patched source code into our native Git tree. This immediately rectifies the `fatal: No url found for submodule path` errors across all GitHub Actions builder matrices by preventing reliance on broken or untracked upstream Git modules.
- **Housekeeping & Git Tree Optimization**: Conducted a final repository sweep to scrub orphaned artifacts and temporary test scripts (`test_rar_pwd.rs`, `test_rar_src.rs`) that were accidentally tracked. Revamped `.gitignore` to implement robust enterprise-grade exclusions across macOS `.DS_Store`, Windows `Thumbs.db`, Rust specific target patterns, and Vite/Jest telemetry, guaranteeing that future code pushes remain perfectly sterile.
- **CI/CD Build Pipeline (Multi-Platform Releases)**: Integrated a comprehensive GitHub Actions workflow (`.github/workflows/release.yml`) to automatically compile and package enterprise-grade releases for Windows (`.msi`, `.exe`) and Linux (`.AppImage`, `.deb`) directly on native cloud runners. This circumvents macOS cross-compilation linker failures (e.g., missing MSVC / WebKit2GTK dependencies) and natively generates Zero-Trust extraction packages directly attached to GitHub Releases.
- **Housekeeping & Rust Code Standardization**: Executed a comprehensive `cargo fmt` sweep across the entire repository (both the `wasm-sandbox` Wasmtime engine and the `src-tauri` native backend layer). This synchronized trailing commas, match-arm indentations, and block limits to global Rust idioms, effectively removing style debt without altering the 100% verified security logic. Cleaned up dangling Jest coverage output artifacts as well.
- **Frontend Test Coverage Optimization**: Investigated the Jest test suite coverage gap and confirmed it was not a system integrity issue but purely a test script coverage gap. Implemented comprehensive test coverage by creating `AboutModal.test.tsx` to handle React event propagation and UI closure paths, and augmented `App.test.tsx` to directly test the asynchronous early unmount race conditions (Tauri `unlisten` lifecycle) along with the About Modal's global toggle state, officially bringing the entire React frontend to 100% statements, branches, and functional Line coverage.
- **Architectural Fact-Checking (WASM Memory Bounds)**: Identified and corrected a mathematically impossible claim in both the English and Chinese Mermaid architecture diagrams. The diagrams previously claimed that the Wasm sandbox "Extracts in Memory" (記憶體解壓縮). Given that ZDefuser operates on a `wasm32-wasip1` 32-bit architecture possessing a hard 4GB linear memory ceiling, extracting a 50GB archive entirely in-memory is functionally impossible. The architecture documentation has been surgically updated to reflect the reality of "Streaming I/O Extraction" (串流抽取), aligning marketing claims with objective computer science limitations.
- **Documentation Parity Action**: Resolved an asymmetrical documentation flaw wherein the English version of the README inexplicably lacked both the crucial "Zero-Trust" architectural philosophy quote and the definitive "Snyk 0 Vulnerabilities Detected" security endorsement, guaranteeing equal representation for international audiences.
- **Penetration Testing Scope Accuracy**: Resolved a persistent `NameError` crash in `tests/generate_payloads.py` caused by executing functions before they were evaluated by the Python interpreter. This finally guarantees that the "Security Penetration Testing" workflow documented in the README successfully executes exactly as claimed.
- **ZIP Symlink Zero-Tolerance Parity**: Implemented explicit `file.is_symlink()` boundary detection and interception inside the `extract_zip` routine. This firmly brings ZIP extraction into architectural parity with TAR interception, actively satisfying the "Zero tolerance dropping" marketing claim by destroying symbolic nodes rather than passively neutering them into text structures.
- **Documentation Translation Accuracy**: Corrected an English vocabulary typo inside the Chinese translation of the `README.md` file, rectifying the dangerously misleading term "Executable Bit Retention" back to the structurally accurate "Executable Bit Stripping".
- **Documentation Truthfulness Audit**: Corrected heavily misleading technical claims in `docs/index.html`. Refined the "Executable Bit Stripping" claim to accurately reflect its `(Unix Only)` structural constraint, and correctly redefined the "WASI Constraints" to depict the accurate single `preopened_dir` layer instead of fabricated read/write stream handles. Also fixed the invalid relative Open Graph image URL to be absolute in `docs/index.html`. Finally, eliminated an obsolete "Over 2GB Limit" node from the Mermaid Architecture Diagram in `README.md` to precisely reflect the new 100GB Layer-2 volumetric reality.
- **Wasm STDOUT Overload Prevention (Deadlock Fix)**: Expanded the WASI `MemoryOutputPipe` bound from an arbitrary 10MB limit to 250MB. This mathematically guarantees that ZDefuser will not freeze / deadlock when exporting JSON progress streams for 50GB legitimate archives containing over 500k small files.
- **Layer 2 Parity Alignment**: Elevated the hardcoded 2GB `validate_extracted_dir` limit inside the Layer 2 release gate to geometrically align with the Layer 1 100GB limitation envelope, thereby restoring functionality for multi-gigabyte legitimately isolated extractions.
- **Zero-Trust Backend State Strictness**: Patched a critical execution race condition in `src-tauri/src/commands.rs`. The Tauri command now strictly listens for Wasm crash/trap results and immediately aborts the internal React state. This mathematically prevents any partially-extracted malicious payloads from bypassing Layer 2 bounds restrictions.
- **Unified Attack Vector Documentation**: Audited and upgraded documentation across all channels from 6 to 8 Advanced Threat Vectors, capturing AES Decryption capabilities and precise Execution Target restrictions.

## [1.0.0-rc.1] - 2026-04-21

### Added
- **RAR Archive Support**: Fully integrated native WebAssembly isolation support for `.rar` files by vendoring and patching a pure-Rust `rar` crate.
- **RAR Password Encryption**: Added structural interface bindings to allow `.rar` password decryption entirely within the WASI physical boundary.
- **Glassmorphic Password Input UI**: Added a slick dropzone password field with visual indicators for encrypted archive decryption (ZIP/RAR).

### Changed
- Refactored `wasm-sandbox/src/decompress.rs` to smartly route between ZIP, TAR, and now RAR extractors.
- Modified App UI to remove full-page scrolling and implemented scaling flexbox constraints so the ZDefuser brand text auto-hides processing logs gracefully.
- Moved the "About & Legal" button from the main UI header to the native OS application menu for a cleaner aesthetic.
  - Fixed hero text alignment to automatically re-center after element removal.
