use boxddd::{
    BodyDef, BodyType, BoxHull, DistanceInput, DistanceJointDef, Quat, QueryFilter,
    ShapeCastPairInput, ShapeDef, ShapeProxy, Sphere, Transform, Vec3, World, WorldDef,
    shape_cast_pair, shape_distance,
};

#[cfg(target_arch = "wasm32")]
use boxddd::{
    Aabb, BoxCastInput, Compound, DebugDrawOptions, DynamicTree, DynamicTreeCastControl,
    DynamicTreeFilter, Error, HeightField, MeshData, RayCastInput, SurfaceMaterial, TaskSystem,
    validate_replay_bytes,
};

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
const ERR_COLLISION: i32 = -9;
const ERR_JOINT: i32 = -10;

#[unsafe(no_mangle)]
pub extern "C" fn boxddd_provider_smoke() -> i32 {
    match run_smoke() {
        Ok(()) => OK,
        Err(code) => code,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn boxddd_provider_drop_millimeters() -> i32 {
    run_drop_millimeters().unwrap_or_else(|code| code)
}

#[unsafe(no_mangle)]
pub extern "C" fn boxddd_provider_ray_hit_millimeters() -> i32 {
    run_ray_hit_millimeters().unwrap_or_else(|code| code)
}

#[unsafe(no_mangle)]
pub extern "C" fn boxddd_provider_shape_cast_permyriad() -> i32 {
    run_shape_cast_permyriad().unwrap_or_else(|code| code)
}

#[unsafe(no_mangle)]
pub extern "C" fn boxddd_provider_joint_error_millimeters() -> i32 {
    run_joint_error_millimeters().unwrap_or_else(|code| code)
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

    run_ray_hit_millimeters()?;
    run_shape_cast_permyriad()?;
    run_joint_error_millimeters()?;

    assert_provider_callback_guardrails(&mut world)?;

    Ok(())
}

fn run_drop_millimeters() -> Result<i32, i32> {
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

    Ok(((start_y - end_y).max(0.0) * 1000.0) as i32)
}

fn run_ray_hit_millimeters() -> Result<i32, i32> {
    let mut world =
        World::new(WorldDef::builder().worker_count(1).build()).map_err(|_| ERR_WORLD)?;
    let body = world.create_body(BodyDef::builder().position(Vec3::ZERO).build());
    let sphere = world.create_sphere_shape(
        body,
        &ShapeDef::builder().density(1.0).build(),
        &Sphere::new([-1.0, 0.0, 0.0], 0.5),
    );
    if !sphere.is_valid() {
        return Err(ERR_SHAPE);
    }
    let cube = world.create_hull_shape(
        body,
        &ShapeDef::builder().density(1.0).build(),
        &BoxHull::cube(0.4),
    );
    if !cube.is_valid() {
        return Err(ERR_SHAPE);
    }

    let hit = world
        .cast_ray_closest([-3.0, 0.0, 0.0], [5.0, 0.0, 0.0], QueryFilter::default())
        .map_err(|_| ERR_QUERY)?
        .ok_or(ERR_QUERY)?;
    if !hit.fraction.is_finite() || !(0.0..=1.0).contains(&hit.fraction) {
        return Err(ERR_QUERY);
    }

    Ok((hit.fraction * 5000.0).round() as i32)
}

fn run_shape_cast_permyriad() -> Result<i32, i32> {
    let sphere_a = ShapeProxy::sphere(0.5).map_err(|_| ERR_COLLISION)?;
    let sphere_b = ShapeProxy::sphere(0.5).map_err(|_| ERR_COLLISION)?;

    let distance = shape_distance(
        DistanceInput::new(
            sphere_a.clone(),
            sphere_b.clone(),
            Transform::new(Vec3::new(1.4, 0.0, 0.0), Quat::IDENTITY),
        )
        .map_err(|_| ERR_COLLISION)?,
    )
    .map_err(|_| ERR_COLLISION)?;
    if !distance.distance.is_finite() || !(0.35..=0.45).contains(&distance.distance) {
        return Err(ERR_COLLISION);
    }

    let cast = shape_cast_pair(
        ShapeCastPairInput::new(
            sphere_a,
            sphere_b,
            Transform::new(Vec3::new(3.0, 0.0, 0.0), Quat::IDENTITY),
            Vec3::new(-4.0, 0.0, 0.0),
        )
        .map_err(|_| ERR_COLLISION)?,
    )
    .map_err(|_| ERR_COLLISION)?;
    if !cast.hit || !cast.fraction.is_finite() || !(0.0..=1.0).contains(&cast.fraction) {
        return Err(ERR_COLLISION);
    }

    Ok((cast.fraction * 10_000.0).round() as i32)
}

fn run_joint_error_millimeters() -> Result<i32, i32> {
    let mut world = World::new(
        WorldDef::builder()
            .gravity(Vec3::ZERO)
            .worker_count(1)
            .build(),
    )
    .map_err(|_| ERR_WORLD)?;
    let anchor = world.create_body(BodyDef::builder().position([0.0, 0.0, 0.0]).build());
    let body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([1.0, 0.0, 0.0])
            .build(),
    );
    let shape = world.create_sphere_shape(
        body,
        &ShapeDef::builder().density(1.0).build(),
        &Sphere::new([0.0, 0.0, 0.0], 0.25),
    );
    if !shape.is_valid() {
        return Err(ERR_SHAPE);
    }

    let joint = world.create_distance_joint(DistanceJointDef::new(anchor, body).length(1.0));
    world
        .try_apply_force_to_center(body, [25.0, 0.0, 0.0], true)
        .map_err(|_| ERR_JOINT)?;
    for _ in 0..60 {
        world.try_step(1.0 / 60.0, 4).map_err(|_| ERR_STEP)?;
    }

    let length = world
        .try_distance_joint_current_length(joint)
        .map_err(|_| ERR_JOINT)?;
    if !length.is_finite() || !(0.5..=1.5).contains(&length) {
        return Err(ERR_JOINT);
    }

    Ok(((length - 1.0).abs() * 1000.0).round() as i32)
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
    let overlaps = world
        .overlap_aabb(aabb, QueryFilter::default())
        .map_err(|_| ERR_CALLBACK_GUARDRAIL)?;
    if overlaps.is_empty() {
        return Err(ERR_CALLBACK_GUARDRAIL);
    }
    let ray_hits = world
        .cast_ray(
            Vec3::new(0.0, 8.0, 0.0),
            Vec3::new(0.0, -16.0, 0.0),
            QueryFilter::default(),
        )
        .map_err(|_| ERR_CALLBACK_GUARDRAIL)?;
    if ray_hits.is_empty() {
        return Err(ERR_CALLBACK_GUARDRAIL);
    }
    let frame = world
        .try_debug_draw_frame(DebugDrawOptions::default())
        .map_err(|_| ERR_CALLBACK_GUARDRAIL)?;
    if !frame
        .events
        .iter()
        .any(|event| matches!(event, boxddd::DebugShapeEvent::Created(_)))
        || !frame.commands.iter().any(|command| {
            matches!(
                command,
                boxddd::DebugDrawCommand::Shape {
                    handle: Some(_),
                    ..
                }
            )
        })
    {
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

    let tree = DynamicTree::new().map_err(|_| ERR_CALLBACK_GUARDRAIL)?;
    if !is_unsupported_on_wasm(tree.query(aabb, DynamicTreeFilter::default())) {
        return Err(ERR_CALLBACK_GUARDRAIL);
    }
    if !is_unsupported_on_wasm(tree.query_closest(
        Vec3::ZERO,
        DynamicTreeFilter::default(),
        1.0,
        |_| 0.0,
    )) {
        return Err(ERR_CALLBACK_GUARDRAIL);
    }
    let ray = RayCastInput::new(Vec3::new(-1.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0))
        .map_err(|_| ERR_CALLBACK_GUARDRAIL)?;
    if !is_unsupported_on_wasm(tree.ray_cast(ray, DynamicTreeFilter::default(), |_| {
        DynamicTreeCastControl::Continue
    })) {
        return Err(ERR_CALLBACK_GUARDRAIL);
    }
    let box_cast =
        BoxCastInput::new(aabb, Vec3::new(1.0, 0.0, 0.0)).map_err(|_| ERR_CALLBACK_GUARDRAIL)?;
    if !is_unsupported_on_wasm(tree.box_cast(box_cast, DynamicTreeFilter::default(), |_| {
        DynamicTreeCastControl::Continue
    })) {
        return Err(ERR_CALLBACK_GUARDRAIL);
    }

    let unit_scale = Vec3::new(1.0, 1.0, 1.0);
    let mesh =
        MeshData::box_mesh(Vec3::ZERO, unit_scale, true).map_err(|_| ERR_CALLBACK_GUARDRAIL)?;
    if !is_unsupported_on_wasm(mesh.query_triangles(aabb, unit_scale)) {
        return Err(ERR_CALLBACK_GUARDRAIL);
    }

    let height_field =
        HeightField::grid(3, 3, unit_scale, false).map_err(|_| ERR_CALLBACK_GUARDRAIL)?;
    if !is_unsupported_on_wasm(height_field.query_triangles(aabb)) {
        return Err(ERR_CALLBACK_GUARDRAIL);
    }

    let compound =
        Compound::single_sphere(Sphere::new(Vec3::ZERO, 0.5), SurfaceMaterial::default())
            .map_err(|_| ERR_CALLBACK_GUARDRAIL)?;
    if !is_unsupported_on_wasm(compound.query_aabb(aabb)) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exported_metrics_cover_core_examples() {
        assert!(run_drop_millimeters().expect("drop metric") > 100);
        assert_eq!(run_ray_hit_millimeters().expect("ray metric"), 1500);
        let shape_cast = run_shape_cast_permyriad().expect("shape-cast metric");
        assert!((4000..=6000).contains(&shape_cast));
        let joint_error = run_joint_error_millimeters().expect("joint metric");
        assert!((0..=500).contains(&joint_error));
    }
}
