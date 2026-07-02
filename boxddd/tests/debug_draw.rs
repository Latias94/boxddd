use boxddd::{
    Aabb, BodyDef, BoxHull, DebugDraw, DebugDrawCommand, DebugDrawOptions, Error, ShapeDef, World,
    WorldDef,
};

fn debug_world() -> World {
    let mut world = World::new(WorldDef::default()).unwrap();
    let body = world.create_body(BodyDef::default());
    world.create_hull_shape(body, &ShapeDef::default(), &BoxHull::cube(1.0));
    world
}

#[test]
fn debug_draw_collects_shape_commands_and_reuses_buffer() {
    let mut world = debug_world();
    let mut commands = vec![DebugDrawCommand::Transform(Default::default())];
    let initial_capacity = commands.capacity();

    world.debug_draw_collect_into(&mut commands, DebugDrawOptions::default());

    assert!(commands.capacity() >= initial_capacity);
    assert!(
        commands
            .iter()
            .any(|command| matches!(command, DebugDrawCommand::Shape { shape: Some(_), .. }))
    );

    let first_len = commands.len();
    world.debug_draw_collect_into(&mut commands, DebugDrawOptions::default());
    assert_eq!(commands.len(), first_len);
}

#[test]
fn debug_draw_options_can_collect_bounds_commands() {
    let mut world = debug_world();
    let mut options = DebugDrawOptions::default();
    options.draw_bounds = true;

    let commands = world.debug_draw_collect(options);

    assert!(
        commands
            .iter()
            .any(|command| matches!(command, DebugDrawCommand::Bounds { .. }))
    );
}

#[test]
fn debug_draw_callback_panic_is_reported_without_crossing_ffi() {
    struct PanickingDrawer;

    impl DebugDraw for PanickingDrawer {
        fn draw_shape(
            &mut self,
            _shape: Option<boxddd::DebugShape>,
            _transform: boxddd::WorldTransform,
            _color: boxddd::HexColor,
        ) -> bool {
            panic!("debug draw panic");
        }
    }

    let mut world = debug_world();
    let result = world.try_debug_draw(&mut PanickingDrawer, DebugDrawOptions::default());

    assert_eq!(result, Err(Error::CallbackPanicked));
}

#[test]
fn debug_draw_respects_callback_guard() {
    let mut world = debug_world();
    let _guard = boxddd::__private::enter_callback_guard_for_test();

    assert_eq!(
        world
            .try_debug_draw_collect(DebugDrawOptions::default())
            .unwrap_err(),
        Error::InCallback
    );
}

#[test]
fn debug_draw_rejects_invalid_bounds() {
    let mut world = debug_world();
    let mut options = DebugDrawOptions::default();
    options.drawing_bounds = Aabb {
        lower_bound: [2.0, 0.0, 0.0].into(),
        upper_bound: [1.0, 1.0, 1.0].into(),
    };

    assert_eq!(
        world.try_debug_draw_collect(options).unwrap_err(),
        Error::InvalidArgument
    );
}
