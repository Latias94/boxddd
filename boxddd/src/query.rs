#![cfg_attr(all(target_arch = "wasm32", boxddd_wasm_provider), allow(dead_code))]

use crate::collision::{ShapeCastInput, ShapeProxy};
use crate::core::{box3d_lock, callback_state};
use crate::error::{Error, Result};
use crate::shapes::Capsule;
use crate::types::{Aabb, Plane, Pos, ShapeId, Vec3};
use crate::world::World;
use boxddd_sys::ffi;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Broad-phase traversal statistics returned by Box3D queries.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TreeStats {
    /// Number of broad-phase tree nodes visited.
    pub node_visits: i32,
    /// Number of broad-phase tree leaves visited.
    pub leaf_visits: i32,
}

impl TreeStats {
    /// Converts raw Box3D data into the safe value type.
    #[inline]
    pub const fn from_raw(raw: ffi::b3TreeStats) -> Self {
        Self {
            node_visits: raw.nodeVisits,
            leaf_visits: raw.leafVisits,
        }
    }
}

/// Collision filter applied to world queries and casts.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct QueryFilter {
    /// Category bits assigned to the query.
    pub category_bits: u64,
    /// Mask bits used to accept candidate shapes.
    pub mask_bits: u64,
    /// User-defined query identifier recorded by Box3D replay.
    pub id: u64,
}

impl QueryFilter {
    /// Creates a new value with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the category bits assigned to the query.
    pub fn category_bits(mut self, category_bits: u64) -> Self {
        self.category_bits = category_bits;
        self
    }

    /// Sets the mask bits used to accept candidate shapes.
    pub fn mask_bits(mut self, mask_bits: u64) -> Self {
        self.mask_bits = mask_bits;
        self
    }

    /// Sets the user-defined query id used by recording and replay diagnostics.
    pub fn id(mut self, id: u64) -> Self {
        self.id = id;
        self
    }

    #[inline]
    pub(crate) fn raw(self) -> ffi::b3QueryFilter {
        ffi::b3QueryFilter {
            categoryBits: self.category_bits,
            maskBits: self.mask_bits,
            id: self.id,
            name: std::ptr::null(),
        }
    }
}

impl Default for QueryFilter {
    fn default() -> Self {
        Self {
            category_bits: u64::MAX,
            mask_bits: u64::MAX,
            id: 0,
        }
    }
}

/// Shape reported by an overlap query.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct QueryHit {
    /// Shape that overlapped the query volume.
    pub shape_id: ShapeId,
}

/// Hit reported by a ray or shape cast callback.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RayHit {
    /// Shape hit by the cast.
    pub shape_id: ShapeId,
    /// World-space point of initial intersection.
    pub point: Pos,
    /// Surface normal at the intersection point.
    pub normal: Vec3,
    /// Fraction along the cast translation at the intersection point.
    pub fraction: f32,
    /// Shape or triangle material id reported by Box3D.
    pub user_material_id: u64,
    /// Triangle index for mesh or height-field hits, or `-1` otherwise.
    pub triangle_index: i32,
    /// Child shape index for compound hits, or `-1` otherwise.
    pub child_index: i32,
}

/// Closest hit reported by [`World::cast_ray_closest`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RayClosestHit {
    /// Shape hit by the ray.
    pub shape_id: ShapeId,
    /// World-space point of initial intersection.
    pub point: Pos,
    /// Surface normal at the intersection point.
    pub normal: Vec3,
    /// Fraction along the ray translation at the intersection point.
    pub fraction: f32,
    /// Shape or triangle material id reported by Box3D.
    pub user_material_id: u64,
    /// Triangle index for mesh or height-field hits, or `-1` otherwise.
    pub triangle_index: i32,
    /// Child shape index for compound hits, or `-1` otherwise.
    pub child_index: i32,
    /// Number of broad-phase tree nodes visited.
    pub node_visits: i32,
    /// Number of broad-phase tree leaves visited.
    pub leaf_visits: i32,
}

/// Closest point result for a body-scoped query.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BodyClosestPoint {
    /// Closest world-space point on the body.
    pub point: Vec3,
    /// Distance from the query point to the body.
    pub distance: f32,
}

