# Changelog

This project contains three crates:

- `boxddd`: safe, ergonomic Rust wrapper over the Box3D C API.
- `boxddd-sys`: low-level FFI bindings plus vendored Box3D sources.
- `bevy_boxddd`: Bevy integration for authoring and visualizing Box3D scenes.

The format is based on Keep a Changelog, and this project follows Semantic Versioning.

## [Unreleased]

### Changed

- `DebugDraw::draw_shape` now returns `()` instead of `bool` because Box3D `0.1.0` does not consume the native `DrawShapeFcn` return value during world debug drawing.

### Documentation

- Documented the current Box3D public API coverage boundary: 538 safe wrappers, 36 raw interop entries, 4 intentionally omitted entries, and 0 deferred entries.
- Expanded FFI lifetime audit notes and tests for event views, debug draw callbacks, and material-mix callback containment.
- Polished README quick-start guidance and removed manual prose line wrapping from README/CHANGELOG.

## [boxddd 0.1.0] - 2026-07-02

### Added

- Initial safe Box3D wrapper with crate-owned math/id/value types and explicit raw interop.
- Safe `World`/`Body` runtime APIs for creation, stepping, transforms, tuning, counters, profiles, and attached resource enumeration.
- Shape creation and runtime APIs for spheres, capsules, hulls, meshes, height fields, compounds, filters, materials, AABBs, mass data, and event toggles.
- Standalone collision helpers for mass/AABB computation, overlap, ray casts, shape casts, and local manifolds.
- Allocation-aware query APIs, including owned `Vec` results, `*_into` reusable buffers, and visitor callbacks.
- Event snapshots/views for bodies, contacts, sensors, and joints.
- Safe callbacks for custom filtering, pre-solve, friction mixing, and restitution mixing, with panic containment across the C ABI.
- Debug draw adapters and collected command buffers.
- Typed joint definitions and runtime APIs for parallel, distance, filter, motor, prismatic, revolute, spherical, weld, and wheel joints.
- Recording and replay APIs for deterministic validation, frame stepping, seek/restart, query metadata, and replay debug drawing.
- Optional `mint`, `glam`, `nalgebra`, `cgmath`, and `serde` support for crate-owned value types.
- Release docs, example catalog, upstream API parity matrix, and CI workflow.

### Notes

- The crate is intentionally a staged `0.x` release. Some upstream Box3D APIs remain raw-only or intentionally omitted when the safe ownership/threading model should not expose them as ordinary safe APIs.
- `World`, native resources, and replay players are `!Send`/`!Sync`; safe task-system callbacks are deferred.

## [boxddd-sys 0.1.0] - 2026-07-02

### Added

- Vendored Box3D C sources built by default with `cc`.
- Pregenerated default and double-precision bindings so normal builds do not require libclang.
- Optional `bindgen` refresh path via `BOXDDD_SYS_FORCE_BINDGEN=1`.
- Native build features for `double-precision`, `disable-simd`, and `validate`.
