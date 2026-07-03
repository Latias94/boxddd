use boxddd::{
    Aabb, BodyDef, BodyType, BoxHull, Capsule, Compound, CompoundChild, Error, HeightField, Hull,
    MeshData, Quat, ShapeDef, ShapeType, Sphere, SurfaceMaterial, Transform, Vec3, World, WorldDef,
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
    let cloned_hull = transformed_hull.try_clone().unwrap();
    assert_ne!(
        cloned_hull.as_hull_data() as *const _,
        transformed_hull.as_hull_data() as *const _
    );
    assert_eq!(
        cloned_hull.as_hull_data().byteCount,
        transformed_hull.as_hull_data().byteCount
    );
    let moved_hull = transformed_hull
        .try_clone_transformed(Transform::new(Vec3::X, Quat::IDENTITY), [1.0, 1.0, 1.0])
        .unwrap();
    assert_ne!(
        moved_hull.as_hull_data().center.x,
        transformed_hull.as_hull_data().center.x
    );
    assert_eq!(
        transformed_hull
            .try_clone_transformed(
                Transform::new(Vec3::ZERO, Quat::new(Vec3::ZERO, f32::NAN)),
                [1.0, 1.0, 1.0],
            )
            .unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        transformed_hull
            .try_clone_transformed(Transform::IDENTITY, [0.0, 1.0, 1.0])
            .unwrap_err(),
        Error::InvalidArgument
    );

    let scaled_box =
        BoxHull::scale_box([1.0, 2.0, 3.0], Transform::IDENTITY, [-2.0, 0.5, 1.5], 0.1).unwrap();
    assert!(scaled_box.half_widths.x >= 0.1);
    assert!(scaled_box.half_widths.y >= 0.1);
    assert!(scaled_box.half_widths.z >= 0.1);
    assert!(scaled_box.transform.is_valid());
    assert_eq!(
        BoxHull::scale_box(
            [1.0, 2.0, 3.0],
            Transform::IDENTITY,
            [f32::NAN, 1.0, 1.0],
            0.1
        )
        .unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        BoxHull::scale_box([1.0, 2.0, 3.0], Transform::IDENTITY, [1.0, 1.0, 1.0], 0.0).unwrap_err(),
        Error::InvalidArgument
    );

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

    let query_bounds = boxddd::Aabb {
        lower_bound: [-2.0, -2.0, -2.0].into(),
        upper_bound: [2.0, 2.0, 2.0].into(),
    };
    let query_hits = compound.query_aabb(query_bounds).unwrap();
    assert_eq!(query_hits.len(), compound.child_count() as usize);
    assert!(query_hits.iter().all(|hit| hit.child_index >= 0));
    assert!(
        query_hits
            .iter()
            .any(|hit| hit.child.shape_type() == ShapeType::Capsule)
    );
    assert!(
        query_hits
            .iter()
            .any(|hit| hit.child.shape_type() == ShapeType::Hull)
    );
    assert!(
        query_hits
            .iter()
            .any(|hit| hit.child.shape_type() == ShapeType::Mesh)
    );
    assert!(
        query_hits
            .iter()
            .any(|hit| hit.child.shape_type() == ShapeType::Sphere)
    );

    let mut reusable_hits = Vec::with_capacity(8);
    compound
        .query_aabb_into(query_bounds, &mut reusable_hits)
        .unwrap();
    assert_eq!(reusable_hits.len(), query_hits.len());

    let mut visited = 0;
    compound
        .visit_query_aabb(query_bounds, |hit| {
            visited += 1;
            assert!(hit.child_index >= 0);
            false
        })
        .unwrap();
    assert_eq!(visited, 1);

    compound
        .visit_query_aabb(query_bounds, |_| {
            assert_eq!(
                compound.query_aabb(query_bounds).unwrap_err(),
                Error::InCallback
            );
            false
        })
        .unwrap();

    assert_eq!(
        compound
            .query_aabb(boxddd::Aabb {
                lower_bound: [1.0, 0.0, 0.0].into(),
                upper_bound: [0.0, 0.0, 0.0].into(),
            })
            .unwrap_err(),
        Error::InvalidArgument
    );

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

#[test]
fn mesh_and_height_field_query_visitors_return_owned_triangles() {
    let mesh = MeshData::box_mesh([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], true).unwrap();
    let broad_bounds = Aabb {
        lower_bound: [-1.25, -1.25, -1.25].into(),
        upper_bound: [1.25, 1.25, 1.25].into(),
    };

    let mesh_hits = mesh.query_triangles(broad_bounds, [1.0, 1.0, 1.0]).unwrap();
    assert!(!mesh_hits.is_empty());
    assert!(mesh_hits.iter().all(|hit| {
        hit.triangle_index >= 0 && hit.a.is_valid() && hit.b.is_valid() && hit.c.is_valid()
    }));

    let mut reusable_hits = Vec::with_capacity(16);
    mesh.query_triangles_into(broad_bounds, [1.0, 1.0, 1.0], &mut reusable_hits)
        .unwrap();
    assert_eq!(reusable_hits.len(), mesh_hits.len());

    let mut visited = 0;
    mesh.visit_triangles(broad_bounds, [1.0, 1.0, 1.0], |hit| {
        visited += 1;
        assert!(hit.triangle_index >= 0);
        false
    })
    .unwrap();
    assert_eq!(visited, 1);

    let narrow_hits = mesh
        .query_triangles(
            Aabb {
                lower_bound: [-0.1, -0.1, -0.1].into(),
                upper_bound: [0.1, 0.1, 0.1].into(),
            },
            [1.0, 1.0, 1.0],
        )
        .unwrap();
    assert!(narrow_hits.len() <= mesh_hits.len());

    assert_eq!(
        mesh.query_triangles(broad_bounds, [0.0, 1.0, 1.0])
            .unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        mesh.query_triangles(
            Aabb {
                lower_bound: [1.0, 0.0, 0.0].into(),
                upper_bound: [0.0, 0.0, 0.0].into(),
            },
            [1.0, 1.0, 1.0],
        )
        .unwrap_err(),
        Error::InvalidArgument
    );

    mesh.visit_triangles(broad_bounds, [1.0, 1.0, 1.0], |_| {
        assert_eq!(
            mesh.query_triangles(broad_bounds, [1.0, 1.0, 1.0])
                .unwrap_err(),
            Error::InCallback
        );
        false
    })
    .unwrap();

    let height_field = HeightField::grid(3, 3, [1.0, 1.0, 1.0], false).unwrap();
    let height_bounds = Aabb {
        lower_bound: [-0.25, -1.0, -0.25].into(),
        upper_bound: [2.25, 1.0, 2.25].into(),
    };
    let height_hits = height_field.query_triangles(height_bounds).unwrap();
    assert!(!height_hits.is_empty());
    assert!(height_hits.iter().all(|hit| {
        hit.triangle_index >= 0 && hit.a.is_valid() && hit.b.is_valid() && hit.c.is_valid()
    }));

    let mut height_reusable_hits = Vec::new();
    height_field
        .query_triangles_into(height_bounds, &mut height_reusable_hits)
        .unwrap();
    assert_eq!(height_reusable_hits.len(), height_hits.len());

    let mut height_visited = 0;
    height_field
        .visit_triangles(height_bounds, |hit| {
            height_visited += 1;
            assert!(hit.triangle_index >= 0);
            assert_eq!(
                height_field.query_triangles(height_bounds).unwrap_err(),
                Error::InCallback
            );
        })
        .unwrap();
    assert_eq!(height_visited, height_hits.len());

    assert_eq!(
        height_field
            .query_triangles(Aabb {
                lower_bound: [1.0, 0.0, 0.0].into(),
                upper_bound: [0.0, 0.0, 0.0].into(),
            })
            .unwrap_err(),
        Error::InvalidArgument
    );
}
