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

## Browser Provider Smoke

Browser-style `wasm32-unknown-unknown` builds use provider mode and a separate
Box3D C provider module:

```bash
BOXDDD_SYS_WASM_MODE=provider cargo check -p boxddd --target wasm32-unknown-unknown
```

The provider smoke verifies the same shared-memory import shape used by browser
apps. It can run headlessly under Node or be packaged into the GitHub Pages demo:

```bash
rustup target add wasm32-unknown-unknown
cargo run -p xtask -- provider-smoke-app

# Requires Emscripten SDK (`emcc`) on PATH or EMSDK set.
cargo run -p xtask -- provider-smoke
cargo run -p xtask -- build-pages-wasm
```

Expected output:

```text
boxddd provider smoke passed: drop_mm=4002, ray_hit_mm=1500, shape_cast_permyriad=5013, joint_error_mm=0
```

`provider-smoke-app` builds the Rust wasm module and records the exact `b3*`
imports it expects from `box3d-sys-v0`. `provider-smoke` additionally builds the
Emscripten provider and runs Node with a shared `WebAssembly.Memory`. This smoke
checks non-callback APIs and asserts that callback-heavy APIs return
`Error::UnsupportedOnWasm` instead of trapping across wasm module tables.

The Pages examples are intentionally core-only: they run falling-body, closest-ray,
shape-cast, and distance-joint probes and display the result on a canvas. Bevy Web
and renderer-specific examples are still deferred until callback/table ownership
and browser packaging mature.
