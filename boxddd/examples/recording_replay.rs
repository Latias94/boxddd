use boxddd::{BodyDef, BodyType, Recording, ShapeDef, Sphere, World, WorldDef};

fn main() -> boxddd::Result<()> {
    let mut world = World::new(WorldDef::builder().gravity([0.0, -9.8, 0.0]).build())?;
    let mut recording = Recording::new()?;
    world.try_start_recording(&mut recording)?;

    let body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([0.0, 2.0, 0.0])
            .build(),
    );
    world.create_sphere_shape(
        body,
        &ShapeDef::builder().density(1.0).build(),
        &Sphere::new([0.0, 0.0, 0.0], 0.25),
    );
    for _ in 0..30 {
        world.try_step(1.0 / 60.0, 4)?;
    }
    world.try_stop_recording(&mut recording)?;

    let mut player = recording.create_player(1)?;
    while player.step_frame()? {}
    println!(
        "replayed {} frames, diverged: {}",
        player.frame(),
        player.has_diverged()
    );
    Ok(())
}
