use boxddd::{
    BodyDef, BodyType, BoxHull, Capsule, Compound, Error, HeightField, Hull, MeshData, ShapeDef,
    ShapeType, Sphere, SurfaceMaterial, Transform, World, WorldDef,
};

#[test]
fn shape_creation_covers_value_and_native_resources() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let static_body = world.create_body(BodyDef::builder().body_type(BodyType::Static).build());
    let def = ShapeDef::builder().density(1.0).build();

    let sphere = world.create_sphere_shape(static_body, &def, &Sphere::new([0.0, 0.0, 0.0], 0.5));
    assert_eq!(world.try_shape_type(sphere).unwrap(), ShapeType::Sphere);

    let capsule = world
        .try_create_capsule_shape(
            static_body,
            &def,
            &Capsule::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.25),
        )
        .unwrap();
    assert_eq!(world.try_shape_type(capsule).unwrap(), ShapeType::Capsule);

    let box_hull = world.create_hull_shape(
        static_body,
        &def,
        &BoxHull::offset(0.5, 0.5, 0.5, [1.0, 0.0, 0.0]),
    );
    assert_eq!(world.try_shape_type(box_hull).unwrap(), ShapeType::Hull);

    let created_hull = Hull::rock(0.5).unwrap();
    let created_hull_shape = world
        .try_create_created_hull_shape(static_body, &def, &created_hull)
        .unwrap();
    drop(created_hull);
    assert!(created_hull_shape.is_valid());
    assert_eq!(
        world.try_shape_type(created_hull_shape).unwrap(),
        ShapeType::Hull
    );

    let transformed_hull = Hull::cylinder(1.0, 0.25, 0.0, 8).unwrap();
    let transformed_hull_shape = world
        .try_create_transformed_hull_shape(
            static_body,
            &def,
            &transformed_hull,
            Transform::default(),
            [1.0, 1.0, 1.0],
        )
        .unwrap();
    assert_eq!(
        world.try_shape_type(transformed_hull_shape).unwrap(),
        ShapeType::Hull
    );

    let mesh_shape = world
        .try_create_mesh_shape(
            static_body,
            &def,
            MeshData::box_mesh([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], true).unwrap(),
            [1.0, 1.0, 1.0],
        )
        .unwrap();
    assert_eq!(world.try_shape_type(mesh_shape).unwrap(), ShapeType::Mesh);

    let height_shape = world
        .try_create_height_field_shape(
            static_body,
            &def,
            HeightField::grid(4, 4, [1.0, 1.0, 1.0], false).unwrap(),
        )
        .unwrap();
    assert_eq!(
        world.try_shape_type(height_shape).unwrap(),
        ShapeType::HeightField
    );

    let compound_shape = world
        .try_create_compound_shape(
            static_body,
            &def,
            Compound::single_sphere(
                Sphere::new([0.0, 0.0, 0.0], 0.25),
                SurfaceMaterial::default(),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(
        world.try_shape_type(compound_shape).unwrap(),
        ShapeType::Compound
    );
}

#[test]
fn borrowed_resource_shapes_are_rejected_on_dynamic_bodies() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let dynamic_body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([0.0, 1.0, 0.0])
            .build(),
    );
    let def = ShapeDef::default();

    assert_eq!(
        world
            .try_create_mesh_shape(
                dynamic_body,
                &def,
                MeshData::box_mesh([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], true).unwrap(),
                [1.0, 1.0, 1.0],
            )
            .unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        world
            .try_create_height_field_shape(
                dynamic_body,
                &def,
                HeightField::grid(4, 4, [1.0, 1.0, 1.0], false).unwrap(),
            )
            .unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        world
            .try_create_compound_shape(
                dynamic_body,
                &def,
                Compound::single_sphere(
                    Sphere::new([0.0, 0.0, 0.0], 0.25),
                    SurfaceMaterial::default(),
                )
                .unwrap(),
            )
            .unwrap_err(),
        Error::InvalidArgument
    );
}
