#![cfg(any(
    feature = "mint",
    feature = "glam",
    feature = "cgmath",
    feature = "nalgebra"
))]

use boxddd::{Aabb, Error, Matrix3, Plane, Pos, Quat, Transform, Vec2, Vec3, WorldTransform};

fn sample_matrix() -> Matrix3 {
    Matrix3 {
        cx: Vec3::X,
        cy: Vec3::Y,
        cz: Vec3::Z,
    }
}

fn sample_aabb() -> Aabb {
    Aabb {
        lower_bound: [-1.0, -2.0, -3.0].into(),
        upper_bound: [1.0, 2.0, 3.0].into(),
    }
}

fn sample_plane() -> Plane {
    Plane {
        normal: Vec3::Y,
        offset: 1.25,
    }
}

#[cfg(feature = "mint")]
#[test]
fn mint_conversions_round_trip_and_validate_inputs() {
    let v2 = Vec2::new(1.0, 2.0);
    let mv2: mint::Vector2<f32> = v2.into();
    assert_eq!(Vec2::from(mv2), v2);

    let v3 = Vec3::new(1.0, 2.0, 3.0);
    let mv3: mint::Vector3<f32> = v3.into();
    assert_eq!(Vec3::from(mv3), v3);

    let pos = Pos::new(4.0, 5.0, 6.0);
    let point: mint::Point3<boxddd::types::PosScalar> = pos.into();
    assert_eq!(Pos::from(point), pos);

    let q = Quat::IDENTITY;
    let mq: mint::Quaternion<f32> = q.into();
    assert_eq!(Quat::try_from(mq).unwrap(), q);

    let t = Transform::new(v3, q);
    let mt: (mint::Vector3<f32>, mint::Quaternion<f32>) = t.into();
    assert_eq!(Transform::try_from(mt).unwrap(), t);

    let wt = WorldTransform::new(pos, q);
    let mwt: (
        mint::Point3<boxddd::types::PosScalar>,
        mint::Quaternion<f32>,
    ) = wt.into();
    assert_eq!(WorldTransform::try_from(mwt).unwrap(), wt);

    let matrix = sample_matrix();
    let mint_matrix: mint::ColumnMatrix3<f32> = matrix.into();
    assert_eq!(Matrix3::try_from(mint_matrix).unwrap(), matrix);

    let aabb = sample_aabb();
    let mint_aabb: (mint::Point3<f32>, mint::Point3<f32>) = aabb.into();
    assert_eq!(Aabb::try_from(mint_aabb).unwrap(), aabb);

    let plane = sample_plane();
    let mint_plane: (mint::Vector3<f32>, f32) = plane.into();
    assert_eq!(Plane::try_from(mint_plane).unwrap(), plane);

    let invalid_quat = mint::Quaternion {
        v: mint::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        s: 2.0,
    };
    assert_eq!(
        Quat::try_from(invalid_quat).unwrap_err(),
        Error::InvalidArgument
    );

    let inverted_aabb = (
        mint::Point3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        mint::Point3 {
            x: -1.0,
            y: 0.0,
            z: 0.0,
        },
    );
    assert_eq!(
        Aabb::try_from(inverted_aabb).unwrap_err(),
        Error::InvalidArgument
    );

    let invalid_plane = (
        mint::Vector3 {
            x: 2.0,
            y: 0.0,
            z: 0.0,
        },
        0.0,
    );
    assert_eq!(
        Plane::try_from(invalid_plane).unwrap_err(),
        Error::InvalidArgument
    );
}

#[cfg(feature = "glam")]
#[test]
fn glam_conversions_round_trip_and_validate_inputs() {
    let v2 = Vec2::new(1.0, 2.0);
    let gv2: glam::Vec2 = v2.into();
    assert_eq!(Vec2::from(gv2), v2);

    let v3 = Vec3::new(1.0, 2.0, 3.0);
    let gv3: glam::Vec3 = v3.into();
    assert_eq!(Vec3::from(gv3), v3);

    let q = Quat::IDENTITY;
    let gq: glam::Quat = q.into();
    assert_eq!(Quat::try_from(gq).unwrap(), q);

    let t = Transform::new(v3, q);
    let gt: (glam::Vec3, glam::Quat) = t.into();
    assert_eq!(Transform::try_from(gt).unwrap(), t);

    let wt = WorldTransform::new(Pos::new(4.0, 5.0, 6.0), q);
    #[cfg(not(feature = "double-precision"))]
    {
        let gwt: (glam::Vec3, glam::Quat) = wt.into();
        assert_eq!(WorldTransform::try_from(gwt).unwrap(), wt);
    }
    #[cfg(feature = "double-precision")]
    {
        let gwt: (glam::DVec3, glam::Quat) = wt.into();
        assert_eq!(WorldTransform::try_from(gwt).unwrap(), wt);
    }

    let matrix = sample_matrix();
    let glam_matrix: glam::Mat3 = matrix.into();
    assert_eq!(Matrix3::try_from(glam_matrix).unwrap(), matrix);

    let aabb = sample_aabb();
    let glam_aabb: (glam::Vec3, glam::Vec3) = aabb.into();
    assert_eq!(Aabb::try_from(glam_aabb).unwrap(), aabb);

    let plane = sample_plane();
    let glam_plane: (glam::Vec3, f32) = plane.into();
    assert_eq!(Plane::try_from(glam_plane).unwrap(), plane);

    assert_eq!(
        Quat::try_from(glam::Quat::from_xyzw(0.0, 0.0, 0.0, 2.0)).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        Transform::try_from((glam::Vec3::new(f32::NAN, 0.0, 0.0), glam::Quat::IDENTITY))
            .unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        Aabb::try_from((glam::Vec3::ONE, -glam::Vec3::ONE)).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        Plane::try_from((glam::Vec3::new(2.0, 0.0, 0.0), 0.0)).unwrap_err(),
        Error::InvalidArgument
    );
}

