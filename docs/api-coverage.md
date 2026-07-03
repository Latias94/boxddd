# Box3D API Coverage

This document describes the safe-wrapper boundary for the vendored Box3D public C API targeted by `boxddd` `0.1.x`.
The machine-checkable source is `boxddd/tests/fixtures/api_coverage_symbols.txt`, and `boxddd/tests/api_coverage.rs` validates it against the vendored headers in `boxddd-sys/third-party/box3d/include/box3d`.

## Status Legend

| Status | Meaning |
|---|---|
| `safe` | Exposed through the safe `boxddd` API with Rust ownership, validation, or callback containment. |
| `raw` | Intentionally left to `boxddd_sys::ffi` or exposed only through explicitly unsafe/raw-named `boxddd` APIs because the API involves process-global state, raw pointers, file IO, debug dumping, or platform hooks. |
| `omitted` | Not part of the safe wrapper contract for this release because it is diagnostic or incompatible with the crate ownership model. |
| `deferred` | Known upstream public API that still needs a focused safe design or implementation unit. |

## Current Snapshot

The current fixture classifies 578 unique upstream `B3_API` functions:

| Status | Count | Typical areas |
|---|---:|---|
| `safe` | 538 | world lifecycle and stepping, body runtime, body/shape scoped queries, dynamic tree, mover collision, explosions, shape creation and runtime introspection, compound/mesh/height-field authoring, query, byte ownership transfer, and readback, shape event/contact/sensor readback, contact data, hull cloning and box scaling, advanced standalone collision, joints, events, world queries, debug draw, recording/replay, deterministic math helpers, core math/value validation |
| `raw` | 36 | allocator/assert/log hooks, timers/sleep/hash, file IO, dump helpers, explicit `boxddd::raw` user data and process-global scalar tuning, file-backed dynamic tree or height-field helpers, low-level debug graph color helper |
| `omitted` | 4 | global world-count diagnostics and redundant shape/joint world-handle getters that do not fit the safe ownership model |
| `deferred` | 0 | no current upstream `B3_API` symbols remain in the deferred bucket |

Counts are intentionally checked by tests instead of maintained only in prose. When the fixture changes, update this snapshot in the same commit.

## Safe Boundary Rules

- Safe APIs must validate handles and scalar/vector inputs before crossing FFI when validation is possible.
- Safe APIs must not expose borrowed Box3D-owned memory beyond the owning `World` or native resource lifetime.
- Callback APIs must follow the existing callback guard pattern: do not unwind through C, return `Error::CallbackPanicked` where panic containment is possible, and return `Error::UnsupportedOnWasm` for provider-mode WASM callback paths that are not sound yet.
- Process-global hooks are not ordinary safe convenience APIs. Allocator, assert, log, timer, and file IO functions stay in `boxddd_sys::ffi`; length-unit and stall-threshold tuning is exposed only through `boxddd::raw` with validation and process-global docs.
- Raw `void*` user data is not a typed Rust ownership mechanism. Storage and retrieval live behind `boxddd::raw` `unsafe fn try_*_raw_user_data` functions; event snapshots may expose `raw_user_data` pointer values but never typed references.
- `CompoundBytes` is an owner for Box3D-created compound allocations only. Its byte slice is for inspection or caller-side copying, not a stable safe deserialization format, and there is no safe `from_slice` or `from_vec` path for arbitrary bytes.
- `World`, native resources, dynamic trees, recording, and replay player types remain single-owner and are not made `Send` or `Sync`.

## Deferred Areas

No vendored upstream `B3_API` symbols are currently classified as `deferred`.
Future upstream additions may reintroduce this bucket temporarily, but any deferred entry must carry a specific implementation-time rationale in `boxddd/tests/fixtures/api_coverage_symbols.txt`.


## How To Update Coverage

1. Add or change the safe/raw implementation.
2. Update `boxddd/tests/fixtures/api_coverage_symbols.txt` so the symbol status matches the public contract.
3. Update this document when the status counts or policy language change.
4. Run `cargo nextest run -p boxddd --test api_coverage`.

The coverage test is deliberately structural. It proves every upstream public symbol is classified, not that every `safe` entry has ideal API ergonomics. API quality is still enforced by the implementation tests, docs, and examples for each wrapper module.
