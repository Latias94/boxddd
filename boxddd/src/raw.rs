//! Explicit raw interop boundary for Box3D APIs that cannot be made ordinary safe Rust APIs.
//!
//! This module is intentionally not re-exported by `boxddd::prelude`. Functions here preserve
//! the crate's handle validation and Box3D global lock, but they still expose native concepts
//! such as process-global settings and untyped `void*` user data.

use crate::core::{box3d_lock, callback_state};
use crate::error::{Error, Result};
use crate::joints::lock_joint_checked;
use crate::types::{BodyId, JointId, ShapeId};
use crate::world::World;
use boxddd_sys::ffi;
use std::ffi::c_void;

/// Sets the raw Box3D `userData` pointer attached to a world.
///
/// # Safety
///
/// The caller must ensure `user_data` remains valid for every native Box3D use and must not rely
/// on `boxddd` to manage, alias-check, or drop the pointed-to value.
pub unsafe fn try_set_world_raw_user_data(world: &mut World, user_data: *mut c_void) -> Result<()> {
    callback_state::check_not_in_callback()?;
    let _guard = box3d_lock::lock();
    world.check_world_valid_locked()?;
    unsafe { ffi::b3World_SetUserData(world.raw(), user_data) };
    Ok(())
}

/// Returns the raw Box3D `userData` pointer attached to a world.
///
/// # Safety
///
/// The returned pointer is not validated by `boxddd`. The caller is responsible for interpreting
/// it only according to the ownership and lifetime contract used when it was stored.
pub unsafe fn try_world_raw_user_data(world: &World) -> Result<*mut c_void> {
    callback_state::check_not_in_callback()?;
    let _guard = box3d_lock::lock();
    world.check_world_valid_locked()?;
    Ok(unsafe { ffi::b3World_GetUserData(world.raw()) })
}

/// Sets the raw Box3D `userData` pointer attached to a body.
///
/// # Safety
///
/// The caller must ensure `user_data` remains valid for every native Box3D use and must not rely
/// on `boxddd` to manage, alias-check, or drop the pointed-to value.
pub unsafe fn try_set_body_raw_user_data(
    world: &mut World,
    body_id: BodyId,
    user_data: *mut c_void,
) -> Result<()> {
    let _guard = world.lock_body_checked(body_id)?;
    unsafe { ffi::b3Body_SetUserData(body_id.into_raw(), user_data) };
    Ok(())
}

/// Returns the raw Box3D `userData` pointer attached to a body.
///
/// # Safety
///
/// The returned pointer is not validated by `boxddd`. The caller is responsible for interpreting
/// it only according to the ownership and lifetime contract used when it was stored.
pub unsafe fn try_body_raw_user_data(world: &World, body_id: BodyId) -> Result<*mut c_void> {
    let _guard = world.lock_body_checked(body_id)?;
    Ok(unsafe { ffi::b3Body_GetUserData(body_id.into_raw()) })
}

/// Sets the raw Box3D `userData` pointer attached to a shape.
///
/// # Safety
///
/// The caller must ensure `user_data` remains valid for every native Box3D use and must not rely
/// on `boxddd` to manage, alias-check, or drop the pointed-to value.
pub unsafe fn try_set_shape_raw_user_data(
    world: &mut World,
    shape_id: ShapeId,
    user_data: *mut c_void,
) -> Result<()> {
    let _guard = world.lock_shape_checked(shape_id)?;
    unsafe { ffi::b3Shape_SetUserData(shape_id.into_raw(), user_data) };
    Ok(())
}

/// Returns the raw Box3D `userData` pointer attached to a shape.
///
/// # Safety
///
/// The returned pointer is not validated by `boxddd`. The caller is responsible for interpreting
/// it only according to the ownership and lifetime contract used when it was stored.
pub unsafe fn try_shape_raw_user_data(world: &World, shape_id: ShapeId) -> Result<*mut c_void> {
    let _guard = world.lock_shape_checked(shape_id)?;
    Ok(unsafe { ffi::b3Shape_GetUserData(shape_id.into_raw()) })
}

/// Sets the raw Box3D `userData` pointer attached to a joint.
///
/// # Safety
///
/// The caller must ensure `user_data` remains valid for every native Box3D use and must not rely
/// on `boxddd` to manage, alias-check, or drop the pointed-to value.
pub unsafe fn try_set_joint_raw_user_data(
    world: &mut World,
    joint_id: JointId,
    user_data: *mut c_void,
) -> Result<()> {
    let _guard = lock_joint_checked(world, joint_id)?;
    unsafe { ffi::b3Joint_SetUserData(joint_id.into_raw(), user_data) };
    Ok(())
}

/// Returns the raw Box3D `userData` pointer attached to a joint.
///
/// # Safety
///
/// The returned pointer is not validated by `boxddd`. The caller is responsible for interpreting
/// it only according to the ownership and lifetime contract used when it was stored.
pub unsafe fn try_joint_raw_user_data(world: &World, joint_id: JointId) -> Result<*mut c_void> {
    let _guard = lock_joint_checked(world, joint_id)?;
    Ok(unsafe { ffi::b3Joint_GetUserData(joint_id.into_raw()) })
}

/// Returns Box3D's process-global length unit scale.
///
/// This is process-global native state. Prefer leaving it unchanged unless interoperating with
/// existing Box3D code that has intentionally changed the unit scale.
pub fn length_units_per_meter() -> f32 {
    try_length_units_per_meter().expect("boxddd raw API called from a Box3D callback")
}

/// Tries to return Box3D's process-global length unit scale.
pub fn try_length_units_per_meter() -> Result<f32> {
    callback_state::check_not_in_callback()?;
    let _guard = box3d_lock::lock();
    Ok(unsafe { ffi::b3GetLengthUnitsPerMeter() })
}

/// Sets Box3D's process-global length unit scale.
///
/// The value must be finite and greater than zero.
pub fn try_set_length_units_per_meter(length_units: f32) -> Result<()> {
    validate_positive(length_units)?;
    callback_state::check_not_in_callback()?;
    let _guard = box3d_lock::lock();
    unsafe { ffi::b3SetLengthUnitsPerMeter(length_units) };
    Ok(())
}

/// Returns Box3D's process-global stall threshold in seconds.
pub fn stall_threshold() -> f32 {
    try_stall_threshold().expect("boxddd raw API called from a Box3D callback")
}

/// Tries to return Box3D's process-global stall threshold in seconds.
pub fn try_stall_threshold() -> Result<f32> {
    callback_state::check_not_in_callback()?;
    let _guard = box3d_lock::lock();
    Ok(unsafe { ffi::b3GetStallThreshold() })
}

/// Sets Box3D's process-global stall threshold in seconds.
///
/// The value must be finite and non-negative.
pub fn try_set_stall_threshold(seconds: f32) -> Result<()> {
    validate_nonnegative(seconds)?;
    callback_state::check_not_in_callback()?;
    let _guard = box3d_lock::lock();
    unsafe { ffi::b3SetStallThreshold(seconds) };
    Ok(())
}

#[inline]
fn validate_positive(value: f32) -> Result<()> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}

#[inline]
fn validate_nonnegative(value: f32) -> Result<()> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}
