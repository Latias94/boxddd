#![allow(rustdoc::broken_intra_doc_links)]
//! Safe, ergonomic Rust bindings for Box3D.
//!
//! This first slice intentionally mirrors the proven shape of `boxdd`: raw Box3D ids are wrapped
//! in crate-owned types, temporary C definitions are exposed through builders, and raw interop is
//! explicit through `from_raw` / `into_raw`.

pub mod body;
pub mod collision;
pub mod core {
    pub(crate) mod box3d_lock;
    pub(crate) mod callback_state;
    pub(crate) mod debug_checks;
    pub(crate) mod ffi_vec;
}
pub mod error;
pub mod query;
pub mod shapes;
pub mod types;
pub mod world;

pub use body::{BodyDef, BodyDefBuilder, BodyType};
pub use collision::{
    CastOutput, LocalManifold, RayCastInput, ShapeCastInput, ShapeProxy, collide_capsules,
    collide_spheres, compute_capsule_aabb, compute_capsule_mass, compute_compound_aabb,
    compute_height_field_aabb, compute_hull_aabb, compute_hull_mass, compute_mesh_aabb,
    compute_sphere_aabb, compute_sphere_mass, overlap_capsule, overlap_compound,
    overlap_height_field, overlap_hull, overlap_mesh, overlap_sphere, ray_cast_capsule,
    ray_cast_compound, ray_cast_height_field, ray_cast_hollow_sphere, ray_cast_hull, ray_cast_mesh,
    ray_cast_sphere, shape_cast_capsule, shape_cast_hull, shape_cast_sphere,
};
pub use error::{ApiError, ApiResult, Error, Result};
pub use query::{QueryFilter, QueryHit, RayHit, TreeStats};
pub use shapes::{
    BoxHull, Capsule, Compound, HeightField, Hull, MeshData, ShapeDef, ShapeDefBuilder, ShapeType,
    Sphere, SurfaceMaterial,
};
pub use types::{
    Aabb, BodyId, Capacity, ContactData, ContactId, Counters, Filter, JointId, Manifold,
    ManifoldPoint, MassData, Matrix3, MotionLocks, Plane, Pos, Profile, Quat, ShapeId, Transform,
    Vec2, Vec3, Version, WorldTransform, is_valid_float,
};
pub use world::{
    World, WorldDef, WorldDefBuilder, allocated_byte_count, is_double_precision, version,
};

#[doc(hidden)]
pub mod __private {
    pub struct CallbackGuard {
        _guard: crate::core::callback_state::CallbackGuard,
    }

    pub fn enter_callback_guard_for_test() -> CallbackGuard {
        CallbackGuard {
            _guard: crate::core::callback_state::CallbackGuard::enter(),
        }
    }
}
