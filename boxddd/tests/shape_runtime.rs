use boxddd::{
    BodyDef, BodyType, Capsule, Error, Filter, Hull, MeshData, ShapeDef, Sphere, SurfaceMaterial,
    World, WorldDef,
};

#[test]
fn shape_runtime_properties_and_geometry_can_be_updated() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::builder().body_type(BodyType::Static).build());
    let shape = world.create_sphere_shape(
        body,
        &ShapeDef::builder().density(1.0).friction(0.3).build(),
        &Sphere::new([0.0, 0.0, 0.0], 0.5),
    );

    assert_eq!(world.try_shape_body(shape).unwrap(), body);
    assert!(!world.try_shape_sensor(shape).unwrap());
    world.try_set_shape_density(shape, 2.0, true).unwrap();
    assert_eq!(world.try_shape_density(shape).unwrap(), 2.0);
    world.try_set_shape_friction(shape, 0.7).unwrap();
    assert_eq!(world.try_shape_friction(shape).unwrap(), 0.7);
    world.try_set_shape_restitution(shape, 0.4).unwrap();
    assert_eq!(world.try_shape_restitution(shape).unwrap(), 0.4);

    let material = SurfaceMaterial {
        friction: 0.2,
        restitution: 0.1,
        ..Default::default()
    };
    world
        .try_set_shape_surface_material(shape, material)
        .unwrap();
    assert_eq!(world.try_shape_surface_material(shape).unwrap(), material);
    assert_eq!(
        world
            .try_set_shape_surface_material(
                shape,
                SurfaceMaterial {
                    rolling_resistance: -0.1,
                    ..Default::default()
                }
            )
            .unwrap_err(),
        Error::InvalidArgument
    );

    let filter = Filter {
        category_bits: 2,
        mask_bits: 4,
        group_index: -1,
    };
    world.try_set_shape_filter(shape, filter, false).unwrap();
    assert_eq!(world.try_shape_filter(shape).unwrap(), filter);
    world.try_enable_shape_sensor_events(shape, true).unwrap();
    world.try_enable_shape_contact_events(shape, true).unwrap();
    world
        .try_enable_shape_pre_solve_events(shape, true)
        .unwrap();
    world.try_enable_shape_hit_events(shape, true).unwrap();

    let replacement_sphere = Sphere::new([1.0, 0.0, 0.0], 0.25);
    world
        .try_set_shape_sphere(shape, &replacement_sphere)
        .unwrap();
    assert_eq!(world.try_shape_sphere(shape).unwrap(), replacement_sphere);
    let _ = world.try_shape_aabb(shape).unwrap();

    let capsule_shape = world
        .try_create_capsule_shape(
            body,
            &ShapeDef::default(),
            &Capsule::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.25),
        )
        .unwrap();
    let replacement_capsule = Capsule::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.2);
    world
        .try_set_shape_capsule(capsule_shape, &replacement_capsule)
        .unwrap();
    assert_eq!(
        world.try_shape_capsule(capsule_shape).unwrap(),
        replacement_capsule
    );

    let hull_shape = world
        .try_create_created_hull_shape(body, &ShapeDef::default(), &Hull::rock(0.5).unwrap())
        .unwrap();
    let new_hull = Hull::cylinder(1.0, 0.25, 0.0, 8).unwrap();
    world.try_set_shape_hull(hull_shape, &new_hull).unwrap();

    let mesh_shape = world
        .try_create_mesh_shape(
            body,
            &ShapeDef::default(),
            MeshData::box_mesh([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], true).unwrap(),
            [1.0, 1.0, 1.0],
        )
        .unwrap();
    assert_eq!(
        world
            .try_set_shape_mesh_material(mesh_shape, 999, SurfaceMaterial::default())
            .unwrap_err(),
        Error::IndexOutOfRange
    );
    world
        .try_set_shape_mesh(
            mesh_shape,
            MeshData::box_mesh([0.0, 0.0, 0.0], [0.5, 0.5, 0.5], true).unwrap(),
            [1.0, 1.0, 1.0],
        )
        .unwrap();

    world.destroy_shape(shape, true);
    assert!(!shape.is_valid());
}
