# boxddd

Safe, ergonomic Rust bindings for Box3D.

`boxddd` is a sibling of `boxdd`, not a 3D feature flag on the 2D crate. Box3D has its own Rust-facing model: 3D vectors, quaternions, hulls, meshes, height fields, compounds, typed joints, allocation-aware queries, debug draw, and deterministic recording/replay.

## Crates

- `boxddd-sys`: low-level FFI for the vendored Box3D C API.
- `boxddd`: safe Rust layer over worlds, bodies, shapes, joints, queries, events, debug draw, recording, and common value types.

## Status

Experimental `0.1.0` release candidate. The safe layer covers the current staged roadmap slice:

- world/body creation, stepping, runtime tuning, counters, profiles, and id validation
- sphere, capsule, hull, mesh, height-field, and compound shape creation/resource ownership
- standalone collision helpers for mass, AABB, overlap, ray cast, shape cast, and local manifolds
- allocation-aware world/body queries with reusable-buffer and visitor forms
- body/contact/sensor/joint events plus custom filter, pre-solve, friction, and restitution callbacks
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

## Features

- `double-precision`: build and bind Box3D in double-precision position mode.
- `disable-simd`: forward `BOX3D_DISABLE_SIMD` to the native build.
- `validate`: forward `BOX3D_VALIDATE` to the native build.
- `serde`: derive serialization for crate-owned value/id/query/debug/replay metadata types. Native resources, event snapshots, and pointer-bearing config wrappers are not serialized implicitly.
- `serialize`: alias for `serde`.
- `mint`: conversions for `Vec2`, `Vec3`, `Pos`, `Quat`, `Transform`, and `WorldTransform`.
- `glam`, `nalgebra`, `cgmath`: conversions for common 3D vector, point, quaternion, and transform representations.

## Build

Default builds use vendored Box3D C sources and pregenerated bindings, so normal builds do not require LLVM or libclang.

```bash
cargo build --workspace
cargo nextest run --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

To refresh bindings for review:

```bash
BOXDDD_SYS_FORCE_BINDGEN=1 cargo check -p boxddd-sys --features bindgen
BOXDDD_SYS_FORCE_BINDGEN=1 cargo check -p boxddd-sys --features "bindgen double-precision"
```

## Examples

The example catalog is grouped in [`boxddd/examples/README.md`](boxddd/examples/README.md).

```bash
cargo run -p boxddd --example hello_world
cargo run -p boxddd --example joints
cargo run -p boxddd --example recording_replay
cargo run -p boxddd --example determinism
cargo run -p boxddd --example mint_interop --features mint
```

## Threading

`World`, native resources, and replay players are intentionally `!Send`/`!Sync`. Keep physics ownership on one thread/task. `WorldDef::builder().worker_count(n)` stores the desired Box3D worker count, but a fully safe task-system callback API is deferred until the callback/threading contract is designed separately.

## Error Handling

The terse APIs panic on misuse such as invalid stale ids. Use the `try_*` APIs at engine/tooling boundaries when invalid input, callback-lock access, or native resource lifetime violations should be handled recoverably as `boxddd::Error`.

## License

`boxddd` and `boxddd-sys` are licensed as MIT OR Apache-2.0. Vendored Box3D is MIT-licensed.
