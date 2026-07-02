use boxddd::{
    Capsule, Error, HeightField, Hull, MeshData, ShapeProxy, Sphere, Transform, Vec3,
    compute_capsule_mass, compute_height_field_aabb, compute_hull_aabb, compute_mesh_aabb,
    compute_sphere_aabb, compute_sphere_mass,
};

#[test]
fn shape_proxy_validates_owned_point_cloud() {
    assert_eq!(
        ShapeProxy::new(Vec::<Vec3>::new(), 0.0).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        ShapeProxy::new(vec![Vec3::ZERO], f32::NAN).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        ShapeProxy::new(vec![Vec3::new(f32::INFINITY, 0.0, 0.0)], 0.0).unwrap_err(),
        Error::InvalidArgument
    );

    let proxy = ShapeProxy::capsule(Vec3::ZERO, Vec3::X, 0.25).unwrap();
    assert_eq!(proxy.points().len(), 2);
    assert_eq!(proxy.radius(), 0.25);
}

#[test]
fn mass_and_aabb_helpers_validate_inputs() {
    let sphere = Sphere::new(Vec3::ZERO, 1.0);
    assert!(compute_sphere_mass(&sphere, 1.0).unwrap().mass > 0.0);
    assert_eq!(
        compute_sphere_mass(&sphere, -1.0).unwrap_err(),
        Error::InvalidArgument
    );

    let capsule = Capsule::new(Vec3::ZERO, Vec3::Y, 0.25);
    assert!(compute_capsule_mass(&capsule, 2.0).unwrap().mass > 0.0);
    let sphere_aabb = compute_sphere_aabb(&sphere, Transform::IDENTITY).unwrap();
    assert!(sphere_aabb.lower_bound.x < sphere_aabb.upper_bound.x);

    let hull = Hull::rock(0.5).unwrap();
    let hull_aabb = compute_hull_aabb(&hull, Transform::IDENTITY).unwrap();
    assert!(hull_aabb.lower_bound.x <= hull_aabb.upper_bound.x);

    let mesh = MeshData::box_mesh(Vec3::ZERO, [1.0, 1.0, 1.0], true).unwrap();
    let mesh_aabb = compute_mesh_aabb(&mesh, Transform::IDENTITY, [1.0, 1.0, 1.0]).unwrap();
    assert!(mesh_aabb.lower_bound.y <= mesh_aabb.upper_bound.y);

    let height = HeightField::grid(4, 4, [1.0, 1.0, 1.0], false).unwrap();
    let height_aabb = compute_height_field_aabb(&height, Transform::IDENTITY).unwrap();
    assert!(height_aabb.lower_bound.z <= height_aabb.upper_bound.z);
}
