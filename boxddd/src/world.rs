use crate::TaskSystem;
use crate::body::{BodyDef, BodyType};
use crate::callbacks::WorldCallbacks;
use crate::core::{box3d_lock, callback_state, debug_checks, ffi_vec, task_system, wasm};
use crate::debug_draw::DebugShapeRegistry;
#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
use crate::debug_draw::{create_debug_shape, destroy_debug_shape};
use crate::error::{Error, Result};
use crate::shapes::{
    BoxHull, Capsule, Compound, HeightField, Hull, MeshData, ShapeDef, ShapeHeightField, ShapeHull,
    ShapeMesh, ShapeType, Sphere, SurfaceMaterial, validate_mesh_scale,
};
use crate::types::{
    Aabb, BodyId, Capacity, ContactData, ContactId, Counters, Filter, JointId, MassData, Matrix3,
    MotionLocks, Pos, Profile, Quat, ShapeId, Vec3, Version, WorldTransform,
};
use boxddd_sys::ffi;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::rc::Rc;

/// Configuration used when creating a Box3D world.
#[derive(Clone, Debug)]
pub struct WorldDef {
    raw: ffi::b3WorldDef,
    task_system: Option<TaskSystem>,
}

impl Default for WorldDef {
    fn default() -> Self {
        Self {
            raw: unsafe { ffi::b3DefaultWorldDef() },
            task_system: None,
        }
    }
}

impl WorldDef {
    /// Starts a builder with Box3D defaults.
    #[inline]
    pub fn builder() -> WorldDefBuilder {
        WorldDefBuilder::new()
    }

    /// Returns the underlying Box3D FFI value.
    #[inline]
    pub fn raw(&self) -> &ffi::b3WorldDef {
        &self.raw
    }

