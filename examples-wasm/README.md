# WASM Examples

This directory documents browser-oriented WASM work that is not part of the
native example catalog.

## Current Runtime Proof

The runnable WASM proof lives in the core crate because it has no browser UI:

```bash
rustup target add wasm32-wasip1
export WASI_SDK_PATH=/path/to/wasi-sdk-33.0-x86_64-linux
export WASI_SYSROOT="$WASI_SDK_PATH/share/wasi-sysroot"
export CC_wasm32_wasip1="$WASI_SDK_PATH/bin/clang"
cargo build -p boxddd --example wasm_smoke --target wasm32-wasip1
wasmtime target/wasm32-wasip1/debug/examples/wasm_smoke.wasm
```

Expected output:

```text
boxddd wasm smoke passed: y 4.000 -> -0.003, hits 2
```

## Browser Provider Scaffold

Browser-style `wasm32-unknown-unknown` builds use provider mode:

```bash
BOXDDD_SYS_WASM_MODE=provider cargo check -p boxddd --target wasm32-unknown-unknown
```

Provider mode generates bindings whose C imports come from module
`box3d-sys-v0`. A future browser example should add:

- a Box3D C provider WASM module exporting the imported `b3*` symbols
- shared `WebAssembly.Memory` wiring between the Rust app module and provider
- a headless JS smoke that calls the same world/body/shape/step assertion
- a visual Bevy Web or renderer-specific example after the runtime smoke is
  stable

This mirrors the `dear-imgui-rs` provider approach without claiming browser
runtime support before the provider module exists.
