#![cfg_attr(all(target_arch = "wasm32", boxddd_wasm_provider), allow(dead_code))]

use crate::collision::{ShapeCastInput, ShapeProxy};
use crate::core::{box3d_lock, callback_state};
use crate::error::{Error, Result};
use crate::shapes::Capsule;
use crate::types::{Aabb, Pos, ShapeId, Vec3};
use crate::world::World;
use boxddd_sys::ffi;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TreeStats {
    pub node_visits: i32,
    pub leaf_visits: i32,
}

impl TreeStats {
    #[inline]
    pub const fn from_raw(raw: ffi::b3TreeStats) -> Self {
        Self {
            node_visits: raw.nodeVisits,
            leaf_visits: raw.leafVisits,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct QueryFilter {
    pub category_bits: u64,
    pub mask_bits: u64,
    pub id: u64,
}

impl QueryFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn category_bits(mut self, category_bits: u64) -> Self {
        self.category_bits = category_bits;
        self
    }

    pub fn mask_bits(mut self, mask_bits: u64) -> Self {
        self.mask_bits = mask_bits;
        self
    }

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

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct QueryHit {
    pub shape_id: ShapeId,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RayHit {
    pub shape_id: ShapeId,
    pub point: Pos,
    pub normal: Vec3,
    pub fraction: f32,
    pub user_material_id: u64,
    pub triangle_index: i32,
    pub child_index: i32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RayClosestHit {
    pub shape_id: ShapeId,
    pub point: Pos,
    pub normal: Vec3,
    pub fraction: f32,
    pub user_material_id: u64,
    pub triangle_index: i32,
    pub child_index: i32,
    pub node_visits: i32,
    pub leaf_visits: i32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BodyClosestPoint {
    pub point: Vec3,
    pub distance: f32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BodyCastHit {
    pub shape_id: ShapeId,
    pub point: Pos,
    pub normal: Vec3,
    pub fraction: f32,
    pub user_material_id: u64,
    pub triangle_index: i32,
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

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ShapeRayHit {
    pub point: Pos,
    pub normal: Vec3,
    pub fraction: f32,
    pub triangle_index: i32,
    pub child_index: i32,
    pub material_index: i32,
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

impl World {
    pub fn overlap_aabb(&self, aabb: Aabb, filter: QueryFilter) -> Result<Vec<QueryHit>> {
        let mut out = Vec::new();
        self.overlap_aabb_into(aabb, filter, &mut out)?;
        Ok(out)
    }

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
