# FFI Lifetime Audit

This document tracks ownership and lifetime decisions for the safe `boxddd`
wrapper over the vendored Box3D C API.

## Current Verdict

No immediate token-guard refactor is required for the currently audited core
paths. The binding already uses the main defensive patterns expected for this
kind of C API:

- `World`, `Recording`, `RecPlayer`, `DynamicTree`, and native geometry owners
  are intentionally `!Send` and `!Sync` through `PhantomData<Rc<()>>`.
- Box3D calls are serialized through the crate-global Box3D lock.
- Safe APIs reject calls made from Box3D callbacks through the callback-state
  guard.
- Callback trampolines catch Rust panics and report them back through
  `Error::CallbackPanicked` instead of unwinding across FFI.
- Body/shape resource sidecars keep `MeshData`, `HeightField`, and `Compound`
  alive while Box3D shapes refer to them.
- Recording bytes are unavailable while a recording is attached to a live world.

## Audited Areas

### Event Views

Official Box3D docs say world event buffers are transient and must not be stored.
The Rust API exposes owned snapshots plus `with_*_events_view` closure APIs.

Status: acceptable.

Reasoning:

- The borrowed event views are only constructed inside a closure call.
- A minimal Rust lifetime probe confirmed that `FnOnce(View<'_>) -> T` does not
  allow returning the borrowed view from the closure in safe Rust.
- Raw event slice APIs are marked `unsafe` and document that callers must not
  store the slices or dereference raw `userData` without upholding validity.

Future hardening:

- Consider adding doc-test `compile_fail` examples to lock in the no-escape
  contract for event views.

### Recording Bytes

Official Box3D docs say `b3Recording_GetData` is valid until the buffer is
modified or destroyed. The Rust API returns `&[u8]` tied to `&Recording` and
blocks byte access while the recording is active on a live world.

Status: acceptable.

Reasoning:

- Borrowed bytes prevent mutable use of the same `Recording`.
- `World::start_recording` requires `&mut Recording`, so safe Rust cannot start
  a recording while a borrowed byte slice exists.
- `Recording::drop` stops active recording sessions when the world is still
  valid.
- `World::drop` detaches active recording registry entries for that world.

Future hardening:

- A RAII `RecordingSession<'world, 'recording>` token could make start/stop
  pairing more explicit, but it is not required to make the current API sound.

### Shape Resource Views

`ShapeHull<'_>`, `ShapeMesh<'_>`, and `ShapeHeightField<'_>` borrow native
geometry owned by a live shape/world.

Status: acceptable.

Reasoning:

- The returned views are tied to `&World`.
- Safe Rust cannot mutably destroy or replace the same shape while those views
  are live.
- Resource-backed mesh, height-field, and compound shapes keep their owned Rust
  resources in the `World` sidecar map until shape/body destruction or shape
  replacement.

Future hardening:

- If raw world handles become more prominent in examples, document that manual
  `boxddd_sys` calls can invalidate safe borrowed views and must stay unsafe.

### Compound Byte Conversion

Box3D converts a compound to bytes by returning the same allocation after
scrubbing internal pointers. It converts bytes back to a compound by mutating the
same allocation in place.

Status: acceptable.

Reasoning:

- `Compound::into_bytes` uses `ManuallyDrop` so the native allocation is not
  freed during transfer to `CompoundBytes`.
- `CompoundBytes::drop` calls `b3DestroyCompound` on the same allocation, which
  matches the upstream representation.
- `CompoundBytes::into_compound` forgets `self` only after successful conversion,
  transferring ownership back to `Compound`.

### Callback Contexts

World callback contexts are boxed and kept inside `WorldCallbacks`; raw Box3D
callback pointers are cleared before the world is destroyed.

Status: acceptable.

Reasoning:

- Context boxes outlive the registered callbacks because they are owned by the
  `World`.
- Callback replacement is performed while the Box3D global lock is held.
- `World::drop` clears raw callbacks before destroying the native world.
- Material-mix callbacks use registry slots and release them when both callbacks
  are cleared.

### Task System

Box3D requires `finishTask` to block until a task completes. The current
`TaskSystem::blocking_threads` adapter joins thread handles in `finishTask`.

Status: acceptable.

Reasoning:

- The task-system context is an `Arc` kept alive by `World`.
- Worker threads temporarily clone the `Arc` from the raw pointer before running.
- Panics in enqueue, task execution, or finish are contained and reported after
  `World::try_step`.

Future hardening:

- Keep the scheduler deliberately conservative. Do not adapt Bevy or async task
  pools unless they can provide the exact blocking `finishTask` semantics.

## Token Guard Decision

Do not introduce a general token guard yet.

Recommended future token candidates:

- `RecordingSession<'world, 'recording>` if start/stop ergonomics become
  important.
- `WorldEvents<'world>` if we decide to expose event iterators outside closures.
- `ShapeGeometryView<'world>` if we expose more borrowed native shape internals.

The current safe API should keep using closure-scoped borrowed views unless a
new public API needs to return borrowed transient data directly.
