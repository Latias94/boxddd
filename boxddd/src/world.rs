use crate::body::{BodyDef, BodyType};
use crate::callbacks::WorldCallbacks;
use crate::core::{box3d_lock, callback_state, debug_checks, ffi_vec};
use crate::debug_draw::{DebugShapeRegistry, create_debug_shape, destroy_debug_shape};
use crate::error::{Error, Result};
use crate::shapes::{
    BoxHull, Capsule, Compound, HeightField, Hull, MeshData, ShapeDef, ShapeType, Sphere,
    SurfaceMaterial,
};
use crate::types::{
    Aabb, BodyId, Capacity, ContactData, Counters, Filter, JointId, MassData, Matrix3, MotionLocks,
    Pos, Profile, Quat, ShapeId, Vec3, Version, WorldTransform,
};
use boxddd_sys::ffi;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
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
    resources: HashMap<ShapeId, ShapeResource>,
    pub(crate) callbacks: WorldCallbacks,
    _debug_shapes: Box<DebugShapeRegistry>,
    _not_send_sync: PhantomData<Rc<()>>,
}

#[derive(Debug)]
enum ShapeResource {
    Mesh { _data: MeshData },
    HeightField { _data: HeightField },
    Compound { _data: Compound },
}

impl World {
    pub fn new(def: WorldDef) -> Result<Self> {
        callback_state::check_not_in_callback()?;
        def.validate()?;

        let debug_shapes = Box::<DebugShapeRegistry>::default();
        let mut raw_def = *def.raw();
        raw_def.createDebugShape = Some(create_debug_shape);
        raw_def.destroyDebugShape = Some(destroy_debug_shape);
        raw_def.userDebugShapeContext =
            (&*debug_shapes) as *const DebugShapeRegistry as *mut std::ffi::c_void;

        let _guard = box3d_lock::lock();
        let raw = unsafe { create_world_raw(&raw_def) };
        if unsafe { ffi::b3World_IsValid(raw) } {
            Ok(Self {
                raw,
                resources: HashMap::new(),
                callbacks: WorldCallbacks::default(),
                _debug_shapes: debug_shapes,
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
        self.callbacks.reset_panics();
        unsafe { ffi::b3World_Step(self.raw, time_step, sub_step_count) };
        if self.callbacks.panicked() {
            return Err(Error::CallbackPanicked);
        }
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
        def.validate()?;
        sphere.validate()?;
        let _guard = self.lock_body_checked(body_id)?;
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
        def.validate()?;
        let _guard = self.lock_body_checked(body_id)?;
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

    pub fn try_create_capsule_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        capsule: &Capsule,
    ) -> Result<ShapeId> {
        callback_state::check_not_in_callback()?;
        def.validate()?;
        capsule.validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        let raw =
            unsafe { ffi::b3CreateCapsuleShape(body_id.into_raw(), def.raw(), capsule.raw()) };
        shape_id_from_raw(raw)
    }

    pub fn create_capsule_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        capsule: &Capsule,
    ) -> ShapeId {
        self.try_create_capsule_shape(body_id, def, capsule)
            .expect("Box3D failed to create capsule shape")
    }

    pub fn try_create_created_hull_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        hull: &Hull,
    ) -> Result<ShapeId> {
        callback_state::check_not_in_callback()?;
        def.validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        let raw = unsafe { ffi::b3CreateHullShape(body_id.into_raw(), def.raw(), hull.as_ptr()) };
        shape_id_from_raw(raw)
    }

    pub fn try_create_transformed_hull_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        hull: &Hull,
        transform: impl Into<crate::types::Transform>,
        scale: impl Into<Vec3>,
    ) -> Result<ShapeId> {
        callback_state::check_not_in_callback()?;
        def.validate()?;
        let transform = transform.into();
        transform.validate()?;
        let scale = scale.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        let raw = unsafe {
            ffi::b3CreateTransformedHullShape(
                body_id.into_raw(),
                def.raw(),
                hull.as_ptr(),
                transform.into_raw(),
                scale.into_raw(),
            )
        };
        shape_id_from_raw(raw)
    }

