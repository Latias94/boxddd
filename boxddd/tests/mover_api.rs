use boxddd::{BodyDef, BodyType, BoxHull, Capsule, QueryFilter, ShapeDef, World, WorldDef};

#[test]
fn cast_mover_returns_fraction_for_simple_static_scene() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::builder().body_type(BodyType::Static).build());
    world.create_hull_shape(body, &ShapeDef::default(), &BoxHull::new(1.0, 0.5, 1.0));
    let mover = Capsule::new([0.0, 2.0, 0.0], [0.0, 3.0, 0.0], 0.25);
    let fraction = world
        .cast_mover(
            [0.0, 0.0, 0.0],
            &mover,
            [0.0, -4.0, 0.0],
            QueryFilter::default(),
        )
        .unwrap();
    assert!((0.0..=1.0).contains(&fraction));
}
