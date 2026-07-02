use boxddd::{BodyDef, BodyType, BoxHull, ShapeDef, World, WorldDef};

#[test]
fn dropping_world_invalidates_body_and_shape_ids() {
    let body;
    let shape;
    {
        let mut world = World::new(WorldDef::default()).unwrap();
        body = world.create_body(
            BodyDef::builder()
                .body_type(BodyType::Dynamic)
                .position([0.0, 1.0, 0.0])
                .build(),
        );
        shape = world.create_hull_shape(
            body,
            &ShapeDef::builder().density(1.0).build(),
            &BoxHull::cube(0.5),
        );
        assert!(body.is_valid());
        assert!(shape.is_valid());
    }

    assert!(!body.is_valid());
    assert!(!shape.is_valid());
}
