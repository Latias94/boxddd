# WASM Support

`boxddd` now has three WASM tiers:

- `wasm32-unknown-unknown` is a browser-oriented compile/import target.
- `wasm32-wasip1` is the first C-backed runtime smoke target.
- provider mode can run as a Node smoke and as published Bevy + egui Web
  examples that share memory between a Rust wasm module and an
  Emscripten-built Box3D C provider.

The supported browser-facing runtime proof is:

- direct Bevy + egui example pages backed by the real
  `bevy_boxddd/examples/testbed_3d` wasm bundle compiled with `wasm-bindgen`.

It does not claim web workers, pthreads, Atomics, or threaded Box3D scheduling
yet.

## Support Matrix

| Surface | Target | Tier | Contract |
|---|---|---|---|
| `boxddd-sys` | `wasm32-unknown-unknown` | compile-only | Uses pregenerated bindings and skips Box3D C compilation. Do not treat the artifact as standalone runnable. |
| `boxddd-sys` | `wasm32-unknown-unknown` with `BOXDDD_SYS_WASM_MODE=provider` | provider import bindings | Generates WASM import bindings for module `box3d-sys-v0`. |
| `boxddd-sys` | `wasm32-wasip1` | C-backed runtime | Compiles vendored Box3D C sources with WASI SDK and links them into the Rust WASI module. |
| `boxddd` | `wasm32-unknown-unknown` | compile-only/provider examples | Safe APIs type-check. `xtask provider-smoke` runs a Rust wasm app against an Emscripten Box3D provider with shared memory, and Pages publishes the same provider shape as live browser examples. |
| `boxddd` | `wasm32-wasip1` | runtime smoke | `wasm_smoke` creates a world, steps a body, runs a query, and exits successfully. |
| `bevy_boxddd` minimal library | `wasm32-unknown-unknown` | compile-only | `--no-default-features` type-checks the library surface. |
| Bevy + egui example pages | browser WASM | provider-backed runtime | `xtask build-pages-wasm` builds `bevy_boxddd/examples/testbed_3d` with `wasm-bindgen` and publishes direct per-scene pages on Pages. |
| Task-system callbacks and replay worker counts | all WASM targets | single-thread only | `TaskSystem::blocking_threads()` is native-only. WASM APIs reject unsupported world and replay worker counts. |

## Compile-Only Checks

```bash
rustup target add wasm32-unknown-unknown
cargo check -p boxddd-sys --target wasm32-unknown-unknown
cargo check -p boxddd --target wasm32-unknown-unknown
cargo check -p bevy_boxddd --target wasm32-unknown-unknown --no-default-features
```

Provider import-mode check:

```bash
BOXDDD_SYS_WASM_MODE=provider cargo check -p boxddd --target wasm32-unknown-unknown
```

Provider mode does not compile Box3D C. It rewrites pregenerated bindings so
extern functions import from the stable module name `box3d-sys-v0`.

## Browser Provider Runtime

The browser runtime has two forms:

- `cargo run -p xtask -- provider-smoke` runs the shared-memory smoke under Node.
- GitHub Pages runs `cargo run -p xtask -- build-pages-wasm` and publishes
  direct Bevy + egui Web example pages.

Prerequisites:

- Rust target: `wasm32-unknown-unknown`
- Node.js
- `wasm-bindgen-cli` matching the workspace `wasm-bindgen` version
- Emscripten SDK (`emcc` on `PATH`, or `EMSDK` set to the emsdk root) for the
  full provider build

Rust-side import check:

```bash
rustup target add wasm32-unknown-unknown
cargo run -p xtask -- provider-smoke-app
```

Full provider smoke:

```bash
cargo run -p xtask -- provider-smoke
```

Pages browser artifacts:

```bash
cargo install wasm-bindgen-cli --version 0.2.126 --locked
cargo run -p xtask -- build-pages-wasm
```

`provider-smoke` builds `examples-wasm/provider-smoke` with
`BOXDDD_SYS_WASM_MODE=provider` and `--import-memory`, extracts the exact
`b3*` imports from the Rust wasm, builds `target/boxddd-provider-smoke/box3d-sys-v0.mjs`
with Emscripten, and runs `target/boxddd-provider-smoke/run-provider-smoke.mjs`
under Node. The runner instantiates both modules with the same
`WebAssembly.Memory` and calls `boxddd_provider_smoke`. The smoke proves ordinary
provider calls, the debug draw callback bridge, and the minimal world-query
bridge for AABB overlap and ray-cast visitors. Shape overlap, shape cast,
mover collision, dynamic-tree visitors, standalone mesh/height-field/compound
geometry visitors, contact/material callbacks, and task callbacks still need
their own provider bridge before they are claimed for browser provider mode.
The safe wrapper returns `Error::UnsupportedOnWasm` for those remaining
callback-heavy APIs instead of allowing a runtime table trap.

