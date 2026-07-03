use boxddd::{
    BodyDef, BodyType, Error, MotionLocks, QueryFilter, ShapeDef, ShapeProxy, Sphere, Vec3, World,
    WorldDef,
};

#[test]
fn world_runtime_tuning_and_metrics_are_safe() {
    let mut world = World::new(WorldDef::builder().gravity([0.0, -9.8, 0.0]).build()).unwrap();

    world.try_enable_sleeping(false).unwrap();
    assert!(!world.sleeping_enabled());
    world.try_enable_sleeping(true).unwrap();
    assert!(world.sleeping_enabled());

    world.try_enable_continuous(false).unwrap();
    assert!(!world.continuous_enabled());
    world.try_enable_continuous(true).unwrap();
    assert!(world.continuous_enabled());

    world.try_enable_warm_starting(false).unwrap();
    assert!(!world.warm_starting_enabled());
    world.try_enable_warm_starting(true).unwrap();
    assert!(world.warm_starting_enabled());

    world.try_enable_speculative(true).unwrap();
    world.try_set_restitution_threshold(2.0).unwrap();
    assert_eq!(world.restitution_threshold(), 2.0);
    world.try_set_hit_event_threshold(3.0).unwrap();
    assert_eq!(world.hit_event_threshold(), 3.0);
    world.try_set_contact_tuning(60.0, 0.7, 10.0).unwrap();
    world.try_set_contact_recycle_distance(0.02).unwrap();
    assert_eq!(world.contact_recycle_distance(), 0.02);
    world.try_set_maximum_linear_speed(120.0).unwrap();
    assert_eq!(world.maximum_linear_speed(), 120.0);
    world.try_set_worker_count(0).unwrap();
    assert!(world.worker_count() >= 0);

    let counters = world.counters();
    assert_eq!(counters.body_count, 0);
    let profile = world.profile();
    assert!(profile.step >= 0.0);
    let capacity = world.max_capacity();
    assert!(capacity.dynamic_body_count >= 0);
    world.try_rebuild_static_tree().unwrap();
}

