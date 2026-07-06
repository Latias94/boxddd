# boxddd Examples

Run examples from the workspace root with `cargo run -p boxddd --example <name>`.

The catalog is grouped by the workflow each example teaches. Start with the first
section if you are new to the crate, then move into integration examples once you
need an engine/render-loop shape.

## Recommended First Examples

- `hello_world.rs`: minimal world, static ground hull, dynamic box hull, stepping, and body position reads.
- `error_handling.rs`: `anyhow::Context` at app boundaries plus recoverable `try_*` errors for invalid user/tooling input.
- `events.rs`: sensor begin/end events, contact and hit events, and closure-scoped event views.
- `body_controls.rs`: dynamic, kinematic, disabled/enabled, force, impulse, motion-lock, and transform-sync body APIs.
- `stats_profile.rs`: world counters, awake-body counts, capacity snapshots, and per-step profile timings.
- `shape_queries.rs`: world, body, and shape-scoped query APIs for editor tools, picking, and gameplay probes.
- `compound_query.rs`: compound child AABB queries, reusable buffers, visitor early-stop, and owned byte transfer.
- `mesh_height_field_query.rs`: mesh and height-field triangle queries for editor tooling and broad-phase probes.
- `wasm_smoke.rs`: minimal no-rendering smoke used by native and WASI runtime checks.

## Queries, Collision, And Broad Phase

- `shape_queries.rs`: creates a body with multiple shapes, then runs world, body, and shape-scoped query APIs.
- `compound_query.rs`: queries a compound resource without creating a world and demonstrates `CompoundBytes` ownership transfer.
- `mesh_height_field_query.rs`: queries standalone mesh and height-field triangles with owned results and visitor callbacks.
- `advanced_collision.rs`: uses standalone distance, shape-cast, local manifold, and plane solver helpers without creating a `World`.
- `continuous_collision.rs`: uses shape-cast, time-of-impact, and bullet-body settings for fast-moving-shape diagnostics.
- `character_mover.rs`: casts a capsule mover, gathers contact planes, solves correction, and clips velocity.
- `dynamic_tree.rs`: owns a standalone broad-phase tree, creates proxies with category bits, runs overlap/closest/ray visitors, and moves proxies.

## Events And Body Control

- `events.rs`: reads sensor, contact, hit, and body-move events through owned snapshots and safe closure-scoped views.
- `body_controls.rs`: demonstrates the body lifecycle knobs most engines expose to gameplay and editor tools.
- `stats_profile.rs`: prints the runtime counters and profile fields most tools surface in debug overlays.

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

## WASM Runtime Smoke

- `wasm_smoke.rs`: creates a single-thread world, steps a falling dynamic box, runs an AABB query, and exits successfully. It is intentionally renderer-free so the same physics smoke can run under `wasm32-wasip1`.

  ```bash
  cargo run -p boxddd --example wasm_smoke
  rustup target add wasm32-wasip1
  export WASI_SDK_PATH=/path/to/wasi-sdk-33.0-x86_64-linux
  export WASI_SYSROOT="$WASI_SDK_PATH/share/wasi-sysroot"
  export CC_wasm32_wasip1="$WASI_SDK_PATH/bin/clang"
  cargo build -p boxddd --example wasm_smoke --target wasm32-wasip1
  wasmtime target/wasm32-wasip1/debug/examples/wasm_smoke.wasm
  ```

## Recording And Replay

- `recording_replay.rs`: records a short simulation, creates a replay player, steps it to the end, and reports divergence status.
- `determinism.rs`: records a simulation and validates replay determinism explicitly with worker count `1`.

## Interop

- `mint_interop.rs`: uses `mint::Vector3`, `mint::Point3`, and `mint::Quaternion` conversions in a normal world setup.
- `glam_interop.rs`: uses `glam::Vec3`, `glam::Quat`, and velocity snapshots in a normal world setup.
- `nalgebra_interop.rs`: uses `nalgebra::Vector3`, `nalgebra::Point3`, `UnitQuaternion`, and `Isometry3` conversions in a normal world setup.

  ```bash
  cargo run -p boxddd --example mint_interop --features mint
  cargo run -p boxddd --example glam_interop --features glam
  cargo run -p boxddd --example nalgebra_interop --features nalgebra
  ```

## Suggested Verification Sweep

```bash
cargo check -p boxddd --examples
cargo run -p boxddd --example hello_world
cargo run -p boxddd --example shape_queries
cargo run -p boxddd --example compound_query
cargo run -p boxddd --example mesh_height_field_query
cargo run -p boxddd --example advanced_collision
cargo run -p boxddd --example continuous_collision
cargo run -p boxddd --example character_mover
cargo run -p boxddd --example dynamic_tree
cargo run -p boxddd --example events
cargo run -p boxddd --example body_controls
cargo run -p boxddd --example stats_profile
cargo run -p boxddd --example wasm_smoke
cargo run -p boxddd --example joints
cargo run -p boxddd --example recording_replay
cargo run -p boxddd --example determinism
cargo run -p boxddd --example error_handling
cargo run -p boxddd --example task_system
cargo run -p boxddd --example physics_thread
cargo run -p boxddd --example mint_interop --features mint
cargo run -p boxddd --example glam_interop --features glam
cargo run -p boxddd --example nalgebra_interop --features nalgebra
cargo run -p boxddd --example tokio_async_bridge --features tokio-example
cargo check -p boxddd --example egui_debug_draw --features egui-example
cargo check -p bevy_boxddd --examples
cargo check -p bevy_boxddd --features debug-gizmos --example debug_draw_overlay_3d
cargo check -p bevy_boxddd --features physics-picking --example physics_picking_3d
cargo check -p bevy_boxddd --features "debug-gizmos physics-picking" --example testbed_3d
```

## Official Box3D Sample Parity

The maintained case-level matrix from vendored Box3D official samples to Rust
examples is
[`docs/upstream-parity/box3d-sample-matrix.md`](https://github.com/Latias94/boxddd/blob/main/docs/upstream-parity/box3d-sample-matrix.md).
It records whether each upstream case is a faithful port, teaching adaptation,
test-only proof, deferred case, or upstream reference.

```bash
cargo run -p xtask -- sample-parity --check
```
