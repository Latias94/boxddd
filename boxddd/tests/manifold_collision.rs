use boxddd::{
    Capsule, RayCastInput, ShapeCastInput, ShapeProxy, Sphere, Transform, Vec3, collide_capsules,
    collide_spheres, ray_cast_capsule, ray_cast_hollow_sphere, ray_cast_sphere, shape_cast_capsule,
    shape_cast_sphere,
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
