use boxddd::{
    Capsule, Compound, Error, HeightField, Hull, MeshData, Sphere, SurfaceMaterial, Vec3,
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
    assert!(HeightField::grid(1, 2, [1.0, 1.0, 1.0], false).is_err());
    assert!(
        Compound::single_sphere(
            Sphere::new(Vec3::ZERO, f32::INFINITY),
            SurfaceMaterial::default()
        )
        .is_err()
    );
}
