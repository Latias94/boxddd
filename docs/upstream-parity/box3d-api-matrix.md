# Box3D API Parity Matrix

This matrix tracks the vendored Box3D C API slice used by `boxddd-sys` and the safe `boxddd` surface for the `0.1.x` release line.

Status legend:

- **Wrapped**: available on the safe `boxddd` API.
- **Raw-only**: intentionally exposed through `boxddd_sys::ffi` only for this release.
- **Unsafe raw**: available through explicit `unsafe`/raw-named `boxddd` APIs, not part of the safe API contract.
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
| Extra math helpers | `b3PointToSegmentDistance`, `b3LineDistance`, `b3SegmentDistance`, `b3Atan2`, `b3ComputeCosSin`, `b3MakeQuatFromMatrix`, `b3ComputeQuatBetweenUnitVectors`, `b3Steiner` | Wrapped | Segment and line distance helpers, deterministic scalar helpers, quaternion constructors, and Steiner inertia are wrapped with validation. |
| World lifecycle | `b3CreateWorld`, `b3DestroyWorld`, `b3World_IsValid`, `b3GetWorldCount`, `b3GetMaxWorldCount` | Wrapped/Not applicable | `World` owns create/destroy/valid; global world-count diagnostics are omitted because they do not fit the safe ownership model. |
| World stepping/drawing | `b3World_Step`, `b3World_Draw` | Wrapped | `try_step`, `step`, debug draw callback and collection APIs. |
| World runtime metrics/tuning | bounds, gravity, sleeping, continuous, warm starting, speculative, thresholds, contact tuning, worker count, profile, counters, capacity, rebuild static tree | Wrapped | Dump/debug-print functions remain raw-only. |
| World user data | `b3World_SetUserData`, `b3World_GetUserData` | Unsafe raw | Exposed through `boxddd::raw`; raw pointer ownership remains entirely caller-defined. |
| World callbacks | custom filter, pre-solve, friction, restitution | Wrapped | Rust callback panics are caught and reentrant safe access is blocked. |
| World task callbacks | `b3EnqueueTaskCallback`, `b3FinishTaskCallback`, `userTaskContext` | Wrapped/Deferred | `TaskSystem::blocking_threads()` provides a safe Rust-owned callback path at world creation. Tokio, Rayon, Bevy Tasks, and arbitrary executor adapters remain deferred until their blocking/deadlock contracts are designed. |
| World queries | `b3World_OverlapAABB`, `b3World_OverlapShape`, `b3World_CastRay`, `b3World_CastRayClosest`, `b3World_CastShape`, `b3World_CastMover` | Wrapped | Owned, reusable-buffer, and visitor variants exist where the upstream callback path supports them. |
| Character mover planes | `b3World_CollideMover`, `b3Body_CollideMover`, `b3SolvePlanes`, `b3ClipVector` | Wrapped | World/body collide-mover plane collection, plane solving, and clipping helpers are wrapped with owned values. |
| Explosion | `b3World_Explode`, `b3DefaultExplosionDef` | Wrapped | `ExplosionDef` validates finite position/radius/falloff/impulse values before applying the world explosion. |
| Memory/stat dump helpers | `b3World_DumpMemoryStats`, `b3World_DumpShapeBounds`, `b3World_DumpAwake`, `b3World_Dump` | Raw-only | Diagnostic printing is not wrapped in the safe API. |
| Recording/replay | `b3CreateRecording`, `b3World_StartRecording`, `b3World_StopRecording`, save/load, validate, `b3RecPlayer_*` except debug-shape callbacks | Wrapped | Replay world id is intentionally read-only and not exposed as a normal `World`. |
| Replay debug-shape callbacks | `b3RecPlayer_SetDebugShapeCallbacks` | Raw-only | Safe replay query drawing reuses the debug draw adapter; custom replay debug-shape lifetime callbacks are deferred. |
| Body lifecycle/type/name/transform | `b3CreateBody`, `b3DestroyBody`, type, name, position, rotation, transforms, local/world point/vector conversion | Wrapped | Safe `BodyDef` validates pointer-sensitive fields through builders. |
| Body user data | `b3Body_SetUserData`, `b3Body_GetUserData` | Unsafe raw | Exposed through `boxddd::raw`; raw pointer ownership remains entirely caller-defined. |
| Body velocity/forces/impulses/mass/damping/sleep/enabled/bullet/motion locks | `b3Body_*` runtime methods | Wrapped | Recoverable `try_*` variants front-load validity and scalar checks. |
| Body attached resources/events/queries | body shapes, joints, contacts, AABB, closest point, ray/shape overlap/casts, collide mover | Wrapped | Shapes/joints/contacts/AABB, body-local closest point, ray casts, shape casts, overlap, and collide mover are wrapped. |
| Shape creation | sphere, capsule, hull, transformed hull, mesh, height field, compound | Wrapped | Native geometry resources use RAII and destroy-order tests. |
| Shape lifecycle/runtime | destroy, valid, type, body, redundant world getter, sensor, density, friction, restitution, material, filter, event toggles/getters, AABB, mass data, geometry getters/setters | Wrapped/Omitted | Mesh material indexing/readback is wrapped with bounds checks; hull, mesh, and height-field readback returns lifetime-bound views. The raw shape-to-world getter is omitted because safe APIs validate ownership through `World`. |
| Shape user data/wind | `b3Shape_SetUserData`, `b3Shape_GetUserData`, `b3Shape_ApplyWind` | Wrapped/Unsafe raw | Pointer user data is exposed through `boxddd::raw`; wind is wrapped with finite/non-negative input validation. |
| Shape contacts/sensors/query helpers | `b3Shape_GetContactData`, `b3Shape_GetSensorData`, `b3Shape_RayCast`, closest point | Wrapped | Contact/sensor buffers, AABB, mass data, direct ray-cast, closest-point, and geometry view helpers are wrapped. |
| Geometry resources | hull/mesh/height field/compound create/destroy helpers | Wrapped/Raw-only | RAII constructors, hull clone/transform helpers, box scaling, arbitrary mesh creation, wave/torus/hollow/platform meshes, custom height fields, compound builders, compound child/material introspection, compound child queries, and `CompoundBytes` owner round-trips are wrapped. File-backed height fields and arbitrary caller-owned compound byte buffers stay raw-only. |
| Dynamic tree | `b3DynamicTree_*` except file save/load | Wrapped/Raw-only | `DynamicTree` owns create/destroy, proxy lifecycle, metrics, rebuild/validation, AABB/closest/ray/box cast visitors, and panic containment. File save/load stays raw file IO. |
| Standalone collision | sphere/capsule/hull/mesh/height field/compound mass/AABB/overlap/ray/shape cast; GJK distance, TOI, sweep transforms, plane solving, clipping, and sphere/capsule/hull/triangle manifolds | Wrapped/Deferred | The value-returning collision helpers are wrapped with validation and owned outputs. Mesh queries support early-stop visitors; height-field triangle queries return owned hits without promising native early stop. |
| Joints common runtime | destroy, valid, type, bodies, redundant world getter, local frames, collide connected, wake, forces/torques, separations, tuning, thresholds | Wrapped/Omitted | Wrong-family typed calls return `Error::WrongJointType`; the raw joint-to-world getter is omitted because safe APIs validate ownership through `World`. |
| Joint user data | `b3Joint_SetUserData`, `b3Joint_GetUserData` | Unsafe raw | Exposed through `boxddd::raw`; definition/event fields use `raw_user_data` naming where pointer values can surface. |
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
| Default definition values | `b3DefaultWorldDef`, `b3DefaultBodyDef`, `b3DefaultShapeDef`, `b3DefaultSurfaceMaterial`, joint defaults, query filter, debug draw | Wrapped | Safe builders keep raw pointer fields out of ordinary user code; `SurfaceMaterial::default` and `QueryFilter::default` mirror upstream defaults without exposing raw pointer fields. |
| Debug draw default/color | `b3DefaultDebugDraw`, `b3GetGraphColor` | Wrapped/Raw-only | Default debug draw is used internally; graph color helper remains a raw low-level diagnostic helper. |

## Release Policy For Deferred APIs

Deferred entries are not hidden unknowns. They are intentionally outside the `0.1.x` safe claim because they need one of:

- raw pointer/user-data ownership policy
- callback/threading design
- allocation and lifetime tests for callback-returned buffers
- representative examples proving the intended workflow

Until then, downstream users can access them explicitly through `boxddd_sys::ffi`.
