use crate::error::{Error, Result};
#[cfg(any(feature = "mint", feature = "cgmath", feature = "nalgebra"))]
use crate::types::PosScalar;
use crate::types::{Aabb, Matrix3, Plane, Pos, Quat, Transform, Vec2, Vec3, WorldTransform};

#[inline]
fn validate_world_transform(transform: WorldTransform) -> Result<WorldTransform> {
    if transform.is_valid() {
        Ok(transform)
    } else {
        Err(Error::InvalidArgument)
    }
}

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
        validate_world_transform(Self::new(value.0.into(), Quat::try_from(value.1)?))
    }
}

#[cfg(feature = "mint")]
impl From<Matrix3> for mint::ColumnMatrix3<f32> {
    #[inline]
    fn from(value: Matrix3) -> Self {
        Self {
            x: value.cx.into(),
            y: value.cy.into(),
            z: value.cz.into(),
        }
    }
}

#[cfg(feature = "mint")]
impl TryFrom<mint::ColumnMatrix3<f32>> for Matrix3 {
    type Error = Error;

    #[inline]
    fn try_from(value: mint::ColumnMatrix3<f32>) -> Result<Self> {
        Self {
            cx: value.x.into(),
            cy: value.y.into(),
            cz: value.z.into(),
        }
        .validate()
    }
}

#[cfg(feature = "mint")]
impl From<Aabb> for (mint::Point3<f32>, mint::Point3<f32>) {
    #[inline]
    fn from(value: Aabb) -> Self {
        (
            mint::Point3 {
                x: value.lower_bound.x,
                y: value.lower_bound.y,
                z: value.lower_bound.z,
            },
            mint::Point3 {
                x: value.upper_bound.x,
                y: value.upper_bound.y,
                z: value.upper_bound.z,
            },
        )
    }
}

#[cfg(feature = "mint")]
impl TryFrom<(mint::Point3<f32>, mint::Point3<f32>)> for Aabb {
    type Error = Error;

    #[inline]
    fn try_from(value: (mint::Point3<f32>, mint::Point3<f32>)) -> Result<Self> {
        Self {
            lower_bound: Vec3::new(value.0.x, value.0.y, value.0.z),
            upper_bound: Vec3::new(value.1.x, value.1.y, value.1.z),
        }
        .validate()
    }
}

#[cfg(feature = "mint")]
impl From<Plane> for (mint::Vector3<f32>, f32) {
    #[inline]
    fn from(value: Plane) -> Self {
        (value.normal.into(), value.offset)
    }
}

#[cfg(feature = "mint")]
impl TryFrom<(mint::Vector3<f32>, f32)> for Plane {
    type Error = Error;

    #[inline]
    fn try_from(value: (mint::Vector3<f32>, f32)) -> Result<Self> {
        Self {
            normal: value.0.into(),
            offset: value.1,
        }
        .validate()
    }
}

#[cfg(feature = "glam")]
impl From<glam::Vec2> for Vec2 {
    #[inline]
    fn from(value: glam::Vec2) -> Self {
        Self::new(value.x, value.y)
    }
}