    pub fn try_create_mesh_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        mesh: MeshData,
        scale: impl Into<Vec3>,
    ) -> Result<ShapeId> {
        if self.try_body_type(body_id)? != BodyType::Static {
            return Err(Error::InvalidArgument);
        }
        callback_state::check_not_in_callback()?;
        def.validate()?;
        let scale = scale.into().validate()?;
        let mesh_ptr = mesh.as_ptr();
        let _guard = self.lock_body_checked(body_id)?;
        let raw = unsafe {
            ffi::b3CreateMeshShape(body_id.into_raw(), def.raw(), mesh_ptr, scale.into_raw())
        };
        let shape_id = shape_id_from_raw(raw)?;
        drop(_guard);
        self.resources
            .insert(shape_id, ShapeResource::Mesh { _data: mesh });
        Ok(shape_id)
    }

    pub fn try_create_height_field_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        height_field: HeightField,
    ) -> Result<ShapeId> {
        if self.try_body_type(body_id)? != BodyType::Static {
            return Err(Error::InvalidArgument);
        }
        callback_state::check_not_in_callback()?;
        def.validate()?;
        let height_field_ptr = height_field.as_ptr();
        let _guard = self.lock_body_checked(body_id)?;
        let raw = unsafe {
            ffi::b3CreateHeightFieldShape(body_id.into_raw(), def.raw(), height_field_ptr)
        };
        let shape_id = shape_id_from_raw(raw)?;
        drop(_guard);
        self.resources.insert(
            shape_id,
            ShapeResource::HeightField {
                _data: height_field,
            },
        );
        Ok(shape_id)
    }

    pub fn try_create_compound_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        compound: Compound,
    ) -> Result<ShapeId> {
        if self.try_body_type(body_id)? != BodyType::Static {
            return Err(Error::InvalidArgument);
        }
        callback_state::check_not_in_callback()?;
        def.validate()?;
        let compound_ptr = compound.as_ptr();
        let mut raw_def = *def.raw();
        let _guard = self.lock_body_checked(body_id)?;
        let raw =
            unsafe { ffi::b3CreateCompoundShape(body_id.into_raw(), &mut raw_def, compound_ptr) };
        let shape_id = shape_id_from_raw(raw)?;
        drop(_guard);
        self.resources
            .insert(shape_id, ShapeResource::Compound { _data: compound });
        Ok(shape_id)
    }

    #[inline]
    pub fn body_position(&self, body_id: BodyId) -> Pos {
        self.try_body_position(body_id).expect("invalid BodyId")
    }

    #[inline]
    pub fn try_body_position(&self, body_id: BodyId) -> Result<Pos> {
        let _guard = self.lock_body_checked(body_id)?;
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
        let _guard = self.lock_body_checked(body_id)?;
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
        let _guard = self.lock_body_checked(body_id)?;
        Ok(WorldTransform::from_raw(unsafe {
            ffi::b3Body_GetTransform(body_id.into_raw())
        }))
    }

    pub fn try_destroy_body(&mut self, body_id: BodyId) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        let shape_ids = body_shape_ids_locked(body_id);
        unsafe { ffi::b3DestroyBody(body_id.into_raw()) };
        drop(_guard);
        for shape_id in shape_ids {
            self.resources.remove(&shape_id);
        }
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

    pub fn try_enable_sleeping(&mut self, enabled: bool) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_EnableSleeping(self.raw, enabled) };
        Ok(())
    }

    pub fn enable_sleeping(&mut self, enabled: bool) {
        self.try_enable_sleeping(enabled)
            .expect("invalid Box3D world");
    }

    pub fn try_sleeping_enabled(&self) -> Result<bool> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_IsSleepingEnabled(self.raw) })
    }

    pub fn sleeping_enabled(&self) -> bool {
        self.try_sleeping_enabled().expect("invalid Box3D world")
    }

    pub fn try_enable_continuous(&mut self, enabled: bool) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_EnableContinuous(self.raw, enabled) };
        Ok(())
    }

    pub fn enable_continuous(&mut self, enabled: bool) {
        self.try_enable_continuous(enabled)
            .expect("invalid Box3D world");
    }

    pub fn try_continuous_enabled(&self) -> Result<bool> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_IsContinuousEnabled(self.raw) })
    }

    pub fn continuous_enabled(&self) -> bool {
        self.try_continuous_enabled().expect("invalid Box3D world")
    }

    pub fn try_set_restitution_threshold(&mut self, value: f32) -> Result<()> {
        validate_nonnegative_scalar(value)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_SetRestitutionThreshold(self.raw, value) };
        Ok(())
    }

    pub fn restitution_threshold(&self) -> f32 {
        self.try_restitution_threshold()
            .expect("invalid Box3D world")
    }

    pub fn try_restitution_threshold(&self) -> Result<f32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_GetRestitutionThreshold(self.raw) })
    }

    pub fn try_set_hit_event_threshold(&mut self, value: f32) -> Result<()> {
        validate_nonnegative_scalar(value)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_SetHitEventThreshold(self.raw, value) };
        Ok(())
    }

    pub fn hit_event_threshold(&self) -> f32 {
        self.try_hit_event_threshold().expect("invalid Box3D world")
    }

    pub fn try_hit_event_threshold(&self) -> Result<f32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_GetHitEventThreshold(self.raw) })
    }

    pub fn try_set_contact_tuning(
        &mut self,
        hertz: f32,
        damping_ratio: f32,
        contact_speed: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        validate_nonnegative_scalar(damping_ratio)?;
        validate_nonnegative_scalar(contact_speed)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_SetContactTuning(self.raw, hertz, damping_ratio, contact_speed) };
        Ok(())
    }

    pub fn try_set_contact_recycle_distance(&mut self, distance: f32) -> Result<()> {
        validate_nonnegative_scalar(distance)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_SetContactRecycleDistance(self.raw, distance) };
        Ok(())
    }

    pub fn contact_recycle_distance(&self) -> f32 {
        self.try_contact_recycle_distance()
            .expect("invalid Box3D world")
    }

    pub fn try_contact_recycle_distance(&self) -> Result<f32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_GetContactRecycleDistance(self.raw) })
    }

    pub fn try_set_maximum_linear_speed(&mut self, speed: f32) -> Result<()> {
        validate_nonnegative_scalar(speed)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_SetMaximumLinearSpeed(self.raw, speed) };
        Ok(())
    }

    pub fn maximum_linear_speed(&self) -> f32 {
        self.try_maximum_linear_speed()
            .expect("invalid Box3D world")
    }

    pub fn try_maximum_linear_speed(&self) -> Result<f32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_GetMaximumLinearSpeed(self.raw) })
    }

    pub fn try_enable_warm_starting(&mut self, enabled: bool) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_EnableWarmStarting(self.raw, enabled) };
        Ok(())
    }

    pub fn warm_starting_enabled(&self) -> bool {
        self.try_warm_starting_enabled()
            .expect("invalid Box3D world")
    }

    pub fn try_warm_starting_enabled(&self) -> Result<bool> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_IsWarmStartingEnabled(self.raw) })
    }

    pub fn try_enable_speculative(&mut self, enabled: bool) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_EnableSpeculative(self.raw, enabled) };
        Ok(())
    }

    pub fn awake_body_count(&self) -> i32 {
        self.try_awake_body_count().expect("invalid Box3D world")
    }

    pub fn try_awake_body_count(&self) -> Result<i32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_GetAwakeBodyCount(self.raw) })
    }

    pub fn max_capacity(&self) -> Capacity {
        self.try_max_capacity().expect("invalid Box3D world")
    }

    pub fn try_max_capacity(&self) -> Result<Capacity> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(Capacity::from_raw(unsafe {
            ffi::b3World_GetMaxCapacity(self.raw)
        }))
    }

    pub fn try_set_worker_count(&mut self, count: i32) -> Result<()> {
        if count < 0 {
            return Err(Error::InvalidArgument);
        }
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_SetWorkerCount(self.raw, count) };
        Ok(())
    }

    pub fn worker_count(&self) -> i32 {
        self.try_worker_count().expect("invalid Box3D world")
    }

    pub fn try_worker_count(&self) -> Result<i32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_GetWorkerCount(self.raw) })
    }

    pub fn try_rebuild_static_tree(&mut self) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_RebuildStaticTree(self.raw) };
        Ok(())
    }

    pub fn try_body_type(&self, body_id: BodyId) -> Result<BodyType> {
        let _guard = self.lock_body_checked(body_id)?;
        BodyType::from_raw(unsafe { ffi::b3Body_GetType(body_id.into_raw()) })
            .ok_or(Error::InvalidArgument)
    }

    pub fn body_type(&self, body_id: BodyId) -> BodyType {
        self.try_body_type(body_id).expect("invalid BodyId")
    }

    pub fn try_set_body_type(&mut self, body_id: BodyId, body_type: BodyType) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetType(body_id.into_raw(), body_type.into_raw()) };
        Ok(())
    }

    pub fn try_set_body_name(&mut self, body_id: BodyId, name: impl Into<Vec<u8>>) -> Result<()> {
        let name = CString::new(name).map_err(|_| Error::NulByteInString)?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetName(body_id.into_raw(), name.as_ptr()) };
        Ok(())
    }

    pub fn try_body_name(&self, body_id: BodyId) -> Result<Option<String>> {
        let _guard = self.lock_body_checked(body_id)?;
        let ptr = unsafe { ffi::b3Body_GetName(body_id.into_raw()) };
        if ptr.is_null() {
            Ok(None)
        } else {
            Ok(Some(
                unsafe { CStr::from_ptr(ptr) }
                    .to_string_lossy()
                    .into_owned(),
            ))
        }
    }

    pub fn try_set_body_transform(
        &mut self,
        body_id: BodyId,
        position: impl Into<Pos>,
        rotation: Quat,
    ) -> Result<()> {
        let position = position.into().validate()?;
        rotation.validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe {
            ffi::b3Body_SetTransform(body_id.into_raw(), position.into_raw(), rotation.into_raw())
        };
        Ok(())
    }

    pub fn try_set_body_target_transform(
        &mut self,
        body_id: BodyId,
        target: WorldTransform,
        time_step: f32,
        wake: bool,
    ) -> Result<()> {
        if !target.is_valid() || !time_step.is_finite() || time_step <= 0.0 {
            return Err(Error::InvalidArgument);
        }
        let _guard = self.lock_body_checked(body_id)?;
        unsafe {
            ffi::b3Body_SetTargetTransform(body_id.into_raw(), target.into_raw(), time_step, wake)
        };
        Ok(())
    }

    pub fn try_body_local_point(
        &self,
        body_id: BodyId,
        world_point: impl Into<Pos>,
    ) -> Result<Vec3> {
        let world_point = world_point.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Body_GetLocalPoint(body_id.into_raw(), world_point.into_raw())
        }))
    }

    pub fn try_body_world_point(
        &self,
        body_id: BodyId,
        local_point: impl Into<Vec3>,
    ) -> Result<Pos> {
        let local_point = local_point.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Pos::from_raw(unsafe {
            ffi::b3Body_GetWorldPoint(body_id.into_raw(), local_point.into_raw())
        }))
    }

    pub fn try_body_local_vector(
        &self,
        body_id: BodyId,
        world_vector: impl Into<Vec3>,
    ) -> Result<Vec3> {
        let world_vector = world_vector.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Body_GetLocalVector(body_id.into_raw(), world_vector.into_raw())
        }))
    }

    pub fn try_body_world_vector(
        &self,
        body_id: BodyId,
        local_vector: impl Into<Vec3>,
    ) -> Result<Vec3> {
        let local_vector = local_vector.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Body_GetWorldVector(body_id.into_raw(), local_vector.into_raw())
        }))
    }

    pub fn try_body_linear_velocity(&self, body_id: BodyId) -> Result<Vec3> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Body_GetLinearVelocity(body_id.into_raw())
        }))
    }

    pub fn body_linear_velocity(&self, body_id: BodyId) -> Vec3 {
        self.try_body_linear_velocity(body_id)
            .expect("invalid BodyId")
    }

    pub fn try_set_body_linear_velocity(
        &mut self,
        body_id: BodyId,
        velocity: impl Into<Vec3>,
    ) -> Result<()> {
        let velocity = velocity.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetLinearVelocity(body_id.into_raw(), velocity.into_raw()) };
        Ok(())
    }

    pub fn try_body_angular_velocity(&self, body_id: BodyId) -> Result<Vec3> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Body_GetAngularVelocity(body_id.into_raw())
        }))
    }

    pub fn body_angular_velocity(&self, body_id: BodyId) -> Vec3 {
        self.try_body_angular_velocity(body_id)
            .expect("invalid BodyId")
    }

    pub fn try_set_body_angular_velocity(
        &mut self,
        body_id: BodyId,
        velocity: impl Into<Vec3>,
    ) -> Result<()> {
        let velocity = velocity.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetAngularVelocity(body_id.into_raw(), velocity.into_raw()) };
        Ok(())
    }

    pub fn try_apply_force(
        &mut self,
        body_id: BodyId,
        force: impl Into<Vec3>,
        point: impl Into<Pos>,
        wake: bool,
    ) -> Result<()> {
        let force = force.into().validate()?;
        let point = point.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe {
            ffi::b3Body_ApplyForce(body_id.into_raw(), force.into_raw(), point.into_raw(), wake)
        };
        Ok(())
    }

    pub fn try_apply_force_to_center(
        &mut self,
        body_id: BodyId,
        force: impl Into<Vec3>,
        wake: bool,
    ) -> Result<()> {
        let force = force.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_ApplyForceToCenter(body_id.into_raw(), force.into_raw(), wake) };
        Ok(())
    }

    pub fn try_apply_torque(
        &mut self,
        body_id: BodyId,
        torque: impl Into<Vec3>,
        wake: bool,
    ) -> Result<()> {
        let torque = torque.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_ApplyTorque(body_id.into_raw(), torque.into_raw(), wake) };
        Ok(())
    }

    pub fn try_apply_linear_impulse(
        &mut self,
        body_id: BodyId,
        impulse: impl Into<Vec3>,
        point: impl Into<Pos>,
        wake: bool,
    ) -> Result<()> {
        let impulse = impulse.into().validate()?;
        let point = point.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe {
            ffi::b3Body_ApplyLinearImpulse(
                body_id.into_raw(),
                impulse.into_raw(),
                point.into_raw(),
                wake,
            )
        };
        Ok(())
    }

    pub fn try_apply_linear_impulse_to_center(
        &mut self,
        body_id: BodyId,
        impulse: impl Into<Vec3>,
        wake: bool,
    ) -> Result<()> {
        let impulse = impulse.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe {
            ffi::b3Body_ApplyLinearImpulseToCenter(body_id.into_raw(), impulse.into_raw(), wake)
        };
        Ok(())
    }

    pub fn try_apply_angular_impulse(
        &mut self,
        body_id: BodyId,
        impulse: impl Into<Vec3>,
        wake: bool,
    ) -> Result<()> {
        let impulse = impulse.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_ApplyAngularImpulse(body_id.into_raw(), impulse.into_raw(), wake) };
        Ok(())
    }

    pub fn try_body_mass(&self, body_id: BodyId) -> Result<f32> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_GetMass(body_id.into_raw()) })
    }

    pub fn try_body_local_rotational_inertia(&self, body_id: BodyId) -> Result<Matrix3> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Matrix3::from_raw(unsafe {
            ffi::b3Body_GetLocalRotationalInertia(body_id.into_raw())
        }))
    }

    pub fn try_body_inverse_mass(&self, body_id: BodyId) -> Result<f32> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_GetInverseMass(body_id.into_raw()) })
    }

    pub fn try_body_world_inverse_rotational_inertia(&self, body_id: BodyId) -> Result<Matrix3> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Matrix3::from_raw(unsafe {
            ffi::b3Body_GetWorldInverseRotationalInertia(body_id.into_raw())
        }))
    }

    pub fn try_body_local_center_of_mass(&self, body_id: BodyId) -> Result<Vec3> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Body_GetLocalCenterOfMass(body_id.into_raw())
        }))
    }

    pub fn try_body_world_center_of_mass(&self, body_id: BodyId) -> Result<Pos> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Pos::from_raw(unsafe {
            ffi::b3Body_GetWorldCenterOfMass(body_id.into_raw())
        }))
    }

    pub fn try_body_mass_data(&self, body_id: BodyId) -> Result<MassData> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(MassData::from_raw(unsafe {
            ffi::b3Body_GetMassData(body_id.into_raw())
        }))
    }

    pub fn try_set_body_mass_data(&mut self, body_id: BodyId, mass_data: MassData) -> Result<()> {
        validate_nonnegative_scalar(mass_data.mass)?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetMassData(body_id.into_raw(), mass_data.into_raw()) };
        Ok(())
    }

    pub fn try_apply_mass_from_shapes(&mut self, body_id: BodyId) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_ApplyMassFromShapes(body_id.into_raw()) };
        Ok(())
    }

    pub fn try_set_body_linear_damping(&mut self, body_id: BodyId, damping: f32) -> Result<()> {
        validate_nonnegative_scalar(damping)?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetLinearDamping(body_id.into_raw(), damping) };
        Ok(())
    }

    pub fn try_body_linear_damping(&self, body_id: BodyId) -> Result<f32> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_GetLinearDamping(body_id.into_raw()) })
    }

    pub fn try_set_body_angular_damping(&mut self, body_id: BodyId, damping: f32) -> Result<()> {
        validate_nonnegative_scalar(damping)?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetAngularDamping(body_id.into_raw(), damping) };
        Ok(())
    }

    pub fn try_body_angular_damping(&self, body_id: BodyId) -> Result<f32> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_GetAngularDamping(body_id.into_raw()) })
    }

    pub fn try_set_body_gravity_scale(&mut self, body_id: BodyId, scale: f32) -> Result<()> {
        validate_scalar(scale)?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetGravityScale(body_id.into_raw(), scale) };
        Ok(())
    }

    pub fn try_body_gravity_scale(&self, body_id: BodyId) -> Result<f32> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_GetGravityScale(body_id.into_raw()) })
    }

    pub fn try_body_awake(&self, body_id: BodyId) -> Result<bool> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_IsAwake(body_id.into_raw()) })
    }

    pub fn try_set_body_awake(&mut self, body_id: BodyId, awake: bool) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetAwake(body_id.into_raw(), awake) };
        Ok(())
    }

    pub fn try_enable_body_sleep(&mut self, body_id: BodyId, enabled: bool) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_EnableSleep(body_id.into_raw(), enabled) };
        Ok(())
    }

    pub fn try_body_sleep_enabled(&self, body_id: BodyId) -> Result<bool> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_IsSleepEnabled(body_id.into_raw()) })
    }

    pub fn try_set_body_sleep_threshold(&mut self, body_id: BodyId, threshold: f32) -> Result<()> {
        validate_nonnegative_scalar(threshold)?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetSleepThreshold(body_id.into_raw(), threshold) };
        Ok(())
    }

    pub fn try_body_sleep_threshold(&self, body_id: BodyId) -> Result<f32> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_GetSleepThreshold(body_id.into_raw()) })
    }

    pub fn try_body_enabled(&self, body_id: BodyId) -> Result<bool> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_IsEnabled(body_id.into_raw()) })
    }

    pub fn try_enable_body(&mut self, body_id: BodyId) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_Enable(body_id.into_raw()) };
        Ok(())
    }

    pub fn try_disable_body(&mut self, body_id: BodyId) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_Disable(body_id.into_raw()) };
        Ok(())
    }

    pub fn try_set_body_motion_locks(&mut self, body_id: BodyId, locks: MotionLocks) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetMotionLocks(body_id.into_raw(), locks.into_raw()) };
        Ok(())
    }

    pub fn try_body_motion_locks(&self, body_id: BodyId) -> Result<MotionLocks> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(MotionLocks::from_raw(unsafe {
            ffi::b3Body_GetMotionLocks(body_id.into_raw())
        }))
    }

    pub fn try_set_body_bullet(&mut self, body_id: BodyId, bullet: bool) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetBullet(body_id.into_raw(), bullet) };
        Ok(())
    }

    pub fn try_body_bullet(&self, body_id: BodyId) -> Result<bool> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_IsBullet(body_id.into_raw()) })
    }

    pub fn try_enable_body_contact_recycling(
        &mut self,
        body_id: BodyId,
        enabled: bool,
    ) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_EnableContactRecycling(body_id.into_raw(), enabled) };
        Ok(())
    }

    pub fn try_body_contact_recycling_enabled(&self, body_id: BodyId) -> Result<bool> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_IsContactRecyclingEnabled(body_id.into_raw()) })
    }

    pub fn try_enable_body_hit_events(&mut self, body_id: BodyId, enabled: bool) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_EnableHitEvents(body_id.into_raw(), enabled) };
        Ok(())
    }

    pub fn try_body_aabb(&self, body_id: BodyId) -> Result<Aabb> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Aabb::from_raw(unsafe {
            ffi::b3Body_ComputeAABB(body_id.into_raw())
        }))
    }

    pub fn body_shapes(&self, body_id: BodyId) -> Vec<ShapeId> {
        self.try_body_shapes(body_id).expect("invalid BodyId")
    }

    pub fn try_body_shapes(&self, body_id: BodyId) -> Result<Vec<ShapeId>> {
        let mut out = Vec::new();
        self.try_body_shapes_into(body_id, &mut out)?;
        Ok(out)
    }

    pub fn try_body_shapes_into(&self, body_id: BodyId, out: &mut Vec<ShapeId>) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_body_belongs_locked(body_id)?;
        let capacity = unsafe { ffi::b3Body_GetShapeCount(body_id.into_raw()) }.max(0) as usize;
        unsafe {
            ffi_vec::fill_from_ffi(out, capacity, |ptr, cap| {
                ffi::b3Body_GetShapes(body_id.into_raw(), ptr.cast(), cap)
            })
        };
        Ok(())
    }

    pub fn try_body_joints(&self, body_id: BodyId) -> Result<Vec<JointId>> {
        let mut out = Vec::new();
        self.try_body_joints_into(body_id, &mut out)?;
        Ok(out)
    }

    pub fn try_body_joints_into(&self, body_id: BodyId, out: &mut Vec<JointId>) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_body_belongs_locked(body_id)?;
        let capacity = unsafe { ffi::b3Body_GetJointCount(body_id.into_raw()) }.max(0) as usize;
        unsafe {
            ffi_vec::fill_from_ffi(out, capacity, |ptr, cap| {
                ffi::b3Body_GetJoints(body_id.into_raw(), ptr.cast(), cap)
            })
        };
        Ok(())
    }

    pub fn try_body_contacts(&self, body_id: BodyId) -> Result<Vec<ContactData>> {
        let mut out = Vec::new();
        self.try_body_contacts_into(body_id, &mut out)?;
        Ok(out)
    }

    pub fn try_body_contacts_into(
        &self,
        body_id: BodyId,
        out: &mut Vec<ContactData>,
    ) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_body_belongs_locked(body_id)?;
        let capacity =
            unsafe { ffi::b3Body_GetContactCapacity(body_id.into_raw()) }.max(0) as usize;
        let raw = unsafe {
            crate::core::ffi_vec::read_from_ffi(capacity, |ptr, cap| {
                ffi::b3Body_GetContactData(body_id.into_raw(), ptr, cap)
            })
        };
        out.clear();
        out.extend(
            raw.into_iter()
                .map(|contact| unsafe { ContactData::from_raw(contact) }),
        );
        Ok(())
    }

    pub fn try_destroy_shape(&mut self, shape_id: ShapeId, update_body_mass: bool) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3DestroyShape(shape_id.into_raw(), update_body_mass) };
        drop(_guard);
        self.resources.remove(&shape_id);
        Ok(())
    }

    pub fn destroy_shape(&mut self, shape_id: ShapeId, update_body_mass: bool) {
        self.try_destroy_shape(shape_id, update_body_mass)
            .expect("invalid ShapeId");
    }

    pub fn try_shape_type(&self, shape_id: ShapeId) -> Result<ShapeType> {
        let _guard = self.lock_shape_checked(shape_id)?;
        ShapeType::from_raw(unsafe { ffi::b3Shape_GetType(shape_id.into_raw()) })
            .ok_or(Error::InvalidArgument)
    }

    pub fn try_shape_body(&self, shape_id: ShapeId) -> Result<BodyId> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(BodyId::from_raw(unsafe {
            ffi::b3Shape_GetBody(shape_id.into_raw())
        }))
    }

    pub fn try_shape_sensor(&self, shape_id: ShapeId) -> Result<bool> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_IsSensor(shape_id.into_raw()) })
    }

    pub fn try_set_shape_density(
        &mut self,
        shape_id: ShapeId,
        density: f32,
        update_body_mass: bool,
    ) -> Result<()> {
        validate_nonnegative_scalar(density)?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetDensity(shape_id.into_raw(), density, update_body_mass) };
        Ok(())
    }

    pub fn try_shape_density(&self, shape_id: ShapeId) -> Result<f32> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_GetDensity(shape_id.into_raw()) })
    }

    pub fn try_set_shape_friction(&mut self, shape_id: ShapeId, friction: f32) -> Result<()> {
        validate_nonnegative_scalar(friction)?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetFriction(shape_id.into_raw(), friction) };
        Ok(())
    }

    pub fn try_shape_friction(&self, shape_id: ShapeId) -> Result<f32> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_GetFriction(shape_id.into_raw()) })
    }

    pub fn try_set_shape_restitution(&mut self, shape_id: ShapeId, restitution: f32) -> Result<()> {
        validate_nonnegative_scalar(restitution)?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetRestitution(shape_id.into_raw(), restitution) };
        Ok(())
    }

    pub fn try_shape_restitution(&self, shape_id: ShapeId) -> Result<f32> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_GetRestitution(shape_id.into_raw()) })
    }

    pub fn try_set_shape_surface_material(
        &mut self,
        shape_id: ShapeId,
        material: SurfaceMaterial,
    ) -> Result<()> {
        material.validate()?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetSurfaceMaterial(shape_id.into_raw(), material.into_raw()) };
        Ok(())
    }

    pub fn try_shape_surface_material(&self, shape_id: ShapeId) -> Result<SurfaceMaterial> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(SurfaceMaterial::from_raw(unsafe {
            ffi::b3Shape_GetSurfaceMaterial(shape_id.into_raw())
        }))
    }

    pub fn try_shape_mesh_material_count(&self, shape_id: ShapeId) -> Result<i32> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_GetMeshMaterialCount(shape_id.into_raw()) })
    }

    pub fn try_set_shape_mesh_material(
        &mut self,
        shape_id: ShapeId,
        index: i32,
        material: SurfaceMaterial,
    ) -> Result<()> {
        material.validate()?;
        let _guard = self.lock_shape_checked(shape_id)?;
        let count = unsafe { ffi::b3Shape_GetMeshMaterialCount(shape_id.into_raw()) };
        if index < 0 || index >= count {
            return Err(Error::IndexOutOfRange);
        }
        unsafe { ffi::b3Shape_SetMeshMaterial(shape_id.into_raw(), material.into_raw(), index) };
        Ok(())
    }

    pub fn try_shape_filter(&self, shape_id: ShapeId) -> Result<Filter> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(Filter::from_raw(unsafe {
            ffi::b3Shape_GetFilter(shape_id.into_raw())
        }))
    }

    pub fn try_set_shape_filter(
        &mut self,
        shape_id: ShapeId,
        filter: Filter,
        invoke_contacts: bool,
    ) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetFilter(shape_id.into_raw(), filter.into_raw(), invoke_contacts) };
        Ok(())
    }

    pub fn try_enable_shape_sensor_events(
        &mut self,
        shape_id: ShapeId,
        enabled: bool,
    ) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_EnableSensorEvents(shape_id.into_raw(), enabled) };
        Ok(())
    }

    pub fn try_enable_shape_contact_events(
        &mut self,
        shape_id: ShapeId,
        enabled: bool,
    ) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_EnableContactEvents(shape_id.into_raw(), enabled) };
        Ok(())
    }

    pub fn try_enable_shape_pre_solve_events(
        &mut self,
        shape_id: ShapeId,
        enabled: bool,
    ) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_EnablePreSolveEvents(shape_id.into_raw(), enabled) };
        Ok(())
    }

    pub fn try_enable_shape_hit_events(&mut self, shape_id: ShapeId, enabled: bool) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_EnableHitEvents(shape_id.into_raw(), enabled) };
        Ok(())
    }

    pub fn try_shape_aabb(&self, shape_id: ShapeId) -> Result<Aabb> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(Aabb::from_raw(unsafe {
            ffi::b3Shape_GetAABB(shape_id.into_raw())
        }))
    }

    pub fn try_shape_sphere(&self, shape_id: ShapeId) -> Result<Sphere> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(Sphere::from_raw(unsafe {
            ffi::b3Shape_GetSphere(shape_id.into_raw())
        }))
    }

    pub fn try_shape_capsule(&self, shape_id: ShapeId) -> Result<Capsule> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(Capsule::from_raw(unsafe {
            ffi::b3Shape_GetCapsule(shape_id.into_raw())
        }))
    }

    pub fn try_set_shape_sphere(&mut self, shape_id: ShapeId, sphere: &Sphere) -> Result<()> {
        sphere.validate()?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetSphere(shape_id.into_raw(), sphere.raw()) };
        Ok(())
    }

    pub fn try_set_shape_capsule(&mut self, shape_id: ShapeId, capsule: &Capsule) -> Result<()> {
        capsule.validate()?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetCapsule(shape_id.into_raw(), capsule.raw()) };
        Ok(())
    }

    pub fn try_set_shape_hull(&mut self, shape_id: ShapeId, hull: &Hull) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetHull(shape_id.into_raw(), hull.as_ptr()) };
        Ok(())
    }

    pub fn try_set_shape_mesh(
        &mut self,
        shape_id: ShapeId,
        mesh: MeshData,
        scale: impl Into<Vec3>,
    ) -> Result<()> {
        let scale = scale.into().validate()?;
        let mesh_ptr = mesh.as_ptr();
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetMesh(shape_id.into_raw(), mesh_ptr, scale.into_raw()) };
        drop(_guard);
        self.resources
            .insert(shape_id, ShapeResource::Mesh { _data: mesh });
        Ok(())
    }

    #[inline]
    pub(crate) fn check_world_valid_locked(&self) -> Result<()> {
        if unsafe { ffi::b3World_IsValid(self.raw) } {
            Ok(())
        } else {
            Err(Error::InvalidWorldId)
        }
    }

    #[inline]
    fn lock_body_checked(&self, body_id: BodyId) -> Result<std::sync::MutexGuard<'static, ()>> {
        callback_state::check_not_in_callback()?;
        let guard = box3d_lock::lock();
        self.check_body_belongs_locked(body_id)?;
        Ok(guard)
    }

    #[inline]
    fn lock_shape_checked(&self, shape_id: ShapeId) -> Result<std::sync::MutexGuard<'static, ()>> {
        callback_state::check_not_in_callback()?;
        let guard = box3d_lock::lock();
        self.check_shape_belongs_locked(shape_id)?;
        Ok(guard)
    }

    #[inline]
    fn check_body_belongs_locked(&self, body_id: BodyId) -> Result<()> {
        self.check_world_valid_locked()?;
        debug_checks::check_body_valid_raw(body_id)?;
        if body_id.world0 == self.world0_locked()? {
            Ok(())
        } else {
            Err(Error::InvalidBodyId)
        }
    }

    #[inline]
    fn check_shape_belongs_locked(&self, shape_id: ShapeId) -> Result<()> {
        self.check_world_valid_locked()?;
        debug_checks::check_shape_valid_raw(shape_id)?;
        if shape_id.world0 == self.world0_locked()? {
            Ok(())
        } else {
            Err(Error::InvalidShapeId)
        }
    }

    #[inline]
    pub(crate) fn check_joint_belongs_locked(&self, joint_id: JointId) -> Result<()> {
        self.check_world_valid_locked()?;
        debug_checks::check_joint_valid_raw(joint_id)?;
        if joint_id.world0 == self.world0_locked()? {
            Ok(())
        } else {
            Err(Error::InvalidJointId)
        }
    }

    #[inline]
    fn world0_locked(&self) -> Result<u16> {
        let world0 = self
            .raw
            .index1
            .checked_sub(1)
            .ok_or(Error::InvalidWorldId)?;
        u16::try_from(world0).map_err(|_| Error::InvalidWorldId)
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

#[inline]
fn validate_scalar(value: f32) -> Result<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}

