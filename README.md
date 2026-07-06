# boxddd

[![CI](https://github.com/Latias94/boxddd/actions/workflows/ci.yml/badge.svg)](https://github.com/Latias94/boxddd/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/boxddd.svg)](https://crates.io/crates/boxddd)
[![Docs.rs](https://docs.rs/boxddd/badge.svg)](https://docs.rs/boxddd)
[![Demo Hub](https://img.shields.io/badge/demo-hub-2ea44f.svg)](https://frankorz.com/boxddd/)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](#license)

`boxddd` is a Rust binding workspace for [Box3D](https://github.com/erincatto/box3d), Erin Catto's 3D physics engine announced in [Announcing Box3D](https://box2d.org/posts/2026/06/announcing-box3d/).

It is the 3D sibling of [`boxdd`](https://github.com/Latias94/boxdd), not a feature flag on the 2D crate.

## What Is boxddd?

`boxddd` gives Rust projects a safe, engine-agnostic layer over Box3D's C API:

- worlds, bodies, shapes, joints, queries, events, debug draw, and recording/replay
- owned Rust value types for vectors, transforms, ids, filters, and debug data
- recoverable `try_*` APIs for tools and engines that cannot panic on invalid input
- optional conversions for `mint`, `glam`, `nalgebra`, and `cgmath`
- a first-party `bevy_boxddd` plugin for Bevy 0.19 apps and teaching examples

## Status

`boxddd` is an experimental `0.x` binding while Box3D itself is new. The latest release is `0.2.0`, which adds the Bevy example hub, browser provider-mode demo support, debug draw frame assets, and release preflight hardening. The native desktop path is the main supported runtime surface today. The safe API covers the primary simulation path and tracks the remaining public Box3D surface in a tested coverage inventory; see [`docs/api-coverage.md`](https://github.com/Latias94/boxddd/blob/main/docs/api-coverage.md).

| Surface | Status |
|---|---|
| Core `boxddd` on Windows, Linux, macOS | Supported and tested |
| `bevy_boxddd` on Windows, Linux, macOS | Supported for native Bevy apps and examples |
| WASM | Experimental provider-mode support; the demo hub exposes direct Bevy + egui Web examples, while native desktop remains the supported runtime commitment |
| Mobile | Not a supported runtime target yet |

The core crate MSRV is Rust `1.92`. `bevy_boxddd` currently requires Rust `1.95` because it tracks Bevy 0.19.

## Version Compatibility

| `boxddd` release | Box3D API target | Vendored Box3D source | Notes |
|---|---|---|---|
| `0.2.0` | [`box3d` `v0.1.0`](https://github.com/erincatto/box3d/tree/v0.1.0) | [`erincatto/box3d@29bf523`](https://github.com/erincatto/box3d/commit/29bf523ce7bc4590aba9f17c9db791cdc5c4397e) | Adds the Bevy example hub, browser provider-mode demos, debug draw frame assets, and release preflight hardening. |
| `0.1.0` | [`box3d` `v0.1.0`](https://github.com/erincatto/box3d/tree/v0.1.0) | [`erincatto/box3d@29bf523`](https://github.com/erincatto/box3d/commit/29bf523ce7bc4590aba9f17c9db791cdc5c4397e) |

The vendored source includes a local single-thread WASM timer portability patch. Native desktop remains the supported runtime path for published releases today.

## Migrating From 0.1.0 To 0.2.0

`0.2.0` has a few intentional breaks, tracked in [`CHANGELOG.md`](https://github.com/Latias94/boxddd/blob/main/CHANGELOG.md): debug draw renderers should move from `DebugDrawCommand::Shape { shape, .. }` to the `DebugDrawFrame` asset/handle model, exhaustive `boxddd::Error` matches need the provider-callback arm, and exhaustive `bevy_boxddd::HullDescriptor` matches need a wildcard arm.

The GitHub Pages examples are real Bevy + egui WASM scenes built through `cargo run -p xtask -- build-pages-wasm`; Pages builds default to the size-focused `wasm-release` profile, use `wasm-opt -Oz` when available, and show download progress while loading the provider and Bevy wasm assets.

## Crates

| Crate | Purpose |
|---|---|
| [`boxddd-sys`](https://github.com/Latias94/boxddd/blob/main/boxddd-sys/README.md) | Low-level FFI for the vendored Box3D C API. |
| [`boxddd`](https://github.com/Latias94/boxddd/tree/main/boxddd) | Safe Rust wrapper for engine-independent physics code. |
| [`bevy_boxddd`](https://github.com/Latias94/boxddd/blob/main/bevy_boxddd/README.md) | Bevy 0.19 plugin with ECS components, fixed-step systems, messages, queries, debug drawing, and windowed 3D examples. |

## Getting Started

Add the core crate when you want to own the physics loop yourself.

```bash
cargo add boxddd
```

```rust
use boxddd::prelude::*;

fn main() -> boxddd::Result<()> {
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
    Ok(())
}
```

Run the smallest core example:

```bash
cargo run -p boxddd --example hello_world
```

## Bevy

Use `bevy_boxddd` when you want Bevy entities and transforms to author the physics scene:

```bash
cargo add bevy_boxddd
cargo add bevy@0.19
```

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

Good first Bevy examples:

```bash
cargo run -p bevy_boxddd --example falling_stack_3d
cargo run -p bevy_boxddd --example advanced_colliders_3d
cargo run -p bevy_boxddd --example joint_gallery_3d
cargo run -p bevy_boxddd --features "debug-gizmos physics-picking" --example testbed_3d
```

See [`bevy_boxddd/README.md`](https://github.com/Latias94/boxddd/blob/main/bevy_boxddd/README.md) for components, messages, fixed-step behavior, debug draw, picking, platform notes, and the full example catalog.
The demo hub at <https://frankorz.com/boxddd/> is the maintained example gallery. Browser entries link directly to individual Bevy + egui Web scenes such as Falling Stack, Query Lab, Debug Draw Inspector, and Stats Dashboard.

## Examples

Start here:

| Example | Command | Teaches |
|---|---|---|
| Core hello world | `cargo run -p boxddd --example hello_world` | World creation, ground, dynamic body, stepping |
| Error handling | `cargo run -p boxddd --example error_handling` | Recoverable `try_*` APIs |
| Events | `cargo run -p boxddd --example events` | Sensor, contact, hit, and body-move event snapshots |
| Body controls | `cargo run -p boxddd --example body_controls` | Body type changes, enable/disable, forces, impulses, and locks |
| Stats/profile | `cargo run -p boxddd --example stats_profile` | World counters, awake-body counts, capacity, and per-step profile timings |
| Shape queries | `cargo run -p boxddd --example shape_queries` | World, body, and shape-scoped query APIs |
| Advanced collision | `cargo run -p boxddd --example advanced_collision` | Standalone distance, shape-cast, manifold, and plane helpers |
| Continuous collision | `cargo run -p boxddd --example continuous_collision` | Shape-cast, time-of-impact, and bullet body diagnostics |
| Character mover | `cargo run -p boxddd --example character_mover` | Capsule mover casts, contact planes, correction, and velocity clipping |
| Dynamic tree | `cargo run -p boxddd --example dynamic_tree` | Standalone broad-phase tree lifecycle, filters, and visitor callbacks |
| Joints | `cargo run -p boxddd --example joints` | Joint creation and runtime reads |
| Recording | `cargo run -p boxddd --example recording_replay` | Recording and replay validation |
| Native debug viewer | `cargo run -p boxddd --example egui_debug_draw --features egui-example` | Consuming debug draw data in an app-owned renderer |
| Bevy testbed | `cargo run -p bevy_boxddd --features "debug-gizmos physics-picking" --example testbed_3d` | Egui-driven scene browser for official Box3D teaching scenes plus Query Lab, Debug Draw Inspector, Material Lab, and Stats Dashboard showcases |

The full catalog lives in [`boxddd/examples/README.md`](https://github.com/Latias94/boxddd/blob/main/boxddd/examples/README.md) and [`bevy_boxddd/examples/README.md`](https://github.com/Latias94/boxddd/blob/main/bevy_boxddd/examples/README.md). The case-level official Box3D sample support matrix lives in [`docs/upstream-parity/box3d-sample-matrix.md`](https://github.com/Latias94/boxddd/blob/main/docs/upstream-parity/box3d-sample-matrix.md); it labels each upstream case as a faithful port, teaching adaptation, test-only proof, deferred case, or upstream reference.

## Design Goals

- Keep `boxddd` engine-agnostic. Bevy support lives in `bevy_boxddd`.
- Prefer safe, typed Rust APIs over exposing raw Box3D handles directly.
- Make invalid ids, stale resources, callback reentry, and unsupported platforms explicit.
- Keep the default build simple: vendored Box3D C sources plus pregenerated bindings, with no normal-user dependency on LLVM or libclang.
- Treat examples as teaching material, not just compile smoke tests.

## Platform Notes

Native Windows, Linux, and macOS are the primary runtime targets. WASM support is early and provider-backed: the demo hub can publish direct Bevy + egui Web example pages through the shared `bevy_boxddd/examples/testbed_3d` wasm bundle. Web workers, pthreads, most callback-heavy Box3D APIs beyond debug draw plus AABB/ray world-query visitors, and threaded scheduling are still deferred.

Normal builds compile the vendored Box3D C sources locally through the Rust `cc` crate and link them into `boxddd-sys`. Users need a working platform C compiler, but they do not need CMake, LLVM, libclang, or bindgen unless they explicitly refresh bindings with `boxddd-sys/bindgen` and `BOXDDD_SYS_FORCE_BINDGEN=1`.

Read more:

- [`docs/platforms/wasm.md`](https://github.com/Latias94/boxddd/blob/main/docs/platforms/wasm.md) for the detailed WASM support matrix
- [`docs/development/ci.md`](https://github.com/Latias94/boxddd/blob/main/docs/development/ci.md) for CI jobs, target checks, and maintainer build commands
- [`docs/upstream-parity/box3d-api-matrix.md`](https://github.com/Latias94/boxddd/blob/main/docs/upstream-parity/box3d-api-matrix.md) for upstream Box3D API parity

## Features

Core feature flags:

- `double-precision`: build and bind Box3D in double-precision position mode.
- `disable-simd`, `validate`: forward the matching Box3D C build options.
- `serde`: serialize crate-owned value/id/query/debug/replay metadata types.
- `mint`, `glam`, `nalgebra`, `cgmath`: enable conversions for common math crates.
- `egui-example`, `tokio-example`: opt into heavier example-only dependencies.

Math interop is feature-gated and covered by runnable examples: `mint_interop`, `glam_interop`, and `nalgebra_interop`.

`bevy_boxddd` feature flags:

- `debug-gizmos`: render collected Box3D debug draw commands through Bevy `Gizmos`.
- `physics-picking`: enable the physics picking example surface.

## Threading, Async, And Errors

`World`, native resources, and replay players are intentionally `!Send`/`!Sync`. Keep physics ownership on one thread or in Bevy's `NonSend<BoxdddPhysicsContext>`, then move plain snapshots across thread or async boundaries.

For async apps, use `spawn_blocking`, channels, and snapshots. See `physics_thread.rs` and `tokio_async_bridge.rs`.

The terse APIs panic on programming misuse such as invalid stale ids. Use `try_*` APIs at engine, editor, scripting, and tooling boundaries where invalid input should become `boxddd::Error`. Unsafe native interop such as raw `void*` user data and process-global scalar tuning lives under `boxddd::raw`, outside the prelude.

## Development

Common local checks:

```bash
cargo fmt --all --check
cargo nextest run --workspace
cargo check --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

More maintainer commands are documented in [`docs/development/ci.md`](docs/development/ci.md).

## License

`boxddd`, `boxddd-sys`, and `bevy_boxddd` are licensed as MIT OR Apache-2.0. Vendored Box3D is MIT-licensed.
