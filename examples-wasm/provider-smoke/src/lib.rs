use boxddd::{BodyDef, BodyType, BoxHull, QueryFilter, ShapeDef, Vec3, World, WorldDef};

#[cfg(target_arch = "wasm32")]
use boxddd::{Aabb, DebugDrawOptions, Error, TaskSystem, validate_replay_bytes};

const OK: i32 = 0;
const ERR_WORLD: i32 = -1;
const ERR_SHAPE: i32 = -2;
const ERR_STEP: i32 = -3;
#[cfg(target_arch = "wasm32")]
const ERR_GUARDRAIL: i32 = -5;
const ERR_MOTION: i32 = -6;
const ERR_QUERY: i32 = -7;
#[cfg(target_arch = "wasm32")]
const ERR_CALLBACK_GUARDRAIL: i32 = -8;

#[unsafe(no_mangle)]
pub extern "C" fn boxddd_provider_smoke() -> i32 {
    match run_smoke() {
        Ok(()) => OK,
        Err(code) => code,
    }
}

fn run_smoke() -> Result<(), i32> {
    assert_wasm_guardrails()?;

    let mut world = World::new(
        WorldDef::builder()
            .gravity(Vec3::new(0.0, -10.0, 0.0))
            .worker_count(1)
            .build(),
    )
    .map_err(|_| ERR_WORLD)?;

    let ground = world.create_body(BodyDef::builder().position([0.0, -1.0, 0.0]).build());
    world.create_hull_shape(ground, &ShapeDef::default(), &BoxHull::new(8.0, 0.5, 8.0));

    let body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([0.0, 4.0, 0.0])
            .build(),
    );
    let shape = world.create_hull_shape(
        body,
        &ShapeDef::builder().density(1.0).friction(0.3).build(),
        &BoxHull::cube(0.5),
    );
    if !shape.is_valid() {
        return Err(ERR_SHAPE);
    }

    let start_y = world.body_position(body).y;
    for _ in 0..60 {
        world.try_step(1.0 / 60.0, 4).map_err(|_| ERR_STEP)?;
    }
    let end_y = world.body_position(body).y;
    if end_y >= start_y - 0.1 {
        return Err(ERR_MOTION);
    }

    let closest = world
        .cast_ray_closest([0.0, 8.0, 0.0], [0.0, -16.0, 0.0], QueryFilter::default())
        .map_err(|_| ERR_QUERY)?;
    if closest.is_none() {
        return Err(ERR_QUERY);
    }

    assert_provider_callback_guardrails(&mut world)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn assert_wasm_guardrails() -> Result<(), i32> {
    if !is_unsupported_on_wasm(TaskSystem::try_blocking_threads()) {
        return Err(ERR_GUARDRAIL);
    }
    if !is_unsupported_on_wasm(World::new(WorldDef::builder().worker_count(2).build())) {
        return Err(ERR_GUARDRAIL);
    }

    let mut world =
        World::new(WorldDef::builder().worker_count(1).build()).map_err(|_| ERR_GUARDRAIL)?;
    if !is_unsupported_on_wasm(world.try_set_worker_count(2)) {
        return Err(ERR_GUARDRAIL);
    }
    if !is_unsupported_on_wasm(validate_replay_bytes(&[0], 2)) {
        return Err(ERR_GUARDRAIL);
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn assert_wasm_guardrails() -> Result<(), i32> {
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn assert_provider_callback_guardrails(world: &mut World) -> Result<(), i32> {
    let aabb = Aabb {
        lower_bound: Vec3::new(-2.0, -2.0, -2.0),
        upper_bound: Vec3::new(2.0, 5.0, 2.0),
    };
    if !is_unsupported_on_wasm(world.overlap_aabb(aabb, QueryFilter::default())) {
        return Err(ERR_CALLBACK_GUARDRAIL);
    }
    if !is_unsupported_on_wasm(world.try_debug_draw_collect(DebugDrawOptions::default())) {
        return Err(ERR_CALLBACK_GUARDRAIL);
    }
    if !is_unsupported_on_wasm(world.try_set_custom_filter(|_, _| true)) {
        return Err(ERR_CALLBACK_GUARDRAIL);
    }
    if !is_unsupported_on_wasm(world.try_set_pre_solve(|_, _, _, _| true)) {
        return Err(ERR_CALLBACK_GUARDRAIL);
    }
    if !is_unsupported_on_wasm(
        world.try_set_friction_callback(|a, b| (a.coefficient * b.coefficient).sqrt()),
    ) {
        return Err(ERR_CALLBACK_GUARDRAIL);
    }
    if !is_unsupported_on_wasm(
        world.try_set_restitution_callback(|a, b| a.coefficient.max(b.coefficient)),
    ) {
        return Err(ERR_CALLBACK_GUARDRAIL);
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn assert_provider_callback_guardrails(_world: &mut World) -> Result<(), i32> {
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn is_unsupported_on_wasm<T>(result: boxddd::Result<T>) -> bool {
    matches!(result, Err(Error::UnsupportedOnWasm))
}