#[inline]
fn validate_nonnegative_scalar(value: f32) -> Result<()> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}

#[inline]
fn shape_id_from_raw(raw: ffi::b3ShapeId) -> Result<ShapeId> {
    if unsafe { ffi::b3Shape_IsValid(raw) } {
        Ok(ShapeId::from_raw(raw))
    } else {
        Err(Error::CreateShapeFailed)
    }
}

fn body_shape_ids_locked(body_id: BodyId) -> Vec<ShapeId> {
    let capacity = unsafe { ffi::b3Body_GetShapeCount(body_id.into_raw()) }.max(0) as usize;
    unsafe {
        ffi_vec::read_from_ffi(capacity, |ptr: *mut ffi::b3ShapeId, cap| {
            ffi::b3Body_GetShapes(body_id.into_raw(), ptr.cast(), cap)
        })
    }
    .into_iter()
    .map(ShapeId::from_raw)
    .collect()
}

impl Drop for World {
    fn drop(&mut self) {
        let _guard = box3d_lock::lock();
        if unsafe { ffi::b3World_IsValid(self.raw) } {
            self.callbacks.clear_raw_callbacks(self.raw);
            unsafe { ffi::b3DestroyWorld(self.raw) };
            crate::recording::detach_world_recording_locked(self.raw);
        }
    }
}

