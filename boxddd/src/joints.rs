use crate::core::{box3d_lock, callback_state, debug_checks};
use crate::error::{Error, Result};
use crate::types::{BodyId, JointId, Quat, Transform, Vec3};
use crate::world::World;
use boxddd_sys::ffi;
use std::ffi::c_void;

mod defs;
pub use defs::*;

macro_rules! family_method {
    ($world:expr, $joint:expr, $ty:expr, $body:block) => {{
        let _guard = lock_typed_joint_checked($world, $joint, $ty)?;
        let result = $body;
        Ok(result)
    }};
}

mod distance;
mod motor;
mod parallel;
mod prismatic;
mod revolute;
mod spherical;
mod wheel;
mod world_api;

fn validate_base(base: &ffi::b3JointDef) -> Result<()> {
    Transform::from_raw(base.localFrameA).validate()?;
    Transform::from_raw(base.localFrameB).validate()?;
    validate_nonnegative_scalar(base.forceThreshold)?;
    validate_nonnegative_scalar(base.torqueThreshold)?;
    validate_nonnegative_scalar(base.constraintHertz)?;
    validate_nonnegative_scalar(base.constraintDampingRatio)?;
    validate_nonnegative_scalar(base.drawScale)
}

#[inline]
fn check_joint_body_pair_valid(body_a: BodyId, body_b: BodyId) -> Result<()> {
    if body_a.world0 == body_b.world0 && body_a != body_b {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}

#[inline]
fn check_joint_targets_world(
    world_id: ffi::b3WorldId,
    body_a: BodyId,
    body_b: BodyId,
) -> Result<()> {
    let world0 = world_id
        .index1
        .checked_sub(1)
        .ok_or(Error::InvalidWorldId)?;
    if body_a.world0 == world0 && body_b.world0 == world0 {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
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
fn validate_range(lower: f32, upper: f32) -> Result<()> {
    validate_scalar(lower)?;
    validate_scalar(upper)?;
    if lower <= upper {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}

#[inline]
fn validate_length_range(lower: f32, upper: f32) -> Result<()> {
    validate_nonnegative_scalar(lower)?;
    validate_nonnegative_scalar(upper)?;
    if lower <= upper {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}

#[inline]
pub(crate) fn lock_joint_checked(
    world: &World,
    joint_id: JointId,
) -> Result<std::sync::MutexGuard<'static, ()>> {
    callback_state::check_not_in_callback()?;
    let guard = box3d_lock::lock();
    world.check_joint_belongs_locked(joint_id)?;
    Ok(guard)
}

#[inline]
fn lock_typed_joint_checked(
    world: &World,
    joint_id: JointId,
    expected: JointType,
) -> Result<std::sync::MutexGuard<'static, ()>> {
    let guard = lock_joint_checked(world, joint_id)?;
    let actual = JointType::from_raw(unsafe { ffi::b3Joint_GetType(joint_id.into_raw()) })
        .ok_or(Error::WrongJointType)?;
    if actual == expected {
        Ok(guard)
    } else {
        Err(Error::WrongJointType)
    }
}

#[inline]
fn joint_id_from_raw(raw: ffi::b3JointId) -> Result<JointId> {
    if unsafe { ffi::b3Joint_IsValid(raw) } {
        Ok(JointId::from_raw(raw))
    } else {
        Err(Error::InvalidJointId)
    }
}
