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
- `task_system.rs`: configures Rust-owned Box3D task callbacks with `TaskSystem::blocking_threads()` and prints scheduler counters.
- `tokio_async_bridge.rs`: uses `tokio::task::spawn_blocking` and async channels so async apps do not hold `World` across async tasks.

  ```bash
  cargo run -p boxddd --example tokio_async_bridge --features tokio-example
  ```

- Bevy integration now lives in the sibling `bevy_boxddd` crate. Use its windowed examples when you want a real Bevy app with camera, light, meshes, messages, joints, queries, debug draw, and gizmos.

  ```bash
  cargo run -p bevy_boxddd --example falling_stack_3d
  cargo run -p bevy_boxddd --example advanced_colliders_3d
  cargo run -p bevy_boxddd --example joint_gallery_3d
  cargo run -p bevy_boxddd --features debug-gizmos --example debug_draw_overlay_3d
  cargo run -p bevy_boxddd --features physics-picking --example physics_picking_3d
  cargo run -p bevy_boxddd --features "debug-gizmos physics-picking" --example testbed_3d
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
cargo run -p boxddd --example task_system
cargo run -p boxddd --example physics_thread
cargo run -p boxddd --example mint_interop --features mint
cargo run -p boxddd --example tokio_async_bridge --features tokio-example
cargo check -p boxddd --example egui_debug_draw --features egui-example
cargo check -p bevy_boxddd --examples
cargo check -p bevy_boxddd --features debug-gizmos --example debug_draw_overlay_3d
cargo check -p bevy_boxddd --features physics-picking --example physics_picking_3d
cargo check -p bevy_boxddd --features "debug-gizmos physics-picking" --example testbed_3d
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
