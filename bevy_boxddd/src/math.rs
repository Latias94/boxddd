//! Bevy math adapters for `boxddd` value types.
//!
//! The core `boxddd` crate stays engine-independent. These helpers live in the
//! Bevy adapter crate so Bevy applications do not need to repeat small
//! conversion functions in every system or example.

use bevy_math::{Quat as BevyQuat, Vec3 as BevyVec3};
use bevy_transform::components::Transform as BevyTransform;

/// Converts a Bevy `Vec3` to a local-space Box3D vector.
#[inline]
pub fn to_boxddd_vec3(value: BevyVec3) -> boxddd::Vec3 {
    boxddd::Vec3::new(value.x, value.y, value.z)
}

/// Converts a Bevy `Vec3` to a Box3D world position.
#[inline]
pub fn to_boxddd_pos(value: BevyVec3) -> boxddd::Pos {
    boxddd::Pos::new(value.x.into(), value.y.into(), value.z.into())
}

/// Converts a Bevy quaternion to a Box3D quaternion without validation.
///
/// Most Bevy rotations are already normalized. Use [`try_to_boxddd_quat`] when
/// accepting arbitrary user input.
#[inline]
pub fn to_boxddd_quat(value: BevyQuat) -> boxddd::Quat {
    boxddd::Quat::new(boxddd::Vec3::new(value.x, value.y, value.z), value.w)
}

/// Converts and validates a Bevy quaternion.
#[inline]
pub fn try_to_boxddd_quat(value: BevyQuat) -> boxddd::Result<boxddd::Quat> {
    to_boxddd_quat(value).validate()
}

/// Converts a Bevy transform to a Box3D local transform, ignoring Bevy scale.
#[inline]
pub fn to_boxddd_local_transform(value: BevyTransform) -> boxddd::Transform {
    boxddd::Transform::new(
        to_boxddd_vec3(value.translation),
        to_boxddd_quat(value.rotation),
    )
}

/// Converts a Bevy transform to a Box3D world transform, ignoring Bevy scale.
#[inline]
pub fn to_boxddd_world_transform(value: BevyTransform) -> boxddd::WorldTransform {
    boxddd::WorldTransform::new(
        to_boxddd_pos(value.translation),
        to_boxddd_quat(value.rotation),
    )
}

/// Converts a Box3D local-space vector to a Bevy `Vec3`.
#[inline]
pub fn to_bevy_vec3(value: boxddd::Vec3) -> BevyVec3 {
    BevyVec3::new(value.x, value.y, value.z)
}

/// Converts a Box3D world position to a Bevy `Vec3`.
///
/// In double-precision builds this intentionally casts the world position down
/// to Bevy's `f32` coordinate type.
#[inline]
pub fn to_bevy_pos(value: boxddd::Pos) -> BevyVec3 {
    BevyVec3::new(value.x as f32, value.y as f32, value.z as f32)
}

/// Converts a Box3D quaternion to a Bevy quaternion.
#[inline]
pub fn to_bevy_quat(value: boxddd::Quat) -> BevyQuat {
    BevyQuat::from_xyzw(value.v.x, value.v.y, value.v.z, value.s)
}

/// Converts a Box3D local transform to a Bevy transform with unit scale.
#[inline]
pub fn to_bevy_local_transform(value: boxddd::Transform) -> BevyTransform {
    BevyTransform::from_translation(to_bevy_vec3(value.p)).with_rotation(to_bevy_quat(value.q))
}

/// Converts a Box3D world transform to a Bevy transform with unit scale.
#[inline]
pub fn to_bevy_transform(value: boxddd::WorldTransform) -> BevyTransform {
    BevyTransform::from_translation(to_bevy_pos(value.p)).with_rotation(to_bevy_quat(value.q))
}

/// Applies a Box3D world transform to an existing Bevy transform, preserving scale.
#[inline]
pub fn apply_boxddd_transform(target: &mut BevyTransform, value: boxddd::WorldTransform) {
    target.translation = to_bevy_pos(value.p);
    target.rotation = to_bevy_quat(value.q);
}

/// Extension methods for converting Bevy vectors to Box3D values.
pub trait BevyVec3BoxdddExt {
    /// Converts this Bevy vector to a local-space Box3D vector.
    fn to_boxddd_vec3(self) -> boxddd::Vec3;

