use boxddd::{Aabb, BodyDef, BodyType, BoxHull, QueryFilter, ShapeDef, Vec3, World, WorldDef};
#[cfg(target_arch = "wasm32")]
use boxddd::{Error, TaskSystem, validate_replay_bytes};

fn main() -> boxddd::Result<()> {
    assert_wasm_thread_guardrails()?;

    let mut world = World::new(
        WorldDef::builder()
            .gravity(Vec3::new(0.0, -10.0, 0.0))
            .worker_count(1)
            .build(),
    )?;

    let ground = world.create_body(BodyDef::builder().position([0.0, -1.0, 0.0]).build());
    world.create_hull_shape(ground, &ShapeDef::default(), &BoxHull::new(8.0, 0.5, 8.0));

    let body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([0.0, 4.0, 0.0])
            .build(),
    );
    world.create_hull_shape(
        body,
        &ShapeDef::builder().density(1.0).friction(0.3).build(),
        &BoxHull::cube(0.5),
    );

    let start_y = world.body_position(body).y;
    for _ in 0..60 {
        world.try_step(1.0 / 60.0, 4)?;
    }
    let end_y = world.body_position(body).y;

    assert!(
        end_y < start_y - 0.1,
        "dynamic body did not fall: start_y={start_y}, end_y={end_y}"
    );

    let hits = world.overlap_aabb(
        Aabb {
            lower_bound: Vec3::new(-2.0, -2.0, -2.0),
            upper_bound: Vec3::new(2.0, 5.0, 2.0),
        },
        QueryFilter::default(),
    )?;
    assert!(
        !hits.is_empty(),
        "overlap query did not report any shapes after stepping"
    );

    println!(
        "boxddd wasm smoke passed: y {:.3} -> {:.3}, hits {}",
        start_y,
        end_y,
        hits.len()
    );
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn assert_wasm_thread_guardrails() -> boxddd::Result<()> {
    assert_eq!(
        TaskSystem::try_blocking_threads().unwrap_err(),
        Error::UnsupportedOnWasm
    );
    assert_eq!(
        World::new(WorldDef::builder().worker_count(2).build()).unwrap_err(),
        Error::UnsupportedOnWasm
    );

    let mut world = World::new(WorldDef::builder().worker_count(1).build())?;
    assert_eq!(
        world.try_set_worker_count(2).unwrap_err(),
        Error::UnsupportedOnWasm
    );
    assert_eq!(
        validate_replay_bytes(&[0], 2).unwrap_err(),
        Error::UnsupportedOnWasm
    );
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn assert_wasm_thread_guardrails() -> boxddd::Result<()> {
    Ok(())
}
