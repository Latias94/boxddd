# Changelog

This project contains three crates:

- `boxddd`: safe, ergonomic Rust wrapper over the Box3D C API.
- `boxddd-sys`: low-level FFI bindings plus vendored Box3D sources.
- `bevy_boxddd`: Bevy integration for authoring and visualizing Box3D scenes.

The format is based on Keep a Changelog, and this project follows Semantic Versioning.

## [Unreleased]

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
