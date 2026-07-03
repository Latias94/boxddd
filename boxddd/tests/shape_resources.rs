use boxddd::{
    BodyDef, BodyType, BoxHull, Capsule, Compound, CompoundChild, Error, HeightField, Hull,
    MeshData, ShapeDef, ShapeType, Sphere, SurfaceMaterial, Transform, Vec3, World, WorldDef,
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
    let hull_view = world.try_shape_hull(box_hull).unwrap();
    assert_eq!(hull_view.vertex_count(), 8);
    assert!(hull_view.surface_area() > 0.0);
    assert_eq!(
        world.try_shape_hull(sphere).unwrap_err(),
        Error::InvalidArgument
    );

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
    let mesh_view = world.try_shape_mesh(mesh_shape).unwrap();
    assert_eq!(mesh_view.scale(), Vec3::new(1.0, 1.0, 1.0));
    assert!(mesh_view.vertex_count() > 0);
    assert!(mesh_view.triangle_count() > 0);

    let custom_mesh = MeshData::builder(vec![Vec3::ZERO, Vec3::X, Vec3::Y], vec![0, 1, 2])
        .material_indices(vec![0])
        .build()
        .unwrap();
    assert_eq!(custom_mesh.material_count(), 1);
    assert_eq!(custom_mesh.triangle_count(), 1);
    assert!(custom_mesh.tree_height() >= 0);
    assert!(
        MeshData::wave_mesh(2, 2, 1.0, 0.25, 1.0, 1.0)
            .unwrap()
            .triangle_count()
            > 0
    );
    assert!(
        MeshData::torus_mesh(8, 8, 1.0, 0.25)
            .unwrap()
            .triangle_count()
            > 0
    );
    assert!(
        MeshData::hollow_box_mesh(Vec3::ZERO, [1.0, 1.0, 1.0])
            .unwrap()
            .triangle_count()
            > 0
    );
    assert!(
        MeshData::platform_mesh(Vec3::ZERO, 1.0, 0.5, 1.0)
            .unwrap()
            .triangle_count()
            > 0
    );

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
    let height_view = world.try_shape_height_field(height_shape).unwrap();
    assert_eq!(height_view.column_count(), 4);
    assert_eq!(height_view.row_count(), 4);

    let custom_height = HeightField::builder(2, 2, vec![0.0, 1.0, 0.5, 1.5])
        .material_indices(vec![boxddd::HEIGHT_FIELD_HOLE])
        .clockwise_winding(true)
        .build([1.0, 1.0, 1.0])
        .unwrap();
    assert_eq!(custom_height.row_count(), 2);
    assert_eq!(custom_height.column_count(), 2);
    assert!(custom_height.byte_count() > 0);
    let wave_height = HeightField::wave(4, 4, [1.0, 1.0, 1.0], 1.0, 1.0, false).unwrap();
    assert_eq!(wave_height.row_count(), 4);
    assert_eq!(wave_height.column_count(), 4);

    let compound_mesh = MeshData::box_mesh([0.0, 0.0, 0.0], [0.25, 0.25, 0.25], true).unwrap();
    let mut compound_builder = Compound::builder();
    compound_builder
        .add_capsule(
            Capsule::new([-1.0, 0.0, 0.0], [-1.0, 0.75, 0.0], 0.2),
            SurfaceMaterial {
                user_material_id: 11,
                ..Default::default()
            },
        )
        .unwrap();
    compound_builder
        .add_hull(
            &transformed_hull,
            Transform::IDENTITY,
            SurfaceMaterial {
                user_material_id: 12,
                ..Default::default()
            },
        )
        .unwrap();
    compound_builder
        .add_mesh(
            &compound_mesh,
            Transform::IDENTITY,
            [1.0, 1.0, 1.0],
            [SurfaceMaterial {
                user_material_id: 13,
                ..Default::default()
            }],
        )
        .unwrap();
    compound_builder
        .add_sphere(
            Sphere::new([1.0, 0.0, 0.0], 0.25),
            SurfaceMaterial {
                user_material_id: 14,
                ..Default::default()
            },
        )
        .unwrap();
    let compound = compound_builder.build().unwrap();
    assert_eq!(compound.child_count(), 4);
    assert_eq!(compound.capsule_count(), 1);
    assert_eq!(compound.hull_count(), 1);
    assert_eq!(compound.mesh_count(), 1);
    assert_eq!(compound.sphere_count(), 1);
    assert_eq!(compound.material_count(), 4);
    assert_eq!(compound.material(0).unwrap().user_material_id, 11);
    assert_eq!(compound.material(3).unwrap().user_material_id, 14);
    assert_eq!(compound.material(4).unwrap_err(), Error::IndexOutOfRange);
    let child: CompoundChild<'_> = compound.child(0).unwrap();
    assert_eq!(child.shape_type(), ShapeType::Capsule);
    assert_eq!(compound.child(1).unwrap().shape_type(), ShapeType::Hull);
    assert_eq!(compound.child(2).unwrap().shape_type(), ShapeType::Mesh);
    assert_eq!(compound.child(3).unwrap().shape_type(), ShapeType::Sphere);
    assert_eq!(compound.child(4).unwrap_err(), Error::IndexOutOfRange);

    let compound_shape = world
        .try_create_compound_shape(static_body, &def, compound)
        .unwrap();
    assert_eq!(
        world.try_shape_type(compound_shape).unwrap(),
        ShapeType::Compound
    );
    assert_eq!(
        world
            .try_shape_compound(compound_shape)
            .unwrap()
            .child_count(),
        4
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

    let sphere = world.create_sphere_shape(dynamic_body, &def, &Sphere::new([0.0, 0.0, 0.0], 0.25));
    assert_eq!(
        world
            .try_set_shape_mesh(
                sphere,
                MeshData::box_mesh([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], true).unwrap(),
                [1.0, 1.0, 1.0],
            )
            .unwrap_err(),
        Error::InvalidArgument
    );
}
