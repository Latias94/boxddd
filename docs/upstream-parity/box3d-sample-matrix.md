# Box3D Official Sample Parity Matrix

This matrix maps the vendored Box3D 0.1.0 official samples to `boxddd` and `bevy_boxddd` teaching material.
It is a case-level parity map, not a promise to port every C++ sample host feature line-for-line.

The source of truth for upstream cases is the vendored sample registration calls in `boxddd-sys/third-party/box3d/samples/sample_*.cpp`.
Run the parity check whenever the vendored Box3D subtree changes:

```bash
cargo run -p xtask -- sample-parity --check
```

## Parity Modes

- **FaithfulPort**: the Rust example or Bevy scene closely follows the official sample's user-visible behavior.
- **TeachingAdaptation**: the Rust or Bevy artifact teaches the same Box3D concept with idiomatic `boxddd` structure.
- **TestOnly**: the case is better proven through a focused test, core example, or deterministic smoke than a visual scene.
- **Deferred**: the case is intentionally not part of the current public teaching surface; the note records the trigger for future work.
- **UpstreamReference**: the case is tied to the official sample host or renderer and remains reference material.

## Category Summary

| Category | Cases | Current Rust strategy |
|---|---:|---|
| Benchmark | 18 | Deferred until a Rust benchmark harness has explicit measurement goals. |
| Bodies | 9 | Covered through body controls and Bevy body-control scenes. |
| Character | 4 | Covered through mover examples and the Bevy character mover scene. |
| Collision | 12 | Covered through query/collision examples, tests, and picking scenes. |
| Compound | 6 | Covered through compound, mesh, height-field, and advanced collider examples. |
| Continuous | 10 | Covered for the common bullet/TOI path; mesh stress cases stay test/deferred. |
| Determinism | 1 | Covered through deterministic recording/replay tests and examples. |
| Events | 6 | Covered through event examples, Bevy messages, and focused tests. |
| Geometry | 5 | Covered through collision geometry tests and collider examples. |
| Issues | 7 | Deferred until a corresponding wrapper regression is fixed. |
| Joints | 16 | Covered through joint gallery/testbed scenes and joint runtime tests. |
| Manifold | 9 | Covered through headless manifold collision tests. |
| Mesh | 9 | Covered through mesh/height-field examples; renderer viewer and creation benchmark are deferred/reference. |
| Ragdoll | 5 | Covered as ragdoll-chain teaching adaptation except pose editing and mesh rig details. |
| Replay | 1 | Covered through recording/replay example and tests, not the upstream ImGui viewer. |
| Robustness | 4 | Deferred until a wrapper regression or robustness target appears. |
| Shapes | 12 | Covered through materials, wind, collision, and collider scenes; conveyor authoring remains deferred. |
| Stacking | 14 | Covered through falling stack, domino, and arch scenes. |
| Tree | 1 | Covered through dynamic tree example and tests. |
| World | 4 | Deferred until far-origin precision behavior becomes user-facing. |

## Deferred Route Summary

Deferred rows are intentional routing decisions, not untriaged backlog. Use this table when deciding whether a future change should remain deferred, become a benchmark, become a regression test, or become a visual scene.

| Bucket | Deferred rows | Route |
|---|---:|---|
| Benchmark stress scenes | 18 | Convert only when `boxddd/benches` has a measured throughput or allocation goal for the specific scenario. Do not turn these into visual examples unless they teach a user-facing workflow. |
| Upstream issue repros | 7 | Convert to a focused regression test only after the upstream behavior reproduces through safe `boxddd` wrappers. Until then they remain upstream references. |
| Robustness scenes | 4 | Convert to release-risk tests when scale, mass-ratio, recovery, or debug-color behavior exposes a Rust API contract or bug. |
| Conveyor scenes | 2 | Convert to a Bevy material/conveyor showcase after tangent velocity or equivalent conveyor authoring is available safely in `bevy_boxddd`. `material-lab` covers material coefficients today, not conveyor motion. |
| Far-world scenes | 4 | Convert after `boxddd` documents a far-origin precision contract and has a visual or headless assertion worth teaching. |
| Ragdoll pose | 1 | Convert only when interactive pose editing or rig import becomes part of the Bevy teaching surface; `ragdoll-chain` remains the current lightweight joint-chain adaptation. |
| Mesh creation benchmark | 1 | Convert with the benchmark bucket when mesh construction throughput has explicit measurement goals. |

