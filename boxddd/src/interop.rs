use crate::error::{Error, Result};
#[cfg(any(
    feature = "mint",
    all(feature = "glam", not(feature = "double-precision")),
    all(feature = "glam", feature = "double-precision"),
    feature = "cgmath",
    feature = "nalgebra"
))]
use crate::types::Pos;
#[cfg(any(feature = "mint", feature = "cgmath", feature = "nalgebra"))]
use crate::types::PosScalar;
#[cfg(any(
    feature = "mint",
    feature = "glam",
    feature = "cgmath",
    feature = "nalgebra"
))]
use crate::types::{Quat, Transform, Vec3};
#[cfg(feature = "mint")]
use crate::types::{Vec2, WorldTransform};

#[cfg(feature = "mint")]
impl From<mint::Vector2<f32>> for Vec2 {
    #[inline]
    fn from(value: mint::Vector2<f32>) -> Self {
        Self::new(value.x, value.y)
    }
}

#[cfg(feature = "mint")]
impl From<Vec2> for mint::Vector2<f32> {
    #[inline]
    fn from(value: Vec2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[cfg(feature = "mint")]
impl From<mint::Vector3<f32>> for Vec3 {
    #[inline]
    fn from(value: mint::Vector3<f32>) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(feature = "mint")]
impl From<Vec3> for mint::Vector3<f32> {
    #[inline]
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[cfg(feature = "mint")]
impl From<mint::Point3<PosScalar>> for Pos {
    #[inline]
    fn from(value: mint::Point3<PosScalar>) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(feature = "mint")]
impl From<Pos> for mint::Point3<PosScalar> {
    #[inline]
    fn from(value: Pos) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[cfg(feature = "mint")]
impl From<Quat> for mint::Quaternion<f32> {
    #[inline]
    fn from(value: Quat) -> Self {
        Self {
            v: value.v.into(),
            s: value.s,
        }
    }
}

#[cfg(feature = "mint")]
impl TryFrom<mint::Quaternion<f32>> for Quat {
    type Error = Error;

    #[inline]
    fn try_from(value: mint::Quaternion<f32>) -> Result<Self> {
        Self::new(value.v.into(), value.s).validate()
    }
}

#[cfg(feature = "mint")]
impl From<Transform> for (mint::Vector3<f32>, mint::Quaternion<f32>) {
    #[inline]
    fn from(value: Transform) -> Self {
        (value.p.into(), value.q.into())
    }
}

#[cfg(feature = "mint")]
impl TryFrom<(mint::Vector3<f32>, mint::Quaternion<f32>)> for Transform {
    type Error = Error;

    #[inline]
    fn try_from(value: (mint::Vector3<f32>, mint::Quaternion<f32>)) -> Result<Self> {
        Self::new(value.0.into(), Quat::try_from(value.1)?).validate()
    }
}

#[cfg(feature = "mint")]
impl From<WorldTransform> for (mint::Point3<PosScalar>, mint::Quaternion<f32>) {
    #[inline]
    fn from(value: WorldTransform) -> Self {
        (value.p.into(), value.q.into())
    }
}

#[cfg(feature = "mint")]
impl TryFrom<(mint::Point3<PosScalar>, mint::Quaternion<f32>)> for WorldTransform {
    type Error = Error;

    #[inline]
    fn try_from(value: (mint::Point3<PosScalar>, mint::Quaternion<f32>)) -> Result<Self> {
        let transform = Self::new(value.0.into(), Quat::try_from(value.1)?);
        if transform.is_valid() {
            Ok(transform)
        } else {
            Err(Error::InvalidArgument)
        }
    }
}

#[cfg(feature = "glam")]
impl From<glam::Vec3> for Vec3 {
    #[inline]
    fn from(value: glam::Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(feature = "glam")]
impl From<Vec3> for glam::Vec3 {
    #[inline]
    fn from(value: Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(all(feature = "glam", not(feature = "double-precision")))]
impl From<glam::Vec3> for Pos {
    #[inline]
    fn from(value: glam::Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(all(feature = "glam", not(feature = "double-precision")))]
impl From<Pos> for glam::Vec3 {
    #[inline]
    fn from(value: Pos) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(all(feature = "glam", feature = "double-precision"))]
impl From<glam::DVec3> for Pos {
    #[inline]
    fn from(value: glam::DVec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(all(feature = "glam", feature = "double-precision"))]
impl From<Pos> for glam::DVec3 {
    #[inline]
    fn from(value: Pos) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(feature = "glam")]
impl From<Quat> for glam::Quat {
    #[inline]
    fn from(value: Quat) -> Self {
        Self::from_xyzw(value.v.x, value.v.y, value.v.z, value.s)
    }
}

#[cfg(feature = "glam")]
impl TryFrom<glam::Quat> for Quat {
    type Error = Error;

    #[inline]
    fn try_from(value: glam::Quat) -> Result<Self> {
        Self::new(Vec3::new(value.x, value.y, value.z), value.w).validate()
    }
}

#[cfg(feature = "glam")]
impl From<Transform> for (glam::Vec3, glam::Quat) {
    #[inline]
    fn from(value: Transform) -> Self {
        (value.p.into(), value.q.into())
    }
}

#[cfg(feature = "glam")]
impl TryFrom<(glam::Vec3, glam::Quat)> for Transform {
    type Error = Error;

    #[inline]
    fn try_from(value: (glam::Vec3, glam::Quat)) -> Result<Self> {
        Self::new(value.0.into(), Quat::try_from(value.1)?).validate()
    }
}

#[cfg(feature = "cgmath")]
impl From<cgmath::Vector3<f32>> for Vec3 {
    #[inline]
    fn from(value: cgmath::Vector3<f32>) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(feature = "cgmath")]
impl From<Vec3> for cgmath::Vector3<f32> {
    #[inline]
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[cfg(feature = "cgmath")]
impl From<cgmath::Point3<PosScalar>> for Pos {
    #[inline]
    fn from(value: cgmath::Point3<PosScalar>) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(feature = "cgmath")]
impl From<Pos> for cgmath::Point3<PosScalar> {
    #[inline]
    fn from(value: Pos) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[cfg(feature = "cgmath")]
impl From<Quat> for cgmath::Quaternion<f32> {
    #[inline]
    fn from(value: Quat) -> Self {
        Self {
            s: value.s,
            v: value.v.into(),
        }
    }
}

#[cfg(feature = "cgmath")]
impl TryFrom<cgmath::Quaternion<f32>> for Quat {
    type Error = Error;

    #[inline]
    fn try_from(value: cgmath::Quaternion<f32>) -> Result<Self> {
        Self::new(value.v.into(), value.s).validate()
    }
}

#[cfg(feature = "cgmath")]
impl From<Transform> for (cgmath::Vector3<f32>, cgmath::Quaternion<f32>) {
    #[inline]
    fn from(value: Transform) -> Self {
        (value.p.into(), value.q.into())
    }
}

#[cfg(feature = "cgmath")]
impl TryFrom<(cgmath::Vector3<f32>, cgmath::Quaternion<f32>)> for Transform {
    type Error = Error;

    #[inline]
    fn try_from(value: (cgmath::Vector3<f32>, cgmath::Quaternion<f32>)) -> Result<Self> {
        Self::new(value.0.into(), Quat::try_from(value.1)?).validate()
    }
}

#[cfg(feature = "nalgebra")]
impl From<nalgebra::Vector3<f32>> for Vec3 {
    #[inline]
    fn from(value: nalgebra::Vector3<f32>) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(feature = "nalgebra")]
impl From<Vec3> for nalgebra::Vector3<f32> {
    #[inline]
    fn from(value: Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(feature = "nalgebra")]
impl From<nalgebra::Point3<PosScalar>> for Pos {
    #[inline]
    fn from(value: nalgebra::Point3<PosScalar>) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(feature = "nalgebra")]
impl From<Pos> for nalgebra::Point3<PosScalar> {
    #[inline]
    fn from(value: Pos) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

#[cfg(feature = "nalgebra")]
impl From<Quat> for nalgebra::Quaternion<f32> {
    #[inline]
    fn from(value: Quat) -> Self {
        Self::new(value.s, value.v.x, value.v.y, value.v.z)
    }
}

#[cfg(feature = "nalgebra")]
impl TryFrom<nalgebra::Quaternion<f32>> for Quat {
    type Error = Error;

    #[inline]
    fn try_from(value: nalgebra::Quaternion<f32>) -> Result<Self> {
        Self::new(Vec3::new(value.i, value.j, value.k), value.w).validate()
    }
}

#[cfg(feature = "nalgebra")]
impl From<Quat> for nalgebra::UnitQuaternion<f32> {
    #[inline]
    fn from(value: Quat) -> Self {
        Self::from_quaternion(value.into())
    }
}

#[cfg(feature = "nalgebra")]
impl From<nalgebra::UnitQuaternion<f32>> for Quat {
    #[inline]
    fn from(value: nalgebra::UnitQuaternion<f32>) -> Self {
        let q = value.quaternion();
        Self::new(Vec3::new(q.i, q.j, q.k), q.w)
    }
}

#[cfg(feature = "nalgebra")]
impl From<Transform> for nalgebra::Isometry3<f32> {
    #[inline]
    fn from(value: Transform) -> Self {
        Self::from_parts(
            nalgebra::Translation3::new(value.p.x, value.p.y, value.p.z),
            value.q.into(),
        )
    }
}

#[cfg(feature = "nalgebra")]
impl From<nalgebra::Isometry3<f32>> for Transform {
    #[inline]
    fn from(value: nalgebra::Isometry3<f32>) -> Self {
        Self::new(value.translation.vector.into(), value.rotation.into())
    }
}
