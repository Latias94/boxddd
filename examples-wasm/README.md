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

## Browser Provider Runtime

Browser-style `wasm32-unknown-unknown` builds use provider mode and a separate
Box3D C provider module:

```bash
BOXDDD_SYS_WASM_MODE=provider cargo check -p boxddd --target wasm32-unknown-unknown
```

The provider smoke verifies the same shared-memory import shape used by browser
apps. It can run headlessly under Node or be packaged into the GitHub Pages demo
alongside the real Bevy + egui Web testbed:

```bash
rustup target add wasm32-unknown-unknown
cargo run -p xtask -- provider-smoke-app

# Requires Emscripten SDK (`emcc`) on PATH or EMSDK set.
# Requires wasm-bindgen-cli for the Bevy Web testbed in build-pages-wasm.
# Uses wasm-opt automatically from PATH or EMSDK/upstream/bin when available.
cargo install wasm-bindgen-cli --version 0.2.126 --locked
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
checks non-callback APIs, proves provider-backed debug draw frame collection, and
asserts that the remaining callback-heavy APIs return `Error::UnsupportedOnWasm`
instead of trapping across wasm module tables.

`build-pages-wasm` publishes two browser surfaces:

- a real `bevy_boxddd/examples/testbed_3d` Bevy + egui app compiled with
  `wasm-bindgen`;
- direct per-scene Pages entries that select the matching Bevy scene by URL and
  share the same Box3D provider runtime.

The Bevy loader reports byte-level download progress for the provider wasm and
the Bevy wasm before instantiating them with a shared `WebAssembly.Memory`. The
Bevy page uses the provider-backed debug draw bridge; other callback-heavy
features still keep their explicit `UnsupportedOnWasm` guardrails until each
bridge is designed and tested.
