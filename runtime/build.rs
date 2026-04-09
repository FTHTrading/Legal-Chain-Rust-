/// Build script for the LEGAL-CHAIN runtime.
/// Uses substrate-wasm-builder to compile the runtime to WASM.

fn main() {
    substrate_wasm_builder::WasmBuilder::build_using_defaults();
}
