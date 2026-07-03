use boxddd::{
    Capsule, Compound, Error, HeightField, Hull, MeshData, MeshDataOptions, ShapeDef, Sphere,
    SurfaceMaterial, Transform, Vec3,
};

#[test]
fn value_and_resource_geometry_reject_invalid_inputs() {
    assert_eq!(
        Capsule::new(Vec3::ZERO, Vec3::X, f32::NAN).validate(),
        Err(Error::InvalidArgument)
    );
    assert_eq!(
        Sphere::new(Vec3::ZERO, 0.0).validate(),
        Err(Error::InvalidArgument)
    );
    assert!(Hull::from_points([Vec3::ZERO, Vec3::X, Vec3::Y], 8).is_err());
    assert!(Hull::cylinder(1.0, 1.0, 0.0, 2).is_err());
    assert!(MeshData::box_mesh(Vec3::ZERO, [1.0, 0.0, 1.0], true).is_err());
    assert!(
        MeshData::from_triangles(
            vec![Vec3::ZERO, Vec3::X, Vec3::Y],
            vec![0, 1, 3],
            None,
            MeshDataOptions::default(),
        )
        .is_err()
    );
    assert!(
        MeshData::from_triangles(
            vec![Vec3::ZERO, Vec3::X, Vec3::Y],
            vec![0, 1, 2],
            Some(&[0, 1]),
            MeshDataOptions::default(),
        )
        .is_err()
    );
    assert!(
        MeshData::from_triangles(
            vec![Vec3::ZERO, Vec3::X, Vec3::new(2.0, 0.0, 0.0)],
            vec![0, 1, 2],
            None,
            MeshDataOptions::default(),
        )
        .is_err()
    );
    assert!(
        MeshData::from_triangles(
            vec![Vec3::ZERO, Vec3::X, Vec3::Y],
            vec![0, 1, 2],
            None,
            MeshDataOptions::default().weld_tolerance(f32::NAN),
        )
        .is_err()
    );
    assert!(MeshData::wave_mesh(2, 2, 1.0, f32::NAN, 1.0, 1.0).is_err());
    assert!(MeshData::torus_mesh(2, 8, 1.0, 0.25).is_err());
    assert!(MeshData::hollow_box_mesh(Vec3::ZERO, [1.0, 0.0, 1.0]).is_err());
    assert!(MeshData::platform_mesh(Vec3::ZERO, 1.0, 0.0, 2.0).is_err());
    assert!(HeightField::grid(1, 2, [1.0, 1.0, 1.0], false).is_err());
    assert!(
        HeightField::from_samples(2, 2, vec![0.0, 1.0, 2.0], [1.0, 1.0, 1.0], None, false).is_err()
    );
    assert!(
        HeightField::from_samples(
            2,
            2,
            vec![0.0, 1.0, 2.0, f32::NAN],
            [1.0, 1.0, 1.0],
            None,
            false,
        )
        .is_err()
    );
    assert!(
        HeightField::from_samples(
            2,
            2,
            vec![0.0, 1.0, 2.0, 3.0],
            [1.0, 1.0, 1.0],
            Some(&[0, 1]),
            false,
        )
        .is_err()
    );
    assert!(HeightField::wave(2, 2, [1.0, 1.0, 1.0], f32::NAN, 1.0, false).is_err());
    assert!(
        Compound::single_sphere(
            Sphere::new(Vec3::ZERO, f32::INFINITY),
            SurfaceMaterial::default()
        )
        .is_err()
    );
    assert!(Compound::builder().build().is_err());
    let mesh = MeshData::box_mesh(Vec3::ZERO, [1.0, 1.0, 1.0], true).unwrap();
    let mut compound_builder = Compound::builder();
    compound_builder
        .add_sphere(Sphere::new(Vec3::ZERO, 0.25), SurfaceMaterial::default())
        .unwrap();
    assert!(
        compound_builder
            .add_mesh(
                &mesh,
                Transform::IDENTITY,
                [0.0, 1.0, 1.0],
                [SurfaceMaterial::default()],
            )
            .is_err()
    );
    assert_eq!(
        SurfaceMaterial {
            friction: -1.0,
            ..Default::default()
        }
        .validate(),
        Err(Error::InvalidArgument)
    );
    assert_eq!(
        SurfaceMaterial {
            restitution: -1.0,
            ..Default::default()
        }
        .validate(),
        Err(Error::InvalidArgument)
    );
    assert_eq!(
        SurfaceMaterial {
            rolling_resistance: -1.0,
            ..Default::default()
        }
        .validate(),
        Err(Error::InvalidArgument)
    );
    assert_eq!(
        ShapeDef::builder().friction(-1.0).build().validate(),
        Err(Error::InvalidArgument)
    );
    assert_eq!(
        ShapeDef::builder().restitution(-1.0).build().validate(),
        Err(Error::InvalidArgument)
    );
}
