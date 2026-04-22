# Changelog

All notable changes to this project will be documented in this file.

## [main] - 2026-04-22

### Added
- **Dynamic Compute Rationing (Fuel)**: Implemented an intelligent Wasmtime fuel allocation engine that scales dynamically with compressed archive size. Safely supports massive (e.g., 50GB) legitimate archives while instantly terminating highly-compressed zero-day logic bombs.
- **RAR RTLO/Unicode Validation Hook**: Added the missing `SecurityContext::is_safe_path` check to the RAR extraction pipeline, successfully clamping down on Unicode Right-to-Left Override spoofing payloads inside `.rar` containers.
- **Enterprise Compliance Section**: Synchronized physical `docs/index.html` and `README.md` to officially declare ZDefuser's Third-Party Notices automated generation schema for MIT/Apache/BSD enterprise adoption.

### Changed
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
