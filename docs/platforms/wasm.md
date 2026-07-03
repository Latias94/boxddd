# WASM Support

`boxddd` now has two WASM tiers:

- `wasm32-unknown-unknown` is a browser-oriented compile/import target.
- `wasm32-wasip1` is the first C-backed runtime smoke target.

The supported runtime tier is intentionally core-only. It proves Box3D C code,
world creation, shape creation, stepping, queries, and teardown in WebAssembly.
It does not claim Bevy Web, browser rendering, web workers, pthreads, Atomics, or
threaded Box3D scheduling yet.

## Support Matrix

| Surface | Target | Tier | Contract |
|---|---|---|---|
| `boxddd-sys` | `wasm32-unknown-unknown` | compile-only | Uses pregenerated bindings and skips Box3D C compilation. Do not treat the artifact as standalone runnable. |
| `boxddd-sys` | `wasm32-unknown-unknown` with `BOXDDD_SYS_WASM_MODE=provider` | provider scaffold | Generates WASM import bindings for module `box3d-sys-v0`. A separate provider module must export the Box3D C symbols and shared memory. |
| `boxddd-sys` | `wasm32-wasip1` | C-backed runtime | Compiles vendored Box3D C sources with WASI SDK and links them into the Rust WASI module. |
| `boxddd` | `wasm32-unknown-unknown` | compile-only/provider | Safe APIs type-check. Provider mode is for browser packaging experiments. |
| `boxddd` | `wasm32-wasip1` | runtime smoke | `wasm_smoke` creates a world, steps a body, runs a query, and exits successfully. |
| `bevy_boxddd` minimal library | `wasm32-unknown-unknown` | compile-only | `--no-default-features` type-checks the library surface. |
| Bevy examples and renderer integrations | browser WASM | deferred | Native examples use windowing/rendering assumptions. Bevy Web needs a separate renderer/input/testbed plan. |
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

## Deferred Browser Work

The browser route follows the same shape as `dear-imgui-rs`: a Rust app WASM
module imports C symbols from a provider module, and both modules share the same
`WebAssembly.Memory`. The current scaffold establishes the import module name
and binding generation path. A future browser plan should add provider artifact
generation, shared-memory patching, a headless JS smoke, and then Bevy Web or
other visual examples.
