use boxddd::{
    Aabb, BodyDef, BodyType, BoxHull, Error, Filter, QueryFilter, ShapeDef, ShapeProxy, Sphere,
    Vec3, World, WorldDef,
};

fn query_world() -> (World, Vec<boxddd::ShapeId>) {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::builder().body_type(BodyType::Static).build());
    let left = world.create_sphere_shape(
        body,
        &ShapeDef::builder().density(1.0).build(),
        &Sphere::new([-1.0, 0.0, 0.0], 0.4),
    );
    let right = world.create_hull_shape(
        body,
        &ShapeDef::builder().density(1.0).build(),
        &BoxHull::cube(0.35),
    );
    (world, vec![left, right])
}

#[test]
fn overlap_aabb_owned_into_and_visitor_agree() {
    let (world, shapes) = query_world();
    let aabb = Aabb {
        lower_bound: [-2.0, -1.0, -1.0].into(),
        upper_bound: [1.0, 1.0, 1.0].into(),
    };

    let owned = world.overlap_aabb(aabb, QueryFilter::default()).unwrap();
    let mut into = Vec::from([boxddd::QueryHit {
        shape_id: Default::default(),
    }]);
    world
        .overlap_aabb_into(aabb, QueryFilter::default(), &mut into)
        .unwrap();
    assert_eq!(owned, into);
    assert!(owned.iter().any(|hit| hit.shape_id == shapes[0]));

    let mut visited = Vec::new();
    world
        .visit_overlap_aabb(aabb, QueryFilter::default(), |shape_id| {
            visited.push(shape_id);
            true
        })
        .unwrap();
    assert_eq!(owned.len(), visited.len());
}

#[test]
fn overlap_shape_and_ray_casts_find_expected_shapes() {
    let (world, shapes) = query_world();
    let proxy = ShapeProxy::sphere(0.5).unwrap();
    let hits = world
        .overlap_shape([0.0, 0.0, 0.0], &proxy, QueryFilter::default())
        .unwrap();
    assert!(hits.iter().any(|hit| hit.shape_id == shapes[1]));

    let all_ray_hits = world
        .cast_ray([-3.0, 0.0, 0.0], [5.0, 0.0, 0.0], QueryFilter::default())
        .unwrap();
    assert!(all_ray_hits.iter().any(|hit| hit.shape_id == shapes[0]));
    let closest = world
        .cast_ray_closest([-3.0, 0.0, 0.0], [5.0, 0.0, 0.0], QueryFilter::default())
        .unwrap()
        .unwrap();
    assert_eq!(closest.shape_id, shapes[0]);

    let shape_hits = world
        .cast_shape(
            [-3.0, 0.0, 0.0],
            boxddd::ShapeCastInput::new(proxy, [5.0, 0.0, 0.0]).unwrap(),
            QueryFilter::default(),
        )
        .unwrap();
    assert!(!shape_hits.is_empty());
}

#[test]
fn query_filter_excludes_shapes_by_mask() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::builder().body_type(BodyType::Static).build());
    let include = ShapeDef::builder()
        .filter(Filter {
            category_bits: 0b10,
            mask_bits: u64::MAX,
            group_index: 0,
        })
        .build();
    let shape = world.create_sphere_shape(body, &include, &Sphere::new(Vec3::ZERO, 0.5));
    let aabb = Aabb {
        lower_bound: [-1.0, -1.0, -1.0].into(),
        upper_bound: [1.0, 1.0, 1.0].into(),
    };

    let included = world
        .overlap_aabb(aabb, QueryFilter::default().mask_bits(0b10))
        .unwrap();
    assert!(included.iter().any(|hit| hit.shape_id == shape));
    let excluded = world
        .overlap_aabb(aabb, QueryFilter::default().mask_bits(0b100))
        .unwrap();
    assert!(excluded.is_empty());
}

#[test]
fn invalid_aabb_is_rejected_before_world_query() {
    let (world, _) = query_world();
    let inverted = Aabb {
        lower_bound: [1.0, 0.0, 0.0].into(),
        upper_bound: [-1.0, 0.0, 0.0].into(),
    };
    assert_eq!(inverted.validate(), Err(Error::InvalidArgument));
    assert_eq!(
        world
            .overlap_aabb(inverted, QueryFilter::default())
            .unwrap_err(),
        Error::InvalidArgument
    );

    let nan = Aabb {
        lower_bound: [f32::NAN, 0.0, 0.0].into(),
        upper_bound: [1.0, 1.0, 1.0].into(),
    };
    assert_eq!(
        world
            .visit_overlap_aabb(nan, QueryFilter::default(), |_| true)
            .unwrap_err(),
        Error::InvalidArgument
    );
}

#[test]
fn pure_global_and_query_defaults_are_callback_safe() {
    let _guard = boxddd::__private::enter_callback_guard_for_test();

    let filter = QueryFilter::default();
    assert_eq!(filter.category_bits, u64::MAX);
    assert_eq!(filter.mask_bits, u64::MAX);
    assert_eq!(filter.id, 0);
    let _ = boxddd::version();
    let _ = boxddd::is_double_precision();
}