/// Hit reported by a body-scoped cast.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BodyCastHit {
    /// Shape on the body hit by the cast.
    pub shape_id: ShapeId,
    /// World-space point of initial intersection.
    pub point: Pos,
    /// Surface normal at the intersection point.
    pub normal: Vec3,
    /// Fraction along the cast translation at the intersection point.
    pub fraction: f32,
    /// Shape or triangle material id reported by Box3D.
    pub user_material_id: u64,
    /// Triangle index for mesh or height-field hits, or `-1` otherwise.
    pub triangle_index: i32,
    /// Narrow-phase iteration count reported by Box3D.
    pub iterations: i32,
}

impl BodyCastHit {
    #[inline]
    pub(crate) fn from_raw(raw: ffi::b3BodyCastResult) -> Option<Self> {
        raw.hit.then(|| Self {
            shape_id: ShapeId::from_raw(raw.shapeId),
            point: Pos::from_raw(raw.point),
            normal: Vec3::from_raw(raw.normal),
            fraction: raw.fraction,
            user_material_id: raw.userMaterialId,
            triangle_index: raw.triangleIndex,
            iterations: raw.iterations,
        })
    }
}

/// Result returned by a ray cast against a single shape.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ShapeRayHit {
    /// World-space point of initial intersection.
    pub point: Pos,
    /// Surface normal at the intersection point.
    pub normal: Vec3,
    /// Fraction along the cast translation at the intersection point.
    pub fraction: f32,
    /// Triangle index for mesh or height-field hits, or `-1` otherwise.
    pub triangle_index: i32,
    /// Child shape index for compound hits, or `-1` otherwise.
    pub child_index: i32,
    /// Material slot index reported by the hit, when applicable.
    pub material_index: i32,
    /// Narrow-phase iteration count reported by Box3D.
    pub iterations: i32,
}

impl ShapeRayHit {
    #[inline]
    pub(crate) fn from_raw(raw: ffi::b3WorldCastOutput) -> Option<Self> {
        raw.hit.then(|| Self {
            point: Pos::from_raw(raw.point),
            normal: Vec3::from_raw(raw.normal),
            fraction: raw.fraction,
            triangle_index: raw.triangleIndex,
            child_index: raw.childIndex,
            material_index: raw.materialIndex,
            iterations: raw.iterations,
        })
    }
}

/// Contact plane gathered for a capsule character mover.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MoverPlane {
    /// Shape that produced the plane.
    pub shape_id: ShapeId,
    /// Plane that constrains the mover at the queried position.
    pub plane: Plane,
    /// World-space contact point reported by Box3D.
    pub point: Vec3,
}

impl MoverPlane {
    #[inline]
    pub(crate) fn from_raw(shape_id: ffi::b3ShapeId, raw: ffi::b3PlaneResult) -> Self {
        Self {
            shape_id: ShapeId::from_raw(shape_id),
            plane: Plane::from_raw(raw.plane),
            point: Vec3::from_raw(raw.point),
        }
    }
}

impl World {
    /// Collects every shape whose bounds overlap `aabb`.
    pub fn overlap_aabb(&self, aabb: Aabb, filter: QueryFilter) -> Result<Vec<QueryHit>> {
        let mut out = Vec::new();
        self.overlap_aabb_into(aabb, filter, &mut out)?;
        Ok(out)
    }

    /// Writes every shape whose bounds overlap `aabb` into `out`, clearing it first.
    pub fn overlap_aabb_into(
        &self,
        aabb: Aabb,
        filter: QueryFilter,
        out: &mut Vec<QueryHit>,
    ) -> Result<TreeStats> {
        out.clear();
        self.visit_overlap_aabb(aabb, filter, |shape_id| {
            out.push(QueryHit { shape_id });
            true
        })
    }

