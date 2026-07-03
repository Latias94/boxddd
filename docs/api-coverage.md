# Box3D API Coverage

This document describes the safe-wrapper boundary for the vendored Box3D public C API targeted by `boxddd` `0.1.x`.
The machine-checkable source is `boxddd/tests/fixtures/api_coverage_symbols.txt`, and `boxddd/tests/api_coverage.rs` validates it against the vendored headers in `boxddd-sys/third-party/box3d/include/box3d`.

## Status Legend

| Status | Meaning |
|---|---|
| `safe` | Exposed through the safe `boxddd` API with Rust ownership, validation, or callback containment. |
| `raw` | Intentionally left to `boxddd_sys::ffi` or a future explicitly unsafe/raw module because the API involves process-global state, raw pointers, file IO, debug dumping, or platform hooks. |
| `omitted` | Not part of the safe wrapper contract for this release because it is diagnostic or incompatible with the crate ownership model. |
| `deferred` | Known upstream public API that still needs a focused safe design or implementation unit. |

## Current Snapshot

The current fixture classifies 578 unique upstream `B3_API` functions:

| Status | Count | Typical areas |
|---|---:|---|
| `safe` | 437 | world lifecycle and stepping, body runtime, shape creation and core properties, joints, events, world queries, debug draw, recording/replay, core math/value validation |
| `raw` | 35 | allocator/assert/log hooks, timers/sleep/hash, file IO, dump helpers, raw `void*` user data, file-backed dynamic tree or height-field helpers |
| `omitted` | 2 | global world-count diagnostics that do not fit the safe ownership model |
| `deferred` | 104 | body/shape scoped queries, mover collision, explosions, dynamic tree, advanced collision, compound/mesh/height-field completion, selected math helpers |

Counts are intentionally checked by tests instead of maintained only in prose. When the fixture changes, update this snapshot in the same commit.

## Safe Boundary Rules

- Safe APIs must validate handles and scalar/vector inputs before crossing FFI when validation is possible.
- Safe APIs must not expose borrowed Box3D-owned memory beyond the owning `World` or native resource lifetime.
- Callback APIs must follow the existing callback guard pattern: do not unwind through C, return `Error::CallbackPanicked` where panic containment is possible, and return `Error::UnsupportedOnWasm` for provider-mode WASM callback paths that are not sound yet.
- Process-global hooks are not ordinary safe convenience APIs. Allocator, assert, log, timer, file IO, global units, and stall threshold functions stay raw or require an explicit raw/unsafe policy.
- Raw `void*` user data is not a typed Rust ownership mechanism. Any exposure must make the raw boundary visible in the name and documentation.
- `World`, native resources, dynamic trees, recording, and replay player types remain single-owner and are not made `Send` or `Sync`.

## High-Priority Deferred Areas

These areas are intentionally visible in the fixture as `deferred` until their implementation units land:

- Body and shape scoped queries: `b3Body_CastRay`, `b3Body_CastShape`, `b3Body_OverlapShape`, `b3Body_GetClosestPoint`, `b3Shape_RayCast`, `b3Shape_GetClosestPoint`, and `b3Shape_ComputeMassData`.
- Runtime helpers: `b3World_CollideMover`, `b3Body_CollideMover`, `b3World_Explode`, and `b3DefaultExplosionDef`.
- Shape inspection and events: shape event-state getters, shape contact/sensor data, mesh material readback, geometry readback, and `b3Shape_ApplyWind`.
- Complex geometry: compound child/material queries, compound byte conversion, arbitrary mesh creation, wave/torus/hollow/platform mesh helpers, and custom height-field creation.
- Advanced collision: GJK distance, shape cast pair input, time of impact, sweep transforms, missing collision pair helpers, plane solving, and clipping.
- Dynamic tree: the independent `b3DynamicTree_*` broadphase API needs a dedicated RAII wrapper.

## How To Update Coverage

1. Add or change the safe/raw implementation.
2. Update `boxddd/tests/fixtures/api_coverage_symbols.txt` so the symbol status matches the public contract.
3. Update this document when the status counts or policy language change.
4. Run `cargo nextest run -p boxddd api_coverage`.

The coverage test is deliberately structural. It proves every upstream public symbol is classified, not that every `safe` entry has ideal API ergonomics. API quality is still enforced by the implementation tests, docs, and examples for each wrapper module.
