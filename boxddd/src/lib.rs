#![allow(rustdoc::broken_intra_doc_links)]
//! Safe, ergonomic Rust bindings for Box3D.
//!
//! This first slice intentionally mirrors the proven shape of `boxdd`: raw Box3D ids are wrapped
//! in crate-owned types, temporary C definitions are exposed through builders, and raw interop is
//! explicit through `from_raw` / `into_raw`.

pub mod body;
pub mod core {
    pub(crate) mod box3d_lock;
    pub(crate) mod callback_state;
    pub(crate) mod debug_checks;
    pub(crate) mod ffi_vec;
}
pub mod error;
pub mod shapes;
pub mod types;
pub mod world;

pub use body::{BodyDef, BodyDefBuilder, BodyType};
pub use error::{ApiError, ApiResult, Error, Result};
pub use shapes::{BoxHull, ShapeDef, ShapeDefBuilder, Sphere, SurfaceMaterial};
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
