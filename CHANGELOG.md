# Changelog

This project contains three crates:

- `boxddd`: safe, ergonomic Rust wrapper over the Box3D C API.
- `boxddd-sys`: low-level FFI bindings plus vendored Box3D sources.
- `bevy_boxddd`: Bevy integration for authoring and visualizing Box3D scenes.

The format is based on Keep a Changelog, and this project follows Semantic Versioning.

## [Unreleased]

### Changed

- GitHub Pages WASM demos now default to the size-focused `wasm-release` profile, run `wasm-opt -Oz` when Binaryen is available, and show byte-level download progress while loading the Box3D provider and Bevy wasm assets.
- `generate-pages` now owns the Bevy testbed entry page and loader script, so `validate-pages` catches stale runtime markup before publishing.

### Migration Notes

- Pages builders that need a different Rust profile can set `BOXDDD_PAGES_WASM_PROFILE=debug`, `release`, or `wasm-release`; set `BOXDDD_PAGES_WASM_OPT=0` to skip optional `wasm-opt` post-processing.

## [0.2.0] - 2026-07-06

This release focuses on browser demo support, Bevy teaching examples, debug draw renderer integration, release preflight coverage, and the first breaking API cleanup after `0.1.0`.

### Added

- Browser demo pages now expose direct Bevy + egui examples from the shared `testbed_3d` scene registry at <https://frankorz.com/boxddd/>.
- The Bevy testbed now includes `boxddd` showcase scenes for Query Lab, Debug Draw Inspector, Material Lab, and Stats Dashboard.
- Query Lab visualizes ray casts, AABB overlaps, sphere shape casts, and capsule mover casts from one Bevy scene; browser provider mode currently supports the ray and AABB visitor paths and labels the remaining visitor paths as unavailable.
- Added a `stats_profile` core example and a Bevy Stats Dashboard scene for world counters, awake body counts, capacity, and per-step profile diagnostics.
- `bevy_boxddd` now exposes Bevy math adapters and prelude extension traits for converting Bevy `Vec3`, `Quat`, and `Transform` values to and from `boxddd` math types.
- Debug draw can now collect lifecycle-aware frames with persistent shape assets, shape create/destroy events, and diagnostics, making renderer integration more practical than consuming command lists alone.
- The Bevy testbed now covers more official Box3D teaching scenarios, including dominoes, arch stacks, wind fields, ragdoll-style joint chains, and cylinder stacks.
- The official Box3D sample matrix is now case-level. Every vendored upstream sample registration, including `Replay / Viewer`, is classified as a faithful port, teaching adaptation, test-only proof, deferred case, or upstream reference.
- `bevy_boxddd` now supports procedural cylinder hull colliders through `HullDescriptor::cylinder` and `Collider::cylinder_hull`.

### Changed

- The root GitHub Pages URL is now the examples index rather than a marketing homepage.
- Example pages and the Bevy testbed now label official Box3D sample coverage separately from `boxddd` showcase entries.
- README and example docs now describe sample support as case-level tracking instead of implying every official sample is a one-to-one clone.
- Release checks now catch stale example pages, official sample matrix drift, and package omissions before publishing.

### Fixed

- WASM provider-mode debug draw can bridge Box3D debug callbacks for the browser demo bundle instead of reporting all debug draw collection as unsupported.
- WASM provider-mode world ray-cast and AABB-overlap visitors now work in browser demos instead of reporting misleading zero-hit results.
- The browser demo path now handles Box3D timer portability correctly.
- Procedural cylinder hull validation now rejects invalid side counts outside Box3D's supported `3..=32` range before reaching FFI.

### Migration Notes

- `DebugDrawCommand::Shape { shape, .. }` has changed to `DebugDrawCommand::Shape { handle, .. }`. Look up owned geometry through `DebugDrawFrame` events and cached `DebugShapeAsset` values. The old `boxddd::DebugShape` metadata type remains as a migration aid for stored metadata, but it is no longer emitted by frame commands.
- Prefer `World::debug_draw_frame`, `try_debug_draw_frame`, or `try_debug_draw_frame_into` when a renderer needs stable shape geometry across frames. The older command collection helpers remain for simpler command-only consumers.
- `Error::ProviderCallbackFailed` is a new public error variant. Exhaustive matches on `boxddd::Error` need a new arm; existing variant order is otherwise preserved for compatibility.
- `HullDescriptor` gained the `Cylinder` variant and is now non-exhaustive. Exhaustive matches in `bevy_boxddd` apps need to add a wildcard arm.

## [0.1.0] - 2026-07-04

### Added

- Initial Rust binding workspace for Box3D `v0.1.0`, including `boxddd-sys`, `boxddd`, and `bevy_boxddd`.
- `boxddd-sys` builds vendored Box3D C sources by default and uses pregenerated bindings for normal builds, including both single-precision and double-precision binding sets.
- `boxddd` provides a safe Rust API for worlds, bodies, shapes, joints, queries, events, debug draw, task-system callbacks, recording, and replay.
- Shape support covers spheres, capsules, hulls, transformed hulls, meshes, height fields, and compounds, with Rust-owned resources kept alive for native shape lifetimes.
- Collision and query helpers cover world/body/shape queries, allocation-aware `*_into` APIs, visitor callbacks, mover helpers, shape casts, time of impact, manifolds, and related geometry utilities.
- Event APIs support owned snapshots, reusable buffers, zero-copy closure-scoped views, and explicit raw escape hatches for bodies, contacts, sensors, and joints.
- Callback APIs cover custom filtering, pre-solve, friction mixing, restitution mixing, debug draw, and task scheduling with Rust panic containment across the C ABI.
- Optional interop features support `mint`, `glam`, `nalgebra`, `cgmath`, and `serde` for crate-owned value types.
- `bevy_boxddd` adds Bevy 0.19 ECS components, fixed-step systems, physics messages, query helpers, optional debug gizmos, optional physics picking, and windowed teaching examples.
- Example coverage includes core headless examples, an egui debug viewer, async/threading examples, math interop examples, and a switchable Bevy 3D testbed.

### Platform Support

- Native Windows, Linux, and macOS are the supported runtime targets for this release.
- Normal builds need a platform C compiler for the vendored Box3D C sources, but do not need CMake, LLVM, libclang, or bindgen.
- WASM support is experimental: compile-only and smoke-test paths exist, but browser apps and Bevy Web are not yet supported runtime targets.

### Known Boundaries

- `World`, native resources, dynamic trees, recordings, and replay players are intentionally `!Send` and `!Sync`.
- Raw `void*` user data, process-global hooks, file helpers, and selected diagnostics stay behind explicit raw APIs or `boxddd_sys::ffi`.
- Mobile targets are compile-only checks today, not supported runtime targets.
