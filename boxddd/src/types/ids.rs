use super::*;
#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BodyId {
    pub index1: i32,
    pub world0: u16,
    pub generation: u16,
}

impl BodyId {
    #[inline]
    pub const fn from_raw(raw: ffi::b3BodyId) -> Self {
        Self {
            index1: raw.index1,
            world0: raw.world0,
            generation: raw.generation,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3BodyId {
        ffi::b3BodyId {
            index1: self.index1,
            world0: self.world0,
            generation: self.generation,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3Body_IsValid(self.into_raw()) }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ShapeId {
    pub index1: i32,
    pub world0: u16,
    pub generation: u16,
}

impl ShapeId {
    #[inline]
    pub const fn from_raw(raw: ffi::b3ShapeId) -> Self {
        Self {
            index1: raw.index1,
            world0: raw.world0,
            generation: raw.generation,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3ShapeId {
        ffi::b3ShapeId {
            index1: self.index1,
            world0: self.world0,
            generation: self.generation,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3Shape_IsValid(self.into_raw()) }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct JointId {
    pub index1: i32,
    pub world0: u16,
    pub generation: u16,
}

impl JointId {
    #[inline]
    pub const fn from_raw(raw: ffi::b3JointId) -> Self {
        Self {
            index1: raw.index1,
            world0: raw.world0,
            generation: raw.generation,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3JointId {
        ffi::b3JointId {
            index1: self.index1,
            world0: self.world0,
            generation: self.generation,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3Joint_IsValid(self.into_raw()) }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ContactId {
    pub index1: i32,
    pub world0: u16,
    pub padding: i16,
    pub generation: u32,
}

impl ContactId {
    #[inline]
    pub const fn from_raw(raw: ffi::b3ContactId) -> Self {
        Self {
            index1: raw.index1,
            world0: raw.world0,
            padding: raw.padding,
            generation: raw.generation,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3ContactId {
        ffi::b3ContactId {
            index1: self.index1,
            world0: self.world0,
            padding: self.padding,
            generation: self.generation,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3Contact_IsValid(self.into_raw()) }
    }
}

const _: () = {
    assert!(::core::mem::size_of::<Vec2>() == ::core::mem::size_of::<ffi::b3Vec2>());
    assert!(::core::mem::align_of::<Vec2>() == ::core::mem::align_of::<ffi::b3Vec2>());
    assert!(::core::mem::size_of::<Vec3>() == ::core::mem::size_of::<ffi::b3Vec3>());
    assert!(::core::mem::align_of::<Vec3>() == ::core::mem::align_of::<ffi::b3Vec3>());
    assert!(::core::mem::size_of::<Quat>() == ::core::mem::size_of::<ffi::b3Quat>());
    assert!(::core::mem::align_of::<Quat>() == ::core::mem::align_of::<ffi::b3Quat>());
    assert!(::core::mem::size_of::<Transform>() == ::core::mem::size_of::<ffi::b3Transform>());
    assert!(::core::mem::align_of::<Transform>() == ::core::mem::align_of::<ffi::b3Transform>());
    assert!(::core::mem::size_of::<Pos>() == ::core::mem::size_of::<ffi::b3Pos>());
    assert!(::core::mem::align_of::<Pos>() == ::core::mem::align_of::<ffi::b3Pos>());
    assert!(
        ::core::mem::size_of::<WorldTransform>() == ::core::mem::size_of::<ffi::b3WorldTransform>()
    );
    assert!(
        ::core::mem::align_of::<WorldTransform>()
            == ::core::mem::align_of::<ffi::b3WorldTransform>()
    );
    assert!(::core::mem::size_of::<Matrix3>() == ::core::mem::size_of::<ffi::b3Matrix3>());
    assert!(::core::mem::align_of::<Matrix3>() == ::core::mem::align_of::<ffi::b3Matrix3>());
    assert!(::core::mem::size_of::<Aabb>() == ::core::mem::size_of::<ffi::b3AABB>());
    assert!(::core::mem::align_of::<Aabb>() == ::core::mem::align_of::<ffi::b3AABB>());
    assert!(::core::mem::size_of::<Plane>() == ::core::mem::size_of::<ffi::b3Plane>());
    assert!(::core::mem::align_of::<Plane>() == ::core::mem::align_of::<ffi::b3Plane>());
    assert!(::core::mem::size_of::<Filter>() == ::core::mem::size_of::<ffi::b3Filter>());
    assert!(::core::mem::align_of::<Filter>() == ::core::mem::align_of::<ffi::b3Filter>());
    assert!(::core::mem::size_of::<MassData>() == ::core::mem::size_of::<ffi::b3MassData>());
    assert!(::core::mem::align_of::<MassData>() == ::core::mem::align_of::<ffi::b3MassData>());
    assert!(::core::mem::size_of::<MotionLocks>() == ::core::mem::size_of::<ffi::b3MotionLocks>());
    assert!(
        ::core::mem::align_of::<MotionLocks>() == ::core::mem::align_of::<ffi::b3MotionLocks>()
    );
    assert!(::core::mem::size_of::<Capacity>() == ::core::mem::size_of::<ffi::b3Capacity>());
    assert!(::core::mem::align_of::<Capacity>() == ::core::mem::align_of::<ffi::b3Capacity>());
    assert!(::core::mem::size_of::<BodyId>() == ::core::mem::size_of::<ffi::b3BodyId>());
    assert!(::core::mem::align_of::<BodyId>() == ::core::mem::align_of::<ffi::b3BodyId>());
    assert!(::core::mem::size_of::<ShapeId>() == ::core::mem::size_of::<ffi::b3ShapeId>());
    assert!(::core::mem::align_of::<ShapeId>() == ::core::mem::align_of::<ffi::b3ShapeId>());
    assert!(::core::mem::size_of::<JointId>() == ::core::mem::size_of::<ffi::b3JointId>());
    assert!(::core::mem::align_of::<JointId>() == ::core::mem::align_of::<ffi::b3JointId>());
    assert!(::core::mem::size_of::<ContactId>() == ::core::mem::size_of::<ffi::b3ContactId>());
    assert!(::core::mem::align_of::<ContactId>() == ::core::mem::align_of::<ffi::b3ContactId>());
};
