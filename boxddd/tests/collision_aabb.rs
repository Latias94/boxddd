use boxddd::{
    Capsule, Compound, HeightField, MeshData, ShapeProxy, Sphere, SurfaceMaterial, Transform, Vec3,
    overlap_capsule, overlap_compound, overlap_height_field, overlap_mesh, overlap_sphere,
};

#[test]
fn overlap_helpers_cover_value_and_resource_shapes() {
    let proxy = ShapeProxy::sphere(0.25).unwrap();
    let sphere = Sphere::new(Vec3::ZERO, 1.0);
    assert!(overlap_sphere(&sphere, Transform::IDENTITY, &proxy).unwrap());

    let capsule = Capsule::new([0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 0.25);
    assert!(overlap_capsule(&capsule, Transform::IDENTITY, &proxy).unwrap());

    let mesh = MeshData::box_mesh(Vec3::ZERO, [1.0, 1.0, 1.0], true).unwrap();
    let _ = overlap_mesh(&mesh, [1.0, 1.0, 1.0], Transform::IDENTITY, &proxy).unwrap();

    let height = HeightField::grid(4, 4, [1.0, 1.0, 1.0], false).unwrap();
    let _ = overlap_height_field(&height, Transform::IDENTITY, &proxy).unwrap();

    let compound =
        Compound::single_sphere(Sphere::new(Vec3::ZERO, 0.5), SurfaceMaterial::default()).unwrap();
    assert!(overlap_compound(&compound, Transform::IDENTITY, &proxy).unwrap());
}
