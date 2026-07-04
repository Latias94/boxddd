# bevy_boxddd

`bevy_boxddd` integrates [`boxddd`](https://github.com/Latias94/boxddd) with Bevy 0.19. It keeps the core `boxddd` crate engine-agnostic while providing ECS authoring, fixed-step systems, messages, queries, debug drawing, and teaching examples for Bevy users.

## Quick Facts

| Item | Value |
|---|---|
| Bevy version | `0.19.0` |
| Rust version | `1.95.0` or newer |
| Physics owner | `NonSend<BoxdddPhysicsContext>` |
| Schedule | `FixedUpdate` |
| Default rendering deps | None |
| Debug rendering | Optional `debug-gizmos` feature |
| Picking example | Optional `physics-picking` feature |
| Testbed UI | `bevy_egui` in the `testbed_3d` example only |
| Threading | Keep `boxddd::World` on the Bevy main thread; move snapshots across threads |

## Quickstart

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
    let body = commands
        .spawn((
            RigidBody::Dynamic,
            Transform::from_xyz(0.0, 4.0, 0.0),
        ))
        .id();

    commands.spawn((Collider::sphere(0.35), ChildOf(body)));
    commands.spawn((Collider::cube(0.4), ChildOf(body)));
}
```

Child collider entities are plugin-owned shapes attached to the nearest parent entity with `BoxdddBody`. A body entity can also carry its own `Collider`.

## Components

- `RigidBody`: `Static`, `Kinematic`, or `Dynamic`.
- `BodySettings`: gravity scale, damping, sleeping, bullet CCD, and motion locks applied to the native body.
- `Collider`: basic `cuboid`, `cube`, `sphere`, and `capsule_y` descriptors plus advanced mesh, height-field, compound, created-hull, and transformed-hull descriptors.
- `PhysicsMaterial`: density, friction, restitution, sensor/contact flags, and Box3D filter data.
- `JointTarget` + `Joint`: declarative joint authoring between two Bevy body entities.
- `TransformSyncMode`: dynamic bodies default to physics-authored transforms; static and kinematic bodies default to Bevy-authored transforms.
- `LinearVelocity`, `AngularVelocity`, `ExternalForce`, `ExternalImpulse`: control inputs applied before stepping.
- `BoxdddBody`, `BoxdddShape`, `BoxdddJoint`: native id components inserted by the plugin after successful creation.

Advanced collider descriptors that create Box3D native resource shapes are static-body only. Attaching them to dynamic bodies emits `BoxdddErrorMessage { operation: CreateShape, error: InvalidArgument, .. }`.

## Queries And Picking

Default library helpers are renderer-free:

```rust
let context = world.get_non_send::<BoxdddPhysicsContext>().unwrap();
let hit = cast_ray_closest(
    context,
    Vec3::new(-2.0, 0.0, 0.0),
    Vec3::new(5.0, 0.0, 0.0),
    boxddd::QueryFilter::default(),
)?;
```

Hits include both the Box3D `ShapeId` and the mapped Bevy entity when the shape is plugin-owned.

## Debug Draw

The plugin always exposes `BoxdddDebugDrawSettings` and `BoxdddDebugDrawCommands`. By default collection is disabled. Enable collection without rendering dependencies:

```rust
commands.insert_resource(BoxdddDebugDrawSettings {
    enabled: true,
    options: boxddd::DebugDrawOptions::default(),
});
```

Enable `debug-gizmos` to render collected commands through Bevy `Gizmos`:

```bash
cargo run -p bevy_boxddd --features debug-gizmos --example debug_draw_overlay_3d
```

## Messages And Errors

The plugin registers Bevy messages for errors, body moves, contact begin/end/hit, and sensor begin/end. Recoverable errors follow `BoxdddPhysicsSettings.error_policy`: message only, message plus log, or panic.

## Examples

The visible examples complement the core headless examples and follow the official Box3D sample parity map in [`docs/upstream-parity/box3d-sample-matrix.md`](https://github.com/Latias94/boxddd/blob/main/docs/upstream-parity/box3d-sample-matrix.md). The complete catalog lives in [`examples/README.md`](examples/README.md).

| Example | Run command | Purpose |
|---|---|---|
| `falling_stack_3d` | `cargo run -p bevy_boxddd --example falling_stack_3d` | Basic windowed stack with plugin-driven transforms. |
| `contact_messages_3d` | `cargo run -p bevy_boxddd --example contact_messages_3d` | Reads contact messages and updates Bevy materials. |
| `debug_gizmos_3d` | `cargo run -p bevy_boxddd --example debug_gizmos_3d` | App-authored collider gizmos without Box3D debug draw collection. |
| `advanced_colliders_3d` | `cargo run -p bevy_boxddd --example advanced_colliders_3d` | Static mesh, height-field, compound, hull-backed colliders, and dynamic bodies falling onto them. |
| `joint_gallery_3d` | `cargo run -p bevy_boxddd --example joint_gallery_3d` | Visible connected bodies using every public declarative joint variant. |
| `debug_draw_overlay_3d` | `cargo run -p bevy_boxddd --features debug-gizmos --example debug_draw_overlay_3d` | Box3D debug draw commands rendered through Bevy `Gizmos`. |
| `physics_picking_3d` | `cargo run -p bevy_boxddd --features physics-picking --example physics_picking_3d` | Camera/cursor ray mapped through Box3D queries, not mesh picking. |
| `testbed_3d` | `cargo run -p bevy_boxddd --features "debug-gizmos physics-picking" --example testbed_3d` | Egui-driven scene browser for stacks, advanced colliders, body controls, continuous collision, character mover probes, materials, joints, contacts, picking, debug draw, dominoes, arch stacks, wind forces, and a ragdoll-lite chain. |

The testbed is the primary teaching surface. Use the left panel to switch scenes and adjust pause, single-step, gravity, solver rate, substeps, sleeping, warm starting, continuous collision, and debug draw presets. The static demo hub at <https://latias94.github.io/boxddd/> mirrors the same scene registry.

## Platform And Concurrency Boundaries

Native desktop targets are the supported runtime target for the Bevy plugin. CI checks Windows, Linux, and macOS native builds through the workspace test suite, and checks the Bevy examples on Linux with the required windowing/audio packages installed.

`wasm32-unknown-unknown` is compile-only for the minimal library surface; windowed examples are native-only in this release. Mobile targets are also compile-only at the core `boxddd` layer and are not yet runtime targets for `bevy_boxddd`.

`boxddd::World` is intentionally non-send and lives in `BoxdddPhysicsContext`. Do not move it into Bevy worker systems. The core crate has `TaskSystem::blocking_threads()` for Box3D's native task callbacks, but Bevy task-pool integration is deferred until that contract has more usage.

## Development Checks

```bash
cargo fmt --all --check
cargo check -p bevy_boxddd --no-default-features
cargo nextest run -p bevy_boxddd
cargo check -p bevy_boxddd --examples
cargo check -p bevy_boxddd --features debug-gizmos --example debug_draw_overlay_3d
cargo check -p bevy_boxddd --features physics-picking --example physics_picking_3d
cargo check -p bevy_boxddd --features "debug-gizmos physics-picking" --example testbed_3d
cargo check -p bevy_boxddd --target wasm32-unknown-unknown --no-default-features
```

Windowed teaching examples:

```bash
cargo run -p bevy_boxddd --example falling_stack_3d
cargo run -p bevy_boxddd --example advanced_colliders_3d
cargo run -p bevy_boxddd --example joint_gallery_3d
cargo run -p bevy_boxddd --features debug-gizmos --example debug_draw_overlay_3d
cargo run -p bevy_boxddd --features physics-picking --example physics_picking_3d
cargo run -p bevy_boxddd --features "debug-gizmos physics-picking" --example testbed_3d
```

On Windows, the teaching examples default to the DX12 backend because some NVIDIA/Vulkan driver and validation-layer combinations emit noisy swapchain validation errors even when the app continues to run. Set `WGPU_BACKEND=vulkan` before running an example if you want to test Vulkan explicitly.
