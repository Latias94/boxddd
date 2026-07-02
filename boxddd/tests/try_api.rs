use boxddd::{BodyDef, BodyType, Error, ShapeDef, Sphere, Vec3, World, WorldDef};
use static_assertions::assert_not_impl_any;

assert_not_impl_any!(World: Send, Sync);

#[test]
fn invalid_world_definition_returns_error() {
    let def = WorldDef::builder().gravity([f32::NAN, 0.0, 0.0]).build();
    assert_eq!(World::new(def).unwrap_err(), Error::InvalidArgument);
}

#[test]
fn callback_guard_blocks_try_apis() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let _guard = boxddd::__private::enter_callback_guard_for_test();

    assert_eq!(
        world.try_set_gravity(Vec3::ZERO).unwrap_err(),
        Error::InCallback
    );
    assert_eq!(world.try_counters().unwrap_err(), Error::InCallback);
}

#[test]
fn stale_body_id_returns_typed_error() {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([0.0, 2.0, 0.0])
            .build(),
    );

    world.destroy_body(body);

    assert_eq!(
        world.try_body_position(body).unwrap_err(),
        Error::InvalidBodyId
    );
}

#[test]
fn invalid_create_inputs_return_invalid_argument() {
    let mut world = World::new(WorldDef::default()).unwrap();

    let bad_body = BodyDef::builder()
        .position([f32::INFINITY, 0.0, 0.0])
        .build();
    assert_eq!(world.try_create_body(bad_body), Err(Error::InvalidArgument));

    let body = world.create_body(BodyDef::default());
    let shape_def = ShapeDef::default();
    let bad_sphere = Sphere::new([0.0, 0.0, 0.0], f32::NAN);
    assert_eq!(
        world.try_create_sphere_shape(body, &shape_def, &bad_sphere),
        Err(Error::InvalidArgument)
    );
}