#[test]
fn body_runtime_setters_getters_and_buffers_work() {
    let mut world = World::new(WorldDef::builder().gravity([0.0, -9.8, 0.0]).build()).unwrap();
    let body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([0.0, 3.0, 0.0])
            .build(),
    );
    let shape = world.create_sphere_shape(
        body,
        &ShapeDef::builder().density(1.0).build(),
        &Sphere::new([0.0, 0.0, 0.0], 0.5),
    );

    assert_eq!(world.body_type(body), BodyType::Dynamic);
    world.try_set_body_name(body, "runtime-body").unwrap();
    assert_eq!(
        world.try_body_name(body).unwrap().as_deref(),
        Some("runtime-body")
    );

    world
        .try_set_body_transform(body, [1.0, 4.0, 2.0], Default::default())
        .unwrap();
    assert_eq!(world.body_position(body).x, 1.0);
    assert_eq!(world.body_transform(body).p.z, 2.0);
    assert_eq!(world.body_rotation(body), Default::default());

    world
        .try_set_body_linear_velocity(body, [1.0, 2.0, 3.0])
        .unwrap();
    assert_eq!(world.body_linear_velocity(body), Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(
        world
            .try_body_local_point_velocity(body, Vec3::ZERO)
            .unwrap(),
        world.body_linear_velocity(body)
    );
    assert_eq!(
        world
            .try_body_world_point_velocity(body, [1.0, 4.0, 2.0])
            .unwrap(),
        world.body_linear_velocity(body)
    );
    world
        .try_set_body_angular_velocity(body, [0.1, 0.2, 0.3])
        .unwrap();
    assert_eq!(world.body_angular_velocity(body), Vec3::new(0.1, 0.2, 0.3));

    world.try_set_body_linear_damping(body, 0.4).unwrap();
    assert_eq!(world.try_body_linear_damping(body).unwrap(), 0.4);
    world.try_set_body_angular_damping(body, 0.5).unwrap();
    assert_eq!(world.try_body_angular_damping(body).unwrap(), 0.5);
    world.try_set_body_gravity_scale(body, 0.25).unwrap();
    assert_eq!(world.try_body_gravity_scale(body).unwrap(), 0.25);

    let locks = MotionLocks::new(true, false, true, false, true, false);
    world.try_set_body_motion_locks(body, locks).unwrap();
    assert_eq!(world.try_body_motion_locks(body).unwrap(), locks);
    world
        .try_set_body_motion_locks(body, MotionLocks::default())
        .unwrap();
    world.try_set_body_bullet(body, true).unwrap();
    assert!(world.try_body_bullet(body).unwrap());
    world
        .try_enable_body_contact_recycling(body, false)
        .unwrap();
    assert!(!world.try_body_contact_recycling_enabled(body).unwrap());
    world.try_enable_body_hit_events(body, true).unwrap();

    assert!(world.try_body_mass(body).unwrap() > 0.0);
    assert!(world.try_body_inverse_mass(body).unwrap() > 0.0);
    let mass_data = world.try_body_mass_data(body).unwrap();
    world.try_set_body_mass_data(body, mass_data).unwrap();
    world.try_apply_mass_from_shapes(body).unwrap();
    let _ = world.try_body_local_rotational_inertia(body).unwrap();
    let _ = world
        .try_body_world_inverse_rotational_inertia(body)
        .unwrap();
    let _ = world.try_body_local_center_of_mass(body).unwrap();
    let _ = world.try_body_world_center_of_mass(body).unwrap();
    let _ = world.try_body_aabb(body).unwrap();

    world
        .try_apply_force_to_center(body, [1.0, 0.0, 0.0], true)
        .unwrap();
    world
        .try_apply_force(body, [0.0, 1.0, 0.0], world.body_position(body), true)
        .unwrap();
    world.try_apply_torque(body, [0.0, 0.0, 1.0], true).unwrap();
    world
        .try_apply_linear_impulse_to_center(body, [1.0, 0.0, 0.0], true)
        .unwrap();
    world
        .try_apply_linear_impulse(body, [0.0, 1.0, 0.0], world.body_position(body), true)
        .unwrap();
    world
        .try_apply_angular_impulse(body, [0.0, 0.0, 0.1], true)
        .unwrap();
    world.step(1.0 / 60.0, 4);
    assert!(world.body_linear_velocity(body).x > 0.0);

    let mut shapes = vec![shape];
    world.try_body_shapes_into(body, &mut shapes).unwrap();
    assert_eq!(shapes, vec![shape]);
    assert_eq!(world.body_shapes(body), vec![shape]);

    let mut joints = Vec::from([Default::default()]);
    world.try_body_joints_into(body, &mut joints).unwrap();
    assert!(joints.is_empty());

    let mut contacts = vec![Default::default()];
    world.try_body_contacts_into(body, &mut contacts).unwrap();
    assert!(contacts.is_empty());
}

#[test]
fn body_runtime_respects_callback_guard() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::default());
    let _guard = boxddd::__private::enter_callback_guard_for_test();

    assert_eq!(
        world.try_body_linear_velocity(body).unwrap_err(),
        boxddd::Error::InCallback
    );
}

#[test]
fn world_rejects_foreign_body_and_shape_handles() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let local = world.create_body(BodyDef::default());

    let mut other = World::new(WorldDef::default()).unwrap();
    let foreign_body = other.create_body(BodyDef::default());
    let foreign_shape = other.create_sphere_shape(
        foreign_body,
        &ShapeDef::default(),
        &Sphere::new(Vec3::ZERO, 0.5),
    );

    assert_eq!(
        world.try_body_position(foreign_body).unwrap_err(),
        Error::InvalidBodyId
    );
    assert_eq!(
        world
            .try_body_closest_point(foreign_body, Vec3::ZERO)
            .unwrap_err(),
        Error::InvalidBodyId
    );
    assert_eq!(
        world
            .try_body_overlap_shape(
                foreign_body,
                Vec3::ZERO,
                &ShapeProxy::sphere(0.5).unwrap(),
                QueryFilter::default()
            )
            .unwrap_err(),
        Error::InvalidBodyId
    );
    assert_eq!(
        world.try_destroy_body(foreign_body).unwrap_err(),
        Error::InvalidBodyId
    );
    assert!(foreign_body.is_valid());
    assert_eq!(
        world
            .try_create_sphere_shape(
                foreign_body,
                &ShapeDef::default(),
                &Sphere::new(Vec3::ZERO, 0.5)
            )
            .unwrap_err(),
        Error::InvalidBodyId
    );
    assert_eq!(
        world.try_shape_body(foreign_shape).unwrap_err(),
        Error::InvalidShapeId
    );
    assert_eq!(
        world.try_destroy_shape(foreign_shape, true).unwrap_err(),
        Error::InvalidShapeId
    );
    assert!(foreign_shape.is_valid());

    world.destroy_body(local);
}
