#![cfg_attr(all(target_arch = "wasm32", boxddd_wasm_provider), allow(dead_code))]

use crate::core::{box3d_lock, callback_state, material_mix_registry};
use crate::error::{Error, Result};
use crate::types::{Pos, ShapeId, Vec3};
use crate::world::World;
use boxddd_sys::ffi;
use std::ffi::c_void;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

type CustomFilterFn = dyn Fn(ShapeId, ShapeId) -> bool + Send + Sync + 'static;
type PreSolveFn = dyn Fn(ShapeId, ShapeId, Pos, Vec3) -> bool + Send + Sync + 'static;
type MaterialMixFn = dyn Fn(MaterialMixInput, MaterialMixInput) -> f32 + Send + Sync + 'static;

#[derive(Copy, Clone, Debug, PartialEq)]
/// Material data passed to a friction or restitution mixing callback.
pub struct MaterialMixInput {
    /// The coefficient currently supplied by Box3D for the material.
    pub coefficient: f32,
    /// User-defined material id stored on the source surface material.
    pub user_material_id: u64,
}
impl MaterialMixInput {
    /// Creates a material mixing callback input.
    #[inline]
    pub const fn new(coefficient: f32, user_material_id: u64) -> Self {
        Self {
            coefficient,
            user_material_id,
        }
    }
}

pub(crate) struct CustomFilterContext {
    callback: Box<CustomFilterFn>,
    panicked: AtomicBool,
}

pub(crate) struct PreSolveContext {
    callback: Box<PreSolveFn>,
    panicked: AtomicBool,
}

pub(crate) struct MaterialMixContext {
    pub(crate) callback: Box<MaterialMixFn>,
    pub(crate) panicked: AtomicBool,
}

#[derive(Default)]
pub(crate) struct WorldCallbacks {
    custom_filter: Option<Box<CustomFilterContext>>,
    pre_solve: Option<Box<PreSolveContext>>,
    friction: Option<Box<MaterialMixContext>>,
    restitution: Option<Box<MaterialMixContext>>,
    material_slot: Option<usize>,
}

impl fmt::Debug for WorldCallbacks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorldCallbacks")
            .field("custom_filter", &self.custom_filter.is_some())
            .field("pre_solve", &self.pre_solve.is_some())
            .field("friction", &self.friction.is_some())
            .field("restitution", &self.restitution.is_some())
            .field("material_slot", &self.material_slot)
            .finish()
    }
}

impl WorldCallbacks {
    pub(crate) fn reset_panics(&self) {
        if let Some(ctx) = self.custom_filter.as_ref() {
            ctx.panicked.store(false, Ordering::Release);
        }
        if let Some(ctx) = self.pre_solve.as_ref() {
            ctx.panicked.store(false, Ordering::Release);
        }
        if let Some(ctx) = self.friction.as_ref() {
            ctx.panicked.store(false, Ordering::Release);
        }
        if let Some(ctx) = self.restitution.as_ref() {
            ctx.panicked.store(false, Ordering::Release);
        }
    }

    pub(crate) fn panicked(&self) -> bool {
        self.custom_filter
            .as_ref()
            .is_some_and(|ctx| ctx.panicked.load(Ordering::Acquire))
            || self
                .pre_solve
                .as_ref()
                .is_some_and(|ctx| ctx.panicked.load(Ordering::Acquire))
            || self
                .friction
                .as_ref()
                .is_some_and(|ctx| ctx.panicked.load(Ordering::Acquire))
            || self
                .restitution
                .as_ref()
                .is_some_and(|ctx| ctx.panicked.load(Ordering::Acquire))
    }

    pub(crate) fn clear_raw_callbacks(&mut self, world: ffi::b3WorldId) {
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = world;
            *self = Self::default();
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            unsafe {
                ffi::b3World_SetCustomFilterCallback(world, None, std::ptr::null_mut());
                ffi::b3World_SetPreSolveCallback(world, None, std::ptr::null_mut());
                ffi::b3World_SetFrictionCallback(world, None);
                ffi::b3World_SetRestitutionCallback(world, None);
            }
            self.custom_filter = None;
            self.pre_solve = None;
            self.friction = None;
            self.restitution = None;
            if let Some(slot) = self.material_slot.take() {
                material_mix_registry::set_friction_ptr(slot, std::ptr::null_mut());
                material_mix_registry::set_restitution_ptr(slot, std::ptr::null_mut());
                material_mix_registry::release_slot(slot);
            }
        }
    }

    fn ensure_material_slot(&mut self) -> Result<usize> {
        if let Some(slot) = self.material_slot {
            return Ok(slot);
        }
        let slot = material_mix_registry::acquire_slot().ok_or(Error::CallbackSlotsExhausted)?;
        self.material_slot = Some(slot);
        Ok(slot)
    }

    fn maybe_release_material_slot(&mut self) {
        let Some(slot) = self.material_slot else {
            return;
        };
        if !material_mix_registry::has_any_callback(slot) {
            material_mix_registry::release_slot(slot);
            self.material_slot = None;
        }
    }
}

