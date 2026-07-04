# Rustdoc Alignment Status

This document tracks whether `boxddd` public rustdoc is semantically aligned with
the vendored Box3D C API documentation. It is intentionally separate from API
coverage: an API can be covered and still have weak or misleading documentation.

## Reference Sources

- `boxddd-sys/third-party/box3d/include/box3d/box3d.h`
- `boxddd-sys/third-party/box3d/include/box3d/types.h`
- `boxddd-sys/third-party/box3d/include/box3d/collision.h`
- `boxddd-sys/third-party/box3d/docs/*.md`

## Alignment Criteria

- Public rustdoc should explain the Rust API behavior, not mechanically restate
  the function name.
- Fallible `try_*` APIs should document what is attempted and the major invalid
  states they report.
- Panicking convenience APIs should document that they panic when Box3D rejects
  the request.
- Callback and event APIs should document transient lifetimes, callback return
  semantics, and panic boundaries.
- FFI-backed resource APIs should document ownership and dependency lifetimes.
- Units and coordinate spaces should be called out when they affect correct use.

## Module Status

| Module | Files | Status | Notes |
| --- | --- | --- | --- |
| Errors | `error.rs` | Aligned | Removed placeholder comments and kept concrete wrapper error semantics. |
| Collision helpers | `collision.rs` | Mostly aligned | Missing-docs clean; proxy docs aligned to GJK/shape-cast terminology. |
| Debug draw | `debug_draw.rs` | Mostly aligned | Callback sink and command docs are useful; still worth checking material/color terminology later. |
| World definition/runtime | `world.rs`, `world/runtime.rs`, `world/creation.rs` | Aligned | Step, gravity, explosion, bounds, profile, counters, simulation toggles, worker count, and creation methods now describe behavior and units. |
| Body API | `world/body_api.rs`, `body.rs` | Mostly aligned | Method-level docs no longer repeat method names; force, transform, mass, query, and list APIs now describe behavior. |
| Shape API | `world/shape_api.rs`, `shapes.rs` | Mostly aligned | Method-level docs now describe material/filter/event/query/geometry behavior; borrowed geometry views document owner lifetime. |
| Query API | `query.rs` | Mostly aligned | World query callback semantics are mostly clear; body/shape query docs need cross-check. |
| Recording/replay | `recording.rs` | Aligned | Recording byte lifetime, active-recording restrictions, replay stepping, divergence, frame queries, and query drawing are documented. |
| Events/callbacks | `events.rs`, `callbacks.rs` | Needs semantic pass | Transient slices and panic boundaries are the key review points. |
| Joints | `joints/*.rs` | Needs semantic pass | Getter docs are passable but should add units/limit/motor semantics where useful. |
| Dynamic tree | `dynamic_tree.rs` | Mostly aligned | Proxy generation and callback semantics are documented; run a final audit later. |
| Task system | `core/task_system.rs` | Mostly aligned | Must preserve the `finishTask` blocking requirement in docs. |

## Related Audits

- `docs/development/ffi-lifetime-audit.md` records the current ownership,
  borrowed-view, callback, recording, compound-byte, and task-system lifetime
  decisions.

## Current Quality Gate

Run these before considering the documentation pass complete:

```powershell
cargo fmt --all --check
cargo rustdoc -p boxddd --all-features -- -D missing_docs
$env:RUSTDOCFLAGS='-D warnings'; cargo doc -p boxddd --all-features --no-deps
```
