use boxddd::{
    Aabb, BodyDef, BodyType, Compound, Error, QueryFilter, ShapeDef, Sphere, SurfaceMaterial,
    World, WorldDef,
};

#[test]
fn query_callback_panic_is_caught_before_crossing_ffi_boundary() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::builder().body_type(BodyType::Static).build());
    world.create_sphere_shape(
        body,
        &ShapeDef::default(),
        &Sphere::new([0.0, 0.0, 0.0], 0.5),
    );
    let aabb = Aabb {
        lower_bound: [-1.0, -1.0, -1.0].into(),
        upper_bound: [1.0, 1.0, 1.0].into(),
    };

    let err = world
        .visit_overlap_aabb(aabb, QueryFilter::default(), |_| panic!("visitor panic"))
        .unwrap_err();
    assert_eq!(err, Error::CallbackPanicked);
}

#[test]
fn compound_query_callback_panic_is_caught_before_crossing_ffi_boundary() {
    let compound = Compound::single_sphere(
        Sphere::new([0.0, 0.0, 0.0], 0.5),
        SurfaceMaterial::default(),
    )
    .unwrap();
    let aabb = Aabb {
        lower_bound: [-1.0, -1.0, -1.0].into(),
        upper_bound: [1.0, 1.0, 1.0].into(),
    };

    let err = compound
        .visit_query_aabb(aabb, |_| panic!("visitor panic"))
        .unwrap_err();
    assert_eq!(err, Error::CallbackPanicked);
}