    /// Validates the value before it is passed to Box3D.
    pub fn validate(&self) -> Result<()> {
        #[cfg(target_arch = "wasm32")]
        if self.raw.workerCount > 1 || self.task_system.is_some() {
            return Err(Error::UnsupportedOnWasm);
        }

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

/// Builder for `WorldDef`.
#[derive(Clone, Debug)]
pub struct WorldDefBuilder {
    def: WorldDef,
}

impl WorldDefBuilder {
    /// Creates a new value with default settings.
    #[inline]
    pub fn new() -> Self {
        Self {
            def: WorldDef::default(),
        }
    }

    /// Sets the gravity vector used by the world, usually in meters per second squared.
    #[inline]
    pub fn gravity(mut self, gravity: impl Into<Vec3>) -> Self {
        self.def.raw.gravity = gravity.into().into_raw();
        self
    }

    /// Sets the number of worker slots Box3D may use while stepping the world.
    #[inline]
    pub fn worker_count(mut self, worker_count: u32) -> Self {
        self.def.raw.workerCount = worker_count.max(1);
        self
    }

    /// Installs the task-system adapter used by Box3D during `World::step`.
    #[inline]
    pub fn task_system(mut self, task_system: TaskSystem) -> Self {
        self.def.task_system = Some(task_system);
        self
    }

    /// Finishes the builder and returns the configured value.
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

/// Configuration for applying an explosion impulse in a world.
#[derive(Clone, Debug)]
pub struct ExplosionDef {
    raw: ffi::b3ExplosionDef,
}

impl Default for ExplosionDef {
    fn default() -> Self {
        Self {
            raw: unsafe { ffi::b3DefaultExplosionDef() },
        }
    }
}

impl ExplosionDef {
    /// Starts a builder with Box3D defaults.
    #[inline]
    pub fn builder() -> ExplosionDefBuilder {
        ExplosionDefBuilder::new()
    }

    /// Returns the underlying Box3D FFI value.
    #[inline]
    pub fn raw(&self) -> &ffi::b3ExplosionDef {
        &self.raw
    }

    /// Validates the value before it is passed to Box3D.
    pub fn validate(&self) -> Result<()> {
        Pos::from_raw(self.raw.position).validate()?;
        if self.raw.radius.is_finite()
            && self.raw.radius > 0.0
            && self.raw.falloff.is_finite()
            && self.raw.falloff >= 0.0
            && self.raw.impulsePerArea.is_finite()
        {
            Ok(())
        } else {
            Err(Error::InvalidArgument)
        }
    }
}

/// Builder for `ExplosionDef`.
#[derive(Clone, Debug)]
pub struct ExplosionDefBuilder {
    def: ExplosionDef,
}

impl ExplosionDefBuilder {
    /// Creates a new value with default settings.
    #[inline]
    pub fn new() -> Self {
        Self {
            def: ExplosionDef::default(),
        }
    }

    /// Sets the collision mask bits matched by the query.
    #[inline]
    pub fn mask_bits(mut self, mask_bits: u64) -> Self {
        self.def.raw.maskBits = mask_bits;
        self
    }

    /// Sets the world-space center of the explosion.
    #[inline]
    pub fn position(mut self, position: impl Into<Pos>) -> Self {
        self.def.raw.position = position.into().into_raw();
        self
    }

    /// Sets the explosion radius.
    #[inline]
    pub fn radius(mut self, radius: f32) -> Self {
        self.def.raw.radius = radius;
        self
    }

    /// Sets how quickly the explosion impulse decays over distance.
    #[inline]
    pub fn falloff(mut self, falloff: f32) -> Self {
        self.def.raw.falloff = falloff;
        self
    }

    /// Sets the impulse applied per affected surface area.
    #[inline]
    pub fn impulse_per_area(mut self, impulse_per_area: f32) -> Self {
        self.def.raw.impulsePerArea = impulse_per_area;
        self
    }

    /// Finishes the builder and returns the configured value.
    #[inline]
    pub fn build(self) -> ExplosionDef {
        self.def
    }
}

impl Default for ExplosionDefBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Owning handle for a Box3D world.
#[derive(Debug)]
pub struct World {
    raw: ffi::b3WorldId,
    resources: HashMap<ShapeId, ShapeResource>,
    pub(crate) callbacks: WorldCallbacks,
    task_system: Option<TaskSystem>,
    pub(crate) debug_shapes: Box<DebugShapeRegistry>,
    _not_send_sync: PhantomData<Rc<()>>,
}

#[derive(Debug)]
enum ShapeResource {
    Mesh { _data: MeshData },
    HeightField { _data: HeightField },
    Compound { _data: Compound },
}

mod body_api;
mod creation;
mod runtime;
mod shape_api;

impl World {
    /// Creates a new value with default settings.
    pub fn new(def: WorldDef) -> Result<Self> {
        callback_state::check_not_in_callback()?;
        def.validate()?;

        let debug_shapes = Box::<DebugShapeRegistry>::default();
        let task_system = def.task_system.clone();
        let mut raw_def = *def.raw();
        if let Some(task_system) = task_system.as_ref() {
            task_system.reset_panics();
            task_system::install_callbacks(&mut raw_def, task_system);
        }
        if !wasm::is_provider_mode() {
            #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
            {
                raw_def.createDebugShape = Some(create_debug_shape);
                raw_def.destroyDebugShape = Some(destroy_debug_shape);
                raw_def.userDebugShapeContext =
                    (&*debug_shapes) as *const DebugShapeRegistry as *mut std::ffi::c_void;
            }
        }

        let _guard = box3d_lock::lock();
        let raw = unsafe { create_world_raw(&raw_def) };
        if unsafe { ffi::b3World_IsValid(raw) } {
            Ok(Self {
                raw,
                resources: HashMap::new(),
                callbacks: WorldCallbacks::default(),
                task_system,
                debug_shapes,
                _not_send_sync: PhantomData,
            })
        } else {
            Err(Error::CreateWorldFailed)
        }
    }

    /// Returns the underlying Box3D FFI value.
    #[inline]
    pub fn raw(&self) -> ffi::b3WorldId {
        self.raw
    }

    /// Returns whether the Box3D handle is still valid.
    #[inline]
    pub fn is_valid(&self) -> bool {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3World_IsValid(self.raw) }
    }
    pub(crate) fn check_world_valid_locked(&self) -> Result<()> {
        if unsafe { ffi::b3World_IsValid(self.raw) } {
            Ok(())
        } else {
            Err(Error::InvalidWorldId)
        }
    }

    #[inline]
    pub(crate) fn lock_body_checked(
        &self,
        body_id: BodyId,
    ) -> Result<std::sync::MutexGuard<'static, ()>> {
        callback_state::check_not_in_callback()?;
        let guard = box3d_lock::lock();
        self.check_body_belongs_locked(body_id)?;
        Ok(guard)
    }

    #[inline]
    pub(crate) fn lock_shape_checked(
        &self,
        shape_id: ShapeId,
    ) -> Result<std::sync::MutexGuard<'static, ()>> {
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
    pub(crate) fn check_contact_belongs_locked(&self, contact_id: ContactId) -> Result<()> {
        self.check_world_valid_locked()?;
        debug_checks::check_contact_valid_raw(contact_id)?;
        if contact_id.world0 == self.world0_locked()? {
            Ok(())
        } else {
            Err(Error::InvalidContactId)
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
fn validate_positive_scalar(value: f32) -> Result<()> {
    if value.is_finite() && value > 0.0 {
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
            self.debug_shapes.clear_all();
            unsafe { ffi::b3DestroyWorld(self.raw) };
            crate::recording::detach_world_recording_locked(self.raw);
        }
    }
}

/// Returns the Box3D runtime version.
#[inline]
pub fn version() -> Version {
    Version::from_raw(unsafe { ffi::b3GetVersion() })
}

/// Returns the number of bytes currently allocated by Box3D.
#[inline]
pub fn allocated_byte_count() -> i32 {
    callback_state::assert_not_in_callback();
    let _guard = box3d_lock::lock();
    unsafe { ffi::b3GetByteCount() }
}

/// Returns whether the linked Box3D library uses double precision.
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

        world
            .try_set_shape_sphere(shape, &Sphere::new(Vec3::ZERO, 0.25))
            .unwrap();
        assert!(world.resources.is_empty());

        world.try_destroy_shape(shape, true).unwrap();
        assert!(world.resources.is_empty());
    }

    #[test]
    fn resource_backed_shapes_keep_body_static() {
        let mut world = World::new(WorldDef::default()).unwrap();
        let body = static_body(&mut world);
        world
            .try_create_height_field_shape(
                body,
                &ShapeDef::default(),
                HeightField::grid(2, 2, [1.0, 1.0, 1.0], false).unwrap(),
            )
            .unwrap();

        assert_eq!(
            world
                .try_set_body_type(body, BodyType::Dynamic)
                .unwrap_err(),
            Error::InvalidArgument
        );
        assert_eq!(world.try_body_type(body).unwrap(), BodyType::Static);

        world
            .try_create_sphere_shape(
                body,
                &ShapeDef::default(),
                &Sphere::new(Vec3::new(2.0, 0.0, 0.0), 0.25),
            )
            .unwrap();
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
