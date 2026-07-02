use boxddd_sys::ffi;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    pub const X: Self = Self::new(1.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3Vec3) -> Self {
        Self {
            x: raw.x,
            y: raw.y,
            z: raw.z,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Vec3 {
        ffi::b3Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3IsValidVec3(self.into_raw()) }
    }
}

impl From<[f32; 3]> for Vec3 {
    #[inline]
    fn from(value: [f32; 3]) -> Self {
        Self::new(value[0], value[1], value[2])
    }
}

impl From<(f32, f32, f32)> for Vec3 {
    #[inline]
    fn from(value: (f32, f32, f32)) -> Self {
        Self::new(value.0, value.1, value.2)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Quat {
    pub v: Vec3,
    pub s: f32,
}

impl Quat {
    pub const IDENTITY: Self = Self {
        v: Vec3::ZERO,
        s: 1.0,
    };

    #[inline]
    pub const fn new(v: Vec3, s: f32) -> Self {
        Self { v, s }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3Quat) -> Self {
        Self {
            v: Vec3::from_raw(raw.v),
            s: raw.s,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Quat {
        ffi::b3Quat {
            v: self.v.into_raw(),
            s: self.s,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3IsValidQuat(self.into_raw()) }
    }
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Transform {
    pub p: Vec3,
    pub q: Quat,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        p: Vec3::ZERO,
        q: Quat::IDENTITY,
    };

    #[inline]
    pub const fn new(p: Vec3, q: Quat) -> Self {
        Self { p, q }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3Transform) -> Self {
        Self {
            p: Vec3::from_raw(raw.p),
            q: Quat::from_raw(raw.q),
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Transform {
        ffi::b3Transform {
            p: self.p.into_raw(),
            q: self.q.into_raw(),
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3IsValidTransform(self.into_raw()) }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub revision: i32,
}

impl Version {
    #[inline]
    pub const fn from_raw(raw: ffi::b3Version) -> Self {
        Self {
            major: raw.major,
            minor: raw.minor,
            revision: raw.revision,
        }
    }
}

#[repr(C)]
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

const _: () = {
    assert!(core::mem::size_of::<Vec3>() == core::mem::size_of::<ffi::b3Vec3>());
    assert!(core::mem::align_of::<Vec3>() == core::mem::align_of::<ffi::b3Vec3>());
    assert!(core::mem::size_of::<Quat>() == core::mem::size_of::<ffi::b3Quat>());
    assert!(core::mem::align_of::<Quat>() == core::mem::align_of::<ffi::b3Quat>());
    assert!(core::mem::size_of::<Transform>() == core::mem::size_of::<ffi::b3Transform>());
    assert!(core::mem::align_of::<Transform>() == core::mem::align_of::<ffi::b3Transform>());
    assert!(core::mem::size_of::<BodyId>() == core::mem::size_of::<ffi::b3BodyId>());
    assert!(core::mem::align_of::<BodyId>() == core::mem::align_of::<ffi::b3BodyId>());
    assert!(core::mem::size_of::<ShapeId>() == core::mem::size_of::<ffi::b3ShapeId>());
    assert!(core::mem::align_of::<ShapeId>() == core::mem::align_of::<ffi::b3ShapeId>());
};
