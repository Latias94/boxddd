use boxddd::{BodyDef, BodyType, Recording, ShapeDef, Sphere, World, WorldDef};

#[test]
fn replay_validation_is_explicit_about_worker_count() {
    let mut world = World::new(WorldDef::builder().gravity([0.0, -9.8, 0.0]).build()).unwrap();
    let mut recording = Recording::new().unwrap();
    world.try_start_recording(&mut recording).unwrap();
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
    for _ in 0..6 {
        world.step(1.0 / 60.0, 4);
    }
    world.try_stop_recording(&mut recording).unwrap();

    assert!(recording.validate_replay(1).unwrap());
    let mut player = recording.create_player(1).unwrap();
    assert_eq!(player.info().worker_count, 1);
    player.set_worker_count(1).unwrap();
    player.restart().unwrap();
    while player.step_frame().unwrap() {}
    assert!(!player.has_diverged());
}
