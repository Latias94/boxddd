use boxddd::{BodyDef, BodyType, BoxHull, Error, ShapeDef, TaskSystem, Vec3, World, WorldDef};
use static_assertions::assert_not_impl_any;

assert_not_impl_any!(World: Send, Sync);

fn populate_parallel_scene(world: &mut World) {
    let ground = world.create_body(BodyDef::builder().position([0.0, -1.0, 0.0]).build());
    world.create_hull_shape(ground, &ShapeDef::default(), &BoxHull::new(30.0, 0.5, 30.0));

    let shape_def = ShapeDef::builder().density(1.0).friction(0.4).build();
    for x in 0..12 {
        for z in 0..12 {
            let body = world.create_body(
                BodyDef::builder()
                    .body_type(BodyType::Dynamic)
                    .position([(x as f32 - 6.0) * 0.8, 3.0, (z as f32 - 6.0) * 0.8])
                    .build(),
            );
            world.create_hull_shape(body, &shape_def, &BoxHull::cube(0.35));
        }
    }
}

fn step_parallel_world(task_system: TaskSystem) -> (TaskSystem, World) {
    let def = WorldDef::builder()
        .gravity(Vec3::new(0.0, -10.0, 0.0))
        .worker_count(2)
        .task_system(task_system.clone())
        .build();
    let mut world = World::new(def).unwrap();
    populate_parallel_scene(&mut world);
    world.try_step(1.0 / 60.0, 4).unwrap();
    (task_system, world)
}

#[test]
fn blocking_task_system_steps_world_through_box3d_callbacks() {
    let (task_system, world) = step_parallel_world(TaskSystem::blocking_threads());

    let stats = task_system.stats();
    assert!(stats.enqueued > 0, "{stats:?}");
    assert_eq!(stats.enqueued, stats.started, "{stats:?}");
    assert_eq!(stats.enqueued, stats.completed, "{stats:?}");
    assert_eq!(stats.enqueued, stats.finished, "{stats:?}");
    assert_eq!(
        boxddd::__private::task_system_guard_rejections_for_test(&task_system),
        0
    );
    assert!(!stats.panicked, "{stats:?}");
    assert!(world.counters().task_count > 0);
}

#[test]
fn world_def_clone_preserves_task_system_context() {
    let task_system = TaskSystem::blocking_threads();
    let def = WorldDef::builder()
        .worker_count(2)
        .task_system(task_system.clone())
        .build();
    let mut world = World::new(def.clone()).unwrap();
    populate_parallel_scene(&mut world);

    world.try_step(1.0 / 60.0, 4).unwrap();

    assert!(task_system.stats().enqueued > 0);
}

#[test]
fn zero_worker_count_is_clamped_by_builder() {
    let def = WorldDef::builder().worker_count(0).build();
    let mut world = World::new(def).unwrap();

    world.try_step(1.0 / 60.0, 4).unwrap();
    assert!(world.worker_count() >= 1);
}

#[test]
fn enqueue_panic_is_reported_as_callback_panic() {
    let task_system = boxddd::__private::task_system_panic_on_enqueue_for_test();
    let def = WorldDef::builder()
        .worker_count(2)
        .task_system(task_system.clone())
        .build();
    let mut world = World::new(def).unwrap();
    populate_parallel_scene(&mut world);

    assert_eq!(world.try_step(1.0 / 60.0, 4), Err(Error::CallbackPanicked));
    assert!(task_system.stats().panicked);
}

#[test]
fn task_panic_is_reported_as_callback_panic() {
    let task_system = boxddd::__private::task_system_panic_on_task_for_test();
    let def = WorldDef::builder()
        .worker_count(2)
        .task_system(task_system.clone())
        .build();
    let mut world = World::new(def).unwrap();
    populate_parallel_scene(&mut world);

    assert_eq!(world.try_step(1.0 / 60.0, 4), Err(Error::CallbackPanicked));
    assert!(task_system.stats().panicked);
}

#[test]
fn finish_panic_is_reported_as_callback_panic() {
    let task_system = boxddd::__private::task_system_panic_on_finish_for_test();
    let def = WorldDef::builder()
        .worker_count(2)
        .task_system(task_system.clone())
        .build();
    let mut world = World::new(def).unwrap();
    populate_parallel_scene(&mut world);

    assert_eq!(world.try_step(1.0 / 60.0, 4), Err(Error::CallbackPanicked));
    assert!(task_system.stats().panicked);
}

#[test]
fn task_callback_runs_under_callback_guard() {
    let task_system = boxddd::__private::task_system_check_callback_guard_for_test();
    let def = WorldDef::builder()
        .worker_count(2)
        .task_system(task_system.clone())
        .build();
    let mut world = World::new(def).unwrap();
    populate_parallel_scene(&mut world);

    world.try_step(1.0 / 60.0, 4).unwrap();

    let stats = task_system.stats();
    assert!(boxddd::__private::task_system_guard_rejections_for_test(&task_system) > 0);
    assert!(!stats.panicked, "{stats:?}");
}