    /// Visits shapes whose bounds overlap `aabb`.
    ///
    /// Returning `false` from `visitor` terminates traversal early.
    pub fn visit_overlap_aabb<F>(
        &self,
        aabb: Aabb,
        filter: QueryFilter,
        visitor: F,
    ) -> Result<TreeStats>
    where
        F: FnMut(ShapeId) -> bool,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = (aabb, filter, visitor);
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            aabb.validate()?;
            let mut ctx = OverlapContext {
                visitor,
                panicked: false,
            };
            let _guard = box3d_lock::lock();
            self.check_world_valid_locked()?;
            let stats = unsafe {
                ffi::b3World_OverlapAABB(
                    self.raw(),
                    aabb.into_raw(),
                    filter.raw(),
                    Some(overlap_trampoline::<F>),
                    (&mut ctx as *mut OverlapContext<_>).cast(),
                )
            };
            if ctx.panicked {
                Err(Error::CallbackPanicked)
            } else {
                Ok(TreeStats::from_raw(stats))
            }
        }
    }

    /// Collects shapes overlapping `proxy` placed at `origin`.
    pub fn overlap_shape(
        &self,
        origin: impl Into<Pos>,
        proxy: &ShapeProxy,
        filter: QueryFilter,
    ) -> Result<Vec<QueryHit>> {
        let mut out = Vec::new();
        self.overlap_shape_into(origin, proxy, filter, &mut out)?;
        Ok(out)
    }

    /// Writes shapes overlapping `proxy` placed at `origin` into `out`, clearing it first.
    pub fn overlap_shape_into(
        &self,
        origin: impl Into<Pos>,
        proxy: &ShapeProxy,
        filter: QueryFilter,
        out: &mut Vec<QueryHit>,
    ) -> Result<TreeStats> {
        out.clear();
        self.visit_overlap_shape(origin, proxy, filter, |shape_id| {
            out.push(QueryHit { shape_id });
            true
        })
    }

    /// Visits shapes overlapping `proxy` placed at `origin`.
    ///
    /// Returning `false` from `visitor` terminates traversal early.
    pub fn visit_overlap_shape<F>(
        &self,
        origin: impl Into<Pos>,
        proxy: &ShapeProxy,
        filter: QueryFilter,
        visitor: F,
    ) -> Result<TreeStats>
    where
        F: FnMut(ShapeId) -> bool,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = (origin, proxy, filter, visitor);
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let origin = origin.into().validate()?;
            let raw_proxy = proxy.raw();
            let mut ctx = OverlapContext {
                visitor,
                panicked: false,
            };
            let _guard = box3d_lock::lock();
            self.check_world_valid_locked()?;
            let stats = unsafe {
                ffi::b3World_OverlapShape(
                    self.raw(),
                    origin.into_raw(),
                    &raw_proxy,
                    filter.raw(),
                    Some(overlap_trampoline::<F>),
                    (&mut ctx as *mut OverlapContext<_>).cast(),
                )
            };
            if ctx.panicked {
                Err(Error::CallbackPanicked)
            } else {
                Ok(TreeStats::from_raw(stats))
            }
        }
    }

    /// Collects hits from a ray cast through the world.
    pub fn cast_ray(
        &self,
        origin: impl Into<Pos>,
        translation: impl Into<Vec3>,
        filter: QueryFilter,
    ) -> Result<Vec<RayHit>> {
        let mut out = Vec::new();
        self.cast_ray_into(origin, translation, filter, &mut out)?;
        Ok(out)
    }

    /// Writes ray-cast hits into `out`, clearing it first.
    pub fn cast_ray_into(
        &self,
        origin: impl Into<Pos>,
        translation: impl Into<Vec3>,
        filter: QueryFilter,
        out: &mut Vec<RayHit>,
    ) -> Result<TreeStats> {
        out.clear();
        self.visit_cast_ray(origin, translation, filter, |hit| {
            out.push(hit);
            1.0
        })
    }

    /// Visits hits from a ray cast through the world.
    ///
    /// The callback follows Box3D ray-cast semantics: return `-1.0` to ignore the hit and continue,
    /// `0.0` to terminate, the hit fraction to clip the ray for closest-hit behavior, or `1.0` to
    /// continue without clipping. Non-finite returns are treated as termination.
    pub fn visit_cast_ray<F>(
        &self,
        origin: impl Into<Pos>,
        translation: impl Into<Vec3>,
        filter: QueryFilter,
        visitor: F,
    ) -> Result<TreeStats>
    where
        F: FnMut(RayHit) -> f32,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = (origin, translation, filter, visitor);
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let origin = origin.into().validate()?;
            let translation = translation.into().validate()?;
            let mut ctx = CastContext {
                visitor,
                panicked: false,
            };
            let _guard = box3d_lock::lock();
            self.check_world_valid_locked()?;
            let stats = unsafe {
                ffi::b3World_CastRay(
                    self.raw(),
                    origin.into_raw(),
                    translation.into_raw(),
                    filter.raw(),
                    Some(cast_trampoline::<F>),
                    (&mut ctx as *mut CastContext<_>).cast(),
                )
            };
            if ctx.panicked {
                Err(Error::CallbackPanicked)
            } else {
                Ok(TreeStats::from_raw(stats))
            }
        }
    }

    /// Returns the closest hit from a ray cast through the world.
    ///
    /// This is the Box3D convenience path for closest-hit queries. It does not provide a callback
    /// for custom per-hit filtering.
    pub fn cast_ray_closest(
        &self,
        origin: impl Into<Pos>,
        translation: impl Into<Vec3>,
        filter: QueryFilter,
    ) -> Result<Option<RayClosestHit>> {
        callback_state::check_not_in_callback()?;
        let origin = origin.into().validate()?;
        let translation = translation.into().validate()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let raw = unsafe {
            ffi::b3World_CastRayClosest(
                self.raw(),
                origin.into_raw(),
                translation.into_raw(),
                filter.raw(),
            )
        };
        if raw.hit {
            Ok(Some(RayClosestHit {
                shape_id: ShapeId::from_raw(raw.shapeId),
                point: Pos::from_raw(raw.point),
                normal: Vec3::from_raw(raw.normal),
                fraction: raw.fraction,
                user_material_id: raw.userMaterialId,
                triangle_index: raw.triangleIndex,
                child_index: raw.childIndex,
                node_visits: raw.nodeVisits,
                leaf_visits: raw.leafVisits,
            }))
        } else {
            Ok(None)
        }
    }

    /// Collects hits from sweeping a shape proxy through the world.
    pub fn cast_shape(
        &self,
        origin: impl Into<Pos>,
        input: ShapeCastInput,
        filter: QueryFilter,
    ) -> Result<Vec<RayHit>> {
        let mut out = Vec::new();
        self.cast_shape_into(origin, input, filter, &mut out)?;
        Ok(out)
    }

    /// Writes shape-cast hits into `out`, clearing it first.
    pub fn cast_shape_into(
        &self,
        origin: impl Into<Pos>,
        input: ShapeCastInput,
        filter: QueryFilter,
        out: &mut Vec<RayHit>,
    ) -> Result<TreeStats> {
        out.clear();
        self.visit_cast_shape(origin, input, filter, |hit| {
            out.push(hit);
            1.0
        })
    }

    /// Visits hits from sweeping a shape proxy through the world.
    ///
    /// The callback uses the same control return values as [`Self::visit_cast_ray`].
    pub fn visit_cast_shape<F>(
        &self,
        origin: impl Into<Pos>,
        input: ShapeCastInput,
        filter: QueryFilter,
        visitor: F,
    ) -> Result<TreeStats>
    where
        F: FnMut(RayHit) -> f32,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = (origin, input, filter, visitor);
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let origin = origin.into().validate()?;
            let input = input.validate()?;
            let raw_input = input.raw();
            let mut ctx = CastContext {
                visitor,
                panicked: false,
            };
            let _guard = box3d_lock::lock();
            self.check_world_valid_locked()?;
            let stats = unsafe {
                ffi::b3World_CastShape(
                    self.raw(),
                    origin.into_raw(),
                    &raw_input.proxy,
                    raw_input.translation,
                    filter.raw(),
                    Some(cast_trampoline::<F>),
                    (&mut ctx as *mut CastContext<_>).cast(),
                )
            };
            if ctx.panicked {
                Err(Error::CallbackPanicked)
            } else {
                Ok(TreeStats::from_raw(stats))
            }
        }
    }

    /// Casts a capsule-shaped character mover through the world.
    ///
    /// Returns the safe travel fraction in `[0, 1]`. Use [`Self::collide_mover`] at the final
    /// position to gather contact planes; Box3D's mover cast is for swept motion, not contact
    /// inspection.
    pub fn cast_mover(
        &self,
        origin: impl Into<Pos>,
        mover: &Capsule,
        translation: impl Into<Vec3>,
        filter: QueryFilter,
    ) -> Result<f32> {
        callback_state::check_not_in_callback()?;
        let origin = origin.into().validate()?;
        let translation = translation.into().validate()?;
        mover.validate()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe {
            ffi::b3World_CastMover(
                self.raw(),
                origin.into_raw(),
                mover.raw(),
                translation.into_raw(),
                filter.raw(),
                None,
                std::ptr::null_mut(),
            )
        })
    }

    /// Collects contact planes for a capsule mover at `origin`.
    pub fn collide_mover(
        &self,
        origin: impl Into<Pos>,
        mover: &Capsule,
        filter: QueryFilter,
    ) -> Result<Vec<MoverPlane>> {
        let mut out = Vec::new();
        self.collide_mover_into(origin, mover, filter, &mut out)?;
        Ok(out)
    }

    /// Writes mover contact planes into `out`, clearing it first.
    pub fn collide_mover_into(
        &self,
        origin: impl Into<Pos>,
        mover: &Capsule,
        filter: QueryFilter,
        out: &mut Vec<MoverPlane>,
    ) -> Result<()> {
        out.clear();
        self.visit_collide_mover(origin, mover, filter, |plane| {
            out.push(plane);
            true
        })
    }

    /// Visits contact planes for a capsule mover at `origin`.
    ///
    /// Returning `false` from `visitor` stops plane collection early.
    pub fn visit_collide_mover<F>(
        &self,
        origin: impl Into<Pos>,
        mover: &Capsule,
        filter: QueryFilter,
        visitor: F,
    ) -> Result<()>
    where
        F: FnMut(MoverPlane) -> bool,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = (origin, mover, filter, visitor);
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let origin = origin.into().validate()?;
            mover.validate()?;
            let mut ctx = MoverPlaneContext {
                visitor,
                panicked: false,
            };
            let _guard = box3d_lock::lock();
            self.check_world_valid_locked()?;
            unsafe {
                ffi::b3World_CollideMover(
                    self.raw(),
                    origin.into_raw(),
                    mover.raw(),
                    filter.raw(),
                    Some(mover_plane_trampoline::<F>),
                    (&mut ctx as *mut MoverPlaneContext<_>).cast(),
                )
            };
            if ctx.panicked {
                Err(Error::CallbackPanicked)
            } else {
                Ok(())
            }
        }
    }
}

