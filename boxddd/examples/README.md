# boxddd Examples

Run examples from the workspace root with `cargo run -p boxddd --example <name>`.

The catalog is grouped by the workflow each example teaches. Start with the first
section if you are new to the crate, then move into integration examples once you
need an engine/render-loop shape.

## Recommended First Examples

- `hello_world.rs`: minimal world, static ground hull, dynamic box hull, stepping, and body position reads.
- `error_handling.rs`: `anyhow::Context` at app boundaries plus recoverable `try_*` errors for invalid user/tooling input.

## Joints

- `joints.rs`: creates a distance joint and reads runtime joint state after simulation.

## App Integration And Ownership

- `physics_thread.rs`: creates and owns `World` inside a dedicated physics thread, then sends plain body snapshots over channels.
- `tokio_async_bridge.rs`: uses `tokio::task::spawn_blocking` and async channels so async apps do not hold `World` across async tasks.

  ```bash
  cargo run -p boxddd --example tokio_async_bridge --features tokio-example
  ```

- `bevy_ecs_integration.rs`: shows the Bevy ECS pattern for a `!Send` Box3D world: use a Bevy non-send resource and copy transforms into ECS components.

  ```bash
  cargo run -p boxddd --example bevy_ecs_integration --features bevy-example
  ```

## Visual Debugging

- `egui_debug_draw.rs`: native egui/wgpu teaching viewer with pause, sub-step controls, and a projected 3D scene. This is intentionally simple and app-owned; it demonstrates how a renderer can consume plain Box3D snapshots without sharing `World`.

  ```bash
  cargo run -p boxddd --example egui_debug_draw --features egui-example
  ```

## Recording And Replay

- `recording_replay.rs`: records a short simulation, creates a replay player, steps it to the end, and reports divergence status.
- `determinism.rs`: records a simulation and validates replay determinism explicitly with worker count `1`.

## Interop

- `mint_interop.rs`: uses `mint::Vector3`, `mint::Point3`, and `mint::Quaternion` conversions in a normal world setup.

  ```bash
  cargo run -p boxddd --example mint_interop --features mint
  ```

## Suggested Verification Sweep

```bash
cargo check -p boxddd --examples
cargo run -p boxddd --example hello_world
cargo run -p boxddd --example joints
cargo run -p boxddd --example recording_replay
cargo run -p boxddd --example determinism
cargo run -p boxddd --example error_handling
cargo run -p boxddd --example physics_thread
cargo run -p boxddd --example mint_interop --features mint
cargo run -p boxddd --example tokio_async_bridge --features tokio-example
cargo run -p boxddd --example bevy_ecs_integration --features bevy-example
cargo check -p boxddd --example egui_debug_draw --features egui-example
```

## Porting Targets From Official Box3D Samples

The upstream Box3D repository groups interactive demos under `samples/`:
`sample_bodies.cpp`, `sample_character.cpp`, `sample_collision.cpp`,
`sample_compound.cpp`, `sample_continuous.cpp`, `sample_events.cpp`,
`sample_geometry.cpp`, `sample_joint.cpp`, `sample_mesh.cpp`, `sample_replay.cpp`,
`sample_robustness.cpp`, `sample_shapes.cpp`, and `sample_world.cpp`.

Good next ports for `boxddd` are:

- world/body laboratory: `sample_world.cpp` + `sample_bodies.cpp`
- geometry showcase: `sample_shapes.cpp`, `sample_mesh.cpp`, `sample_compound.cpp`
- query/collision teaching: `sample_collision.cpp`, `sample_manifold.cpp`
- gameplay-like scenes: `sample_character.cpp`, `sample_joint.cpp`, `sample_continuous.cpp`
- diagnostics: `sample_events.cpp`, `sample_replay.cpp`, `sample_determinism.cpp`
