#![allow(rustdoc::broken_intra_doc_links)]
//! Safe, ergonomic Rust bindings for Box3D.
//!
//! This first slice intentionally mirrors the proven shape of `boxdd`: raw Box3D ids are wrapped
//! in crate-owned types, temporary C definitions are exposed through builders, and raw interop is
//! explicit through `from_raw` / `into_raw`.

pub mod body;
pub mod error;
pub mod shapes;
pub mod types;
pub mod world;

pub use body::{BodyDef, BodyDefBuilder, BodyType};
pub use error::{Error, Result};
pub use shapes::{BoxHull, ShapeDef, ShapeDefBuilder, Sphere, SurfaceMaterial};
pub use types::{BodyId, Quat, ShapeId, Transform, Vec3, Version};
pub use world::{
    World, WorldDef, WorldDefBuilder, allocated_byte_count, is_double_precision, version,
};
