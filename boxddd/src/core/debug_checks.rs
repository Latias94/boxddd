use crate::error::{Error, Result};
use crate::types::{BodyId, JointId, ShapeId};
use boxddd_sys::ffi;

#[inline]
pub(crate) fn check_body_valid_raw(id: BodyId) -> Result<()> {
    if unsafe { ffi::b3Body_IsValid(id.into_raw()) } {
        Ok(())
    } else {
        Err(Error::InvalidBodyId)
    }
}

#[inline]
pub(crate) fn check_shape_valid_raw(id: ShapeId) -> Result<()> {
    if unsafe { ffi::b3Shape_IsValid(id.into_raw()) } {
        Ok(())
    } else {
        Err(Error::InvalidShapeId)
    }
}

#[inline]
pub(crate) fn check_joint_valid_raw(id: JointId) -> Result<()> {
    if unsafe { ffi::b3Joint_IsValid(id.into_raw()) } {
        Ok(())
    } else {
        Err(Error::InvalidJointId)
    }
}
