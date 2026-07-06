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
| `safe` | 538 | world lifecycle and stepping, body runtime, body/shape scoped queries, dynamic tree, mover collision, explosions, shape creation and runtime introspection, compound/mesh/height-field authoring, query, byte ownership transfer, and readback, shape event/contact/sensor readback, contact data, hull cloning and box scaling, advanced standalone collision, joints, events, world queries, debug draw, recording/replay, safe recording load, deterministic math helpers, core math/value validation |
| `raw` | 35 | allocator/assert/log hooks, timers/sleep/hash, low-level file IO, dump helpers, explicit `boxddd::raw` user data and process-global scalar tuning, file-backed dynamic tree or height-field helpers, native recording save helper, low-level debug graph color helper |
| `omitted` | 5 | global world-count diagnostics and redundant body/shape/joint world-handle getters that do not fit the safe ownership model |
| `deferred` | 0 | no current upstream `B3_API` symbols remain in the deferred bucket |

Counts are intentionally checked by tests instead of maintained only in prose. When the fixture changes, update this snapshot in the same commit.

Safe wrapper coverage is 538 of 578 public `B3_API` symbols, or about 93%.
The remaining 40 symbols are not unplanned API gaps: they are classified as
explicit raw interop or intentionally omitted from the safe ownership model.

## Current Non-Safe Boundary

| Boundary | Count | Why this is not ordinary safe API |
|---|---:|---|
| Base hooks, timers, sleep, hash, and binary file helpers | 12 | Upstream treats these as platform, diagnostics, determinism, or file helpers; allocator/assert/log hooks are process-global callbacks. |
| Raw `void*` user data on worlds, bodies, shapes, and joints | 8 | Untyped pointers cannot express Rust ownership or aliasing; access stays under `boxddd::raw` `unsafe` functions with handle validation. |
| World dump/debug file helpers | 4 | Debug dump output is file/diagnostic oriented rather than a stable safe data model. |
| Native recording save helper and debug-shape callback hook | 2 | `Recording::load_from_file` is safe and `Recording::save_to_file` writes validated bytes through Rust IO. The upstream native save helper and raw debug-shape callback installation remain low-level interop. |
| Dynamic-tree and height-field file helpers | 4 | File-backed native helpers do not provide a Rust-owned resource contract. |
| Process-global tuning | 4 | Length-unit and stall-threshold tuning is process-global state and stays in `boxddd::raw` with validation and docs. |
| Low-level graph-color diagnostic helper | 1 | Useful for native diagnostics, not part of the public safe simulation model. |
| Omitted diagnostics and redundant world-handle getters | 5 | Global world-count diagnostics and body/shape/joint world getters duplicate ownership already represented by `World`. |

## Safe Boundary Rules

- Safe APIs must validate handles and scalar/vector inputs before crossing FFI when validation is possible.
- Safe APIs must not expose borrowed Box3D-owned memory beyond the owning `World` or native resource lifetime.
- Callback APIs must follow the existing callback guard pattern: do not unwind through C, return `Error::CallbackPanicked` where panic containment is possible, and return `Error::UnsupportedOnWasm` for provider-mode WASM callback paths that are not sound yet, including standalone mesh, height-field, and compound geometry visitors until their bridges land.
- Process-global hooks are not ordinary safe convenience APIs. Allocator, assert, log, timer, low-level binary file helpers, native dump helpers, and the upstream native recording save helper stay in `boxddd_sys::ffi`; length-unit and stall-threshold tuning is exposed only through `boxddd::raw` with validation and process-global docs.
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
