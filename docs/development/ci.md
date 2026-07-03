# CI And Maintainer Checks

This document keeps the detailed build matrix out of the top-level README while
preserving the commands maintainers need when changing the binding, CI, platform
support, or vendored Box3D build.

## Target Support

| Target | CI coverage | Notes |
|---|---|---|
| `x86_64` Windows MSVC | tests | Primary Windows target. Vendored Box3D C sources are compiled by `boxddd-sys`. |
| `x86_64` Linux GNU | tests | Primary Linux target. CI installs the native windowing/audio packages needed by Bevy examples. |
| `aarch64` macOS | tests | Primary Apple desktop target on GitHub-hosted macOS runners. |
| `x86_64-pc-windows-gnu` | link check | CI builds `boxddd-sys` tests with MSYS2/MinGW to catch GNU linker regressions. |
| `armv7-unknown-linux-gnueabihf` | compile-only | FFI signedness sentinel for pregenerated bindings. Native C linking is skipped. |
| `aarch64-apple-ios` | compile-only | Rust wrapper and pregenerated bindings are type-checked. Native C linking is skipped. |
| `aarch64-apple-ios-sim` | compile-only | Simulator compile sentinel. Native C linking is skipped. |
| `aarch64-linux-android` | compile-only | Android compile sentinel. Native C linking is skipped. |
| `wasm32-unknown-unknown` | compile-only + provider smoke | Browser-oriented target. Default checks skip Box3D C; provider mode imports Box3D from module `box3d-sys-v0` and CI runs a headless shared-memory smoke. |
| `wasm32-wasip1` | runtime smoke | CI builds vendored Box3D C with WASI SDK and runs `boxddd/examples/wasm_smoke.rs` under wasmtime. |

See [`../platforms/wasm.md`](../platforms/wasm.md) for the exact WASM matrix.

## CI Coverage

The GitHub Actions workflow is shaped like a native binding crate gate rather
than a single workspace smoke test:

- format check on stable Rust
- native `cargo nextest run --workspace` on Windows, Linux, and macOS
- double-precision `boxddd-sys` ABI checks and layout tests
- Bevy example compile checks, including debug draw, picking, and the 3D testbed
- docs.rs paths for `boxddd-sys`, `boxddd`, and `bevy_boxddd`
- no-default-feature and optional interop feature checks
- package checks for all publishable crates
- forced bindgen refresh checks for single and double precision
- default `boxddd-sys` dependency checks proving `bindgen` and `clang-sys` are not required for normal users
- Windows GNU, armv7, mobile, and WASM compile/link sentinels
- C-backed `wasm32-wasip1` runtime smoke with WASI SDK and wasmtime
- browser-style provider smoke with Emscripten, shared `WebAssembly.Memory`, and Node

## Local Workspace Checks

```bash
cargo fmt --all --check
cargo build --workspace
cargo nextest run --workspace
cargo check --workspace --all-features
cargo check -p bevy_boxddd --examples
cargo check -p bevy_boxddd --features "debug-gizmos physics-picking" --example testbed_3d
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

## Binding Checks

Default builds use vendored Box3D C sources and pregenerated bindings, so normal
builds do not require LLVM or libclang.

Useful CI-equivalent binding checks:

```bash
BOXDDD_SYS_SKIP_CC=1 cargo check -p boxddd-sys
cargo check -p boxddd-sys --features double-precision
cargo nextest run -p boxddd-sys --features double-precision --test layout
BOXDDD_SYS_FORCE_BINDGEN=1 cargo check -p boxddd-sys --features bindgen
BOXDDD_SYS_FORCE_BINDGEN=1 cargo check -p boxddd-sys --features "bindgen double-precision"
```

When checking release packaging, publish or stage the dependency chain in order.
`cargo package` prepares publishable manifests and resolves workspace
dependencies through the registry, so `boxddd` expects `boxddd-sys` to already
exist there, and `bevy_boxddd` expects `boxddd` to exist there.

```bash
cargo package -p boxddd-sys --allow-dirty

# After boxddd-sys is published or available in a local registry:
cargo package -p boxddd --allow-dirty

# After boxddd is published or available in a local registry:
cargo package -p bevy_boxddd --allow-dirty
```

## Cross-Target Compile-Only Checks

```bash
rustup target add armv7-unknown-linux-gnueabihf wasm32-unknown-unknown aarch64-linux-android
BOXDDD_SYS_SKIP_CC=1 cargo check -p boxddd-sys -p boxddd --target armv7-unknown-linux-gnueabihf
BOXDDD_SYS_SKIP_CC=1 cargo check -p boxddd-sys -p boxddd --target aarch64-linux-android
cargo check -p boxddd --target wasm32-unknown-unknown
BOXDDD_SYS_WASM_MODE=provider cargo check -p boxddd --target wasm32-unknown-unknown
cargo check -p bevy_boxddd --target wasm32-unknown-unknown --no-default-features
```

On macOS, CI also runs:

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
BOXDDD_SYS_SKIP_CC=1 cargo check -p boxddd-sys -p boxddd --target aarch64-apple-ios
BOXDDD_SYS_SKIP_CC=1 cargo check -p boxddd-sys -p boxddd --target aarch64-apple-ios-sim
```

## WASM Runtime Smokes

C-backed WASI runtime smoke:

```bash
rustup target add wasm32-wasip1
export WASI_SDK_PATH=/path/to/wasi-sdk-33.0-x86_64-linux
export WASI_SYSROOT="$WASI_SDK_PATH/share/wasi-sysroot"
export CC_wasm32_wasip1="$WASI_SDK_PATH/bin/clang"
cargo build -p boxddd --example wasm_smoke --target wasm32-wasip1
wasmtime target/wasm32-wasip1/debug/examples/wasm_smoke.wasm
```

Browser-style provider smoke:

```bash
rustup target add wasm32-unknown-unknown
cargo run -p xtask -- provider-smoke-app

# Full provider smoke also requires Emscripten SDK on PATH or EMSDK set.
cargo run -p xtask -- provider-smoke
```
