use boxddd::{Aabb, BodyDef, BodyType, BoxHull, QueryFilter, ShapeDef, Vec3, World, WorldDef};

fn main() -> boxddd::Result<()> {
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
