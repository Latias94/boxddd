<div align="center">

# boxddd-sys - Low-level FFI for Box3D

</div>

`boxddd-sys` builds the vendored Box3D C sources from `third-party/box3d` and exposes the raw C API in `boxddd_sys::ffi`.
High-level, safe Rust wrappers live in the companion `boxddd` crate.

## Build

- Default: builds vendored Box3D C sources with `cc`.
- External link: disable default features to skip vendored C compilation and link an external `box3d` library instead.
- Bindings: uses checked-in pregenerated bindings so normal builds do not require LLVM or libclang.
- Refresh: enable the `bindgen` feature and set `BOXDDD_SYS_FORCE_BINDGEN=1`.
- Docs.rs/offline docs: uses pregenerated bindings and skips native C compilation.
- WASM compile-only: `wasm32-unknown-unknown` skips C compilation by default.
- WASM runtime smoke: `wasm32-wasip1` builds vendored C sources when `WASI_SYSROOT` or `WASI_SDK_PATH` points at WASI SDK.
- WASM provider: `BOXDDD_SYS_WASM_MODE=provider` generates import bindings for module `box3d-sys-v0`; `xtask provider-smoke` builds an Emscripten provider and runs the shared-memory smoke.

## Features

- `build-from-source`: compile vendored Box3D C sources. Enabled by default.
- `bindgen`: enable bindgen-based binding refresh.
- `double-precision`: build Box3D with `BOX3D_DOUBLE_PRECISION` and use matching pregenerated bindings.
- `disable-simd`: define `BOX3D_DISABLE_SIMD`.
- `validate`: define `BOX3D_VALIDATE`.

## Environment

- `BOXDDD_SYS_FORCE_BINDGEN=1`: regenerate bindings into Cargo's `OUT_DIR`; requires `--features bindgen`.
- `BOXDDD_SYS_SKIP_CC=1`: skip native C compilation for check-only workflows.
- `BOXDDD_SYS_WASM_MODE`: override wasm mode. Accepted values are `compile-only`, `source`, and `provider`.
- `BOXDDD_SYS_SKIP_CC=1` is rejected with `BOXDDD_SYS_WASM_MODE=source` because source mode is the runtime-capable WASM path.
- `BOXDDD_SYS_LINK_LIB`: external library name used when `build-from-source` is disabled. Defaults to `box3d`.
- `BOXDDD_SYS_LINK_SEARCH`: optional native library search directory used when `build-from-source` is disabled.
- `WASI_SYSROOT`: WASI libc sysroot used by `wasm32-wasip1` source builds.
- `WASI_SDK_PATH`: WASI SDK root. If `WASI_SYSROOT` is unset, `build.rs` uses `$WASI_SDK_PATH/share/wasi-sysroot`.
- `DOCS_RS=1` or `--cfg docsrs`: skip native C compilation for documentation.

## WASM Commands

Compile-only browser target:

```bash
rustup target add wasm32-unknown-unknown
cargo check -p boxddd-sys --target wasm32-unknown-unknown
```

Provider import-mode check:

```bash
BOXDDD_SYS_WASM_MODE=provider cargo check -p boxddd-sys --target wasm32-unknown-unknown
```

Browser-style provider smoke:

```bash
rustup target add wasm32-unknown-unknown
cargo run -p xtask -- provider-smoke-app

# Full smoke requires Emscripten SDK (`emcc`) on PATH or EMSDK set.
cargo run -p xtask -- provider-smoke
```

The provider smoke verifies shared memory and imported Box3D symbols without
cross-module callback function pointers. Callback-heavy APIs need a dedicated
shared-table or dynamic-linking path before browser provider mode can claim
them.

C-backed WASI source build:

```bash
rustup target add wasm32-wasip1
export WASI_SDK_PATH=/path/to/wasi-sdk-33.0-x86_64-linux
export WASI_SYSROOT="$WASI_SDK_PATH/share/wasi-sysroot"
export CC_wasm32_wasip1="$WASI_SDK_PATH/bin/clang"
cargo build -p boxddd --example wasm_smoke --target wasm32-wasip1
```

## Notes

The pregenerated bindings are ABI-mode specific. If `double-precision` is enabled, the crate uses `bindings_pregenerated_double.rs`; otherwise it uses `bindings_pregenerated.rs`.

## License

This crate is licensed as MIT OR Apache-2.0. Vendored Box3D is MIT-licensed.
