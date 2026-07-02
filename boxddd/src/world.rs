use crate::body::BodyDef;
use crate::core::{box3d_lock, callback_state, debug_checks};
use crate::error::{Error, Result};
use crate::shapes::{BoxHull, ShapeDef, Sphere};
use crate::types::{
    Aabb, BodyId, Counters, Pos, Profile, Quat, ShapeId, Vec3, Version, WorldTransform,
};
use boxddd_sys::ffi;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct WorldDef {
    raw: ffi::b3WorldDef,
}

impl Default for WorldDef {
    fn default() -> Self {
        Self {
            raw: unsafe { ffi::b3DefaultWorldDef() },
        }
    }
}

impl WorldDef {
    #[inline]
    pub fn builder() -> WorldDefBuilder {
        WorldDefBuilder::new()
    }

    #[inline]
    pub fn raw(&self) -> &ffi::b3WorldDef {
        &self.raw
    }

    pub fn validate(&self) -> Result<()> {
        Vec3::from_raw(self.raw.gravity).validate()?;
        if self.raw.restitutionThreshold.is_finite()
            && self.raw.hitEventThreshold.is_finite()
            && self.raw.contactHertz.is_finite()
            && self.raw.contactDampingRatio.is_finite()
            && self.raw.contactSpeed.is_finite()
            && self.raw.maximumLinearSpeed.is_finite()
        {
            Ok(())
        } else {
            Err(Error::InvalidArgument)
        }
    }
}

#[derive(Clone, Debug)]
pub struct WorldDefBuilder {
    def: WorldDef,
}

impl WorldDefBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            def: WorldDef::default(),
        }
    }

    #[inline]
    pub fn gravity(mut self, gravity: impl Into<Vec3>) -> Self {
        self.def.raw.gravity = gravity.into().into_raw();
        self
    }

    #[inline]
    pub fn worker_count(mut self, worker_count: u32) -> Self {
        self.def.raw.workerCount = worker_count;
        self
    }

    #[inline]
    pub fn build(self) -> WorldDef {
        self.def
    }
}

impl Default for WorldDefBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct World {
    raw: ffi::b3WorldId,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl World {
    pub fn new(def: WorldDef) -> Result<Self> {
        callback_state::check_not_in_callback()?;
        def.validate()?;

        let _guard = box3d_lock::lock();
        let raw = unsafe { create_world_raw(def.raw()) };
        if unsafe { ffi::b3World_IsValid(raw) } {
            Ok(Self {
                raw,
                _not_send_sync: PhantomData,
            })
        } else {
            Err(Error::CreateWorldFailed)
        }
    }

    #[inline]
    pub fn raw(&self) -> ffi::b3WorldId {
        self.raw
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3World_IsValid(self.raw) }
    }

    #[inline]
    pub fn step(&mut self, time_step: f32, sub_step_count: i32) {
        self.try_step(time_step, sub_step_count)
            .expect("Box3D failed to step world");
    }

