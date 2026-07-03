use boxddd::{
    Capsule, CollisionPlane, DistanceInput, Error, HeightField, Hull, MeshData, Plane, Quat,
    ShapeCastInput, ShapeCastPairInput, ShapeProxy, Sphere, Sweep, TimeOfImpactInput, Transform,
    Vec3, collide_sphere_and_triangle, compute_capsule_mass, compute_height_field_aabb,
    compute_hull_aabb, compute_mesh_aabb, compute_sphere_aabb, compute_sphere_mass,
    shape_cast_pair, shape_cast_sphere, shape_distance, solve_planes, sweep_transform,
    time_of_impact,
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

#[test]
fn advanced_collision_helpers_validate_inputs() {
    let sphere = ShapeProxy::sphere(1.0).unwrap();
    let invalid_transform = Transform::new(Vec3::ZERO, Quat::new(Vec3::ZERO, f32::NAN));
    assert_eq!(
        DistanceInput::new(sphere.clone(), sphere.clone(), invalid_transform).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        shape_distance(
            DistanceInput::new(sphere.clone(), sphere.clone(), Transform::IDENTITY).unwrap()
        )
        .unwrap()
        .distance,
        0.0
    );
    assert_eq!(
        ShapeCastPairInput::with_options(
            sphere.clone(),
            sphere.clone(),
            Transform::IDENTITY,
            [f32::NAN, 0.0, 0.0],
            1.0,
            false,
        )
        .unwrap_err(),
        Error::InvalidArgument
    );
    assert!(
        shape_cast_pair(
            ShapeCastPairInput::new(sphere.clone(), sphere.clone(), Transform::IDENTITY, Vec3::X)
                .unwrap(),
        )
        .is_ok()
    );
    let mut mutated_shape_cast = ShapeCastInput::new(sphere.clone(), [1.0, 0.0, 0.0]).unwrap();
    mutated_shape_cast.max_fraction = f32::NAN;
    assert_eq!(
        shape_cast_sphere(&Sphere::new(Vec3::ZERO, 0.5), mutated_shape_cast).unwrap_err(),
        Error::InvalidArgument
    );

    assert_eq!(
        Sweep::new(
            Vec3::ZERO,
            Vec3::ZERO,
            Vec3::ZERO,
            Quat::new(Vec3::ZERO, f32::NAN),
            Quat::IDENTITY,
        )
        .unwrap_err(),
        Error::InvalidArgument
    );
    let invalid_sweep = Sweep {
        local_center: Vec3::ZERO,
        c1: Vec3::ZERO,
        c2: Vec3::ZERO,
        q1: Quat::new(Vec3::ZERO, f32::NAN),
        q2: Quat::IDENTITY,
    };
    assert_eq!(
        sweep_transform(invalid_sweep, 0.0).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        sweep_transform(
            Sweep::new(
                Vec3::ZERO,
                Vec3::ZERO,
                Vec3::ZERO,
                Quat::IDENTITY,
                Quat::IDENTITY,
            )
            .unwrap(),
            f32::NAN,
        )
        .unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        TimeOfImpactInput::with_max_fraction(
            sphere.clone(),
            sphere,
            Sweep::default(),
            Sweep::default(),
            f32::NAN,
        )
        .unwrap_err(),
        Error::InvalidArgument
    );
    assert!(
        time_of_impact(
            TimeOfImpactInput::new(
                ShapeProxy::sphere(1.0).unwrap(),
                ShapeProxy::sphere(1.0).unwrap(),
                Sweep::default(),
                Sweep::default(),
            )
            .unwrap()
        )
        .is_ok()
    );

    assert_eq!(
        CollisionPlane::new(
            Plane {
                normal: Vec3::new(f32::NAN, 0.0, 0.0),
                offset: 0.0,
            },
            1.0,
            true,
        )
        .unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        CollisionPlane::new(
            Plane {
                normal: Vec3::new(2.0, 0.0, 0.0),
                offset: 0.0,
            },
            1.0,
            true,
        )
        .unwrap_err(),
        Error::InvalidArgument
    );
    let mut empty_planes = [];
    assert_eq!(
        solve_planes(Vec3::ZERO, &mut empty_planes).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        collide_sphere_and_triangle(
            &Sphere::new(Vec3::ZERO, 1.0),
            [Vec3::ZERO, Vec3::X, Vec3::new(2.0, 0.0, 0.0)],
        )
        .unwrap_err(),
        Error::InvalidArgument
    );
}
