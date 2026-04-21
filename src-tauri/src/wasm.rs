
use wasmtime::*;
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::pipe::{MemoryInputPipe, MemoryOutputPipe};
use wasmtime_wasi::{WasiCtxBuilder, DirPerms, FilePerms};
use tokio::sync::mpsc;
use crate::sandbox::SandboxEnv;

const WASM_BYTES: &[u8] = include_bytes!("../../target/wasm32-wasip1/release/wasm-sandbox.wasm");

struct MyState {
    wasi: WasiP1Ctx,
}

pub fn run_wasm_sandbox(env: &SandboxEnv, host_json_cmd: String, tx: mpsc::Sender<String>) -> Result<(), String> {
    let mut config = Config::new();
    config.consume_fuel(true); // Limit compute usage

    let engine = Engine::new(&config).map_err(|e| format!("Engine init error: {}", e))?;
    let mut linker = Linker::new(&engine);
    preview1::add_to_linker_sync(&mut linker, |s: &mut MyState| &mut s.wasi)
        .map_err(|e| format!("WASI Linker error: {}", e))?;

    let module = Module::from_binary(&engine, WASM_BYTES)
        .map_err(|e| format!("Load Wasm error: {}", e))?;

    let sandbox_path = env.path();

    let mut builder = WasiCtxBuilder::new();
    builder.preopened_dir(sandbox_path, "/sandbox", DirPerms::all(), FilePerms::all())
        .map_err(|e| format!("Preopen error: {}", e))?;
    
    let stdin = MemoryInputPipe::new(host_json_cmd.into_bytes());
    let stdout = MemoryOutputPipe::new(10 * 1024 * 1024);
    
    builder.stdin(stdin);
    builder.stdout(stdout.clone());

    let wasi_ctx = builder.build_p1();
    let mut store = Store::new(&engine, MyState { wasi: wasi_ctx });
    
    // Set 10_000_000_000 units of fuel ~ equivalent to roughly 10s of heavy computation depending on clock.
    let _ = store.set_fuel(5_000_000_000);
    
    let instance = linker.instantiate(&mut store, &module)
        .map_err(|e| format!("Instantiate error: {}", e))?;
    
    let func = instance.get_typed_func::<(), ()>(&mut store, "_start")
        .map_err(|e| format!("Find _start error: {}", e))?;
        
    let result = func.call(&mut store, ());
    
    if let Err(e) = result {
        let _ = tx.blocking_send(format!(
            r#"{{"type": "error", "code": "WASM_TRAP", "details": "Execution crashed or ran out of fuel: {}"}}"#, e
        ));
    }

    let output_bytes = stdout.contents();
    if let Ok(contents) = String::from_utf8(output_bytes.into()) {
        for line in contents.lines() {
            if !line.trim().is_empty() {
                tx.blocking_send(line.to_string()).ok();
            }
        }
    }

    Ok(())
}
// Force host rebuild after updating Wasm payload
