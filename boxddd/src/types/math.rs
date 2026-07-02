use super::*;

#[inline]
pub fn is_valid_float(value: f32) -> bool {
    unsafe { ffi::b3IsValidFloat(value) }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3Vec2) -> Self {
        Self { x: raw.x, y: raw.y }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Vec2 {
        ffi::b3Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl From<[f32; 2]> for Vec2 {
    #[inline]
    fn from(value: [f32; 2]) -> Self {
        Self::new(value[0], value[1])
    }
}

impl From<(f32, f32)> for Vec2 {
    #[inline]
    fn from(value: (f32, f32)) -> Self {
        Self::new(value.0, value.1)
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    #[inline]
    pub fn validate(self) -> crate::error::Result<Self> {
        if self.is_valid() {
            Ok(self)
        } else {
            Err(crate::error::Error::InvalidArgument)
        }
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    #[inline]
    pub fn validate(self) -> crate::error::Result<Self> {
        if self.is_valid() {
            Ok(self)
        } else {
            Err(crate::error::Error::InvalidArgument)
        }
    }
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    #[inline]
    pub fn validate(self) -> crate::error::Result<Self> {
        if self.is_valid() {
            Ok(self)
        } else {
            Err(crate::error::Error::InvalidArgument)
        }
    }
}

#[cfg(not(feature = "double-precision"))]
pub type PosScalar = f32;
#[cfg(feature = "double-precision")]
pub type PosScalar = f64;

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Pos {
    pub x: PosScalar,
    pub y: PosScalar,
    pub z: PosScalar,
}

impl Pos {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    #[inline]
    pub const fn new(x: PosScalar, y: PosScalar, z: PosScalar) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3Pos) -> Self {
        Self {
            x: raw.x,
            y: raw.y,
            z: raw.z,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Pos {
        ffi::b3Pos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3IsValidPosition(self.into_raw()) }
    }

    #[inline]
    pub fn validate(self) -> crate::error::Result<Self> {
        if self.is_valid() {
            Ok(self)
        } else {
            Err(crate::error::Error::InvalidArgument)
        }
    }
}

impl From<Vec3> for Pos {
    #[inline]
    fn from(value: Vec3) -> Self {
        Self::new(value.x.into(), value.y.into(), value.z.into())
    }
}

impl From<[f32; 3]> for Pos {
    #[inline]
    fn from(value: [f32; 3]) -> Self {
        Self::from(Vec3::from(value))
    }
}

impl From<(f32, f32, f32)> for Pos {
    #[inline]
    fn from(value: (f32, f32, f32)) -> Self {
        Self::from(Vec3::from(value))
    }
}

#[cfg(feature = "double-precision")]
impl From<[f64; 3]> for Pos {
    #[inline]
    fn from(value: [f64; 3]) -> Self {
        Self::new(value[0], value[1], value[2])
    }
}

#[cfg(feature = "double-precision")]
impl From<(f64, f64, f64)> for Pos {
    #[inline]
    fn from(value: (f64, f64, f64)) -> Self {
        Self::new(value.0, value.1, value.2)
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct WorldTransform {
    pub p: Pos,
    pub q: Quat,
}

impl WorldTransform {
    pub const IDENTITY: Self = Self {
        p: Pos::ZERO,
        q: Quat::IDENTITY,
    };

    #[inline]
    pub const fn new(p: Pos, q: Quat) -> Self {
        Self { p, q }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3WorldTransform) -> Self {
        Self {
            p: Pos::from_raw(raw.p),
            q: Quat::from_raw(raw.q),
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3WorldTransform {
        ffi::b3WorldTransform {
            p: self.p.into_raw(),
            q: self.q.into_raw(),
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3IsValidWorldTransform(self.into_raw()) }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Matrix3 {
    pub cx: Vec3,
    pub cy: Vec3,
    pub cz: Vec3,
}

impl Matrix3 {
    #[inline]
    pub const fn from_raw(raw: ffi::b3Matrix3) -> Self {
        Self {
            cx: Vec3::from_raw(raw.cx),
            cy: Vec3::from_raw(raw.cy),
            cz: Vec3::from_raw(raw.cz),
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Matrix3 {
        ffi::b3Matrix3 {
            cx: self.cx.into_raw(),
            cy: self.cy.into_raw(),
            cz: self.cz.into_raw(),
        }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Aabb {
    pub lower_bound: Vec3,
    pub upper_bound: Vec3,
}

impl Aabb {
    #[inline]
    pub const fn from_raw(raw: ffi::b3AABB) -> Self {
        Self {
            lower_bound: Vec3::from_raw(raw.lowerBound),
            upper_bound: Vec3::from_raw(raw.upperBound),
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3AABB {
        ffi::b3AABB {
            lowerBound: self.lower_bound.into_raw(),
            upperBound: self.upper_bound.into_raw(),
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3IsValidAABB(self.into_raw()) }
    }

    #[inline]
    pub fn validate(self) -> Result<Self> {
        if self.is_valid() {
            Ok(self)
        } else {
            Err(Error::InvalidArgument)
        }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Plane {
    pub normal: Vec3,
    pub offset: f32,
}

impl Plane {
    #[inline]
    pub const fn from_raw(raw: ffi::b3Plane) -> Self {
        Self {
            normal: Vec3::from_raw(raw.normal),
            offset: raw.offset,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Plane {
        ffi::b3Plane {
            normal: self.normal.into_raw(),
            offset: self.offset,
        }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    pub category_bits: u64,
    pub mask_bits: u64,
    pub group_index: i32,
}

impl Filter {
    #[inline]
    pub const fn from_raw(raw: ffi::b3Filter) -> Self {
        Self {
            category_bits: raw.categoryBits,
            mask_bits: raw.maskBits,
            group_index: raw.groupIndex,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Filter {
        ffi::b3Filter {
            categoryBits: self.category_bits,
            maskBits: self.mask_bits,
            groupIndex: self.group_index,
        }
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::from_raw(unsafe { ffi::b3DefaultFilter() })
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct MassData {
    pub mass: f32,
    pub center: Vec3,
    pub inertia: Matrix3,
}

impl MassData {
    #[inline]
    pub const fn from_raw(raw: ffi::b3MassData) -> Self {
        Self {
            mass: raw.mass,
            center: Vec3::from_raw(raw.center),
            inertia: Matrix3::from_raw(raw.inertia),
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3MassData {
        ffi::b3MassData {
            mass: self.mass,
            center: self.center.into_raw(),
            inertia: self.inertia.into_raw(),
        }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct MotionLocks {
    pub linear_x: bool,
    pub linear_y: bool,
    pub linear_z: bool,
    pub angular_x: bool,
    pub angular_y: bool,
    pub angular_z: bool,
}

impl MotionLocks {
    #[inline]
    pub const fn new(
        linear_x: bool,
        linear_y: bool,
        linear_z: bool,
        angular_x: bool,
        angular_y: bool,
        angular_z: bool,
    ) -> Self {
        Self {
            linear_x,
            linear_y,
            linear_z,
            angular_x,
            angular_y,
            angular_z,
        }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3MotionLocks) -> Self {
        Self {
            linear_x: raw.linearX,
            linear_y: raw.linearY,
            linear_z: raw.linearZ,
            angular_x: raw.angularX,
            angular_y: raw.angularY,
            angular_z: raw.angularZ,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3MotionLocks {
        ffi::b3MotionLocks {
            linearX: self.linear_x,
            linearY: self.linear_y,
            linearZ: self.linear_z,
            angularX: self.angular_x,
            angularY: self.angular_y,
            angularZ: self.angular_z,
        }
    }
}
