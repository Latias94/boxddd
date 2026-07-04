use boxddd::prelude::*;

fn main() -> boxddd::Result<()> {
    let moving_sphere = ShapeProxy::sphere(0.25)?;
    let target_sphere = ShapeProxy::sphere(0.5)?;

    let cast = shape_cast_pair(ShapeCastPairInput::new(
        target_sphere.clone(),
        moving_sphere.clone(),
        Transform::new(Vec3::new(-4.0, 0.0, 0.0), Quat::IDENTITY),
        Vec3::new(8.0, 0.0, 0.0),
    )?)?;

    println!(
        "shape cast: hit={}, fraction={:.3}, point={:?}, normal={:?}",
        cast.hit, cast.fraction, cast.point, cast.normal
    );
    assert!(cast.hit, "expected the swept sphere to hit");

    let sweep_a = Sweep::default();
    let sweep_b = Sweep::new(
        Vec3::ZERO,
        Vec3::new(-4.0, 0.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Quat::IDENTITY,
        Quat::IDENTITY,
    )?;
    let toi = time_of_impact(TimeOfImpactInput::new(
        target_sphere,
        moving_sphere,
        sweep_a,
        sweep_b,
    )?)?;

    let impact_transform = get_sweep_transform(&sweep_b, toi.fraction)?;
    println!(
        "time of impact: state={:?}, fraction={:.3}, distance={:.3}, impact_position={:?}, used_fallback={}",
        toi.state, toi.fraction, toi.distance, impact_transform.p, toi.used_fallback
    );
    assert_eq!(toi.state, TimeOfImpactState::Hit);

    let mut world = World::new(WorldDef::default())?;
    let wall = world.create_body(BodyDef::builder().position([0.0, 0.0, 0.0]).build());
    world.create_hull_shape(wall, &ShapeDef::default(), &BoxHull::new(0.05, 2.0, 2.0));

    let bullet = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([-3.0, 0.0, 0.0])
            .linear_velocity([60.0, 0.0, 0.0])
            .gravity_scale(0.0)
            .bullet(true)
            .build(),
    );
    world.create_sphere_shape(
        bullet,
        &ShapeDef::builder().density(1.0).build(),
        &Sphere::new(Vec3::ZERO, 0.15),
    );

    for _ in 0..10 {
        world.step(1.0 / 120.0, 4);
    }

    println!(
        "bullet world body: bullet={}, position={:?}, velocity={:?}",
        world.try_body_bullet(bullet)?,
        world.try_body_position(bullet)?,
        world.try_body_linear_velocity(bullet)?
    );
    assert!(world.try_body_bullet(bullet)?);

    Ok(())
}
