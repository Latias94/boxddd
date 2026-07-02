use boxddd::{Aabb, BodyDef, BodyType, QueryFilter, ShapeDef, Sphere, World, WorldDef};

#[test]
fn query_buffers_are_cleared_and_capacity_is_reused() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::builder().body_type(BodyType::Static).build());
    world.create_sphere_shape(
        body,
        &ShapeDef::default(),
        &Sphere::new([0.0, 0.0, 0.0], 0.5),
    );

    let mut hits = Vec::with_capacity(8);
    hits.push(boxddd::QueryHit {
        shape_id: Default::default(),
    });
    let before_capacity = hits.capacity();
    let hit_aabb = Aabb {
        lower_bound: [-1.0, -1.0, -1.0].into(),
        upper_bound: [1.0, 1.0, 1.0].into(),
    };
    world
        .overlap_aabb_into(hit_aabb, QueryFilter::default(), &mut hits)
        .unwrap();
    assert_eq!(hits.capacity(), before_capacity);
    assert_eq!(hits.len(), 1);

    let miss_aabb = Aabb {
        lower_bound: [10.0, 10.0, 10.0].into(),
        upper_bound: [11.0, 11.0, 11.0].into(),
    };
    world
        .overlap_aabb_into(miss_aabb, QueryFilter::default(), &mut hits)
        .unwrap();
    assert_eq!(hits.capacity(), before_capacity);
    assert!(hits.is_empty());
}