#[cfg(feature = "cgmath")]
#[test]
fn cgmath_conversions_round_trip_and_validate_inputs() {
    let v2 = Vec2::new(1.0, 2.0);
    let cv2: cgmath::Vector2<f32> = v2.into();
    assert_eq!(Vec2::from(cv2), v2);

    let v3 = Vec3::new(1.0, 2.0, 3.0);
    let cv3: cgmath::Vector3<f32> = v3.into();
    assert_eq!(Vec3::from(cv3), v3);

    let pos = Pos::new(4.0, 5.0, 6.0);
    let point: cgmath::Point3<boxddd::types::PosScalar> = pos.into();
    assert_eq!(Pos::from(point), pos);

    let q = Quat::IDENTITY;
    let cq: cgmath::Quaternion<f32> = q.into();
    assert_eq!(Quat::try_from(cq).unwrap(), q);

    let t = Transform::new(v3, q);
    let ct: (cgmath::Vector3<f32>, cgmath::Quaternion<f32>) = t.into();
    assert_eq!(Transform::try_from(ct).unwrap(), t);

    let wt = WorldTransform::new(pos, q);
    let cwt: (
        cgmath::Point3<boxddd::types::PosScalar>,
        cgmath::Quaternion<f32>,
    ) = wt.into();
    assert_eq!(WorldTransform::try_from(cwt).unwrap(), wt);

    let matrix = sample_matrix();
    let cgmath_matrix: cgmath::Matrix3<f32> = matrix.into();
    assert_eq!(Matrix3::try_from(cgmath_matrix).unwrap(), matrix);

    let aabb = sample_aabb();
    let cgmath_aabb: (cgmath::Point3<f32>, cgmath::Point3<f32>) = aabb.into();
    assert_eq!(Aabb::try_from(cgmath_aabb).unwrap(), aabb);

    let plane = sample_plane();
    let cgmath_plane: (cgmath::Vector3<f32>, f32) = plane.into();
    assert_eq!(Plane::try_from(cgmath_plane).unwrap(), plane);

    let invalid_quat = cgmath::Quaternion {
        s: 2.0,
        v: cgmath::Vector3::new(0.0, 0.0, 0.0),
    };
    assert_eq!(
        Quat::try_from(invalid_quat).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        Aabb::try_from((
            cgmath::Point3::new(1.0, 0.0, 0.0),
            cgmath::Point3::new(-1.0, 0.0, 0.0)
        ))
        .unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        Plane::try_from((cgmath::Vector3::new(2.0, 0.0, 0.0), 0.0)).unwrap_err(),
        Error::InvalidArgument
    );
}

#[cfg(feature = "nalgebra")]
#[test]
fn nalgebra_conversions_round_trip_and_validate_inputs() {
    let v2 = Vec2::new(1.0, 2.0);
    let nv2: nalgebra::Vector2<f32> = v2.into();
    assert_eq!(Vec2::from(nv2), v2);

    let v3 = Vec3::new(1.0, 2.0, 3.0);
    let nv3: nalgebra::Vector3<f32> = v3.into();
    assert_eq!(Vec3::from(nv3), v3);

    let pos = Pos::new(4.0, 5.0, 6.0);
    let point: nalgebra::Point3<boxddd::types::PosScalar> = pos.into();
    assert_eq!(Pos::from(point), pos);

    let q = Quat::IDENTITY;
    let nq: nalgebra::Quaternion<f32> = q.into();
    assert_eq!(Quat::try_from(nq).unwrap(), q);

    let unit: nalgebra::UnitQuaternion<f32> = q.into();
    assert_eq!(Quat::from(unit), q);

    let t = Transform::new(v3, q);
    let iso: nalgebra::Isometry3<f32> = t.into();
    assert_eq!(Transform::try_from(iso).unwrap(), t);

    #[cfg(not(feature = "double-precision"))]
    {
        let wt = WorldTransform::new(pos, q);
        let world_iso: nalgebra::Isometry3<f32> = wt.into();
        assert_eq!(WorldTransform::try_from(world_iso).unwrap(), wt);
    }

    let matrix = sample_matrix();
    let nalgebra_matrix: nalgebra::Matrix3<f32> = matrix.into();
    assert_eq!(Matrix3::try_from(nalgebra_matrix).unwrap(), matrix);

    let aabb = sample_aabb();
    let nalgebra_aabb: (nalgebra::Point3<f32>, nalgebra::Point3<f32>) = aabb.into();
    assert_eq!(Aabb::try_from(nalgebra_aabb).unwrap(), aabb);

    let plane = sample_plane();
    let nalgebra_plane: (nalgebra::Vector3<f32>, f32) = plane.into();
    assert_eq!(Plane::try_from(nalgebra_plane).unwrap(), plane);

    assert_eq!(
        Quat::try_from(nalgebra::Quaternion::new(2.0, 0.0, 0.0, 0.0)).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        Transform::try_from(nalgebra::Isometry3::from_parts(
            nalgebra::Translation3::new(f32::NAN, 0.0, 0.0),
            nalgebra::UnitQuaternion::identity(),
        ))
        .unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        Aabb::try_from((
            nalgebra::Point3::new(1.0, 0.0, 0.0),
            nalgebra::Point3::new(-1.0, 0.0, 0.0)
        ))
        .unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        Plane::try_from((nalgebra::Vector3::new(2.0, 0.0, 0.0), 0.0)).unwrap_err(),
        Error::InvalidArgument
    );
}
