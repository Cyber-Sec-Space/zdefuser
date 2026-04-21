# 🛡 ZDefuser

> Zero-Trust Sandboxed Extraction for macOS & Windows.  
> 專屬工程師與資安研究員的終極物理隔離解壓縮防護傘。

---

[English](#english) | [繁體中文](#繁體中文)

---

<br>

<h2 id="english">English</h2>

**ZDefuser** is a highly secure archive extraction tool built for engineers and security researchers. By combining unidirectional WebAssembly (Wasm) isolation technology with the native OS interface, it analyzes and extracts `.zip`, `.rar`, and `.tar` files of unknown origins within a "purely physically isolated" sandbox. This effectively blocks malicious payloads from penetrating or damaging the host system during the exact moment of decompression.

### Why ZDefuser?
Traditional OS archiving tools run with full native file permissions, providing hackers a perfect window for exploitation. ZDefuser drops the payload into a WebAssembly sandbox—completely cut off from networking and native OS calls. This "sterile extraction" guarantees immunity against the **6 classic compression attack vectors**:

1. 💣 **Zip Bomb Defusion**: Enforces strict resource ratios (max 100x inflation / 100GB limits) to completely intercept memory exhaustion attacks.
2. 🚫 **Path Traversal Blocking**: Drops any hazardous relative paths like `../../etc/passwd` attempting to escape the sandbox and overwrite the system.
3. 🌀 **Resource Exhaustion/Infinite Loops (CPU DoS)**: Integrates Wasmtime's `Fuel` (instruction quota) limit to forcibly interrupt mathematical algorithm loops that attempt to stall the CPU.
4. 🔗 **Symlink Attack Isolation**: Zero tolerance for illegal directory references and symbolic shortcuts, protecting host private keys and config files from stealthy exfiltration.
5. 🛡 **RCE / Buffer Overflow Immunity**: Even if the internal extraction engine suffers a buffer overflow, it will only trigger a Wasm linear memory trap crash. It is physically impossible to penetrate the host.
6. 🔒 **Executable Bit Stripping**: Through the Layer-3 Release Gate, any stealthy `+x` script permissions implanted by attackers are forcibly stripped, downgrading malicious executables to harmless text chunks.

### Tech Stack
* **Host**: [Tauri v2](https://v2.tauri.app/) (Rust)
* **Isolated Virtual Machine (Sandbox)**: [Wasmtime v29](https://wasmtime.dev/) (`wasm32-wasip1`)
* **Frontend UI**: React + TypeScript + Vite + Vanilla CSS (Dark minimal aesthetics)
* **Inter-process Communication**: Async Tokio MPSC Channels

### Development & Build Instructions
Ensure you have the Node.js and Rust toolchains (including the `wasm32-wasip1` target) installed.

```bash
# 1. Clone the repository
git clone https://github.com/your-repo/zdefuser.git

# 2. Install frontend dependencies
npm install

# 3. Prepare the WASM Sandbox environment
rustup target add wasm32-wasip1
cd wasm-sandbox
cargo build --target wasm32-wasip1 --release
cd ..

# 4. Kickstart developer mode
npm run tauri dev
```

### Security Penetration Testing
Included with built-in realistic penetration verification payloads.  
You can run the script `python3 tests/generate_payloads.py` to generate authentic malicious archives, including Zip Bombs, Path Traversals, and executable hijacks, and witness the defensive mechanisms in action via the UI interface.

---

<br>

<h2 id="繁體中文">繁體中文</h2>

**ZDefuser** 是一個為工程師與資安研究員打造的極致安全解壓縮工具。透過整合 WebAssembly (Wasm) 單向隔離技術與原生作業系統介面，它能在「純物理隔離」的虛擬沙箱內剖析未知來源的 `.zip`、`.rar` 與 `.tar` 檔案，有效阻斷惡意程式在解壓縮瞬間造成的系統滲透與破壞。

### 為什麼需要 ZDefuser？
傳統的作業系統解壓工具具備過高的原生檔案權限，這讓駭客有機可乘。ZDefuser 將檔案丟進無實體網路、無作業系統呼叫權限的 WebAssembly 沙箱中進行「無菌抽取」，徹底免疫以下**六大傳統解壓縮攻擊向量**：

1. 💣 **解壓炸彈 (Zip Bomb) 攔截**：採用硬性資源比例上限 (最高防護達 `100 倍膨脹 / 100 GB`) 阻斷記憶體溢出攻擊。
2. 🚫 **目錄穿越 (Path Traversal)**：攔截所有 `../../etc/passwd` 等企圖跳脫沙箱覆寫系統檔案的危險路徑。
3. 🌀 **資源耗盡/死迴圈 (CPU DoS)**：內建 Wasmtime `Fuel` (執行指令配額) 極限，強制中斷企圖卡死 CPU 的惡意壓縮演算法。
4. 🔗 **符號連結 (Symlink) 隔離**：對非法目錄參照與符號捷徑做到零容忍丟棄，防護私鑰與配置檔遭竊。
5. 🛡 **任意代碼執行 (RCE/Buffer Overflow) 免疫**：解壓引擎就算發生緩衝區溢位，也只會導致 Wasm 線性記憶體 (Linear Memory) 陷阱崩潰，不可能滲透宿主機。
6. 🔒 **剝除可執行權限 (Executable Bit Retention)**：經過 Layer 3 釋放閘道 (Release Gate)，駭客植入的隱形 `+x` 可執行權限會被強制扒除，將執行檔降級為無害純文字。

### 核心技術棧 (Tech Stack)
* **宿主架構 (Host)**: [Tauri v2](https://v2.tauri.app/) (Rust)
* **隔離虛擬機 (Sandbox)**: [Wasmtime v29](https://wasmtime.dev/) (`wasm32-wasip1`)
* **使用者介面 (Frontend)**: React + TypeScript + Vite + Vanilla CSS (極黑幾何美學)
* **通訊層**: 異步 Tokio 管道 (Async MPSC Channels)

### 如何安裝與建置 (Development)
首先確保您已安裝了 Node.js 與 Rust 工具鏈（含 `wasm32-wasip1` target）。

```bash
# 1. 複製專案
git clone https://github.com/your-repo/zdefuser.git

# 2. 安裝前端依賴
npm install

# 3. 準備 WASM 沙箱環境
rustup target add wasm32-wasip1
cd wasm-sandbox
cargo build --target wasm32-wasip1 --release
cd ..

# 4. 啟動開發者模式
npm run tauri dev
```

### 測試驅動 (Security Payloads)
內建真實駭客測試包 (Penetration Verification Payloads)：  
您可以透過執行測試指令碼 `python3 tests/generate_payloads.py`，產生包含 Zip Bomb、Path Traversal 與可執行權限劫持的真實惡意壓縮包，並透過介面親自見證攔截防禦機制。

---

> _"In a zero-trust world, even air doesn't pass verification without inspecting its atoms."_ 
> _「在零信任的世界裡，就連空氣也必須先檢驗過原子才能放行。」_

**Audited securely by Snyk 🐶 - 0 Vulnerabilities Detected.**
