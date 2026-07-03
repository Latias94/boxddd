# boxddd

[![CI](https://github.com/Latias94/boxddd/actions/workflows/ci.yml/badge.svg)](https://github.com/Latias94/boxddd/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/boxddd.svg)](https://crates.io/crates/boxddd)
[![Docs.rs](https://docs.rs/boxddd/badge.svg)](https://docs.rs/boxddd)

`boxddd` is a Rust binding workspace for [Box3D](https://github.com/erincatto/box3d), Erin Catto's 3D physics engine announced in [Announcing Box3D](https://box2d.org/posts/2026/06/announcing-box3d/). It is the 3D sibling of [`boxdd`](https://github.com/Latias94/boxdd), not a feature flag on the 2D crate.

## Crates

| Crate | Purpose |
|---|---|
| `boxddd-sys` | Low-level FFI for the vendored Box3D C API. |
| `boxddd` | Engine-agnostic safe Rust layer over worlds, bodies, shapes, joints, queries, events, debug draw, recording, and common value types. |
| `bevy_boxddd` | First-party Bevy 0.19 plugin crate with components, fixed-step systems, messages, and visible 3D examples. |

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
| `wasm32-unknown-unknown` | compile-only + provider check | Browser-oriented target. Default checks skip Box3D C; `BOXDDD_SYS_WASM_MODE=provider` checks import bindings for module `box3d-sys-v0`. |
| `wasm32-wasip1` | runtime smoke | CI builds vendored Box3D C with WASI SDK and runs `boxddd/examples/wasm_smoke.rs` under wasmtime. |

See [`docs/platforms/wasm.md`](docs/platforms/wasm.md) for the exact WASM matrix.

## CI Coverage

The GitHub Actions workflow is intentionally shaped like a native binding crate gate rather than a single workspace smoke test:

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

## Status

Experimental `0.1.0` release candidate. The safe layer currently covers:

- world/body creation, stepping, runtime tuning, counters, profiles, and id validation
- sphere, capsule, hull, mesh, height-field, and compound shape creation/resource ownership
- standalone collision helpers for mass, AABB, overlap, ray cast, shape cast, and local manifolds
- allocation-aware world/body queries with reusable-buffer and visitor forms
- body/contact/sensor/joint events plus custom filter, pre-solve, friction, and restitution callbacks
- safe Box3D task-system callback configuration through `TaskSystem::blocking_threads()`
- debug draw collection and callback adapters that catch Rust panics before the C ABI boundary
- typed joint creation/runtime APIs for Box3D's joint families
- recording/replay validation, frame stepping, query inspection, and replay debug drawing
- optional `mint`, `glam`, `nalgebra`, `cgmath`, and `serde` support for crate-owned value types

Some upstream public APIs intentionally remain raw-only or deferred. See [`docs/upstream-parity/box3d-api-matrix.md`](docs/upstream-parity/box3d-api-matrix.md).

## Quickstart

```rust
use boxddd::prelude::*;

let mut world = World::new(
    WorldDef::builder()
        .gravity(Vec3::new(0.0, -10.0, 0.0))
        .build(),
)?;

let ground = world.create_body(BodyDef::builder().position([0.0, -10.0, 0.0]).build());
world.create_hull_shape(ground, &ShapeDef::default(), &BoxHull::new(50.0, 10.0, 50.0));

let body = world.create_body(
    BodyDef::builder()
        .body_type(BodyType::Dynamic)
        .position([0.0, 4.0, 0.0])
        .build(),
);
world.create_hull_shape(
    body,
    &ShapeDef::builder().density(1.0).friction(0.3).build(),
    &BoxHull::cube(1.0),
);

world.step(1.0 / 60.0, 4);
# Ok::<(), boxddd::Error>(())
```

## Bevy Quickstart

```rust
use bevy::prelude::*;
use bevy_boxddd::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BoxdddPhysicsPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(8.0, 0.25, 8.0),
        Transform::from_xyz(0.0, -0.25, 0.0),
    ));

    commands.spawn((
        RigidBody::Dynamic,
        Collider::cube(0.5),
        Transform::from_xyz(0.0, 4.0, 0.0),
    ));
}
```

See [`bevy_boxddd/README.md`](bevy_boxddd/README.md) for Bevy components, messages, fixed-step behavior, and examples.

## Examples

Core examples are listed in [`boxddd/examples/README.md`](boxddd/examples/README.md).

```bash
cargo run -p boxddd --example hello_world
cargo run -p boxddd --example joints
cargo run -p boxddd --example recording_replay
cargo run -p boxddd --example determinism
cargo run -p boxddd --example error_handling
cargo run -p boxddd --example task_system
cargo run -p boxddd --example wasm_smoke
cargo run -p boxddd --example physics_thread
cargo run -p boxddd --example tokio_async_bridge --features tokio-example
cargo run -p boxddd --example egui_debug_draw --features egui-example
cargo run -p bevy_boxddd --example falling_stack_3d
cargo run -p bevy_boxddd --example contact_messages_3d
cargo run -p bevy_boxddd --example debug_gizmos_3d
cargo run -p bevy_boxddd --example advanced_colliders_3d
cargo run -p bevy_boxddd --example joint_gallery_3d
cargo run -p bevy_boxddd --features debug-gizmos --example debug_draw_overlay_3d
cargo run -p bevy_boxddd --features physics-picking --example physics_picking_3d
cargo run -p bevy_boxddd --features "debug-gizmos physics-picking" --example testbed_3d
```

## Features

- `double-precision`: build and bind Box3D in double-precision position mode.
- `disable-simd`: forward `BOX3D_DISABLE_SIMD` to the native build.
- `validate`: forward `BOX3D_VALIDATE` to the native build.
- `serde`: derive serialization for crate-owned value/id/query/debug/replay metadata types.
- `serialize`: alias for `serde`.
- `mint`: conversions for `Vec2`, `Vec3`, `Pos`, `Quat`, `Transform`, and `WorldTransform`.
- `glam`, `nalgebra`, `cgmath`: conversions for common 3D vector, point, quaternion, and transform representations.
- `tokio-example`: enables the Tokio async bridge example.
- `egui-example`: enables the native egui/wgpu visual debug example.

`bevy_boxddd` feature flags:

- `debug-gizmos`: enables the Bevy `Gizmos` bridge for collected Box3D debug draw commands.
- `physics-picking`: marks the native camera/cursor physics picking example. Core query helpers remain available without this feature.

## Build

Default builds use vendored Box3D C sources and pregenerated bindings, so normal builds do not require LLVM or libclang.

```bash
cargo fmt --all --check
cargo build --workspace
cargo nextest run --workspace
cargo check -p bevy_boxddd --examples
cargo check -p bevy_boxddd --features "debug-gizmos physics-picking" --example testbed_3d
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

Useful CI-equivalent binding checks:

```bash
BOXDDD_SYS_SKIP_CC=1 cargo check -p boxddd-sys
cargo check -p boxddd-sys --features double-precision
cargo nextest run -p boxddd-sys --features double-precision --test layout
BOXDDD_SYS_FORCE_BINDGEN=1 cargo check -p boxddd-sys --features bindgen
BOXDDD_SYS_FORCE_BINDGEN=1 cargo check -p boxddd-sys --features "bindgen double-precision"
```

Cross-target compile-only checks:

```bash
rustup target add armv7-unknown-linux-gnueabihf wasm32-unknown-unknown aarch64-linux-android
BOXDDD_SYS_SKIP_CC=1 cargo check -p boxddd-sys -p boxddd --target armv7-unknown-linux-gnueabihf
BOXDDD_SYS_SKIP_CC=1 cargo check -p boxddd-sys -p boxddd --target aarch64-linux-android
cargo check -p boxddd --target wasm32-unknown-unknown
BOXDDD_SYS_WASM_MODE=provider cargo check -p boxddd --target wasm32-unknown-unknown
cargo check -p bevy_boxddd --target wasm32-unknown-unknown --no-default-features
```

C-backed WASI runtime smoke:

```bash
rustup target add wasm32-wasip1
export WASI_SDK_PATH=/path/to/wasi-sdk-33.0-x86_64-linux
export WASI_SYSROOT="$WASI_SDK_PATH/share/wasi-sysroot"
export CC_wasm32_wasip1="$WASI_SDK_PATH/bin/clang"
cargo build -p boxddd --example wasm_smoke --target wasm32-wasip1
wasmtime target/wasm32-wasip1/debug/examples/wasm_smoke.wasm
```

On macOS, CI also runs:

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
BOXDDD_SYS_SKIP_CC=1 cargo check -p boxddd-sys -p boxddd --target aarch64-apple-ios
BOXDDD_SYS_SKIP_CC=1 cargo check -p boxddd-sys -p boxddd --target aarch64-apple-ios-sim
```

## Threading And Async

`World`, native resources, and replay players are intentionally `!Send`/`!Sync`. Keep physics ownership on one thread or one Bevy non-send resource.

Box3D can use its internal scheduler when only `worker_count` is configured on native targets. On WASM, `boxddd` keeps the supported contract single-threaded: `TaskSystem::blocking_threads()` is native-only and checked worker-count APIs reject unsupported values above one. When you need Rust-owned task callbacks on native targets, configure them at world creation:

```rust
let task_system = TaskSystem::blocking_threads();
let mut world = World::new(
    WorldDef::builder()
        .worker_count(2)
        .task_system(task_system.clone())
        .build(),
)?;
world.try_step(1.0 / 60.0, 4)?;
println!("{:?}", task_system.stats());
# Ok::<(), boxddd::Error>(())
```

`TaskSystem::blocking_threads()` runs Box3D tasks on blocking OS threads and joins them from Box3D's `finishTask` callback. `World::try_step` must therefore run on a thread that is allowed to block. Do not call it from a job system that cannot park or yield while waiting for child work, because Box3D requires `finishTask` to wait for completion.

For async apps, do not hold `World` across async tasks. Use `spawn_blocking`, channels, and plain snapshots such as body positions or transforms. See `physics_thread.rs` and `tokio_async_bridge.rs`.

`bevy_boxddd` stores `boxddd::World` as a Bevy `NonSend` resource and steps it from `FixedUpdate`. Bevy task-pool integration is intentionally separate from the core callback API.

## Error Handling

The terse core APIs panic on misuse such as invalid stale ids. Use the `try_*` APIs at engine/tooling boundaries when invalid input, callback-lock access, or native resource lifetime violations should be handled recoverably as `boxddd::Error`.

`bevy_boxddd` reports recoverable integration failures through Bevy messages and an optional log/panic policy.

## License

`boxddd`, `boxddd-sys`, and `bevy_boxddd` are licensed as MIT OR Apache-2.0. Vendored Box3D is MIT-licensed.
