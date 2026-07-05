#![allow(rustdoc::broken_intra_doc_links)]
//! Safe, ergonomic Rust bindings for Box3D.
//!
//! The crate wraps the primary Box3D simulation APIs with Rust-owned value types, builders,
//! recoverable `try_*` methods, and callback panic containment.
//! Low-level process-global hooks and raw `void*` user data stay outside the ordinary safe API;
//! APIs with an explicit policy live under `boxddd::raw` and are not re-exported by the prelude.
//!
//! See the repository's `docs/api-coverage.md` for the tested upstream API coverage inventory.

#[doc = "Body definitions, body types, and body builder types."]
pub mod body;
#[doc = "World callback adapters for custom filtering and material mixing."]
pub mod callbacks;
#[doc = "Standalone collision, cast, overlap, distance, manifold, and plane helpers."]
pub mod collision;
mod core {
    pub(crate) mod box3d_lock;
    pub(crate) mod callback_state;
    pub(crate) mod debug_checks;
    pub(crate) mod ffi_vec;
    pub(crate) mod material_mix_registry;
    pub(crate) mod task_system;
    pub(crate) mod wasm;
}
#[doc = "Collected debug draw commands and debug draw option types."]
pub mod debug_draw;
#[doc = "Standalone broad-phase dynamic tree wrapper."]
pub mod dynamic_tree;
#[doc = "Error and result types returned by the safe wrapper."]
pub mod error;
#[doc = "Body, contact, joint, and sensor event snapshots read from a world."]
pub mod events;
#[cfg(any(
    feature = "mint",
    feature = "glam",
    feature = "cgmath",
    feature = "nalgebra"
))]
mod interop;
#[doc = "Joint definitions and joint-specific world APIs."]
pub mod joints;
#[doc = "Common imports for applications using `boxddd`."]
pub mod prelude;
#[doc = "World, body, shape, and tree query result types."]
pub mod query;
#[doc = "Explicit raw interop boundary for APIs that expose native Box3D concepts."]
pub mod raw;
#[doc = "Recording and replay support for Box3D simulation traces."]
pub mod recording;
#[doc = "Shape descriptors, native resource owners, and shape data builders."]
pub mod shapes;
#[doc = "Value types for ids, math, contacts, stats, filters, and transforms."]
pub mod types;
#[doc = "World creation, stepping, body APIs, shape APIs, and global metadata."]
pub mod world;

