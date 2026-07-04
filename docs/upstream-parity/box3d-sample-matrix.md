# Box3D Official Sample Parity Matrix

This matrix maps the vendored Box3D 0.1.0 official samples to `boxddd` and
`bevy_boxddd` teaching material. It is a concept-parity map, not a promise to
port every C++ sample line-for-line.

The source of truth is the vendored sample registration calls in
`boxddd-sys/third-party/box3d/samples/sample_*.cpp`. If the vendored Box3D
subtree is updated, refresh this matrix in the same change.

## Status Legend

- **Covered**: the concept has a maintained Rust example, Bevy scene, or focused
  test that teaches the same API surface.
- **Planned**: the concept is in scope for the documentation/example parity plan.
- **Partial**: an adjacent example exists, but the category needs a clearer
  teaching path or documentation link.
- **Deferred**: not part of the first release teaching surface; keep it as
  upstream reference material or future issue work.

## Category Matrix

| Official category | Vendored samples | Rust coverage | Status | Notes |
|---|---|---|---|---|
| Benchmark | Large Pyramid, Wide Pyramid, Many Pyramids, Rain, Large World, Joint Grid, Falling Boxes, Candy Cups, Explosion, Height Field, Falling Trees, Sensor, Washer, Hull, Chains, Destruction, Junkyard | CI and examples exercise representative stacks, joints, events, and meshes; no benchmark parity harness yet | Deferred | These are performance and stress scenes. Add benchmark parity only after the public teaching surface is stable. |
| Bodies | Body Type, Spinning Book, Gyroscopic Torque, Weeble, Disable, Cast, Kinematic, Lock Mixing, Fixed Rotation | `boxddd/examples/hello_world.rs`, `bevy_boxddd/examples/falling_stack_3d.rs`, planned `boxddd/examples/body_controls.rs` | Planned | The planned core example should teach body type, kinematic motion, disable/enable, locks, force, impulse, and casts without renderer dependencies. |
| Character | CapsulePlane, MoverOverlap, Mover, Rigid Body | `boxddd/tests/mover_api.rs`, planned `boxddd/examples/character_mover.rs`, planned Bevy testbed character/controller-style scene | Planned | Keep this focused on mover-plane queries and controller integration data, not a full game character framework. |
| Collision | Ray Curtain, Cast World, Mesh Scale, Shape Cast, Overlap World, Long Ray Cast, Initial Overlap, Shape Cast Debug, Distance Debug, Shape Distance, Time of Impact, Capsule Cast Ray | `boxddd/examples/shape_queries.rs`, `boxddd/examples/advanced_collision.rs`, `bevy_boxddd/examples/physics_picking_3d.rs`, planned `boxddd/examples/continuous_collision.rs` | Partial | Existing examples cover queries and standalone collision; the planned continuous example should make fast-shape diagnostics explicit. |
| Compound | Simple, Spheres, Hulls, Tile Floor, Mesh Tile, Village | `boxddd/examples/compound_query.rs`, `boxddd/examples/mesh_height_field_query.rs`, `bevy_boxddd/examples/advanced_colliders_3d.rs`, `bevy_boxddd/examples/testbed_3d` | Covered | Bevy examples should separate static resource-backed collider showcases from dynamic falling bodies. |
| Continuous | Thin Wall, Bounce House, Spinning Stick, Bullet vs Stack, Needle Mesh, Mesh Drop, Mesh Drop Unit Test, Hump Mesh, Is Fast, Stall | `boxddd/tests/collision_validation.rs`, `boxddd/examples/advanced_collision.rs`, planned `boxddd/examples/continuous_collision.rs` | Planned | Prefer deterministic headless diagnostics over a visual-only demo. |
| Determinism | Falling Ragdolls | `boxddd/examples/determinism.rs`, `boxddd/examples/recording_replay.rs`, `boxddd/tests/determinism.rs` | Covered | The Rust examples teach deterministic replay rather than the exact ragdoll scene. |
| Events | Sensor Visit, Hit, Move, Joint, Persistent Contact, Sensor Hits | `bevy_boxddd/examples/contact_messages_3d.rs`, `boxddd/tests/events_and_sensors.rs`, planned `boxddd/examples/events.rs` | Planned | Core example should show owned snapshots and closure-scoped event views. |
| Geometry | Box Hull, Hull, Hull Reduction, Hull Transform, Capsule Mass | `boxddd/examples/advanced_collision.rs`, `bevy_boxddd/examples/advanced_colliders_3d.rs`, planned Bevy shape/material scene | Partial | Geometry teaching should emphasize ownership of hull data and transformed hull scale. |
| Issues | Dump Loader, Crash, Multiple Prismatic, Hull Crash, Convex Jitter, s&box mover, Capsule Mesh | Regression tests when a specific upstream issue maps to a safe wrapper bug | Deferred | Issue samples are upstream repro material, not first-run examples. Port only when fixing the corresponding bug. |
| Joints | Distance Joint, Filter, Motor Joint, Top Down Friction, Prismatic, Spherical, Parallel Spring, Revolute, Weld, Wheel, Ball and Chain, Door, Bridge, Motion Locks, Driving, Gear Lift | `boxddd/examples/joints.rs`, `boxddd/tests/joints.rs`, `boxddd/tests/joint_runtime.rs`, `bevy_boxddd/examples/joint_gallery_3d.rs`, `bevy_boxddd/tests/joints.rs` | Covered | Runtime docs still need unit and wrong-family semantics, but example coverage exists. |
| Manifold | Sphere vs Sphere, Capsule vs Sphere, Hull vs Sphere, Triangle vs Sphere, Capsule vs Capsule, Capsule vs Hull, Triangle vs Capsule, Hull vs Hull, Triangle vs Hull | `boxddd/tests/manifold_collision.rs`, `boxddd/examples/advanced_collision.rs` | Covered | This is better as headless collision output than a separate Bevy scene. |
| Mesh | Grid, Big Box, Box, Reflection, Height Field, Viewer, Creation Benchmark, Voxel, Hollow Box | `boxddd/examples/mesh_height_field_query.rs`, `bevy_boxddd/examples/advanced_colliders_3d.rs`, `bevy_boxddd/examples/testbed_3d` | Covered | Viewer and creation benchmark samples remain upstream reference material. |
| Ragdoll | Box, Mesh, Pile, Incline, Pose | Joint and shape primitives in `boxddd/examples/joints.rs` and Bevy joint scenes | Deferred | Full ragdoll authoring is a higher-level asset/rigging topic, not core binding parity for 0.1.0. |
| Replay | Viewer | `boxddd/examples/recording_replay.rs`, `boxddd/tests/recording.rs` | Covered | The Rust teaching path focuses on deterministic headless recording and replay instead of cloning the upstream ImGui timeline viewer. |
| Robustness | HighMassRatio1, Tiny Pyramid, Overlap Recovery, Overflow Color Pile | Focused tests when a robustness case becomes a wrapper regression | Deferred | Keep these as upstream stress references until there is a Rust API bug or benchmark target. |
| Shapes | Inclined Plane, Rolling Resistance, High Resistance, Isotropic Friction, Slide Twist, Restitution, Static Invoke, Conveyor Belt, Conveyor Mesh, Wind, Wind Drop, Wind Flap | `boxddd/examples/hello_world.rs`, `boxddd/examples/advanced_collision.rs`, `bevy_boxddd/examples/advanced_colliders_3d.rs`, planned Bevy shape/material scene | Partial | A visible Bevy scene should make materials, restitution, wind, and conveyor-style behavior easier to inspect. |
| Stacking | Card House Thick, Card House, Sphere Stack, Capsule Stack, Single Box, Cylinder, Cylinder Stack, Box Stack, Jenga Stack, Dominoes, Wedge, Arch, Double Domino, Pyramid2D | `bevy_boxddd/examples/falling_stack_3d.rs`, `bevy_boxddd/examples/testbed_3d`, core `hello_world` smoke | Covered | Use Bevy for visual teaching; keep the core crate renderer-free. |
| Tree | Benchmark | `boxddd/examples/dynamic_tree.rs`, `boxddd/tests/dynamic_tree.rs` | Covered | The Rust example teaches standalone broad-phase lifecycle and callbacks without adopting the upstream benchmark UI. |
| World | Far Stack, Far Pyramid, Far Ragdolls, Far Mesh Drop | `boxddd/examples/hello_world.rs`, `boxddd/examples/determinism.rs`, Bevy stack scenes, planned `boxddd/examples/body_controls.rs` | Partial | Far-origin examples should stay deferred unless world-origin/precision behavior becomes a user-facing issue. |

## Porting Priorities

1. Core headless examples for events, body controls, continuous collision, and
   character mover data.
2. Bevy visible scenes for static-vs-dynamic advanced colliders, shape/material
   behavior, events, joints, picking, and debug draw.
3. Focused tests for concepts that are better proven headlessly than rendered:
   manifolds, determinism, dynamic tree, recording/replay, and callback/lifetime
   behavior.
4. Deferred benchmark, issue, robustness, and ragdoll work only after a concrete
   release goal requires it.
