use boxddd::prelude::*;

fn main() -> boxddd::Result<()> {
    let mut world = World::new(
        WorldDef::builder()
            .gravity(Vec3::new(0.0, -10.0, 0.0))
            .build(),
    )?;

    let ground = world.create_body(BodyDef::builder().position([0.0, -0.5, 0.0]).build());
    world.create_hull_shape(ground, &ShapeDef::default(), &BoxHull::new(8.0, 0.5, 8.0));

    let dynamic_body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([-1.5, 2.5, 0.0])
            .build(),
    );
    world.create_hull_shape(
        dynamic_body,
        &ShapeDef::builder().density(1.0).friction(0.4).build(),
        &BoxHull::cube(0.5),
    );

    let kinematic_body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Kinematic)
            .position([1.5, 1.0, 0.0])
            .linear_velocity([0.75, 0.0, 0.0])
            .build(),
    );
    world.create_sphere_shape(
        kinematic_body,
        &ShapeDef::builder().density(1.0).build(),
        &Sphere::new(Vec3::ZERO, 0.35),
    );

    world.try_disable_body(dynamic_body)?;
    world.step(1.0 / 60.0, 4);
    let disabled_state = world.try_body_enabled(dynamic_body)?;
    println!("dynamic enabled after disable: {disabled_state}");
    assert!(!disabled_state, "expected disabled body to report disabled");

    world.try_enable_body(dynamic_body)?;
    let enabled_state = world.try_body_enabled(dynamic_body)?;
    println!("dynamic enabled after enable: {enabled_state}");
    assert!(enabled_state, "expected re-enabled body to report enabled");

    world.try_set_body_motion_locks(
        dynamic_body,
        MotionLocks::new(false, false, true, true, true, false),
    )?;
    world.try_apply_force_to_center(dynamic_body, [25.0, 0.0, 0.0], true)?;
    world.try_apply_linear_impulse_to_center(dynamic_body, [0.0, 4.0, 0.0], true)?;

    for _ in 0..60 {
        world.step(1.0 / 60.0, 4);
    }

    let dynamic_transform = world.try_body_transform(dynamic_body)?;
    let dynamic_velocity = world.try_body_linear_velocity(dynamic_body)?;
    let kinematic_position = world.try_body_position(kinematic_body)?;

    println!(
        "dynamic after force+impulse: position={:?}, velocity={:?}, locks={:?}",
        dynamic_transform.p,
        dynamic_velocity,
        world.try_body_motion_locks(dynamic_body)?
    );
    println!("kinematic body moved to: {kinematic_position:?}");

    world.try_set_body_type(kinematic_body, BodyType::Dynamic)?;
    world.try_set_body_linear_velocity(kinematic_body, [0.0, 0.0, 0.0])?;
    world.try_set_body_transform(kinematic_body, [0.0, 3.0, 0.0], Quat::IDENTITY)?;
    world.step(1.0 / 60.0, 4);
    let converted_type = world.try_body_type(kinematic_body)?;
    println!(
        "converted kinematic body type={:?}, synced position={:?}",
        converted_type,
        world.try_body_position(kinematic_body)?
    );
    assert_eq!(converted_type, BodyType::Dynamic);

    Ok(())
}
