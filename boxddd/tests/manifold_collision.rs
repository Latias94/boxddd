use boxddd::{
    Capsule, CollisionPlane, DistanceInput, Hull, Plane, Quat, RayCastInput, ShapeCastInput,
    ShapeCastPairInput, ShapeProxy, Sphere, Sweep, TimeOfImpactInput, TimeOfImpactState, Transform,
    Vec3, clip_vector, collide_capsule_and_sphere, collide_capsule_and_triangle, collide_capsules,
    collide_hull_and_capsule, collide_hull_and_sphere, collide_hull_and_triangle, collide_hulls,
    collide_sphere_and_triangle, collide_spheres, get_sweep_transform, ray_cast_capsule,
    ray_cast_hollow_sphere, ray_cast_sphere, shape_cast_capsule, shape_cast_pair,
    shape_cast_sphere, shape_distance, solve_planes, time_of_impact,
};

#[test]
fn ray_casts_report_expected_hits_and_misses() {
    let sphere = Sphere::new(Vec3::ZERO, 1.0);
    let hit = ray_cast_sphere(
        &sphere,
        RayCastInput::new([-3.0, 0.0, 0.0], [6.0, 0.0, 0.0]).unwrap(),
    )
    .unwrap();
    assert!(hit.hit);
    assert!(hit.fraction > 0.0 && hit.fraction < 1.0);
    assert!(hit.point.x < 0.0);

    let miss = ray_cast_sphere(
        &sphere,
        RayCastInput::new([-3.0, 3.0, 0.0], [6.0, 0.0, 0.0]).unwrap(),
    )
    .unwrap();
    assert!(!miss.hit);

    let hollow = ray_cast_hollow_sphere(
        &sphere,
        RayCastInput::new([0.0, 0.0, 0.0], [2.0, 0.0, 0.0]).unwrap(),
    )
    .unwrap();
    assert!(hollow.hit);

    let capsule = Capsule::new([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.25);
    assert!(
        ray_cast_capsule(
            &capsule,
            RayCastInput::new([0.0, -2.0, 0.0], [0.0, 4.0, 0.0]).unwrap()
        )
        .unwrap()
        .hit
    );
}

#[test]
fn shape_casts_and_local_manifolds_are_owned_values() {
    let sphere = Sphere::new(Vec3::ZERO, 1.0);
    let moving_proxy = ShapeProxy::new(vec![Vec3::new(-3.0, 0.0, 0.0)], 0.25).unwrap();
    let cast = shape_cast_sphere(
        &sphere,
        ShapeCastInput::new(moving_proxy, [6.0, 0.0, 0.0]).unwrap(),
    )
    .unwrap();
    assert!(cast.hit);
    assert!(cast.fraction > 0.0 && cast.fraction < 1.0);
    assert!(cast.point.is_valid());
    assert!(cast.normal.is_valid());

    let miss_proxy = ShapeProxy::new(vec![Vec3::new(-3.0, 3.0, 0.0)], 0.25).unwrap();
    let miss = shape_cast_sphere(
        &sphere,
        ShapeCastInput::new(miss_proxy.clone(), [6.0, 0.0, 0.0]).unwrap(),
    )
    .unwrap();
    assert!(!miss.hit);

    let capsule = Capsule::new([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.25);
    let capsule_cast = shape_cast_capsule(
        &capsule,
        ShapeCastInput::new(miss_proxy, [0.0, 3.0, 0.0]).unwrap(),
    )
    .unwrap();
    assert!(capsule_cast.fraction >= 0.0);

    let contact = collide_spheres(
        &Sphere::new(Vec3::ZERO, 1.0),
        &Sphere::new(Vec3::ZERO, 1.0),
        Transform::IDENTITY,
    )
    .unwrap();
    assert!(!contact.points.is_empty());

    let capsule_contact = collide_capsules(&capsule, &capsule, Transform::IDENTITY).unwrap();
    assert!(capsule_contact.points.len() <= boxddd::collision::MAX_LOCAL_MANIFOLD_POINTS);
}

#[test]
fn distance_pair_cast_toi_and_plane_helpers_are_owned_values() {
    let sphere_a = ShapeProxy::sphere(1.0).unwrap();
    let sphere_b = ShapeProxy::sphere(0.5).unwrap();
    let separated = shape_distance(
        DistanceInput::new(
            sphere_a.clone(),
            sphere_b.clone(),
            Transform::new(Vec3::new(3.0, 0.0, 0.0), Quat::IDENTITY),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(separated.distance > 1.0);
    assert!(separated.point_a.is_valid());
    assert!(separated.point_b.is_valid());
    assert!(separated.iterations >= 0);

    let overlapped = shape_distance(
        DistanceInput::new(sphere_a.clone(), sphere_b.clone(), Transform::IDENTITY).unwrap(),
    )
    .unwrap();
    assert_eq!(overlapped.distance, 0.0);
    assert!(
        DistanceInput::new(
            sphere_a.clone(),
            sphere_b.clone(),
            Transform::new(Vec3::new(f32::NAN, 0.0, 0.0), Quat::IDENTITY),
        )
        .is_err()
    );

    let pair_hit = shape_cast_pair(
        ShapeCastPairInput::new(
            sphere_a.clone(),
            sphere_b.clone(),
            Transform::new(Vec3::new(-3.0, 0.0, 0.0), Quat::IDENTITY),
            [6.0, 0.0, 0.0],
        )
        .unwrap(),
    )
    .unwrap();
    assert!(pair_hit.hit);
    assert!(pair_hit.fraction > 0.0 && pair_hit.fraction < 1.0);

    let pair_miss = shape_cast_pair(
        ShapeCastPairInput::new(
            sphere_a.clone(),
            sphere_b.clone(),
            Transform::new(Vec3::new(-3.0, 3.0, 0.0), Quat::IDENTITY),
            [-6.0, 0.0, 0.0],
        )
        .unwrap(),
    )
    .unwrap();
    assert!(!pair_miss.hit);
    assert!(
        ShapeCastPairInput::new(
            sphere_a.clone(),
            sphere_b.clone(),
            Transform::IDENTITY,
            [f32::NAN, 0.0, 0.0],
        )
        .is_err()
    );

    let sweep_a = Sweep::default();
    let sweep_b = Sweep::new(
        Vec3::ZERO,
        Vec3::new(-3.0, 0.0, 0.0),
        Vec3::new(3.0, 0.0, 0.0),
        Quat::IDENTITY,
        Quat::IDENTITY,
    )
    .unwrap();
    let midpoint = get_sweep_transform(&sweep_b, 0.5).unwrap();
    assert!(midpoint.p.x.abs() < 0.01);

    let toi = time_of_impact(TimeOfImpactInput::new(sphere_a, sphere_b, sweep_a, sweep_b).unwrap())
        .unwrap();
    assert_eq!(toi.state, TimeOfImpactState::Hit);
    assert!(toi.fraction > 0.0 && toi.fraction < 1.0);
    assert_eq!(
        Sweep::new(
            Vec3::new(f32::NAN, 0.0, 0.0),
            Vec3::ZERO,
            Vec3::ZERO,
            Quat::IDENTITY,
            Quat::IDENTITY,
        )
        .unwrap_err(),
        boxddd::Error::InvalidArgument
    );

    let plane = CollisionPlane::new(
        Plane {
            normal: Vec3::Y,
            offset: 0.0,
        },
        f32::MAX,
        true,
    )
    .unwrap();
    let mut planes = [plane];
    let solved = solve_planes([1.0, -1.0, 0.0], &mut planes).unwrap();
    assert!(solved.delta.is_valid());
    let clipped = clip_vector([1.0, -1.0, 0.0], &planes).unwrap();
    assert!(clipped.is_valid());
    assert!(clipped.y >= -1.0);
    assert!(
        CollisionPlane::new(
            Plane {
                normal: Vec3::new(f32::NAN, 0.0, 0.0),
                offset: 0.0,
            },
            f32::MAX,
            true,
        )
        .is_err()
    );
}

#[test]
fn missing_collision_pairs_return_owned_manifolds() {
    let sphere = Sphere::new(Vec3::ZERO, 0.5);
    let capsule = Capsule::new([0.0, -0.5, 0.0], [0.0, 0.5, 0.0], 0.25);
    let hull_a = Hull::cylinder(1.0, 0.5, 0.0, 8).unwrap();
    let hull_b = Hull::rock(0.5).unwrap();
    let triangle = [
        Vec3::new(-1.0, -1.0, 0.0),
        Vec3::new(1.0, -1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    ];

    assert!(
        !collide_capsule_and_sphere(&capsule, &sphere, Transform::IDENTITY)
            .unwrap()
            .points
            .is_empty()
    );
    assert!(
        !collide_hull_and_sphere(&hull_a, &sphere, Transform::IDENTITY)
            .unwrap()
            .points
            .is_empty()
    );
    assert!(
        !collide_hull_and_capsule(&hull_a, &capsule, Transform::IDENTITY)
            .unwrap()
            .points
            .is_empty()
    );
    assert!(
        !collide_hulls(&hull_a, &hull_b, Transform::IDENTITY)
            .unwrap()
            .points
            .is_empty()
    );
    assert!(
        !collide_capsule_and_triangle(&capsule, triangle)
            .unwrap()
            .points
            .is_empty()
    );
    assert!(
        !collide_hull_and_triangle(&hull_a, triangle, 0)
            .unwrap()
            .points
            .is_empty()
    );
    assert!(
        !collide_sphere_and_triangle(&sphere, triangle)
            .unwrap()
            .points
            .is_empty()
    );
}