#[cfg(feature = "glam")]
impl From<Vec2> for glam::Vec2 {
    #[inline]
    fn from(value: Vec2) -> Self {
        Self::new(value.x, value.y)
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

#[cfg(all(feature = "glam", not(feature = "double-precision")))]
impl From<WorldTransform> for (glam::Vec3, glam::Quat) {
    #[inline]
    fn from(value: WorldTransform) -> Self {
        (value.p.into(), value.q.into())
    }
}

#[cfg(all(feature = "glam", not(feature = "double-precision")))]
impl TryFrom<(glam::Vec3, glam::Quat)> for WorldTransform {
    type Error = Error;

    #[inline]
    fn try_from(value: (glam::Vec3, glam::Quat)) -> Result<Self> {
        validate_world_transform(Self::new(value.0.into(), Quat::try_from(value.1)?))
    }
}

#[cfg(all(feature = "glam", feature = "double-precision"))]
impl From<WorldTransform> for (glam::DVec3, glam::Quat) {
    #[inline]
    fn from(value: WorldTransform) -> Self {
        (value.p.into(), value.q.into())
    }
}

#[cfg(all(feature = "glam", feature = "double-precision"))]
impl TryFrom<(glam::DVec3, glam::Quat)> for WorldTransform {
    type Error = Error;

    #[inline]
    fn try_from(value: (glam::DVec3, glam::Quat)) -> Result<Self> {
        validate_world_transform(Self::new(value.0.into(), Quat::try_from(value.1)?))
    }
}

#[cfg(feature = "glam")]
impl From<Matrix3> for glam::Mat3 {
    #[inline]
    fn from(value: Matrix3) -> Self {
        Self::from_cols(value.cx.into(), value.cy.into(), value.cz.into())
    }
}

#[cfg(feature = "glam")]
impl TryFrom<glam::Mat3> for Matrix3 {
    type Error = Error;

    #[inline]
    fn try_from(value: glam::Mat3) -> Result<Self> {
        Self {
            cx: value.x_axis.into(),
            cy: value.y_axis.into(),
            cz: value.z_axis.into(),
        }
        .validate()
    }
}

#[cfg(feature = "glam")]
impl From<Aabb> for (glam::Vec3, glam::Vec3) {
    #[inline]
    fn from(value: Aabb) -> Self {
        (value.lower_bound.into(), value.upper_bound.into())
    }
}

#[cfg(feature = "glam")]
impl TryFrom<(glam::Vec3, glam::Vec3)> for Aabb {
    type Error = Error;

    #[inline]
    fn try_from(value: (glam::Vec3, glam::Vec3)) -> Result<Self> {
        Self {
            lower_bound: value.0.into(),
            upper_bound: value.1.into(),
        }
        .validate()
    }
}

#[cfg(feature = "glam")]
impl From<Plane> for (glam::Vec3, f32) {
    #[inline]
    fn from(value: Plane) -> Self {
        (value.normal.into(), value.offset)
    }
}

#[cfg(feature = "glam")]
impl TryFrom<(glam::Vec3, f32)> for Plane {
    type Error = Error;

    #[inline]
    fn try_from(value: (glam::Vec3, f32)) -> Result<Self> {
        Self {
            normal: value.0.into(),
            offset: value.1,
        }
        .validate()
    }
}

#[cfg(feature = "cgmath")]
impl From<cgmath::Vector2<f32>> for Vec2 {
    #[inline]
    fn from(value: cgmath::Vector2<f32>) -> Self {
        Self::new(value.x, value.y)
    }
}

#[cfg(feature = "cgmath")]
impl From<Vec2> for cgmath::Vector2<f32> {
    #[inline]
    fn from(value: Vec2) -> Self {
        Self::new(value.x, value.y)
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

#[cfg(feature = "cgmath")]
impl From<WorldTransform> for (cgmath::Point3<PosScalar>, cgmath::Quaternion<f32>) {
    #[inline]
    fn from(value: WorldTransform) -> Self {
        (value.p.into(), value.q.into())
    }
}

#[cfg(feature = "cgmath")]
impl TryFrom<(cgmath::Point3<PosScalar>, cgmath::Quaternion<f32>)> for WorldTransform {
    type Error = Error;

    #[inline]
    fn try_from(value: (cgmath::Point3<PosScalar>, cgmath::Quaternion<f32>)) -> Result<Self> {
        validate_world_transform(Self::new(value.0.into(), Quat::try_from(value.1)?))
    }
}

#[cfg(feature = "cgmath")]
impl From<Matrix3> for cgmath::Matrix3<f32> {
    #[inline]
    fn from(value: Matrix3) -> Self {
        Self::from_cols(value.cx.into(), value.cy.into(), value.cz.into())
    }
}

#[cfg(feature = "cgmath")]
impl TryFrom<cgmath::Matrix3<f32>> for Matrix3 {
    type Error = Error;

    #[inline]
    fn try_from(value: cgmath::Matrix3<f32>) -> Result<Self> {
        Self {
            cx: value.x.into(),
            cy: value.y.into(),
            cz: value.z.into(),
        }
        .validate()
    }
}

#[cfg(feature = "cgmath")]
impl From<Aabb> for (cgmath::Point3<f32>, cgmath::Point3<f32>) {
    #[inline]
    fn from(value: Aabb) -> Self {
        (
            cgmath::Point3::new(
                value.lower_bound.x,
                value.lower_bound.y,
                value.lower_bound.z,
            ),
            cgmath::Point3::new(
                value.upper_bound.x,
                value.upper_bound.y,
                value.upper_bound.z,
            ),
        )
    }
}

#[cfg(feature = "cgmath")]
impl TryFrom<(cgmath::Point3<f32>, cgmath::Point3<f32>)> for Aabb {
    type Error = Error;

    #[inline]
    fn try_from(value: (cgmath::Point3<f32>, cgmath::Point3<f32>)) -> Result<Self> {
        Self {
            lower_bound: Vec3::new(value.0.x, value.0.y, value.0.z),
            upper_bound: Vec3::new(value.1.x, value.1.y, value.1.z),
        }
        .validate()
    }
}

#[cfg(feature = "cgmath")]
impl From<Plane> for (cgmath::Vector3<f32>, f32) {
    #[inline]
    fn from(value: Plane) -> Self {
        (value.normal.into(), value.offset)
    }
}

#[cfg(feature = "cgmath")]
impl TryFrom<(cgmath::Vector3<f32>, f32)> for Plane {
    type Error = Error;

    #[inline]
    fn try_from(value: (cgmath::Vector3<f32>, f32)) -> Result<Self> {
        Self {
            normal: value.0.into(),
            offset: value.1,
        }
        .validate()
    }
}

#[cfg(feature = "nalgebra")]
impl From<nalgebra::Vector2<f32>> for Vec2 {
    #[inline]
    fn from(value: nalgebra::Vector2<f32>) -> Self {
        Self::new(value.x, value.y)
    }
}

#[cfg(feature = "nalgebra")]
impl From<Vec2> for nalgebra::Vector2<f32> {
    #[inline]
    fn from(value: Vec2) -> Self {
        Self::new(value.x, value.y)
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
impl TryFrom<nalgebra::Isometry3<f32>> for Transform {
    type Error = Error;

    #[inline]
    fn try_from(value: nalgebra::Isometry3<f32>) -> Result<Self> {
        Self::new(value.translation.vector.into(), value.rotation.into()).validate()
    }
}

#[cfg(all(feature = "nalgebra", not(feature = "double-precision")))]
impl From<WorldTransform> for nalgebra::Isometry3<f32> {
    #[inline]
    fn from(value: WorldTransform) -> Self {
        Self::from_parts(
            nalgebra::Translation3::new(value.p.x, value.p.y, value.p.z),
            value.q.into(),
        )
    }
}

#[cfg(all(feature = "nalgebra", not(feature = "double-precision")))]
impl TryFrom<nalgebra::Isometry3<f32>> for WorldTransform {
    type Error = Error;

    #[inline]
    fn try_from(value: nalgebra::Isometry3<f32>) -> Result<Self> {
        validate_world_transform(Self::new(
            Pos::from(Vec3::from(value.translation.vector)),
            value.rotation.into(),
        ))
    }
}

#[cfg(feature = "nalgebra")]
impl From<Matrix3> for nalgebra::Matrix3<f32> {
    #[inline]
    fn from(value: Matrix3) -> Self {
        Self::from_columns(&[value.cx.into(), value.cy.into(), value.cz.into()])
    }
}

#[cfg(feature = "nalgebra")]
impl TryFrom<nalgebra::Matrix3<f32>> for Matrix3 {
    type Error = Error;

    #[inline]
    fn try_from(value: nalgebra::Matrix3<f32>) -> Result<Self> {
        Self {
            cx: Vec3::new(value[(0, 0)], value[(1, 0)], value[(2, 0)]),
            cy: Vec3::new(value[(0, 1)], value[(1, 1)], value[(2, 1)]),
            cz: Vec3::new(value[(0, 2)], value[(1, 2)], value[(2, 2)]),
        }
        .validate()
    }
}

#[cfg(feature = "nalgebra")]
impl From<Aabb> for (nalgebra::Point3<f32>, nalgebra::Point3<f32>) {
    #[inline]
    fn from(value: Aabb) -> Self {
        (
            nalgebra::Point3::new(
                value.lower_bound.x,
                value.lower_bound.y,
                value.lower_bound.z,
            ),
            nalgebra::Point3::new(
                value.upper_bound.x,
                value.upper_bound.y,
                value.upper_bound.z,
            ),
        )
    }
}

#[cfg(feature = "nalgebra")]
impl TryFrom<(nalgebra::Point3<f32>, nalgebra::Point3<f32>)> for Aabb {
    type Error = Error;

    #[inline]
    fn try_from(value: (nalgebra::Point3<f32>, nalgebra::Point3<f32>)) -> Result<Self> {
        Self {
            lower_bound: Vec3::new(value.0.x, value.0.y, value.0.z),
            upper_bound: Vec3::new(value.1.x, value.1.y, value.1.z),
        }
        .validate()
    }
}

#[cfg(feature = "nalgebra")]
impl From<Plane> for (nalgebra::Vector3<f32>, f32) {
    #[inline]
    fn from(value: Plane) -> Self {
        (value.normal.into(), value.offset)
    }
}

#[cfg(feature = "nalgebra")]
impl TryFrom<(nalgebra::Vector3<f32>, f32)> for Plane {
    type Error = Error;

    #[inline]
    fn try_from(value: (nalgebra::Vector3<f32>, f32)) -> Result<Self> {
        Self {
            normal: value.0.into(),
            offset: value.1,
        }
        .validate()
    }
}
