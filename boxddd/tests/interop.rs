#[cfg(feature = "mint")]
#[test]
fn mint_conversions_round_trip_and_validate_quaternions() {
    use boxddd::{Error, Quat, Transform, Vec3, WorldTransform};

    let v = Vec3::new(1.0, 2.0, 3.0);
    let mv: mint::Vector3<f32> = v.into();
    assert_eq!(Vec3::from(mv), v);

    let q = Quat::IDENTITY;
    let mq: mint::Quaternion<f32> = q.into();
    assert_eq!(Quat::try_from(mq).unwrap(), q);

    let t = Transform::new(v, q);
    let mt: (mint::Vector3<f32>, mint::Quaternion<f32>) = t.into();
    assert_eq!(Transform::try_from(mt).unwrap(), t);

    let wt = WorldTransform::new([4.0, 5.0, 6.0].into(), q);
    let mwt: (
        mint::Point3<boxddd::types::PosScalar>,
        mint::Quaternion<f32>,
    ) = wt.into();
    assert_eq!(WorldTransform::try_from(mwt).unwrap(), wt);

    let invalid = mint::Quaternion {
        v: mint::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        s: 2.0,
    };
    assert_eq!(Quat::try_from(invalid).unwrap_err(), Error::InvalidArgument);
}

#[cfg(feature = "glam")]
#[test]
fn glam_conversions_round_trip_and_validate_quaternions() {
    use boxddd::{Error, Quat, Transform, Vec3};

    let v = Vec3::new(1.0, 2.0, 3.0);
    let gv: glam::Vec3 = v.into();
    assert_eq!(Vec3::from(gv), v);

    let q = Quat::IDENTITY;
    let gq: glam::Quat = q.into();
    assert_eq!(Quat::try_from(gq).unwrap(), q);

    let t = Transform::new(v, q);
    let gt: (glam::Vec3, glam::Quat) = t.into();
    assert_eq!(Transform::try_from(gt).unwrap(), t);

    assert_eq!(
        Quat::try_from(glam::Quat::from_xyzw(0.0, 0.0, 0.0, 2.0)).unwrap_err(),
        Error::InvalidArgument
    );
}

#[cfg(feature = "cgmath")]
#[test]
fn cgmath_conversions_round_trip_and_validate_quaternions() {
    use boxddd::{Error, Quat, Transform, Vec3};

    let v = Vec3::new(1.0, 2.0, 3.0);
    let cv: cgmath::Vector3<f32> = v.into();
    assert_eq!(Vec3::from(cv), v);

    let q = Quat::IDENTITY;
    let cq: cgmath::Quaternion<f32> = q.into();
    assert_eq!(Quat::try_from(cq).unwrap(), q);

    let t = Transform::new(v, q);
    let ct: (cgmath::Vector3<f32>, cgmath::Quaternion<f32>) = t.into();
    assert_eq!(Transform::try_from(ct).unwrap(), t);

    let invalid = cgmath::Quaternion {
        s: 2.0,
        v: cgmath::Vector3::new(0.0, 0.0, 0.0),
    };
    assert_eq!(Quat::try_from(invalid).unwrap_err(), Error::InvalidArgument);
}

#[cfg(feature = "nalgebra")]
#[test]
fn nalgebra_conversions_round_trip_and_validate_quaternions() {
    use boxddd::{Error, Quat, Transform, Vec3};

    let v = Vec3::new(1.0, 2.0, 3.0);
    let nv: nalgebra::Vector3<f32> = v.into();
    assert_eq!(Vec3::from(nv), v);

    let q = Quat::IDENTITY;
    let nq: nalgebra::Quaternion<f32> = q.into();
    assert_eq!(Quat::try_from(nq).unwrap(), q);

    let t = Transform::new(v, q);
    let iso: nalgebra::Isometry3<f32> = t.into();
    assert_eq!(Transform::from(iso), t);

    assert_eq!(
        Quat::try_from(nalgebra::Quaternion::new(2.0, 0.0, 0.0, 0.0)).unwrap_err(),
        Error::InvalidArgument
    );
}