    #[inline]
    pub fn try_step(&mut self, time_step: f32, sub_step_count: i32) -> Result<()> {
        callback_state::check_not_in_callback()?;
        if !time_step.is_finite() || time_step < 0.0 || sub_step_count < 0 {
            return Err(Error::InvalidArgument);
        }
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3World_Step(self.raw, time_step, sub_step_count) };
        Ok(())
    }

    #[inline]
    pub fn gravity(&self) -> Vec3 {
        self.try_gravity().expect("invalid Box3D world")
    }

    #[inline]
    pub fn try_gravity(&self) -> Result<Vec3> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        if !unsafe { ffi::b3World_IsValid(self.raw) } {
            return Err(Error::InvalidWorldId);
        }
        Ok(Vec3::from_raw(unsafe { ffi::b3World_GetGravity(self.raw) }))
    }

    #[inline]
    pub fn set_gravity(&mut self, gravity: impl Into<Vec3>) {
        self.try_set_gravity(gravity)
            .expect("invalid gravity or Box3D world");
    }

    #[inline]
    pub fn try_set_gravity(&mut self, gravity: impl Into<Vec3>) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let gravity = gravity.into().validate()?;
        let _guard = box3d_lock::lock();
        if !unsafe { ffi::b3World_IsValid(self.raw) } {
            return Err(Error::InvalidWorldId);
        }
        unsafe { ffi::b3World_SetGravity(self.raw, gravity.into_raw()) };
        Ok(())
    }

    pub fn try_create_body(&mut self, def: BodyDef) -> Result<BodyId> {
        callback_state::check_not_in_callback()?;
        def.validate()?;
        let _guard = box3d_lock::lock();
        let raw = unsafe { ffi::b3CreateBody(self.raw, def.raw()) };
        if unsafe { ffi::b3Body_IsValid(raw) } {
            Ok(BodyId::from_raw(raw))
        } else {
            Err(Error::CreateBodyFailed)
        }
    }

    pub fn create_body(&mut self, def: BodyDef) -> BodyId {
        self.try_create_body(def)
            .expect("Box3D failed to create body")
    }

    pub fn try_create_sphere_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        sphere: &Sphere,
    ) -> Result<ShapeId> {
        callback_state::check_not_in_callback()?;
        debug_checks::check_body_valid(body_id)?;
        def.validate()?;
        sphere.validate()?;
        let _guard = box3d_lock::lock();
        let raw = unsafe { ffi::b3CreateSphereShape(body_id.into_raw(), def.raw(), sphere.raw()) };
        if unsafe { ffi::b3Shape_IsValid(raw) } {
            Ok(ShapeId::from_raw(raw))
        } else {
            Err(Error::CreateShapeFailed)
        }
    }

    pub fn create_sphere_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        sphere: &Sphere,
    ) -> ShapeId {
        self.try_create_sphere_shape(body_id, def, sphere)
            .expect("Box3D failed to create sphere shape")
    }

    pub fn try_create_hull_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        hull: &BoxHull,
    ) -> Result<ShapeId> {
        callback_state::check_not_in_callback()?;
        debug_checks::check_body_valid(body_id)?;
        def.validate()?;
        let _guard = box3d_lock::lock();
        let raw =
            unsafe { ffi::b3CreateHullShape(body_id.into_raw(), def.raw(), hull.hull_data()) };
        if unsafe { ffi::b3Shape_IsValid(raw) } {
            Ok(ShapeId::from_raw(raw))
        } else {
            Err(Error::CreateShapeFailed)
        }
    }

    pub fn create_hull_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        hull: &BoxHull,
    ) -> ShapeId {
        self.try_create_hull_shape(body_id, def, hull)
            .expect("Box3D failed to create hull shape")
    }

    #[inline]
    pub fn body_position(&self, body_id: BodyId) -> Pos {
        self.try_body_position(body_id).expect("invalid BodyId")
    }

    #[inline]
    pub fn try_body_position(&self, body_id: BodyId) -> Result<Pos> {
        debug_checks::check_body_valid(body_id)?;
        let _guard = box3d_lock::lock();
        Ok(Pos::from_raw(unsafe {
            ffi::b3Body_GetPosition(body_id.into_raw())
        }))
    }

    #[inline]
    pub fn body_rotation(&self, body_id: BodyId) -> Quat {
        self.try_body_rotation(body_id).expect("invalid BodyId")
    }

    #[inline]
    pub fn try_body_rotation(&self, body_id: BodyId) -> Result<Quat> {
        debug_checks::check_body_valid(body_id)?;
        let _guard = box3d_lock::lock();
        Ok(Quat::from_raw(unsafe {
            ffi::b3Body_GetRotation(body_id.into_raw())
        }))
    }

    #[inline]
    pub fn body_transform(&self, body_id: BodyId) -> WorldTransform {
        self.try_body_transform(body_id).expect("invalid BodyId")
    }

    #[inline]
    pub fn try_body_transform(&self, body_id: BodyId) -> Result<WorldTransform> {
        debug_checks::check_body_valid(body_id)?;
        let _guard = box3d_lock::lock();
        Ok(WorldTransform::from_raw(unsafe {
            ffi::b3Body_GetTransform(body_id.into_raw())
        }))
    }

    pub fn try_destroy_body(&mut self, body_id: BodyId) -> Result<()> {
        debug_checks::check_body_valid(body_id)?;
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3DestroyBody(body_id.into_raw()) };
        Ok(())
    }

    #[track_caller]
    pub fn destroy_body(&mut self, body_id: BodyId) {
        self.try_destroy_body(body_id).expect("invalid BodyId");
    }

    #[inline]
    pub fn bounds(&self) -> Aabb {
        self.try_bounds().expect("invalid Box3D world")
    }

    #[inline]
    pub fn try_bounds(&self) -> Result<Aabb> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        if !unsafe { ffi::b3World_IsValid(self.raw) } {
            return Err(Error::InvalidWorldId);
        }
        Ok(Aabb::from_raw(unsafe { ffi::b3World_GetBounds(self.raw) }))
    }

    #[inline]
    pub fn profile(&self) -> Profile {
        self.try_profile().expect("invalid Box3D world")
    }

    #[inline]
    pub fn try_profile(&self) -> Result<Profile> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        if !unsafe { ffi::b3World_IsValid(self.raw) } {
            return Err(Error::InvalidWorldId);
        }
        Ok(Profile::from_raw(unsafe {
            ffi::b3World_GetProfile(self.raw)
        }))
    }

    #[inline]
    pub fn counters(&self) -> Counters {
        self.try_counters().expect("invalid Box3D world")
    }

    #[inline]
    pub fn try_counters(&self) -> Result<Counters> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        if !unsafe { ffi::b3World_IsValid(self.raw) } {
            return Err(Error::InvalidWorldId);
        }
        Ok(Counters::from_raw(unsafe {
            ffi::b3World_GetCounters(self.raw)
        }))
    }
}

#[cfg(not(feature = "double-precision"))]
#[inline]
unsafe fn create_world_raw(def: *const ffi::b3WorldDef) -> ffi::b3WorldId {
    unsafe { ffi::b3CreateWorld(def) }
}

#[cfg(feature = "double-precision")]
#[inline]
unsafe fn create_world_raw(def: *const ffi::b3WorldDef) -> ffi::b3WorldId {
    unsafe { ffi::b3CreateWorldDoublePrecision(def) }
}

impl Drop for World {
    fn drop(&mut self) {
        let _guard = box3d_lock::lock();
        if unsafe { ffi::b3World_IsValid(self.raw) } {
            unsafe { ffi::b3DestroyWorld(self.raw) };
        }
    }
}

#[inline]
pub fn version() -> Version {
    let _guard = box3d_lock::lock();
    Version::from_raw(unsafe { ffi::b3GetVersion() })
}

#[inline]
pub fn allocated_byte_count() -> i32 {
    let _guard = box3d_lock::lock();
    unsafe { ffi::b3GetByteCount() }
}

#[inline]
pub fn is_double_precision() -> bool {
    let _guard = box3d_lock::lock();
    unsafe { ffi::b3IsDoublePrecision() }
}
