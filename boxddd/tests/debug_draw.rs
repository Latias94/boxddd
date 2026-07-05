use boxddd::{
    Aabb, BodyDef, BodyType, BoxHull, Capsule, Compound, DebugDrawCommand, DebugDrawOptions,
    DebugShapeEvent, DebugShapeGeometry, Error, HeightField, HexColor, MeshData, ShapeDef, Sphere,
    SurfaceMaterial, Vec3, World, WorldDef,
};

fn debug_world() -> World {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::default());
    world.create_hull_shape(body, &ShapeDef::default(), &BoxHull::cube(1.0));
    world
}

#[test]
fn debug_draw_collects_shape_commands_and_reuses_buffer() {
    let mut world = debug_world();
    let mut commands = vec![DebugDrawCommand::Transform(Default::default())];
    let initial_capacity = commands.capacity();

    world.debug_draw_collect_into(&mut commands, DebugDrawOptions::default());

    assert!(commands.capacity() >= initial_capacity);
    assert!(commands.iter().any(|command| matches!(
        command,
        DebugDrawCommand::Shape {
            handle: Some(_),
            ..
        }
    )));

    let first_len = commands.len();
    world.debug_draw_collect_into(&mut commands, DebugDrawOptions::default());
    assert_eq!(commands.len(), first_len);
}

#[test]
fn debug_draw_frame_emits_shape_asset_events_and_reuses_handles() {
    let mut world = debug_world();

    let first = world
        .try_debug_draw_frame(DebugDrawOptions::default())
        .unwrap();
    let created: Vec<_> = first
        .events
        .iter()
        .filter_map(|event| match event {
            DebugShapeEvent::Created(asset) => Some(asset),
            _ => None,
        })
        .collect();
    assert_eq!(created.len(), 1);
    assert!(matches!(
        created[0].geometry,
        DebugShapeGeometry::Hull { .. }
    ));
    let handle = created[0].handle;
    assert!(first.commands.iter().any(|command| {
        matches!(command, DebugDrawCommand::Shape { handle: Some(seen), .. } if *seen == handle)
    }));

    let second = world
        .try_debug_draw_frame(DebugDrawOptions::default())
        .unwrap();
    assert!(
        second
            .events
            .iter()
            .all(|event| !matches!(event, DebugShapeEvent::Created(_)))
    );
    assert!(second.commands.iter().any(|command| {
        matches!(command, DebugDrawCommand::Shape { handle: Some(seen), .. } if *seen == handle)
    }));
}

#[test]
fn debug_draw_frame_queues_destroy_events_for_drawn_shapes() {
    let mut world = debug_world();
    let first = world
        .try_debug_draw_frame(DebugDrawOptions::default())
        .unwrap();
    let handle = first
        .events
        .iter()
        .find_map(|event| match event {
            DebugShapeEvent::Created(asset) => Some(asset.handle),
            _ => None,
        })
        .expect("first frame should create a debug shape");
    let shape_id = first
        .events
        .iter()
        .find_map(|event| match event {
            DebugShapeEvent::Created(asset) => Some(asset.shape_id),
            _ => None,
        })
        .expect("first frame should include source shape id");

    world.destroy_shape(shape_id, true);
    let second = world
        .try_debug_draw_frame(DebugDrawOptions::default())
        .unwrap();

    assert!(second.events.iter().any(|event| {
        matches!(event, DebugShapeEvent::Destroyed { handle: seen } if *seen == handle)
    }));
    assert!(second.commands.iter().all(|command| {
        !matches!(command, DebugDrawCommand::Shape { handle: Some(seen), .. } if *seen == handle)
    }));
}

#[test]
fn debug_draw_frame_copies_sphere_geometry() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::default());
    let sphere = Sphere::new([1.0, 2.0, 3.0], 0.75);
    world.create_sphere_shape(body, &ShapeDef::default(), &sphere);

    let frame = world
        .try_debug_draw_frame(DebugDrawOptions::default())
        .unwrap();
    let asset = frame
        .events
        .iter()
        .find_map(|event| match event {
            DebugShapeEvent::Created(asset) => Some(asset),
            _ => None,
        })
        .expect("sphere debug shape asset should be created");

    assert!(matches!(
        asset.geometry,
        DebugShapeGeometry::Sphere {
            center,
            radius
        } if center == [1.0, 2.0, 3.0].into() && radius == 0.75
    ));
}

#[test]
fn debug_draw_frame_copies_capsule_geometry() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::default());
    let capsule = Capsule::new([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.25);
    world.create_capsule_shape(body, &ShapeDef::default(), &capsule);

    let frame = world
        .try_debug_draw_frame(DebugDrawOptions::default())
        .unwrap();
    let asset = frame
        .events
        .iter()
        .find_map(|event| match event {
            DebugShapeEvent::Created(asset) => Some(asset),
            _ => None,
        })
        .expect("capsule debug shape asset should be created");

    assert!(matches!(
        asset.geometry,
        DebugShapeGeometry::Capsule {
            center1,
            center2,
            radius
        } if center1 == [-1.0, 0.0, 0.0].into()
            && center2 == [1.0, 0.0, 0.0].into()
            && radius == 0.25
    ));
}