## Official Case Matrix

| Category | Official sample | Source location | Parity mode | Target | Notes |
|---|---|---|---|---|---|
| Benchmark | Large Pyramid | `sample_benchmark.cpp:44` | Deferred | Upstream benchmark reference | Stress benchmark; port when `boxddd` has benchmark harness goals. |
| Benchmark | Wide Pyramid | `sample_benchmark.cpp:68` | Deferred | Upstream benchmark reference | Stress benchmark; port when pyramid breadth is a measured target. |
| Benchmark | Many Pyramids | `sample_benchmark.cpp:101` | Deferred | Upstream benchmark reference | Stress benchmark; port when multi-island throughput is measured. |
| Benchmark | Rain | `sample_benchmark.cpp:154` | Deferred | Upstream benchmark reference | Stress benchmark; port when spawn-rate performance is measured. |
| Benchmark | Large World | `sample_benchmark.cpp:203` | Deferred | Upstream benchmark reference | Large-world stress case; port with a precision/performance benchmark. |
| Benchmark | Joint Grid | `sample_benchmark.cpp:248` | Deferred | Upstream benchmark reference | Joint throughput benchmark; current joint examples are teaching coverage. |
| Benchmark | Falling Boxes | `sample_benchmark.cpp:293` | Deferred | Upstream benchmark reference | Box throughput benchmark; visible stack scenes cover teaching only. |
| Benchmark | Candy Cups | `sample_benchmark.cpp:371` | Deferred | Upstream benchmark reference | Convex stress scene; port when hull benchmark coverage is needed. |
| Benchmark | Explosion | `sample_benchmark.cpp:488` | Deferred | Upstream benchmark reference | High-force stress case; port when force-field benchmark coverage is needed. |
| Benchmark | Height Field | `sample_benchmark.cpp:658` | Deferred | Upstream benchmark reference | Height-field throughput benchmark; teaching covered elsewhere. |
| Benchmark | Falling Trees | `sample_benchmark.cpp:730` | Deferred | Upstream benchmark reference | Broad dynamic stress case; port when scenario has a benchmark target. |
| Benchmark | Sensor | `sample_benchmark.cpp:963` | Deferred | Upstream benchmark reference | Sensor throughput benchmark; event semantics are covered by examples/tests. |
| Benchmark | Washer | `sample_benchmark.cpp:990` | Deferred | Upstream benchmark reference | Stress scene; port only with a measured benchmark story. |
| Benchmark | Large World | `sample_benchmark.cpp:1022` | Deferred | Upstream benchmark reference | Second upstream large-world registration; source location disambiguates it. |
| Benchmark | Hull | `sample_benchmark.cpp:1110` | Deferred | Upstream benchmark reference | Hull creation/query benchmark; safe hull API is covered by tests/examples. |
| Benchmark | Chains | `sample_benchmark.cpp:1222` | Deferred | Upstream benchmark reference | Chain stress benchmark; joint teaching coverage lives in Bevy scenes. |
| Benchmark | Destruction | `sample_benchmark.cpp:1416` | Deferred | Upstream benchmark reference | Destruction stress benchmark; port when destruction throughput is measured. |
| Benchmark | Junkyard | `sample_benchmark.cpp:1451` | Deferred | Upstream benchmark reference | Large mixed stress scene; not a first-release teaching target. |
| Bodies | Body Type | `sample_bodies.cpp:269` | TeachingAdaptation | `boxddd/examples/body_controls.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#body-controls` | Teaches static/dynamic/kinematic body authoring through Rust and Bevy. |
| Bodies | Spinning Book | `sample_bodies.cpp:317` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#body-controls` | Visual body-control scene covers angular motion teaching. |
| Bodies | Gyroscopic Torque | `sample_bodies.cpp:356` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#body-controls` | Bevy body-control scene is the current visible angular-torque teaching path. |
| Bodies | Weeble | `sample_bodies.cpp:457` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#body-controls` | Body-control scene covers center-of-mass and stability behavior at teaching level. |
| Bodies | Disable | `sample_bodies.cpp:566` | TeachingAdaptation | `boxddd/examples/body_controls.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#body-controls` | Safe enable/disable lifecycle is covered by the body-controls example. |
| Bodies | Cast | `sample_bodies.cpp:753` | TestOnly | `boxddd/examples/shape_queries.rs`, `boxddd/tests/world_and_queries.rs` | Body and shape query APIs are covered headlessly. |
| Bodies | Kinematic | `sample_bodies.cpp:829` | TeachingAdaptation | `boxddd/examples/body_controls.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#body-controls` | Kinematic body motion is part of the body-controls teaching path. |
| Bodies | Lock Mixing | `sample_bodies.cpp:911` | TeachingAdaptation | `boxddd/examples/body_controls.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#body-controls` | Motion locks are covered in core and Bevy body-control examples. |
| Bodies | Fixed Rotation | `sample_bodies.cpp:960` | TeachingAdaptation | `boxddd/examples/body_controls.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#body-controls` | Fixed rotation maps to the safe motion-lock API. |
| Character | CapsulePlane | `sample_character.cpp:145` | TestOnly | `boxddd/tests/mover_api.rs`, `boxddd/examples/character_mover.rs` | Capsule mover plane behavior is covered through the mover API test/example. |
| Character | MoverOverlap | `sample_character.cpp:312` | TestOnly | `boxddd/tests/mover_api.rs`, `boxddd/examples/character_mover.rs` | Mover overlap behavior is covered by focused core mover coverage. |
| Character | Mover | `sample_character.cpp:590` | TeachingAdaptation | `boxddd/examples/character_mover.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#character-mover` | Core and Bevy examples teach mover casts and obstacle probes. |
| Character | Rigid Body | `sample_character.cpp:1671` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#character-mover` | Bevy scene shows rigid-body character-style interaction without a framework promise. |
| Collision | Ray Curtain | `sample_collision.cpp:118` | TestOnly | `boxddd/examples/shape_queries.rs`, `boxddd/tests/world_and_queries.rs` | Ray casting is covered through query examples and tests. |
| Collision | Cast World | `sample_collision.cpp:772` | TestOnly | `boxddd/examples/shape_queries.rs`, `boxddd/tests/world_and_queries.rs` | World casts are covered through safe query APIs. |
| Collision | Mesh Scale | `sample_collision.cpp:888` | TeachingAdaptation | `boxddd/examples/mesh_height_field_query.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Mesh/height-field query and Bevy collider scenes teach the concept. |
| Collision | Shape Cast | `sample_collision.cpp:1144` | TestOnly | `boxddd/examples/advanced_collision.rs`, `boxddd/tests/collision_validation.rs` | Standalone shape-cast coverage is headless and deterministic. |
| Collision | Overlap World | `sample_collision.cpp:1329` | TestOnly | `boxddd/examples/shape_queries.rs`, `boxddd/tests/world_and_queries.rs` | Overlap queries are covered by query examples and tests. |
| Collision | Long Ray Cast | `sample_collision.cpp:1573` | TestOnly | `boxddd/examples/shape_queries.rs`, `boxddd/tests/world_and_queries.rs` | Long ray behavior is part of query coverage rather than a separate scene. |
| Collision | Initial Overlap | `sample_collision.cpp:1680` | TestOnly | `boxddd/examples/advanced_collision.rs`, `boxddd/tests/collision_validation.rs` | Initial-overlap diagnostics belong in deterministic collision tests. |
| Collision | Shape Cast Debug | `sample_collision.cpp:1791` | TestOnly | `boxddd/examples/advanced_collision.rs`, `boxddd/tests/collision_validation.rs` | Debug-style shape-cast data is covered by the core collision example/test. |
| Collision | Distance Debug | `sample_collision.cpp:2048` | TestOnly | `boxddd/examples/advanced_collision.rs`, `boxddd/tests/collision_validation.rs` | Distance diagnostics are covered headlessly. |
| Collision | Shape Distance | `sample_collision.cpp:2496` | TestOnly | `boxddd/examples/advanced_collision.rs`, `boxddd/tests/collision_validation.rs` | Shape distance maps to the standalone collision helper coverage. |
| Collision | Time of Impact | `sample_collision.cpp:2724` | TestOnly | `boxddd/examples/continuous_collision.rs`, `boxddd/tests/collision_validation.rs` | TOI is covered by continuous collision diagnostics. |
| Collision | Capsule Cast Ray | `sample_collision.cpp:2798` | TestOnly | `boxddd/examples/advanced_collision.rs`, `boxddd/tests/collision_validation.rs` | Capsule ray casting is covered in core collision diagnostics. |
| Compound | Simple | `sample_compound.cpp:106` | TeachingAdaptation | `boxddd/examples/compound_query.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Compound creation/query is covered in core and Bevy examples. |
| Compound | Spheres | `sample_compound.cpp:167` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Advanced collider scene includes sphere-backed collider cases. |
| Compound | Hulls | `sample_compound.cpp:241` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Hull compound behavior is covered as a visual collider adaptation. |
| Compound | Tile Floor | `sample_compound.cpp:359` | TeachingAdaptation | `boxddd/examples/compound_query.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Static compound/tile concepts map to the advanced colliders scene. |
| Compound | Mesh Tile | `sample_compound.cpp:475` | TeachingAdaptation | `boxddd/examples/mesh_height_field_query.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Mesh tile behavior is covered through mesh/height-field examples. |
| Compound | Village | `sample_compound.cpp:798` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Large compound village is represented by smaller collider teaching cases. |
| Continuous | Thin Wall | `sample_continuous.cpp:73` | TeachingAdaptation | `boxddd/examples/continuous_collision.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#continuous-collision` | Bullet bodies against thin obstacles are covered in core and Bevy. |
| Continuous | Bounce House | `sample_continuous.cpp:147` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#continuous-collision` | Bevy continuous scene teaches repeated fast-body collision. |
| Continuous | Spinning Stick | `sample_continuous.cpp:189` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#continuous-collision` | Fast angular motion is represented in the continuous collision scene. |
| Continuous | Bullet vs Stack | `sample_continuous.cpp:279` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#continuous-collision` | Bullet-vs-stack behavior is a Bevy scene target. |
| Continuous | Needle Mesh | `sample_continuous.cpp:390` | TestOnly | `boxddd/examples/continuous_collision.rs`, `boxddd/tests/collision_validation.rs` | Mesh CCD edge case is kept deterministic in core coverage. |
| Continuous | Mesh Drop | `sample_continuous.cpp:742` | TeachingAdaptation | `boxddd/examples/continuous_collision.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Mesh drop is represented by mesh collider plus CCD teaching paths. |
| Continuous | Mesh Drop Unit Test | `sample_continuous.cpp:796` | TestOnly | `boxddd/tests/collision_validation.rs` | Upstream unit-test case maps to Rust collision validation. |
| Continuous | Hump Mesh | `sample_continuous.cpp:909` | TestOnly | `boxddd/examples/continuous_collision.rs`, `boxddd/tests/collision_validation.rs` | Mesh CCD stress behavior is covered headlessly. |
| Continuous | Is Fast | `sample_continuous.cpp:994` | TestOnly | `boxddd/examples/continuous_collision.rs`, `boxddd/tests/collision_validation.rs` | Fast-body classification belongs in core continuous collision coverage. |
| Continuous | Stall | `sample_continuous.cpp:1078` | TestOnly | `boxddd/tests/collision_validation.rs` | Stall regression coverage is headless, not a visual teaching scene. |
| Determinism | Falling Ragdolls | `sample_determinism.cpp:59` | TestOnly | `boxddd/examples/determinism.rs`, `boxddd/tests/determinism.rs` | Rust teaches deterministic replay instead of cloning the exact ragdoll scene. |
| Events | Sensor Visit | `sample_events.cpp:81` | TestOnly | `boxddd/examples/events.rs`, `boxddd/tests/events_and_sensors.rs` | Sensor visit semantics are covered by safe event snapshots/tests. |
| Events | Hit | `sample_events.cpp:245` | TeachingAdaptation | `boxddd/examples/events.rs`, `bevy_boxddd/examples/contact_messages_3d.rs` | Hit events are shown through core and Bevy message examples. |
| Events | Move | `sample_events.cpp:328` | TestOnly | `boxddd/examples/events.rs`, `boxddd/tests/events_and_sensors.rs` | Body move events are covered by core event tests/examples. |
| Events | Joint | `sample_events.cpp:571` | TestOnly | `boxddd/tests/events_and_sensors.rs`, `boxddd/tests/joint_runtime.rs` | Joint event semantics stay test-focused because generation can be threshold-sensitive. |
| Events | Persistent Contact | `sample_events.cpp:670` | TeachingAdaptation | `boxddd/examples/events.rs`, `bevy_boxddd/examples/contact_messages_3d.rs` | Persistent contact teaching is covered through contact message flow. |
| Events | Sensor Hits | `sample_events.cpp:901` | TeachingAdaptation | `boxddd/examples/events.rs`, `bevy_boxddd/examples/contact_messages_3d.rs` | Sensor hit teaching is represented by event examples and Bevy messages. |
| Geometry | Box Hull | `sample_geometry.cpp:136` | TestOnly | `boxddd/tests/shape_geometry_validation.rs`, `boxddd/examples/advanced_collision.rs` | Hull construction is covered by shape geometry validation. |
| Geometry | Hull | `sample_geometry.cpp:231` | TestOnly | `boxddd/tests/shape_geometry_validation.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Hull geometry is covered by validation and collider scenes. |
| Geometry | Hull Reduction | `sample_geometry.cpp:360` | TestOnly | `boxddd/tests/shape_geometry_validation.rs` | Hull reduction is an algorithmic geometry validation case. |
| Geometry | Hull Transform | `sample_geometry.cpp:488` | TestOnly | `boxddd/tests/shape_geometry_validation.rs`, `boxddd/examples/advanced_collision.rs` | Transformed hull behavior is covered through geometry/collision tests. |
| Geometry | Capsule Mass | `sample_geometry.cpp:648` | TestOnly | `boxddd/tests/shape_geometry_validation.rs` | Capsule mass is a deterministic shape geometry check. |
| Issues | Dump Loader | `sample_issues.cpp:50` | Deferred | Upstream issue reference | Port when this upstream issue maps to a safe-wrapper bug. |
| Issues | Crash | `sample_issues.cpp:116` | Deferred | Upstream issue reference | Port when this crash reproduces in `boxddd`. |
| Issues | Multiple Prismatic | `sample_issues.cpp:172` | Deferred | Upstream issue reference | Port when a prismatic wrapper regression appears. |
| Issues | Hull Crash | `sample_issues.cpp:273` | Deferred | Upstream issue reference | Port when this hull crash maps to Rust API behavior. |
| Issues | Convex Jitter | `sample_issues.cpp:383` | Deferred | Upstream issue reference | Port when jitter becomes a `boxddd` regression target. |
| Issues | s&box mover | `sample_issues.cpp:461` | Deferred | Upstream issue reference | Port when the mover issue reproduces through safe APIs. |
| Issues | Capsule Mesh | `sample_issues.cpp:532` | Deferred | Upstream issue reference | Port when capsule-mesh behavior becomes a wrapper regression. |
| Joints | Distance Joint | `sample_joint.cpp:236` | TeachingAdaptation | `boxddd/examples/joints.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#joints` | Distance joints are covered by core and Bevy joint examples. |
| Joints | Filter | `sample_joint.cpp:276` | TestOnly | `boxddd/tests/joints.rs`, `bevy_boxddd/tests/joints.rs` | Filter joint semantics are test-backed. |
| Joints | Motor Joint | `sample_joint.cpp:445` | TeachingAdaptation | `bevy_boxddd/examples/joint_gallery_3d.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#joints` | Motor-style joint behavior is covered through Bevy joint gallery. |
| Joints | Top Down Friction | `sample_joint.cpp:550` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#joints` | Top-down friction maps to joint/material teaching in the Bevy testbed. |
| Joints | Prismatic | `sample_joint.cpp:718` | TeachingAdaptation | `bevy_boxddd/examples/joint_gallery_3d.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#joints` | Prismatic joints are part of the public Bevy joint gallery. |
| Joints | Spherical | `sample_joint.cpp:897` | TeachingAdaptation | `bevy_boxddd/examples/joint_gallery_3d.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#joints` | Spherical joints are covered in the joint gallery and testbed. |
| Joints | Parallel Spring | `sample_joint.cpp:1013` | TeachingAdaptation | `bevy_boxddd/examples/joint_gallery_3d.rs`, `boxddd/tests/joint_new_apis.rs` | Parallel spring API is covered by joint examples/tests. |
| Joints | Revolute | `sample_joint.cpp:1191` | TeachingAdaptation | `bevy_boxddd/examples/joint_gallery_3d.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#joints` | Revolute joints are covered visually. |
| Joints | Weld | `sample_joint.cpp:1290` | TeachingAdaptation | `bevy_boxddd/examples/joint_gallery_3d.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#joints` | Weld joints are covered visually. |
| Joints | Wheel | `sample_joint.cpp:1536` | TeachingAdaptation | `bevy_boxddd/examples/joint_gallery_3d.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#joints` | Wheel joints are covered visually. |
| Joints | Ball and Chain | `sample_joint.cpp:1602` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#ragdoll-chain`, `boxddd/tests/joints.rs` | Chain-style joint teaching is covered by ragdoll/chain examples. |
| Joints | Door | `sample_joint.cpp:1831` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#joints` | Door-style revolute behavior is represented in joint gallery scenes. |
| Joints | Bridge | `sample_joint.cpp:1954` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#joints` | Bridge-style connected bodies are represented by joint-gallery teaching. |
| Joints | Motion Locks | `sample_joint.cpp:2235` | TeachingAdaptation | `boxddd/examples/body_controls.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#body-controls` | Motion locks are covered through body controls. |
| Joints | Driving | `sample_joint.cpp:2624` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#joints` | Driving behavior maps to visible joint-gallery interaction. |
| Joints | Gear Lift | `sample_joint.cpp:3100` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#joints` | Gear lift is represented as advanced joint-gallery teaching, not a host clone. |
| Manifold | Sphere vs Sphere | `sample_manifold.cpp:392` | TestOnly | `boxddd/tests/manifold_collision.rs` | Manifold outputs are better asserted headlessly. |
| Manifold | Capsule vs Sphere | `sample_manifold.cpp:430` | TestOnly | `boxddd/tests/manifold_collision.rs` | Manifold outputs are better asserted headlessly. |
| Manifold | Hull vs Sphere | `sample_manifold.cpp:473` | TestOnly | `boxddd/tests/manifold_collision.rs` | Manifold outputs are better asserted headlessly. |
| Manifold | Triangle vs Sphere | `sample_manifold.cpp:526` | TestOnly | `boxddd/tests/manifold_collision.rs` | Manifold outputs are better asserted headlessly. |
| Manifold | Capsule vs Capsule | `sample_manifold.cpp:562` | TestOnly | `boxddd/tests/manifold_collision.rs` | Manifold outputs are better asserted headlessly. |
| Manifold | Capsule vs Hull | `sample_manifold.cpp:621` | TestOnly | `boxddd/tests/manifold_collision.rs` | Manifold outputs are better asserted headlessly. |
| Manifold | Triangle vs Capsule | `sample_manifold.cpp:681` | TestOnly | `boxddd/tests/manifold_collision.rs` | Manifold outputs are better asserted headlessly. |
| Manifold | Hull vs Hull | `sample_manifold.cpp:814` | TestOnly | `boxddd/tests/manifold_collision.rs` | Manifold outputs are better asserted headlessly. |
| Manifold | Triangle vs Hull | `sample_manifold.cpp:923` | TestOnly | `boxddd/tests/manifold_collision.rs` | Manifold outputs are better asserted headlessly. |
| Mesh | Grid | `sample_mesh.cpp:208` | TeachingAdaptation | `boxddd/examples/mesh_height_field_query.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Mesh grid behavior is covered by mesh and collider teaching paths. |
| Mesh | Big Box | `sample_mesh.cpp:382` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Large mesh collider behavior is represented by advanced colliders. |
| Mesh | Box | `sample_mesh.cpp:551` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Box mesh teaching is included in advanced colliders. |
| Mesh | Reflection | `sample_mesh.cpp:740` | TestOnly | `boxddd/examples/mesh_height_field_query.rs`, `boxddd/tests/shape_resources.rs` | Mesh resource behavior is covered by core examples/tests. |
| Mesh | Height Field | `sample_mesh.cpp:995` | TeachingAdaptation | `boxddd/examples/mesh_height_field_query.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Height fields are covered in core and Bevy examples. |
| Mesh | Viewer | `sample_mesh.cpp:1298` | UpstreamReference | Official renderer viewer | The upstream viewer is host UI; Rust mesh teaching lives in examples. |
| Mesh | Creation Benchmark | `sample_mesh.cpp:1379` | Deferred | Upstream benchmark reference | Port when mesh creation throughput is measured. |
| Mesh | Voxel | `sample_mesh.cpp:1495` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Voxel-style mesh teaching is represented by advanced colliders. |
| Mesh | Hollow Box | `sample_mesh.cpp:1586` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#advanced-colliders` | Hollow mesh concepts are represented by advanced collider teaching. |
| Ragdoll | Box | `sample_ragdoll.cpp:79` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#ragdoll-chain` | Ragdoll box behavior maps to a lightweight capsule joint chain. |
| Ragdoll | Mesh | `sample_ragdoll.cpp:205` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#ragdoll-chain` | Mesh rig details are simplified into a visible joint-chain teaching scene. |
| Ragdoll | Pile | `sample_ragdoll.cpp:262` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#ragdoll-chain` | Pile behavior is represented by the ragdoll-chain scene. |
| Ragdoll | Incline | `sample_ragdoll.cpp:337` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#ragdoll-chain` | Inclined ragdoll behavior is represented at teaching level. |
| Ragdoll | Pose | `sample_ragdoll.cpp:463` | Deferred | Upstream ragdoll pose reference | Pose editing and rig import are outside current teaching scope. |
| Replay | Viewer | `sample_replay.cpp:1744` | TestOnly | `boxddd/examples/recording_replay.rs`, `boxddd/tests/recording.rs` | Rust covers deterministic recording/replay, not the upstream ImGui viewer. |
| Robustness | HighMassRatio1 | `sample_robustness.cpp:70` | Deferred | Upstream robustness reference | Port when high-mass-ratio behavior becomes a wrapper regression target. |
| Robustness | Tiny Pyramid | `sample_robustness.cpp:129` | Deferred | Upstream robustness reference | Port when tiny-scale stability needs release proof. |
| Robustness | Overlap Recovery | `sample_robustness.cpp:241` | Deferred | Upstream robustness reference | Port when overlap recovery exposes a Rust API bug. |
| Robustness | Overflow Color Pile | `sample_robustness.cpp:281` | Deferred | Upstream robustness reference | Port when debug color overflow or pile robustness becomes release-relevant. |
| Shapes | Inclined Plane | `sample_shapes.cpp:54` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#materials` | Inclined-plane material behavior is represented by material scenes. |
| Shapes | Rolling Resistance | `sample_shapes.cpp:110` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#materials` | Rolling/friction behavior is taught through material variants. |
| Shapes | High Resistance | `sample_shapes.cpp:149` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#materials` | High friction/resistance maps to the materials scene. |
| Shapes | Isotropic Friction | `sample_shapes.cpp:193` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#materials` | Friction behavior is covered by material variants. |
| Shapes | Slide Twist | `sample_shapes.cpp:239` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#materials` | Sliding and twisting are represented by Bevy material demonstrations. |
| Shapes | Restitution | `sample_shapes.cpp:337` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#materials` | Restitution variants are visible in the materials scene. |
| Shapes | Static Invoke | `sample_shapes.cpp:436` | TestOnly | `boxddd/tests/world_runtime.rs`, `boxddd/examples/body_controls.rs` | Static-body invocation behavior is covered through runtime/body tests. |
| Shapes | Conveyor Belt | `sample_shapes.cpp:486` | Deferred | Upstream conveyor reference | Port when tangent-velocity/conveyor authoring is added to `bevy_boxddd`. |
| Shapes | Conveyor Mesh | `sample_shapes.cpp:675` | Deferred | Upstream conveyor mesh reference | Port with conveyor authoring and mesh-material teaching support. |
| Shapes | Wind | `sample_shapes.cpp:842` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#wind-field` | Wind force field is represented by the Bevy wind scene. |
| Shapes | Wind Drop | `sample_shapes.cpp:903` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#wind-field` | Wind drop maps to the wind-field scene. |
| Shapes | Wind Flap | `sample_shapes.cpp:1019` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#wind-field` | Wind flap is represented by force-field teaching, not cloth/soft-body behavior. |
| Stacking | Card House Thick | `sample_stacking.cpp:89` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#arch-stack` | Card-house stability maps to the arch/stack teaching scenes. |
| Stacking | Card House | `sample_stacking.cpp:163` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#arch-stack` | Card-house stability maps to the arch/stack teaching scenes. |
| Stacking | Sphere Stack | `sample_stacking.cpp:224` | TeachingAdaptation | `bevy_boxddd/examples/falling_stack_3d.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#falling-stack` | Falling stack scenes cover stacked dynamic shapes. |
| Stacking | Capsule Stack | `sample_stacking.cpp:268` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#falling-stack` | Capsule-style stacks are represented by falling stack teaching. |
| Stacking | Single Box | `sample_stacking.cpp:313` | FaithfulPort | `bevy_boxddd/examples/falling_stack_3d.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#falling-stack` | The simplest official stack case is directly represented by falling box examples. |
| Stacking | Cylinder | `sample_stacking.cpp:364` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#falling-stack` | Cylinder stack behavior is represented by dynamic-shape stack teaching. |
| Stacking | Cylinder Stack | `sample_stacking.cpp:422` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#falling-stack` | Cylinder-stack teaching maps to falling stack variants. |
| Stacking | Box Stack | `sample_stacking.cpp:477` | TeachingAdaptation | `bevy_boxddd/examples/falling_stack_3d.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#falling-stack` | Box stacks are covered in standalone and testbed Bevy examples. |
| Stacking | Jenga Stack | `sample_stacking.cpp:566` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#falling-stack` | Jenga-style instability maps to the falling-stack teaching scene. |
| Stacking | Dominoes | `sample_stacking.cpp:632` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#domino-run` | Domino behavior is directly represented by the domino-run scene. |
| Stacking | Wedge | `sample_stacking.cpp:677` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#arch-stack` | Wedge/arch stability is represented by arch-stack teaching. |
| Stacking | Arch | `sample_stacking.cpp:833` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#arch-stack` | Arch behavior is directly represented by the arch-stack scene. |
| Stacking | Double Domino | `sample_stacking.cpp:879` | TeachingAdaptation | `bevy_boxddd/examples/testbed_3d/scenes.rs#domino-run` | Double-domino behavior is represented by domino-run teaching. |
| Stacking | Pyramid2D | `sample_stacking.cpp:924` | TeachingAdaptation | `bevy_boxddd/examples/falling_stack_3d.rs`, `bevy_boxddd/examples/testbed_3d/scenes.rs#falling-stack` | Pyramid stack behavior maps to the falling-stack scene. |
| Tree | Benchmark | `sample_tree.cpp:710` | TestOnly | `boxddd/examples/dynamic_tree.rs`, `boxddd/tests/dynamic_tree.rs` | Dynamic tree lifecycle and query behavior are covered headlessly. |
| World | Far Stack | `sample_world.cpp:122` | Deferred | Upstream far-world reference | Port when far-origin precision becomes user-facing. |
| World | Far Pyramid | `sample_world.cpp:182` | Deferred | Upstream far-world reference | Port when far-origin precision becomes user-facing. |
| World | Far Ragdolls | `sample_world.cpp:245` | Deferred | Upstream far-world reference | Port when far-origin ragdoll precision becomes user-facing. |
| World | Far Mesh Drop | `sample_world.cpp:308` | Deferred | Upstream far-world reference | Port when far-origin mesh precision becomes user-facing. |