unsafe extern "C" fn custom_filter_trampoline(
    shape_id_a: ffi::b3ShapeId,
    shape_id_b: ffi::b3ShapeId,
    context: *mut c_void,
) -> bool {
    if context.is_null() {
        return true;
    }
    let ctx = unsafe { &*(context as *const CustomFilterContext) };
    if ctx.panicked.load(Ordering::Relaxed) {
        return true;
    }

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = callback_state::CallbackGuard::enter();
        (ctx.callback)(ShapeId::from_raw(shape_id_a), ShapeId::from_raw(shape_id_b))
    })) {
        Ok(enabled) => enabled,
        Err(_) => {
            ctx.panicked.store(true, Ordering::SeqCst);
            true
        }
    }
}

unsafe extern "C" fn pre_solve_trampoline(
    shape_id_a: ffi::b3ShapeId,
    shape_id_b: ffi::b3ShapeId,
    point: ffi::b3Pos,
    normal: ffi::b3Vec3,
    context: *mut c_void,
) -> bool {
    if context.is_null() {
        return true;
    }
    let ctx = unsafe { &*(context as *const PreSolveContext) };
    if ctx.panicked.load(Ordering::Relaxed) {
        return true;
    }

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = callback_state::CallbackGuard::enter();
        (ctx.callback)(
            ShapeId::from_raw(shape_id_a),
            ShapeId::from_raw(shape_id_b),
            Pos::from_raw(point),
            Vec3::from_raw(normal),
        )
    })) {
        Ok(enabled) => enabled,
        Err(_) => {
            ctx.panicked.store(true, Ordering::SeqCst);
            true
        }
    }
}

impl World {
    /// Registers a custom contact filter callback.
    ///
    /// Box3D calls this when an awake dynamic contact pair is considered and at
    /// least one shape has custom filtering enabled. Return `true` to allow the
    /// collision, or `false` to disable it.
    ///
    /// The callback must be thread-safe and must not mutate the world. Safe
    /// callbacks do not receive a `World` capability, and `World` itself is not
    /// `Send` or `Sync`, so safe Rust cannot move a live world handle into the
    /// callback:
    ///
    /// ```compile_fail
    /// use boxddd::{ShapeId, World};
    ///
    /// fn register(world: &mut World, captured: World) {
    ///     world.set_custom_filter(move |_: ShapeId, _: ShapeId| {
    ///         let _ = captured.body_events();
    ///         true
    ///     });
    /// }
    /// ```
    ///
    /// The runtime callback guard remains as an FFI reentrancy boundary for raw
    /// handles, global state, and future callback surfaces. A panic raised by
    /// the callback is caught and reported after stepping as
    /// [`Error::CallbackPanicked`].
    ///
    /// Panics if the callback cannot be registered; use
    /// [`Self::try_set_custom_filter`] to handle errors explicitly.
    pub fn set_custom_filter<F>(&mut self, callback: F)
    where
        F: Fn(ShapeId, ShapeId) -> bool + Send + Sync + 'static,
    {
        self.try_set_custom_filter(callback)
            .expect("failed to register Box3D custom filter callback");
    }

    /// Tries to register a custom contact filter callback.
    ///
    /// This cannot be called from inside another Box3D callback. On Emscripten
    /// provider builds, custom Rust callbacks are reported as
    /// [`Error::UnsupportedOnWasm`].
    pub fn try_set_custom_filter<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(ShapeId, ShapeId) -> bool + Send + Sync + 'static,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = callback;
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let context = Box::new(CustomFilterContext {
                callback: Box::new(callback),
                panicked: AtomicBool::new(false),
            });
            let context_ptr = (&*context) as *const CustomFilterContext as *mut c_void;