#[test]
fn debug_draw_frame_copies_mesh_geometry() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::builder().body_type(BodyType::Static).build());
    let mesh = MeshData::box_mesh(Vec3::ZERO, [0.5, 0.5, 0.5], true).unwrap();
    world
        .try_create_mesh_shape(body, &ShapeDef::default(), mesh, [1.0, 2.0, 1.0])
        .unwrap();

    let frame = world
        .try_debug_draw_frame(DebugDrawOptions::default())
        .unwrap();
    let asset = frame
        .events
        .iter()
        .find_map(|event| match event {
            DebugShapeEvent::Created(asset) => Some(asset),
            _ => None,
        })
        .expect("mesh debug shape asset should be created");

    assert!(matches!(
        &asset.geometry,
        DebugShapeGeometry::Mesh { mesh, scale }
            if !mesh.vertices.is_empty()
            && !mesh.triangles.is_empty()
            && *scale == [1.0, 2.0, 1.0].into()
    ));
}

#[test]
fn debug_draw_frame_copies_height_field_geometry() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::builder().body_type(BodyType::Static).build());
    let height_field = HeightField::grid(3, 3, [1.0, 1.0, 1.0], false).unwrap();
    world
        .try_create_height_field_shape(body, &ShapeDef::default(), height_field)
        .unwrap();

    let frame = world
        .try_debug_draw_frame(DebugDrawOptions::default())
        .unwrap();
    let asset = frame
        .events
        .iter()
        .find_map(|event| match event {
            DebugShapeEvent::Created(asset) => Some(asset),
            _ => None,
        })
        .expect("height-field debug shape asset should be created");

    assert!(matches!(
        &asset.geometry,
        DebugShapeGeometry::HeightField { mesh }
            if mesh.vertices.len() == 9 && mesh.triangles.len() == 8
    ));
}

#[test]
fn debug_draw_frame_flattens_compound_children() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::builder().body_type(BodyType::Static).build());
    let compound = Compound::builder()
        .sphere(
            Sphere::new([0.0, 0.0, 0.0], 0.5),
            SurfaceMaterial::default(),
        )
        .capsule(
            Capsule::new([0.0, -0.5, 0.0], [0.0, 0.5, 0.0], 0.1),
            SurfaceMaterial::default(),
        )
        .build()
        .unwrap();
    world
        .try_create_compound_shape(body, &ShapeDef::default(), compound)
        .unwrap();

    let frame = world
        .try_debug_draw_frame(DebugDrawOptions::default())
        .unwrap();
    let asset = frame
        .events
        .iter()
        .find_map(|event| match event {
            DebugShapeEvent::Created(asset) => Some(asset),
            _ => None,
        })
        .expect("compound debug shape asset should be created");

    match &asset.geometry {
        DebugShapeGeometry::Compound { children } => {
            assert_eq!(children.len(), 2);
            assert!(
                children
                    .iter()
                    .any(|child| matches!(child.geometry, DebugShapeGeometry::Sphere { .. }))
            );
            assert!(
                children
                    .iter()
                    .any(|child| matches!(child.geometry, DebugShapeGeometry::Capsule { .. }))
            );
        }
        other => panic!("expected compound debug geometry, got {other:?}"),
    }
}

#[test]
fn hex_color_preserves_box3d_material_payload() {
    let raw = 0x05_ab_cd_ef_u32;
    let color = HexColor::from_raw(raw);

    assert_eq!(color.raw_u32(), raw);
    assert_eq!(color.rgb_u32(), 0x00_ab_cd_ef);
    assert_eq!(color.into_raw(), raw);
    assert_eq!(HexColor::from_rgb_u32(raw).raw_u32(), 0x00_ab_cd_ef);
}

#[test]
fn debug_draw_options_can_collect_bounds_commands() {
    let mut world = debug_world();
    let mut options = DebugDrawOptions::default();
    options.draw_bounds = true;

    let commands = world.debug_draw_collect(options);

    assert!(
        commands
            .iter()
            .any(|command| matches!(command, DebugDrawCommand::Bounds { .. }))
    );
}

#[test]
fn debug_draw_shape_callback_visits_shape_commands() {
    let mut world = World::new(WorldDef::default()).unwrap();
    for offset in [-2.0, 0.0, 2.0] {
        let body = world.create_body(BodyDef::builder().position([offset, 0.0, 0.0]).build());
        world.create_hull_shape(body, &ShapeDef::default(), &BoxHull::cube(0.2));
    }

    let baseline_shape_count = world
        .debug_draw_collect(DebugDrawOptions::default())
        .iter()
        .filter(|command| matches!(command, DebugDrawCommand::Shape { .. }))
        .count();
    assert!(baseline_shape_count > 1);

    let frame = world
        .try_debug_draw_frame(DebugDrawOptions::default())
        .unwrap();

    assert_eq!(
        frame
            .commands
            .iter()
            .filter(|command| matches!(command, DebugDrawCommand::Shape { .. }))
            .count(),
        baseline_shape_count
    );
}

#[test]
fn debug_draw_respects_callback_guard() {
    let mut world = debug_world();
    let _guard = boxddd::__private::enter_callback_guard_for_test();

    assert_eq!(
        world
            .try_debug_draw_collect(DebugDrawOptions::default())
            .unwrap_err(),
        Error::InCallback
    );
}

#[test]
fn debug_draw_rejects_invalid_bounds() {
    let mut world = debug_world();
    let mut options = DebugDrawOptions::default();
    options.drawing_bounds = Aabb {
        lower_bound: [2.0, 0.0, 0.0].into(),
        upper_bound: [1.0, 1.0, 1.0].into(),
    };

    assert_eq!(
        world.try_debug_draw_collect(options).unwrap_err(),
        Error::InvalidArgument
    );
}
