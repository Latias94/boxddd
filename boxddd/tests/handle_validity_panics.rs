use boxddd::{BodyDef, BodyType, World, WorldDef};

#[test]
#[should_panic(expected = "invalid BodyId")]
fn stale_body_id_panics_at_safe_boundary() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([0.0, 2.0, 0.0])
            .build(),
    );

    world.destroy_body(body);
    let _ = world.body_position(body);
}
