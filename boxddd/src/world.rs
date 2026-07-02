use crate::body::BodyDef;
use crate::error::{Error, Result};
use crate::shapes::{BoxHull, ShapeDef, Sphere};
use crate::types::{BodyId, Quat, ShapeId, Vec3, Version};
use boxddd_sys::ffi;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard};

static BOX3D_LOCK: Mutex<()> = Mutex::new(());

fn lock_box3d() -> MutexGuard<'static, ()> {
    BOX3D_LOCK.lock().expect("Box3D global lock poisoned")
}

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

pub struct World {
    raw: ffi::b3WorldId,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl World {
    pub fn new(def: WorldDef) -> Result<Self> {
        if !Vec3::from_raw(def.raw.gravity).is_valid() {
            return Err(Error::InvalidArgument);
        }

        let _guard = lock_box3d();
        let raw = unsafe { ffi::b3CreateWorld(def.raw()) };
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
        let _guard = lock_box3d();
        unsafe { ffi::b3World_IsValid(self.raw) }
    }

    #[inline]
    pub fn step(&mut self, time_step: f32, sub_step_count: i32) {
        let _guard = lock_box3d();
        unsafe { ffi::b3World_Step(self.raw, time_step, sub_step_count) };
    }

    #[inline]
    pub fn gravity(&self) -> Vec3 {
        let _guard = lock_box3d();
        Vec3::from_raw(unsafe { ffi::b3World_GetGravity(self.raw) })
    }

    #[inline]
    pub fn set_gravity(&mut self, gravity: impl Into<Vec3>) {
        let _guard = lock_box3d();
        unsafe { ffi::b3World_SetGravity(self.raw, gravity.into().into_raw()) };
    }

    pub fn try_create_body(&mut self, def: BodyDef) -> Result<BodyId> {
        let _guard = lock_box3d();
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
        let _guard = lock_box3d();
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
        let _guard = lock_box3d();
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
    pub fn body_position(&self, body_id: BodyId) -> Vec3 {
        let _guard = lock_box3d();
        Vec3::from_raw(unsafe { ffi::b3Body_GetPosition(body_id.into_raw()) })
    }

    #[inline]
    pub fn body_rotation(&self, body_id: BodyId) -> Quat {
        let _guard = lock_box3d();
        Quat::from_raw(unsafe { ffi::b3Body_GetRotation(body_id.into_raw()) })
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _guard = lock_box3d();
        if unsafe { ffi::b3World_IsValid(self.raw) } {
            unsafe { ffi::b3DestroyWorld(self.raw) };
        }
    }
}

#[inline]
pub fn version() -> Version {
    let _guard = lock_box3d();
    Version::from_raw(unsafe { ffi::b3GetVersion() })
}

#[inline]
pub fn allocated_byte_count() -> i32 {
    let _guard = lock_box3d();
    unsafe { ffi::b3GetByteCount() }
}

#[inline]
pub fn is_double_precision() -> bool {
    let _guard = lock_box3d();
    unsafe { ffi::b3IsDoublePrecision() }
}