struct OverlapContext<F> {
    visitor: F,
    panicked: bool,
}

unsafe extern "C" fn overlap_trampoline<F>(
    shape_id: ffi::b3ShapeId,
    context: *mut std::ffi::c_void,
) -> bool
where
    F: FnMut(ShapeId) -> bool,
{
    let _guard = callback_state::CallbackGuard::enter();
    let ctx = unsafe { &mut *context.cast::<OverlapContext<F>>() };
    match catch_unwind(AssertUnwindSafe(|| {
        (ctx.visitor)(ShapeId::from_raw(shape_id))
    })) {
        Ok(keep_going) => keep_going,
        Err(_) => {
            ctx.panicked = true;
            false
        }
    }
}

struct CastContext<F> {
    visitor: F,
    panicked: bool,
}

unsafe extern "C" fn cast_trampoline<F>(
    shape_id: ffi::b3ShapeId,
    point: ffi::b3Pos,
    normal: ffi::b3Vec3,
    fraction: f32,
    user_material_id: u64,
    triangle_index: i32,
    child_index: i32,
    context: *mut std::ffi::c_void,
) -> f32
where
    F: FnMut(RayHit) -> f32,
{
    let _guard = callback_state::CallbackGuard::enter();
    let ctx = unsafe { &mut *context.cast::<CastContext<F>>() };
    let hit = RayHit {
        shape_id: ShapeId::from_raw(shape_id),
        point: Pos::from_raw(point),
        normal: Vec3::from_raw(normal),
        fraction,
        user_material_id,
        triangle_index,
        child_index,
    };
    match catch_unwind(AssertUnwindSafe(|| (ctx.visitor)(hit))) {
        Ok(next_fraction) if next_fraction.is_finite() => next_fraction,
        Ok(_) => 0.0,
        Err(_) => {
            ctx.panicked = true;
            0.0
        }
    }
}

struct MoverPlaneContext<F> {
    visitor: F,
    panicked: bool,
}

unsafe extern "C" fn mover_plane_trampoline<F>(
    shape_id: ffi::b3ShapeId,
    plane: *const ffi::b3PlaneResult,
    plane_count: i32,
    context: *mut std::ffi::c_void,
) -> bool
where
    F: FnMut(MoverPlane) -> bool,
{
    let _guard = callback_state::CallbackGuard::enter();
    let ctx = unsafe { &mut *context.cast::<MoverPlaneContext<F>>() };
    if plane.is_null() || plane_count <= 0 {
        return true;
    }

    let planes = unsafe { std::slice::from_raw_parts(plane, plane_count as usize) };
    for raw_plane in planes {
        let mover_plane = MoverPlane::from_raw(shape_id, *raw_plane);
        match catch_unwind(AssertUnwindSafe(|| (ctx.visitor)(mover_plane))) {
            Ok(true) => {}
            Ok(false) => return false,
            Err(_) => {
                ctx.panicked = true;
                return false;
            }
        }
    }

    true
}
