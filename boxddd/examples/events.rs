use boxddd::prelude::*;

fn main() -> boxddd::Result<()> {
    sensor_events()?;
    contact_and_body_events()?;
    Ok(())
}

fn sensor_events() -> boxddd::Result<()> {
    let mut world = World::new(WorldDef::default())?;

    let sensor_body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Static)
            .position([1.5, 0.0, 0.0])
            .build(),
    );
    let sensor_shape = world.create_hull_shape(
        sensor_body,
        &ShapeDef::builder()
            .sensor(true)
            .enable_sensor_events(true)
            .build(),
        &BoxHull::new(0.5, 4.0, 4.0),
    );

    let visitor_body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([7.4, 0.0, 0.0])
            .linear_velocity([-20.0, 0.0, 0.0])
            .gravity_scale(0.0)
            .bullet(true)
            .build(),
    );
    let visitor_shape = world.create_sphere_shape(
        visitor_body,
        &ShapeDef::builder()
            .density(1.0)
            .enable_sensor_events(true)
            .build(),
        &Sphere::new(Vec3::ZERO, 0.2),
    );

    let mut begin_seen = false;
    let mut end_seen = false;
    let mut reusable = SensorEvents::default();

    for _ in 0..120 {
        world.step(1.0 / 60.0, 4);
        world.sensor_events_into(&mut reusable);

        begin_seen |= reusable.begin.iter().any(|event| {
            [event.sensor_shape, event.visitor_shape].contains(&sensor_shape)
                && [event.sensor_shape, event.visitor_shape].contains(&visitor_shape)
        });
        end_seen |= reusable.end.iter().any(|event| {
            [event.sensor_shape, event.visitor_shape].contains(&sensor_shape)
                && [event.sensor_shape, event.visitor_shape].contains(&visitor_shape)
        });

        if begin_seen && end_seen {
            break;
        }
    }

    let view_count = world.try_with_sensor_events_view(|begin, end| begin.count() + end.count())?;
    println!(
        "sensor events: begin_seen={begin_seen}, end_seen={end_seen}, current_frame_count={view_count}"
    );
    assert!(begin_seen, "expected a sensor begin event");
    assert!(end_seen, "expected a sensor end event");

    Ok(())
}

fn contact_and_body_events() -> boxddd::Result<()> {
    let mut world = World::new(
        WorldDef::builder()
            .gravity(Vec3::new(0.0, -10.0, 0.0))
            .build(),
    )?;
    world.try_set_hit_event_threshold(1.0)?;

    let ground = world.create_body(BodyDef::builder().position([0.0, -0.5, 0.0]).build());
    let ground_shape = world.create_hull_shape(
        ground,
        &ShapeDef::builder().user_material_id(11).build(),
        &BoxHull::new(8.0, 0.5, 8.0),
    );

    let falling_body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([0.0, 4.0, 0.0])
            .build(),
    );
    let falling_shape = world.create_sphere_shape(
        falling_body,
        &ShapeDef::builder()
            .density(1.0)
            .restitution(0.6)
            .user_material_id(7)
            .enable_contact_events(true)
            .enable_hit_events(true)
            .build(),
        &Sphere::new(Vec3::ZERO, 0.5),
    );

    let mut contact_events = ContactEvents::default();
    let mut body_moves = Vec::new();
    let mut begin_seen = false;
    let mut hit_seen = false;
    let mut moved_seen = false;

    for _ in 0..180 {
        world.step(1.0 / 60.0, 4);
        world.contact_events_into(&mut contact_events);
        world.body_events_into(&mut body_moves);

        begin_seen |= contact_events.begin.iter().any(|event| {
            [event.shape_a, event.shape_b].contains(&ground_shape)
                && [event.shape_a, event.shape_b].contains(&falling_shape)
        });
        hit_seen |= contact_events.hit.iter().any(|event| {
            [event.shape_a, event.shape_b].contains(&ground_shape)
                && [event.shape_a, event.shape_b].contains(&falling_shape)
                && event.approach_speed > 0.0
        });
        moved_seen |= body_moves.iter().any(|event| event.body_id == falling_body);

        if begin_seen && hit_seen && moved_seen {
            break;
        }
    }

    let body_view_count = world.try_with_body_events_view(|events| events.count())?;
    let contact_view_count = world.try_with_contact_events_view(|begin, end, hit| {
        begin.count() + end.count() + hit.count()
    })?;
    println!(
        "contact/body events: begin_seen={begin_seen}, hit_seen={hit_seen}, moved_seen={moved_seen}, current_body_moves={body_view_count}, current_contact_events={contact_view_count}"
    );
    assert!(begin_seen, "expected a contact begin event");
    assert!(hit_seen, "expected a hit event");
    assert!(moved_seen, "expected a body move event");

    Ok(())
}
