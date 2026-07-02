<div align="center">

# boxddd-sys - Low-level FFI for Box3D

</div>

`boxddd-sys` builds the vendored Box3D C sources from `third-party/box3d` and exposes the raw C API in `boxddd_sys::ffi`.
High-level, safe Rust wrappers live in the companion `boxddd` crate.

## Build

- Default: builds vendored Box3D C sources with `cc`.
- Bindings: uses checked-in pregenerated bindings so normal builds do not require LLVM or libclang.
- Refresh: enable the `bindgen` feature and set `BOXDDD_SYS_FORCE_BINDGEN=1`.
- Docs.rs/offline docs: uses pregenerated bindings and skips native C compilation.

## Features

- `build-from-source`: compile vendored Box3D C sources. Enabled by default.
- `bindgen`: enable bindgen-based binding refresh.
- `double-precision`: build Box3D with `BOX3D_DOUBLE_PRECISION` and use matching pregenerated bindings.
- `disable-simd`: define `BOX3D_DISABLE_SIMD`.
- `validate`: define `BOX3D_VALIDATE`.

## Environment

- `BOXDDD_SYS_FORCE_BINDGEN=1`: regenerate bindings into Cargo's `OUT_DIR`; requires `--features bindgen`.
- `BOXDDD_SYS_SKIP_CC=1`: skip native C compilation for check-only workflows.
- `DOCS_RS=1` or `--cfg docsrs`: skip native C compilation for documentation.

## Notes

The pregenerated bindings are ABI-mode specific. If `double-precision` is enabled, the crate uses `bindings_pregenerated_double.rs`; otherwise it uses `bindings_pregenerated.rs`.

## License

This crate is licensed as MIT OR Apache-2.0. Vendored Box3D is MIT-licensed.