    /// Converts this Bevy vector to a Box3D world position.
    fn to_boxddd_pos(self) -> boxddd::Pos;
}

impl BevyVec3BoxdddExt for BevyVec3 {
    #[inline]
    fn to_boxddd_vec3(self) -> boxddd::Vec3 {
        to_boxddd_vec3(self)
    }

    #[inline]
    fn to_boxddd_pos(self) -> boxddd::Pos {
        to_boxddd_pos(self)
    }
}

/// Extension methods for converting Bevy quaternions to Box3D values.
pub trait BevyQuatBoxdddExt {
    /// Converts this Bevy quaternion without validation.
    fn to_boxddd_quat(self) -> boxddd::Quat;

    /// Converts and validates this Bevy quaternion.
    fn try_to_boxddd_quat(self) -> boxddd::Result<boxddd::Quat>;
}

impl BevyQuatBoxdddExt for BevyQuat {
    #[inline]
    fn to_boxddd_quat(self) -> boxddd::Quat {
        to_boxddd_quat(self)
    }

    #[inline]
    fn try_to_boxddd_quat(self) -> boxddd::Result<boxddd::Quat> {
        try_to_boxddd_quat(self)
    }
}

/// Extension methods for converting Bevy transforms to Box3D transforms.
pub trait BevyTransformBoxdddExt {
    /// Converts translation and rotation to a Box3D local transform.
    fn to_boxddd_local_transform(self) -> boxddd::Transform;

    /// Converts translation and rotation to a Box3D world transform.
    fn to_boxddd_world_transform(self) -> boxddd::WorldTransform;
}

impl BevyTransformBoxdddExt for BevyTransform {
    #[inline]
    fn to_boxddd_local_transform(self) -> boxddd::Transform {
        to_boxddd_local_transform(self)
    }

    #[inline]
    fn to_boxddd_world_transform(self) -> boxddd::WorldTransform {
        to_boxddd_world_transform(self)
    }
}

/// Extension method for converting Box3D vectors and positions to Bevy vectors.
pub trait BoxdddVec3BevyExt {
    /// Converts this value to Bevy's `Vec3`.
    fn to_bevy_vec3(self) -> BevyVec3;
}

impl BoxdddVec3BevyExt for boxddd::Vec3 {
    #[inline]
    fn to_bevy_vec3(self) -> BevyVec3 {
        to_bevy_vec3(self)
    }
}

impl BoxdddVec3BevyExt for boxddd::Pos {
    #[inline]
    fn to_bevy_vec3(self) -> BevyVec3 {
        to_bevy_pos(self)
    }
}

/// Extension method for converting Box3D quaternions to Bevy quaternions.
pub trait BoxdddQuatBevyExt {
    /// Converts this value to Bevy's `Quat`.
    fn to_bevy_quat(self) -> BevyQuat;
}

impl BoxdddQuatBevyExt for boxddd::Quat {
    #[inline]
    fn to_bevy_quat(self) -> BevyQuat {
        to_bevy_quat(self)
    }
}

/// Extension methods for converting Box3D transforms to Bevy transforms.
pub trait BoxdddTransformBevyExt {
    /// Converts this value to a Bevy transform with unit scale.
    fn to_bevy_transform(self) -> BevyTransform;

    /// Applies translation and rotation to an existing Bevy transform.
    fn apply_to_bevy_transform(self, target: &mut BevyTransform);
}

impl BoxdddTransformBevyExt for boxddd::Transform {
    #[inline]
    fn to_bevy_transform(self) -> BevyTransform {
        to_bevy_local_transform(self)
    }

    #[inline]
    fn apply_to_bevy_transform(self, target: &mut BevyTransform) {
        target.translation = to_bevy_vec3(self.p);
        target.rotation = to_bevy_quat(self.q);
    }
}

impl BoxdddTransformBevyExt for boxddd::WorldTransform {
    #[inline]
    fn to_bevy_transform(self) -> BevyTransform {
        to_bevy_transform(self)
    }

    #[inline]
    fn apply_to_bevy_transform(self, target: &mut BevyTransform) {
        apply_boxddd_transform(target, self);
    }
}
