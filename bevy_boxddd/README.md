# bevy_boxddd

`bevy_boxddd` integrates [`boxddd`](https://github.com/Latias94/boxddd) with Bevy 0.19. It is a separate crate so non-Bevy users can keep using the engine-agnostic `boxddd` bindings without Bevy dependencies.

## Quick Facts

| Item | Value |
|---|---|
| Bevy version | `0.19.0` |
| Rust version | `1.95.0` or newer |
| Physics owner | `NonSend<BoxdddPhysicsContext>` |
| Schedule | `FixedUpdate` |
| Rendering | Examples only; the library has no renderer dependency |
| Async/threading | Keep `boxddd::World` on the Bevy main thread; move snapshots across threads |

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

## Examples

| Example | Run command | Purpose |
|---|---|---|
| `falling_stack_3d` | `cargo run -p bevy_boxddd --example falling_stack_3d` | Windowed 3D scene with ground, cubes, spheres, camera, light, and plugin-driven transforms. |
| `contact_messages_3d` | `cargo run -p bevy_boxddd --example contact_messages_3d` | Reads `BoxdddContactBeginMessage` / `BoxdddContactEndMessage` and updates Bevy materials from plugin messages. |
| `debug_gizmos_3d` | `cargo run -p bevy_boxddd --example debug_gizmos_3d` | Draws collider outlines with Bevy `Gizmos` without putting debug rendering into the plugin core. |

## Components

Spawn an entity with:

- `RigidBody`: `Static`, `Kinematic`, or `Dynamic`.
- `Collider`: `cuboid`, `cube`, `sphere`, or `capsule_y`.
- `PhysicsMaterial`: density, friction, restitution, sensor/contact flags, and Box3D filter data.
- `TransformSyncMode`: default dynamic bodies are physics-authored; static and kinematic bodies are Bevy-authored.
- `LinearVelocity`, `AngularVelocity`, `ExternalForce`, `ExternalImpulse`: control inputs applied before stepping.

The plugin inserts `BoxdddBody` and `BoxdddShape` after native resource creation. The first slice supports one collider component per entity.

## Messages And Errors

The plugin registers Bevy messages for:

- `BoxdddErrorMessage`
- `BoxdddBodyMoveMessage`
- `BoxdddContactBeginMessage`
- `BoxdddContactEndMessage`
- `BoxdddContactHitMessage`
- `BoxdddSensorBeginMessage`
- `BoxdddSensorEndMessage`

Recoverable errors follow `BoxdddPhysicsSettings.error_policy`: message only, message plus log, or panic for debugging.

## Development Checks

```bash
cargo check -p bevy_boxddd --no-default-features
cargo nextest run -p bevy_boxddd
cargo check -p bevy_boxddd --examples
```