            let _guard = box3d_lock::lock();
            self.check_world_valid_locked()?;
            self.callbacks.custom_filter = Some(context);
            unsafe {
                ffi::b3World_SetCustomFilterCallback(
                    self.raw(),
                    Some(custom_filter_trampoline),
                    context_ptr,
                )
            };
            Ok(())
        }
    }

    /// Clears the custom contact filter callback.
    ///
    /// Panics if Box3D rejects the operation; use `try_clear_custom_filter` for
    /// fallible code paths.
    pub fn clear_custom_filter(&mut self) {
        self.try_clear_custom_filter()
            .expect("failed to clear Box3D custom filter callback");
    }

    /// Tries to clear the custom contact filter callback.
    ///
    /// This cannot be called from inside another Box3D callback.
    pub fn try_clear_custom_filter(&mut self) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe {
            ffi::b3World_SetCustomFilterCallback(self.raw(), None, std::ptr::null_mut());
        }
        self.callbacks.custom_filter = None;
        Ok(())
    }

    /// Registers a pre-solve callback.
    ///
    /// Box3D calls this after contact update and before solving when a dynamic
    /// non-sensor shape has pre-solve events enabled. Return `true` to keep the
    /// contact enabled for the current step, or `false` to disable it for this
    /// step.
    ///
    /// The point and normal are the limited CCD-compatible contact data Box3D
    /// exposes to this callback, not the full manifold. The callback must be
    /// thread-safe and must not mutate the world. Safe `World` methods reject
    /// calls made from inside Box3D callbacks, and a panic raised by the
    /// callback is caught and reported after stepping as
    /// [`Error::CallbackPanicked`].
    ///
    /// Panics if the callback cannot be registered; use
    /// [`Self::try_set_pre_solve`] to handle errors explicitly.
    pub fn set_pre_solve<F>(&mut self, callback: F)
    where
        F: Fn(ShapeId, ShapeId, Pos, Vec3) -> bool + Send + Sync + 'static,
    {
        self.try_set_pre_solve(callback)
            .expect("failed to register Box3D pre-solve callback");
    }

    /// Tries to register a pre-solve callback.
    ///
    /// This cannot be called from inside another Box3D callback. On Emscripten
    /// provider builds, custom Rust callbacks are reported as
    /// [`Error::UnsupportedOnWasm`].
    pub fn try_set_pre_solve<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(ShapeId, ShapeId, Pos, Vec3) -> bool + Send + Sync + 'static,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = callback;
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let context = Box::new(PreSolveContext {
                callback: Box::new(callback),
                panicked: AtomicBool::new(false),
            });
            let context_ptr = (&*context) as *const PreSolveContext as *mut c_void;

            let _guard = box3d_lock::lock();
            self.check_world_valid_locked()?;
            self.callbacks.pre_solve = Some(context);
            unsafe {
                ffi::b3World_SetPreSolveCallback(
                    self.raw(),
                    Some(pre_solve_trampoline),
                    context_ptr,
                )
            };
            Ok(())
        }
    }

    /// Clears the pre-solve callback.
    ///
    /// Panics if Box3D rejects the operation.
    pub fn clear_pre_solve(&mut self) {
        self.try_clear_pre_solve()
            .expect("failed to clear Box3D pre-solve callback");
    }

    /// Tries to clear the pre-solve callback.
    ///
    /// This cannot be called from inside another Box3D callback.
    pub fn try_clear_pre_solve(&mut self) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe {
            ffi::b3World_SetPreSolveCallback(self.raw(), None, std::ptr::null_mut());
        }
        self.callbacks.pre_solve = None;
        Ok(())
    }

    /// Registers a friction mixing callback.
    ///
    /// Box3D calls this from worker threads while mixing two shape materials.
    /// The default upstream behavior is `sqrt(friction_a * friction_b)`.
    ///
    /// The callback receives only the two material inputs and must not mutate
    /// Box3D or application state. Panics are caught and reported after stepping
    /// as [`Error::CallbackPanicked`]. If the callback returns a non-finite
    /// coefficient, `boxddd` falls back to the default mix instead of reporting
    /// a panic.
    ///
    /// Panics if the callback cannot be registered; use
    /// [`Self::try_set_friction_callback`] to handle errors explicitly.
    pub fn set_friction_callback<F>(&mut self, callback: F)
    where
        F: Fn(MaterialMixInput, MaterialMixInput) -> f32 + Send + Sync + 'static,
    {
        self.try_set_friction_callback(callback)
            .expect("failed to register Box3D friction callback");
    }

    /// Tries to register a friction mixing callback.
    ///
    /// On Emscripten provider builds, custom Rust callbacks are reported as
    /// [`Error::UnsupportedOnWasm`].
    pub fn try_set_friction_callback<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(MaterialMixInput, MaterialMixInput) -> f32 + Send + Sync + 'static,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = callback;
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let _guard = box3d_lock::lock();
            self.check_world_valid_locked()?;
            let slot = self.callbacks.ensure_material_slot()?;
            let context = Box::new(MaterialMixContext {
                callback: Box::new(callback),
                panicked: AtomicBool::new(false),
            });
            let ptr = (&*context) as *const MaterialMixContext as *mut MaterialMixContext;
            material_mix_registry::set_friction_ptr(slot, ptr);
            self.callbacks.friction = Some(context);
            unsafe {
                ffi::b3World_SetFrictionCallback(
                    self.raw(),
                    material_mix_registry::friction_callback(slot),
                )
            };
            Ok(())
        }
    }

    /// Clears the friction mixing callback.
    ///
    /// Panics if Box3D rejects the operation.
    pub fn clear_friction_callback(&mut self) {
        self.try_clear_friction_callback()
            .expect("failed to clear Box3D friction callback");
    }

    /// Tries to clear the friction mixing callback.
    ///
    /// This cannot be called from inside another Box3D callback.
    pub fn try_clear_friction_callback(&mut self) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_SetFrictionCallback(self.raw(), None) };
        if let Some(slot) = self.callbacks.material_slot {
            material_mix_registry::set_friction_ptr(slot, std::ptr::null_mut());
        }
        self.callbacks.friction = None;
        self.callbacks.maybe_release_material_slot();
        Ok(())
    }

    /// Registers a restitution mixing callback.
    ///
    /// Box3D calls this from worker threads while mixing two shape materials.
    /// The default upstream behavior is `max(restitution_a, restitution_b)`.
    ///
    /// The callback receives only the two material inputs and must not mutate
    /// Box3D or application state. Panics are caught and reported after stepping
    /// as [`Error::CallbackPanicked`]. If the callback returns a non-finite
    /// coefficient, `boxddd` falls back to the default mix instead of reporting
    /// a panic.
    ///
    /// Panics if the callback cannot be registered; use
    /// [`Self::try_set_restitution_callback`] to handle errors explicitly.
    pub fn set_restitution_callback<F>(&mut self, callback: F)
    where
        F: Fn(MaterialMixInput, MaterialMixInput) -> f32 + Send + Sync + 'static,
    {
        self.try_set_restitution_callback(callback)
            .expect("failed to register Box3D restitution callback");
    }

    /// Tries to register a restitution mixing callback.
    ///
    /// On Emscripten provider builds, custom Rust callbacks are reported as
    /// [`Error::UnsupportedOnWasm`].
    pub fn try_set_restitution_callback<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(MaterialMixInput, MaterialMixInput) -> f32 + Send + Sync + 'static,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = callback;
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let _guard = box3d_lock::lock();
            self.check_world_valid_locked()?;
            let slot = self.callbacks.ensure_material_slot()?;
            let context = Box::new(MaterialMixContext {
                callback: Box::new(callback),
                panicked: AtomicBool::new(false),
            });
            let ptr = (&*context) as *const MaterialMixContext as *mut MaterialMixContext;
            material_mix_registry::set_restitution_ptr(slot, ptr);
            self.callbacks.restitution = Some(context);
            unsafe {
                ffi::b3World_SetRestitutionCallback(
                    self.raw(),
                    material_mix_registry::restitution_callback(slot),
                )
            };
            Ok(())
        }
    }

    /// Clears the restitution mixing callback.
    ///
    /// Panics if Box3D rejects the operation.
    pub fn clear_restitution_callback(&mut self) {
        self.try_clear_restitution_callback()
            .expect("failed to clear Box3D restitution callback");
    }

    /// Tries to clear the restitution mixing callback.
    ///
    /// This cannot be called from inside another Box3D callback.
    pub fn try_clear_restitution_callback(&mut self) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_SetRestitutionCallback(self.raw(), None) };
        if let Some(slot) = self.callbacks.material_slot {
            material_mix_registry::set_restitution_ptr(slot, std::ptr::null_mut());
        }
        self.callbacks.restitution = None;
        self.callbacks.maybe_release_material_slot();
        Ok(())
    }
}
