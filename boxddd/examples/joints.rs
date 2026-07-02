use boxddd::{BodyDef, BodyType, DistanceJointDef, ShapeDef, Sphere, Vec3, World, WorldDef};

fn main() -> boxddd::Result<()> {
    let mut world = World::new(WorldDef::builder().gravity(Vec3::ZERO).build())?;
    let anchor = world.create_body(BodyDef::builder().position([0.0, 0.0, 0.0]).build());
    let body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([1.0, 0.0, 0.0])
            .build(),
    );
    world.create_sphere_shape(
        body,
        &ShapeDef::builder().density(1.0).build(),
        &Sphere::new([0.0, 0.0, 0.0], 0.25),
    );

    let joint = world.create_distance_joint(DistanceJointDef::new(anchor, body).length(1.0));
    world.try_apply_force_to_center(body, [25.0, 0.0, 0.0], true)?;
    for _ in 0..60 {
        world.try_step(1.0 / 60.0, 4)?;
    }

    println!(
        "distance joint length after one second: {:.3}",
        world.try_distance_joint_current_length(joint)?
    );
    Ok(())
}
