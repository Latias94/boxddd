# bevy_boxddd Examples

Run examples from the workspace root with `cargo run -p bevy_boxddd --example <name>`.

These examples are native desktop teaching apps. They use Bevy windows, cameras, lights, and simple meshes so users can see how `boxddd` bodies, shapes, joints, events, queries, debug draw, and picking fit into a real Bevy app.

## Start Here

```bash
cargo run -p bevy_boxddd --example falling_stack_3d
cargo run -p bevy_boxddd --example advanced_colliders_3d
cargo run -p bevy_boxddd --example joint_gallery_3d
cargo run -p bevy_boxddd --features "debug-gizmos physics-picking" --example testbed_3d
```

On Windows, examples default to DX12. Set `WGPU_BACKEND=vulkan` only when you intentionally want to test Vulkan.

## Catalog

| Example | Command | Teaches |
|---|---|---|
| Falling stack | `cargo run -p bevy_boxddd --example falling_stack_3d` | Dynamic rigid bodies, static floor, transform sync |
| Advanced colliders | `cargo run -p bevy_boxddd --example advanced_colliders_3d` | Static mesh, height-field, compound, sphere, and hull-backed colliders |
| Joint gallery | `cargo run -p bevy_boxddd --example joint_gallery_3d` | Declarative Bevy joint authoring with visible connected bodies |
| Contact messages | `cargo run -p bevy_boxddd --example contact_messages_3d` | Bevy messages for contact begin/end/hit events |
| Debug gizmos | `cargo run -p bevy_boxddd --example debug_gizmos_3d` | App-authored collider gizmos without Box3D debug draw collection |
| Debug draw overlay | `cargo run -p bevy_boxddd --features debug-gizmos --example debug_draw_overlay_3d` | Box3D debug draw commands rendered through Bevy `Gizmos` |
| Physics picking | `cargo run -p bevy_boxddd --features physics-picking --example physics_picking_3d` | Camera cursor rays resolved through Box3D queries |
| Testbed | `cargo run -p bevy_boxddd --features "debug-gizmos physics-picking" --example testbed_3d` | Switchable scene browser with egui controls, picking, debug draw, and representative Box3D sample concepts |

## Testbed Scenes

The testbed is the primary visual learning surface. Use the left panel to switch scenes and adjust pause, single-step, gravity, solver rate, substeps, sleeping, warm starting, continuous collision, and debug draw presets.

Current official-parity scenes: Falling Stack, Advanced Colliders, Body Controls, Continuous Collision, Character Mover, Materials, Joints, Contacts And Sensors, Ray Picking, Debug Draw, Domino Run, Arch Stack, Wind Field, and Ragdoll Chain.

Current boxddd showcase entries: Query Lab, Debug Draw Inspector, and Material Lab. They include live egui controls for Box3D ray casts, AABB overlaps, shape casts, mover casts, debug draw frame inspection, and native shape material tuning, and are not counted as official Box3D sample ports. Browser Query Lab entries label unbridged provider-mode visitor queries as unavailable instead of reporting misleading zero-hit results.

The static demo hub at <https://frankorz.com/boxddd/> mirrors this scene registry.

## Headless Validation

CI validates the scene registry without opening a GPU window:

```bash
cargo nextest run -p bevy_boxddd --test testbed
```

Compile the visible examples locally with:

```bash
cargo check -p bevy_boxddd --examples
cargo check -p bevy_boxddd --features debug-gizmos --example debug_draw_overlay_3d
cargo check -p bevy_boxddd --features physics-picking --example physics_picking_3d
cargo check -p bevy_boxddd --features "debug-gizmos physics-picking" --example testbed_3d
```

## Upstream Parity

The examples follow the case-level official Box3D sample matrix in [`docs/upstream-parity/box3d-sample-matrix.md`](https://github.com/Latias94/boxddd/blob/main/docs/upstream-parity/box3d-sample-matrix.md). The goal is readable Rust and Bevy teaching coverage with honest labels for faithful ports, teaching adaptations, test-only proofs, deferred cases, and upstream references.