pub use body::{BodyDef, BodyDefBuilder, BodyType};
pub use callbacks::MaterialMixInput;
pub use collision::{
    BoxCastInput, CastOutput, CollisionPlane, DistanceInput, DistanceOutput, LocalManifold,
    PlaneSolverResult, RayCastInput, ShapeCastInput, ShapeCastPairInput, ShapeProxy, Sweep,
    TimeOfImpactInput, TimeOfImpactOutput, TimeOfImpactState, clip_vector,
    collide_capsule_and_sphere, collide_capsule_and_triangle, collide_capsules,
    collide_hull_and_capsule, collide_hull_and_sphere, collide_hull_and_triangle, collide_hulls,
    collide_sphere_and_triangle, collide_spheres, compute_capsule_aabb, compute_capsule_mass,
    compute_compound_aabb, compute_height_field_aabb, compute_hull_aabb, compute_hull_mass,
    compute_mesh_aabb, compute_sphere_aabb, compute_sphere_mass, get_sweep_transform,
    overlap_capsule, overlap_compound, overlap_height_field, overlap_hull, overlap_mesh,
    overlap_sphere, ray_cast_capsule, ray_cast_compound, ray_cast_height_field,
    ray_cast_hollow_sphere, ray_cast_hull, ray_cast_mesh, ray_cast_sphere, shape_cast_capsule,
    shape_cast_compound, shape_cast_height_field, shape_cast_hull, shape_cast_mesh,
    shape_cast_pair, shape_cast_sphere, shape_distance, solve_planes, sweep_transform,
    time_of_impact,
};
pub use core::task_system::{TaskSystem, TaskSystemStats};
pub use debug_draw::{
    DebugCompoundChild, DebugDraw, DebugDrawCommand, DebugDrawDiagnostic, DebugDrawFrame,
    DebugDrawOptions, DebugHullFace, DebugMesh, DebugMeshTriangle, DebugShapeAsset,
    DebugShapeEvent, DebugShapeGeometry, DebugShapeHandle, HexColor,
};
pub use dynamic_tree::{
    DynamicTree, DynamicTreeBoxCastHit, DynamicTreeCastControl, DynamicTreeClosestHit,
    DynamicTreeClosestResult, DynamicTreeFilter, DynamicTreeHit, DynamicTreeProxy,
    DynamicTreeProxyId, DynamicTreeRayCastHit,
};
pub use error::{ApiError, ApiResult, Error, Result};
pub use events::{
    BodyMoveEvent, BodyMoveIter, ContactBeginIter, ContactBeginTouch, ContactBeginTouchEvent,
    ContactEndIter, ContactEndTouch, ContactEndTouchEvent, ContactEvents, ContactHit,
    ContactHitEvent, ContactHitIter, JointEvent, JointEventIter, SensorBeginIter, SensorBeginTouch,
    SensorBeginTouchEvent, SensorEndIter, SensorEndTouch, SensorEndTouchEvent, SensorEvents,
};
pub use joints::{
    DistanceJointDef, FilterJointDef, JointTuning, JointType, MotorJointDef, ParallelJointDef,
    PrismaticJointDef, RevoluteJointDef, SphericalJointDef, WeldJointDef, WheelJointDef,
};
pub use query::{
    BodyCastHit, BodyClosestPoint, MoverPlane, QueryFilter, QueryHit, RayHit, ShapeRayHit,
    TreeStats,
};
pub use recording::{
    RecPlayer, RecPlayerInfo, RecQueryHit, RecQueryInfo, RecQueryType, Recording, ReplayWorldId,
    validate_replay_bytes,
};
pub use shapes::{
    BoxHull, Capsule, Compound, CompoundBuilder, CompoundBytes, CompoundCapsule, CompoundChild,
    CompoundChildShape, CompoundHull, CompoundMesh, CompoundQueryHit, CompoundSphere,
    HEIGHT_FIELD_HOLE, HeightField, HeightFieldBuilder, Hull, MAX_COMPOUND_MESH_MATERIALS,
    MeshData, MeshDataBuilder, MeshDataOptions, MeshTriangleHit, ScaledBox, ShapeDef,
    ShapeDefBuilder, ShapeHeightField, ShapeHull, ShapeMesh, ShapeType, Sphere, SurfaceMaterial,
};
pub use types::{
    Aabb, BodyId, Capacity, ContactData, ContactId, CosSin, Counters, Filter, JointId, Manifold,
    ManifoldPoint, MassData, Matrix3, MotionLocks, Plane, Pos, Profile, Quat,
    SegmentDistanceResult, ShapeId, Transform, Vec2, Vec3, Version, WorldTransform,
    closest_point_on_segment, compute_cos_sin, deterministic_atan2, is_valid_float, line_distance,
    segment_distance, steiner_inertia,
};
pub use world::{
    ExplosionDef, ExplosionDefBuilder, World, WorldDef, WorldDefBuilder, allocated_byte_count,
    is_double_precision, version,
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

    pub fn task_system_panic_on_enqueue_for_test() -> crate::TaskSystem {
        crate::TaskSystem::__panic_on_enqueue_for_test()
    }

    pub fn task_system_panic_on_task_for_test() -> crate::TaskSystem {
        crate::TaskSystem::__panic_on_task_for_test()
    }

    pub fn task_system_panic_on_finish_for_test() -> crate::TaskSystem {
        crate::TaskSystem::__panic_on_finish_for_test()
    }

    pub fn task_system_check_callback_guard_for_test() -> crate::TaskSystem {
        crate::TaskSystem::__check_callback_guard_for_test()
    }

    pub fn task_system_guard_rejections_for_test(task_system: &crate::TaskSystem) -> usize {
        task_system.__guard_rejections_for_test()
    }
}
