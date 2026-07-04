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
| Bodies | Body Type, Spinning Book, Gyroscopic Torque, Weeble, Disable, Cast, Kinematic, Lock Mixing, Fixed Rotation | `boxddd/examples/hello_world.rs`, `boxddd/examples/body_controls.rs`, `bevy_boxddd/examples/falling_stack_3d.rs` | Partial | The core example teaches body type, kinematic motion, disable/enable, locks, force, impulse, and transform synchronization without renderer dependencies. Gyroscopic and Weeble-style visual behavior remains better suited to Bevy. |
| Character | CapsulePlane, MoverOverlap, Mover, Rigid Body | `boxddd/tests/mover_api.rs`, `boxddd/examples/character_mover.rs`, `bevy_boxddd/examples/testbed_3d` character mover scene | Partial | Core coverage focuses on mover-plane queries and controller integration data; Bevy visualizes a capsule mover probe without becoming a full character-controller framework. |
| Collision | Ray Curtain, Cast World, Mesh Scale, Shape Cast, Overlap World, Long Ray Cast, Initial Overlap, Shape Cast Debug, Distance Debug, Shape Distance, Time of Impact, Capsule Cast Ray | `boxddd/examples/shape_queries.rs`, `boxddd/examples/advanced_collision.rs`, `boxddd/examples/continuous_collision.rs`, `bevy_boxddd/examples/physics_picking_3d.rs` | Covered | Core examples cover queries, standalone collision, and fast-shape diagnostics; Bevy covers visible picking-style inspection. |
| Compound | Simple, Spheres, Hulls, Tile Floor, Mesh Tile, Village | `boxddd/examples/compound_query.rs`, `boxddd/examples/mesh_height_field_query.rs`, `bevy_boxddd/examples/advanced_colliders_3d.rs`, `bevy_boxddd/examples/testbed_3d` | Covered | Bevy advanced collider scenes separate static resource-backed collider showcases from dynamic falling bodies. |
| Continuous | Thin Wall, Bounce House, Spinning Stick, Bullet vs Stack, Needle Mesh, Mesh Drop, Mesh Drop Unit Test, Hump Mesh, Is Fast, Stall | `boxddd/tests/collision_validation.rs`, `boxddd/examples/advanced_collision.rs`, `boxddd/examples/continuous_collision.rs`, `bevy_boxddd/examples/testbed_3d` continuous collision scene | Partial | Core examples provide deterministic shape-cast and time-of-impact diagnostics; Bevy visualizes bullet-style fast bodies against a thin wall and stack. Richer mesh CCD stress scenes remain upstream reference material. |
| Determinism | Falling Ragdolls | `boxddd/examples/determinism.rs`, `boxddd/examples/recording_replay.rs`, `boxddd/tests/determinism.rs` | Covered | The Rust examples teach deterministic replay rather than the exact ragdoll scene. |
| Events | Sensor Visit, Hit, Move, Joint, Persistent Contact, Sensor Hits | `bevy_boxddd/examples/contact_messages_3d.rs`, `boxddd/tests/events_and_sensors.rs`, `boxddd/examples/events.rs` | Partial | Core example shows owned snapshots and closure-scoped event views for sensor, contact, hit, and body-move events; joint events stay test/doc focused because threshold-driven generation is less deterministic as a first-run smoke. |
| Geometry | Box Hull, Hull, Hull Reduction, Hull Transform, Capsule Mass | `boxddd/examples/advanced_collision.rs`, `bevy_boxddd/examples/advanced_colliders_3d.rs`, `bevy_boxddd/examples/testbed_3d` | Partial | Geometry teaching emphasizes ownership of hull data and transformed hull scale; hull-reduction internals remain upstream reference material. |
| Issues | Dump Loader, Crash, Multiple Prismatic, Hull Crash, Convex Jitter, s&box mover, Capsule Mesh | Regression tests when a specific upstream issue maps to a safe wrapper bug | Deferred | Issue samples are upstream repro material, not first-run examples. Port only when fixing the corresponding bug. |
| Joints | Distance Joint, Filter, Motor Joint, Top Down Friction, Prismatic, Spherical, Parallel Spring, Revolute, Weld, Wheel, Ball and Chain, Door, Bridge, Motion Locks, Driving, Gear Lift | `boxddd/examples/joints.rs`, `boxddd/tests/joints.rs`, `boxddd/tests/joint_runtime.rs`, `bevy_boxddd/examples/joint_gallery_3d.rs`, `bevy_boxddd/examples/testbed_3d`, `bevy_boxddd/tests/joints.rs` | Covered | Core tests cover runtime semantics, and Bevy joint gallery/testbed scenes create every public declarative joint variant. |
| Manifold | Sphere vs Sphere, Capsule vs Sphere, Hull vs Sphere, Triangle vs Sphere, Capsule vs Capsule, Capsule vs Hull, Triangle vs Capsule, Hull vs Hull, Triangle vs Hull | `boxddd/tests/manifold_collision.rs`, `boxddd/examples/advanced_collision.rs` | Covered | This is better as headless collision output than a separate Bevy scene. |
| Mesh | Grid, Big Box, Box, Reflection, Height Field, Viewer, Creation Benchmark, Voxel, Hollow Box | `boxddd/examples/mesh_height_field_query.rs`, `bevy_boxddd/examples/advanced_colliders_3d.rs`, `bevy_boxddd/examples/testbed_3d` | Covered | Viewer and creation benchmark samples remain upstream reference material. |
| Ragdoll | Box, Mesh, Pile, Incline, Pose | Joint and shape primitives in `boxddd/examples/joints.rs`; `bevy_boxddd/examples/testbed_3d` includes a ragdoll-lite capsule chain | Partial | The Bevy scene demonstrates joint-chain behavior without promising full rig import, pose editing, or mesh ragdoll authoring. |
| Replay | Viewer | `boxddd/examples/recording_replay.rs`, `boxddd/tests/recording.rs` | Covered | The Rust teaching path focuses on deterministic headless recording and replay instead of cloning the upstream ImGui timeline viewer. |
| Robustness | HighMassRatio1, Tiny Pyramid, Overlap Recovery, Overflow Color Pile | Focused tests when a robustness case becomes a wrapper regression | Deferred | Keep these as upstream stress references until there is a Rust API bug or benchmark target. |
| Shapes | Inclined Plane, Rolling Resistance, High Resistance, Isotropic Friction, Slide Twist, Restitution, Static Invoke, Conveyor Belt, Conveyor Mesh, Wind, Wind Drop, Wind Flap | `boxddd/examples/hello_world.rs`, `boxddd/examples/advanced_collision.rs`, `bevy_boxddd/examples/advanced_colliders_3d.rs`, `bevy_boxddd/examples/testbed_3d` materials and wind-field scenes | Partial | Bevy now includes a wind-style force field using safe `ExternalForce` components. Rolling resistance and conveyor tangent-velocity authoring remain future Bevy-surface work. |
| Stacking | Card House Thick, Card House, Sphere Stack, Capsule Stack, Single Box, Cylinder, Cylinder Stack, Box Stack, Jenga Stack, Dominoes, Wedge, Arch, Double Domino, Pyramid2D | `bevy_boxddd/examples/falling_stack_3d.rs`, `bevy_boxddd/examples/testbed_3d`, core `hello_world` smoke | Covered | The Bevy testbed includes falling stacks, dominoes, and an arch-style block stack while keeping the core crate renderer-free. |
| Tree | Benchmark | `boxddd/examples/dynamic_tree.rs`, `boxddd/tests/dynamic_tree.rs` | Covered | The Rust example teaches standalone broad-phase lifecycle and callbacks without adopting the upstream benchmark UI. |
| World | Far Stack, Far Pyramid, Far Ragdolls, Far Mesh Drop | `boxddd/examples/hello_world.rs`, `boxddd/examples/determinism.rs`, `boxddd/examples/body_controls.rs`, Bevy stack scenes | Partial | Far-origin examples should stay deferred unless world-origin/precision behavior becomes a user-facing issue. |

## Porting Priorities

1. Focused tests for concepts that are better proven headlessly than rendered:
   manifolds, determinism, dynamic tree, recording/replay, and callback/lifetime
   behavior.
2. Deferred benchmark, issue, robustness, and ragdoll work only after a concrete
   release goal requires it.
