use boxddd::{
    Aabb, BodyDef, BodyType, BoxHull, DebugDrawOptions, Error, QueryFilter, RecPlayer, Recording,
    ShapeDef, Sphere, Vec3, World, WorldDef, validate_replay_bytes,
};
use std::path::PathBuf;

fn record_basic_scene(frame_count: usize) -> Recording {
    let mut world = World::new(WorldDef::builder().gravity([0.0, -10.0, 0.0]).build()).unwrap();
    let mut recording = Recording::new().unwrap();
    world.try_start_recording(&mut recording).unwrap();

    let ground = world.create_body(BodyDef::builder().position([0.0, -1.0, 0.0]).build());
    world.create_hull_shape(ground, &ShapeDef::default(), &BoxHull::new(10.0, 0.5, 10.0));
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

    for _ in 0..frame_count {
        world.step(1.0 / 60.0, 4);
    }
    world.try_stop_recording(&mut recording).unwrap();
    recording
}

fn temp_recording_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("boxddd-{name}-{}.b3rec", std::process::id()))
}

#[test]
fn recording_round_trips_and_player_controls_work() {
    let recording = record_basic_scene(8);
    assert!(!recording.is_empty());
    assert!(recording.bytes().unwrap().len() > 128);
    assert!(recording.validate_replay(1).unwrap());

    let mut player = recording.create_player(1).unwrap();
    assert!(player.world_id().is_valid());
    assert_eq!(player.frame(), 0);
    assert!(player.frame_count() >= 8);
    let info = player.info();
    assert_eq!(info.frame_count, player.frame_count());
    assert_eq!(info.sub_step_count, 4);
    assert_eq!(info.worker_count, 1);

    assert!(player.step_frame().unwrap());
    assert_eq!(player.frame(), 1);
    player.seek_frame(player.frame_count()).unwrap();
    assert_eq!(player.frame(), player.frame_count());
    assert!(!player.step_frame().unwrap());
    assert!(player.is_at_end());
    assert!(!player.has_diverged());
    assert_eq!(player.diverge_frame(), None);
    assert!(player.body_count() >= 2);
    assert!(player.body_id(0).unwrap().is_some());

    player.restart().unwrap();
    assert_eq!(player.frame(), 0);
    assert!(!player.is_at_end());
    let world_id = player.world_id();
    player.seek_frame(1).unwrap();
    player.restart().unwrap();
    assert_eq!(player.world_id(), world_id);

    player.set_worker_count(1).unwrap();
    player.set_keyframe_policy(256 * 1024, 4).unwrap();
    assert_eq!(player.keyframe_budget(), 256 * 1024);
    assert_eq!(player.keyframe_min_interval(), 4);
    assert!(player.keyframe_interval() >= 4);
    assert!(player.keyframe_bytes() <= player.keyframe_budget());
}

#[test]
fn recording_can_be_saved_loaded_and_validated_from_bytes() {
    let recording = record_basic_scene(4);
    let bytes = recording.to_vec().unwrap();
    assert!(validate_replay_bytes(&bytes, 1).unwrap());

    let path = temp_recording_path("roundtrip");
    let _ = std::fs::remove_file(&path);
    recording.save_to_file(&path).unwrap();
    let loaded = Recording::load_from_file(&path).unwrap();
    assert_eq!(loaded.bytes().unwrap(), bytes.as_slice());
    assert!(loaded.validate_replay(1).unwrap());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn recording_buffer_survives_world_drop_while_active() {
    let mut recording = Recording::new().unwrap();
    {
        let mut world = World::new(WorldDef::default()).unwrap();
        world.try_start_recording(&mut recording).unwrap();
        let body = world.create_body(BodyDef::builder().body_type(BodyType::Dynamic).build());
        world.create_sphere_shape(
            body,
            &ShapeDef::builder().density(1.0).build(),
            &Sphere::new([0.0, 0.0, 0.0], 0.25),
        );
        world.step(1.0 / 60.0, 4);
        assert_eq!(
            recording.bytes().unwrap_err(),
            Error::ResourceLifetimeViolation
        );
    }

    assert!(!recording.bytes().unwrap().is_empty());
    assert!(recording.validate_replay(1).unwrap());
}

#[test]
fn world_rejects_a_second_active_recording() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let mut first = Recording::new().unwrap();
    let mut second = Recording::new().unwrap();

    world.try_start_recording(&mut first).unwrap();
    assert_eq!(
        world.try_start_recording(&mut second).unwrap_err(),
        Error::ResourceLifetimeViolation
    );

    world.step(1.0 / 60.0, 4);
    world.try_stop_recording(&mut first).unwrap();
    assert!(first.validate_replay(1).unwrap());
    assert!(second.bytes().unwrap().is_empty());
}

#[test]
fn stopping_with_the_wrong_recording_does_not_detach_the_active_recording() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let mut active = Recording::new().unwrap();
    let mut inactive = Recording::new().unwrap();

    world.try_start_recording(&mut active).unwrap();
    world.step(1.0 / 60.0, 4);
    assert_eq!(
        world.try_stop_recording(&mut inactive).unwrap_err(),
        Error::ResourceLifetimeViolation
    );
    assert_eq!(
        active.bytes().unwrap_err(),
        Error::ResourceLifetimeViolation
    );

    world.try_stop_recording(&mut active).unwrap();
    assert!(!active.bytes().unwrap().is_empty());
    assert!(active.validate_replay(1).unwrap());
}

#[test]
fn malformed_recordings_fail_without_panicking() {
    let malformed = [0_u8; 64];
    assert_eq!(
        RecPlayer::from_bytes(&malformed, 1).unwrap_err(),
        Error::CreateRecPlayerFailed
    );
    assert!(!validate_replay_bytes(&malformed, 1).unwrap());
    assert_eq!(
        validate_replay_bytes(&[], 1).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        RecPlayer::from_bytes(&malformed, 0).unwrap_err(),
        Error::InvalidArgument
    );
}

#[test]
fn replay_exposes_recorded_queries_and_draws_them() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::builder().position([0.0, 0.0, 0.0]).build());
    world.create_sphere_shape(
        body,
        &ShapeDef::default(),
        &Sphere::new([0.0, 0.0, 0.0], 0.5),
    );
    let mut recording = Recording::new().unwrap();
    world.try_start_recording(&mut recording).unwrap();

    let aabb = Aabb {
        lower_bound: Vec3::new(-1.0, -1.0, -1.0),
        upper_bound: Vec3::new(1.0, 1.0, 1.0),
    };
    for _ in 0..3 {
        let hits = world.overlap_aabb(aabb, QueryFilter::new().id(42)).unwrap();
        assert_eq!(hits.len(), 1);
        world.step(1.0 / 60.0, 4);
    }
    world.try_stop_recording(&mut recording).unwrap();

    let mut player = recording.create_player(1).unwrap();
    player.seek_frame(1).unwrap();
    assert_eq!(player.frame_query_count(), 1);
    let query = player.frame_query(0).unwrap();
    assert_eq!(query.id, 42);
    assert_eq!(query.hit_count, 1);
    let hit = player.frame_query_hit(0, 0).unwrap();
    assert!(hit.shape_id.is_valid());

    let commands = player
        .draw_frame_queries_collect(DebugDrawOptions::default(), None, None)
        .unwrap();
    assert!(!commands.is_empty());
}