#[inline]
pub fn version() -> Version {
    Version::from_raw(unsafe { ffi::b3GetVersion() })
}

#[inline]
pub fn allocated_byte_count() -> i32 {
    callback_state::assert_not_in_callback();
    let _guard = box3d_lock::lock();
    unsafe { ffi::b3GetByteCount() }
}

#[inline]
pub fn is_double_precision() -> bool {
    unsafe { ffi::b3IsDoublePrecision() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn static_body(world: &mut World) -> BodyId {
        world.create_body(BodyDef::builder().body_type(BodyType::Static).build())
    }

    #[test]
    fn shape_resources_are_removed_on_shape_destroy_and_mesh_replace() {
        let mut world = World::new(WorldDef::default()).unwrap();
        let body = static_body(&mut world);
        let shape = world
            .try_create_mesh_shape(
                body,
                &ShapeDef::default(),
                MeshData::box_mesh(Vec3::ZERO, [1.0, 1.0, 1.0], true).unwrap(),
                [1.0, 1.0, 1.0],
            )
            .unwrap();
        assert_eq!(world.resources.len(), 1);

        world
            .try_set_shape_mesh(
                shape,
                MeshData::box_mesh(Vec3::ZERO, [0.5, 0.5, 0.5], true).unwrap(),
                [1.0, 1.0, 1.0],
            )
            .unwrap();
        assert_eq!(world.resources.len(), 1);

        world.try_destroy_shape(shape, true).unwrap();
        assert!(world.resources.is_empty());
    }

    #[test]
    fn shape_resources_are_removed_on_body_destroy() {
        let mut world = World::new(WorldDef::default()).unwrap();
        let body = static_body(&mut world);
        world
            .try_create_mesh_shape(
                body,
                &ShapeDef::default(),
                MeshData::box_mesh(Vec3::ZERO, [1.0, 1.0, 1.0], true).unwrap(),
                [1.0, 1.0, 1.0],
            )
            .unwrap();
        world
            .try_create_height_field_shape(
                body,
                &ShapeDef::default(),
                HeightField::grid(2, 2, [1.0, 1.0, 1.0], false).unwrap(),
            )
            .unwrap();
        world
            .try_create_compound_shape(
                body,
                &ShapeDef::default(),
                Compound::single_sphere(Sphere::new(Vec3::ZERO, 0.25), SurfaceMaterial::default())
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(world.resources.len(), 3);

        world.try_destroy_body(body).unwrap();
        assert!(world.resources.is_empty());
    }
}
