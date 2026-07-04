# boxddd-sys

Low-level Rust FFI for the vendored [Box3D](https://github.com/erincatto/box3d) C API.

Most users should depend on [`boxddd`](https://crates.io/crates/boxddd) instead. Use `boxddd-sys` directly when you need raw C symbols, custom native linking, or binding-generation maintenance.

## Build Contract

Default builds compile the vendored Box3D C sources with the Rust `cc` crate and link the resulting static library into `boxddd-sys`.

Normal users need a platform C compiler, such as MSVC Build Tools on Windows, Clang on macOS, or GCC/Clang on Linux. Normal users do not need CMake, LLVM, libclang, or bindgen because checked-in pregenerated bindings are used by default.

## Features

- `build-from-source`: compile vendored Box3D C sources. Enabled by default.
- `bindgen`: allow regenerating bindings when `BOXDDD_SYS_FORCE_BINDGEN=1` is set.
- `double-precision`: build Box3D with `BOX3D_DOUBLE_PRECISION` and use matching pregenerated bindings.
- `disable-simd`: define `BOX3D_DISABLE_SIMD`.
- `validate`: define `BOX3D_VALIDATE`.

## Native Linking

Disable default features to skip vendored C compilation and link an external `box3d` library:

```toml
boxddd-sys = { version = "0.1", default-features = false }
```

Optional environment variables:

- `BOXDDD_SYS_LINK_LIB`: external library name. Defaults to `box3d`.
- `BOXDDD_SYS_LINK_SEARCH`: native library search directory.

## Binding Refresh

Pregenerated bindings are ABI-mode specific. The default build uses `bindings_pregenerated.rs`; `double-precision` uses `bindings_pregenerated_double.rs`.

Regenerate bindings only when maintaining this crate:

```bash
BOXDDD_SYS_FORCE_BINDGEN=1 cargo check -p boxddd-sys --features bindgen
BOXDDD_SYS_FORCE_BINDGEN=1 cargo check -p boxddd-sys --features "bindgen double-precision"
```

## WASM

WASM support is early and core-only.

| Target | Status |
|---|---|
| `wasm32-unknown-unknown` | Compile-only by default. Provider mode imports Box3D symbols from module `box3d-sys-v0`. |
| `wasm32-wasip1` | Runtime-capable source build when a WASI SDK sysroot is configured. |
| Browser visual demos | Not supported yet. |

Useful environment variables:

- `BOXDDD_SYS_WASM_MODE`: `compile-only`, `source`, or `provider`.
- `WASI_SYSROOT`: WASI libc sysroot for `wasm32-wasip1` source builds.
- `WASI_SDK_PATH`: WASI SDK root. Used as `$WASI_SDK_PATH/share/wasi-sysroot` when `WASI_SYSROOT` is unset.

Detailed WASM commands live in the workspace documentation: <https://github.com/Latias94/boxddd/blob/main/docs/platforms/wasm.md>.

## Check-Only Builds

`BOXDDD_SYS_SKIP_CC=1` skips native C compilation for check-only workflows. Do not use it for normal runnable native builds.

```bash
BOXDDD_SYS_SKIP_CC=1 cargo check -p boxddd-sys
```

## License

This crate is licensed as MIT OR Apache-2.0. Vendored Box3D is MIT-licensed.
