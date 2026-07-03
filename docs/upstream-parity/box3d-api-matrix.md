# Box3D API Parity Matrix

This matrix tracks the vendored Box3D C API slice used by `boxddd-sys` and the safe `boxddd` surface for the `0.1.x` release line.

Status legend:

- **Wrapped**: available on the safe `boxddd` API.
- **Raw-only**: intentionally exposed through `boxddd_sys::ffi` only for this release.
- **Deferred**: known upstream API that needs a focused safe design before it should be wrapped.
- **Not applicable**: internal/package concern, not a safe wrapper target.

## Summary

| Upstream area | Representative symbols | 0.1 status | Notes |
| --- | --- | --- | --- |
| Raw bindings/build | all `b3*` symbols in vendored headers | Wrapped in `boxddd-sys` | Pregenerated default and double-precision bindings are checked in; `bindgen` refresh is explicit. |
| Version/build metadata | `b3GetVersion`, `b3IsDoublePrecision`, `b3GetByteCount` | Wrapped | `version`, `is_double_precision`, `allocated_byte_count`. |
| Allocator/assert/log hooks | `b3SetAllocator`, `b3SetAssertFcn`, `b3SetLogFcn` | Raw-only | Process-global hooks need a separate initialization/ownership policy. |
| Timing/files/platform helpers | `b3GetTicks`, `b3Sleep`, `b3Hash`, `b3ReadBinaryFile`, `b3WriteBinaryFile` | Raw-only | Not required for the safe physics model; use Rust std/time/io unless a Box3D-specific reason appears. |
| Scalar/vector validation | `b3IsValidFloat`, `b3IsValidVec3`, `b3IsValidQuat`, `b3IsValidTransform`, `b3IsValidPosition`, `b3IsValidWorldTransform` | Wrapped | Value types expose `is_valid`/`validate` where applicable. |
| Extra math helpers | `b3Atan2`, `b3ComputeCosSin`, `b3MakeQuatFromMatrix`, `b3ComputeQuatBetweenUnitVectors`, `b3Steiner`, segment-distance helpers | Deferred | Safe math-helper module can be added after the core physics wrapper stabilizes. |
| World lifecycle | `b3CreateWorld`, `b3DestroyWorld`, `b3World_IsValid`, `b3GetWorldCount`, `b3GetMaxWorldCount` | Wrapped/Raw-only | `World` owns create/destroy/valid; global world-count diagnostics remain raw-only. |
| World stepping/drawing | `b3World_Step`, `b3World_Draw` | Wrapped | `try_step`, `step`, debug draw callback and collection APIs. |
| World runtime metrics/tuning | bounds, gravity, sleeping, continuous, warm starting, speculative, thresholds, contact tuning, worker count, profile, counters, capacity, rebuild static tree | Wrapped | Dump/debug-print functions remain raw-only. |
| World user data | `b3World_SetUserData`, `b3World_GetUserData` | Raw-only | Raw pointers are not part of the safe 0.1 ownership model. |
| World callbacks | custom filter, pre-solve, friction, restitution | Wrapped | Rust callback panics are caught and reentrant safe access is blocked. |
| World task callbacks | `b3EnqueueTaskCallback`, `b3FinishTaskCallback`, `userTaskContext` | Wrapped/Deferred | `TaskSystem::blocking_threads()` provides a safe Rust-owned callback path at world creation. Tokio, Rayon, Bevy Tasks, and arbitrary executor adapters remain deferred until their blocking/deadlock contracts are designed. |
| World queries | `b3World_OverlapAABB`, `b3World_OverlapShape`, `b3World_CastRay`, `b3World_CastRayClosest`, `b3World_CastShape`, `b3World_CastMover` | Wrapped | Owned, reusable-buffer, and visitor variants exist where the upstream callback path supports them. |
| Character mover planes | `b3World_CollideMover`, `b3Body_CollideMover`, `b3SolvePlanes`, `b3ClipVector` | Wrapped/Deferred | World/body collide-mover plane collection is wrapped; plane solving and clipping helpers remain deferred to the advanced collision unit. |
| Explosion | `b3World_Explode`, `b3DefaultExplosionDef` | Wrapped | `ExplosionDef` validates finite position/radius/falloff/impulse values before applying the world explosion. |
| Memory/stat dump helpers | `b3World_DumpMemoryStats`, `b3World_DumpShapeBounds`, `b3World_DumpAwake`, `b3World_Dump` | Raw-only | Diagnostic printing is not wrapped in the safe API. |
| Recording/replay | `b3CreateRecording`, `b3World_StartRecording`, `b3World_StopRecording`, save/load, validate, `b3RecPlayer_*` except debug-shape callbacks | Wrapped | Replay world id is intentionally read-only and not exposed as a normal `World`. |
| Replay debug-shape callbacks | `b3RecPlayer_SetDebugShapeCallbacks` | Raw-only | Safe replay query drawing reuses the debug draw adapter; custom replay debug-shape lifetime callbacks are deferred. |
| Body lifecycle/type/name/transform | `b3CreateBody`, `b3DestroyBody`, type, name, position, rotation, transforms, local/world point/vector conversion | Wrapped | Safe `BodyDef` validates pointer-sensitive fields through builders. |
| Body user data | `b3Body_SetUserData`, `b3Body_GetUserData` | Raw-only | Raw pointer ownership is deliberately explicit. |
| Body velocity/forces/impulses/mass/damping/sleep/enabled/bullet/motion locks | `b3Body_*` runtime methods | Wrapped | Recoverable `try_*` variants front-load validity and scalar checks. |
| Body attached resources/events/queries | body shapes, joints, contacts, AABB, closest point, ray/shape overlap/casts, collide mover | Wrapped | Shapes/joints/contacts/AABB, body-local closest point, ray casts, shape casts, overlap, and collide mover are wrapped. |
| Shape creation | sphere, capsule, hull, transformed hull, mesh, height field, compound | Wrapped | Native geometry resources use RAII and destroy-order tests. |
| Shape lifecycle/runtime | destroy, valid, type, body/world, sensor, density, friction, restitution, material, filter, event toggles/getters, AABB, mass data, geometry getters/setters | Wrapped | Mesh material indexing/readback is wrapped with bounds checks; hull, mesh, and height-field readback returns lifetime-bound views. |
| Shape user data/wind | `b3Shape_SetUserData`, `b3Shape_GetUserData`, `b3Shape_ApplyWind` | Wrapped/Raw-only | Pointer user data stays raw; wind is wrapped with finite/non-negative input validation. |
| Shape contacts/sensors/query helpers | `b3Shape_GetContactData`, `b3Shape_GetSensorData`, `b3Shape_RayCast`, closest point | Wrapped | Contact/sensor buffers, AABB, mass data, direct ray-cast, closest-point, and geometry view helpers are wrapped. |
| Geometry resources | hull/mesh/height field/compound create/destroy helpers | Wrapped/Deferred | Core RAII constructors are wrapped; specialized wave/torus/hollow/platform/load/dump/byte-conversion helpers are deferred. |
| Dynamic tree | `b3DynamicTree_*` | Deferred | Needs an owned safe tree type; not required for world-level query use. |
| Standalone collision | sphere/capsule/hull/mesh/height field/compound mass/AABB/overlap/ray/shape cast; sphere/capsule manifolds | Wrapped/Deferred | Current safe layer covers the common gameplay/tested subset. GJK distance, TOI, sweep, triangle/hull manifold variants, and mesh/height-field query callbacks remain deferred. |
| Joints common runtime | destroy, valid, type, bodies, world, local frames, collide connected, wake, forces/torques, separations, tuning, thresholds | Wrapped | Wrong-family typed calls return `Error::WrongJointType`. |
| Joint user data | `b3Joint_SetUserData`, `b3Joint_GetUserData` | Raw-only | Raw pointer ownership is explicit. |
| Parallel joint | create, spring, damping, max torque | Wrapped | Typed definition and runtime APIs. |
| Distance joint | create, length, spring/range, limit/range, motor/speed/force/current length | Wrapped | Typed definition and runtime APIs. |
| Motor joint | create, linear/angular velocity, max velocity force/torque, spring tuning | Wrapped | Typed definition and runtime APIs. |
| Filter joint | create | Wrapped | No runtime family-specific state beyond common joint APIs. |
| Prismatic joint | create, spring, target translation, limits, motor, translation/speed | Wrapped | Typed definition and runtime APIs. |
| Revolute joint | create, spring, target/angle, limits, motor speed/torque | Wrapped | Typed definition and runtime APIs. |
| Spherical joint | create, cone/twist limits, spring, target rotation, motor velocity/torque | Wrapped | Typed definition and runtime APIs. |
| Weld joint | create, linear/angular spring tuning | Wrapped | Typed definition and runtime APIs. |
| Wheel joint | create, suspension, spin motor, steering and limits | Wrapped | Typed definition and runtime APIs. |
| Contact id/data | `b3Contact_IsValid`, `b3Contact_GetData` | Wrapped | Safe `ContactId`, `ContactData`, and `Manifold` value types. |
| Default definition values | `b3DefaultWorldDef`, `b3DefaultBodyDef`, `b3DefaultShapeDef`, joint defaults, query filter, debug draw | Wrapped | Safe builders keep raw pointer fields out of ordinary user code. |
| Debug draw default/color | `b3DefaultDebugDraw`, `b3GetGraphColor` | Wrapped/Raw-only | Default debug draw is used internally; graph color helper remains raw-only. |

## Release Policy For Deferred APIs

Deferred entries are not hidden unknowns. They are intentionally outside the `0.1.x` safe claim because they need one of:

- a new owned native resource type (`b3DynamicTree`)
- raw pointer/user-data ownership policy
- callback/threading design
- allocation and lifetime tests for callback-returned buffers
- representative examples proving the intended workflow

Until then, downstream users can access them explicitly through `boxddd_sys::ffi`.