`build-pages-wasm` also builds `bevy_boxddd/examples/testbed_3d` in provider
mode, runs `wasm-bindgen`, extracts the Bevy bundle's actual Box3D imports, and
generates a small JavaScript shim that forwards those imports to the shared
Emscripten provider. The Pages Bevy entries are therefore real Bevy + egui
applications selected by URL, not JavaScript-drawn core probes. Debug draw
collection uses the provider callback bridge, and Query Lab can use the bridged
AABB overlap and ray-cast visitor paths. Other callback-heavy tools remain
blocked with `UnsupportedOnWasm` until their bridges are designed. Query Lab
surfaces those limitations in its egui diagnostics instead of treating
unsupported visitor queries as empty results.

Expected output:

```text
boxddd provider smoke passed: drop_mm=4002, ray_hit_mm=1500, shape_cast_permyriad=5013, joint_error_mm=0
```

Provider mode currently supports non-callback calls such as world/body/shape
creation, stepping, body inspection, closest-ray casts, standalone collision
helpers, distance joint solving, debug draw frame collection, world AABB overlap
visitors, and world ray-cast visitors. Shape overlap, shape cast, mover
collision planes, `DynamicTree` visitor queries/casts, standalone
mesh/height-field/compound geometry visitors, contact/material callbacks, and
Rust-owned task callbacks are blocked with `Error::UnsupportedOnWasm` until
cross-module function-table ownership is designed for each surface.

## C-Backed WASI Runtime Smoke

Prerequisites:

- Rust target: `wasm32-wasip1`
- WASI SDK 33 or newer
- A WASI runner such as `wasmtime`

Linux/macOS shell:

```bash
rustup target add wasm32-wasip1
export WASI_SDK_PATH=/path/to/wasi-sdk-33.0-x86_64-linux
export WASI_SYSROOT="$WASI_SDK_PATH/share/wasi-sysroot"
export CC_wasm32_wasip1="$WASI_SDK_PATH/bin/clang"
cargo build -p boxddd --example wasm_smoke --target wasm32-wasip1
wasmtime target/wasm32-wasip1/debug/examples/wasm_smoke.wasm
```

PowerShell:

```powershell
rustup target add wasm32-wasip1
$env:WASI_SDK_PATH = "C:\path\to\wasi-sdk-33.0-x86_64-windows"
$env:WASI_SYSROOT = "$env:WASI_SDK_PATH\share\wasi-sysroot"
$env:CC_wasm32_wasip1 = "$env:WASI_SDK_PATH\bin\clang.exe"
cargo build -p boxddd --example wasm_smoke --target wasm32-wasip1
wasmtime target/wasm32-wasip1/debug/examples/wasm_smoke.wasm
```

Expected output:

```text
boxddd wasm smoke passed: y 4.000 -> -0.003, hits 2
```

If `WASI_SYSROOT` or `WASI_SDK_PATH` is missing, `boxddd-sys` fails early with
an actionable error instead of letting clang fail later on missing libc headers.

## Current CI Gates

CI separates WASM support into visible jobs:

- `WASM compile-only matrix`: checks `boxddd-sys`, `boxddd`, minimal
  `bevy_boxddd`, and provider import-mode type checking for
  `wasm32-unknown-unknown`.
- `WASM runtime smoke (WASI)`: installs WASI SDK, builds `wasm_smoke` for
  `wasm32-wasip1`, and runs it under `wasmtime`.
- `WASM provider smoke`: installs Emscripten SDK, builds the provider-mode Rust
  smoke and Box3D C provider, and runs the shared-memory Node smoke.
- `Pages`: installs Emscripten SDK and `wasm-bindgen-cli`, then builds direct
  Bevy + egui Web example pages for GitHub Pages.

## Remaining Browser Work

The browser route follows the same shape as `dear-imgui-rs`: a Rust app WASM
module imports C symbols from a provider module, and both modules share the same
`WebAssembly.Memory`. The current browser surfaces prove the memory/import part
of this runtime contract through direct Bevy + egui Web example pages.

Callback-heavy APIs need an explicit provider bridge instead of passing Rust
closure pointers directly into another wasm module. The first bridge is debug
draw collection; the remaining design direction is documented in
[`wasm-callbacks.md`](wasm-callbacks.md): continue with query visitor
trampolines, keep typed `Error::UnsupportedOnWasm` for unimplemented surfaces,
and leave task-system worker callbacks until the blocking `finishTask` and
browser worker policy are fully specified.
