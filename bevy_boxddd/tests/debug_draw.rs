use bevy_app::App;
use bevy_boxddd::prelude::*;
use bevy_ecs::message::Messages;
use bevy_math::Vec3;
use bevy_time::{TimePlugin, TimeUpdateStrategy};
use bevy_transform::components::Transform;

fn physics_app(debug_settings: BoxdddDebugDrawSettings) -> App {
    let mut app = App::new();
    app.add_plugins(TimePlugin)
        .insert_resource(TimeUpdateStrategy::FixedTimesteps(1))
        .insert_resource(debug_settings)
        .add_plugins(BoxdddPhysicsPlugin::new(BoxdddPhysicsSettings::default()));
    app
}

fn run_fixed_frames(app: &mut App, count: usize) {
    for _ in 0..count {
        app.update();
    }
}

fn dynamic_body(app: &mut App, position: Vec3) -> bevy_ecs::entity::Entity {
    app.world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.3),
            Transform::from_translation(position),
        ))
        .id()
}

fn first_shape_handle(debug_frame: &BoxdddDebugDrawFrame) -> Option<boxddd::DebugShapeHandle> {
    debug_frame
        .commands()
        .iter()
        .find_map(|command| match command {
            boxddd::DebugDrawCommand::Shape {
                handle: Some(handle),
                ..
            } => Some(*handle),
            _ => None,
        })
}

#[test]
fn debug_draw_collects_shape_events_commands_and_cache_entries() {
    let mut options = boxddd::DebugDrawOptions::default();
    options.draw_joints = false;
    let mut app = physics_app(BoxdddDebugDrawSettings {
        enabled: true,
        options,
    });

    dynamic_body(&mut app, Vec3::new(0.0, 1.0, 0.0));
    run_fixed_frames(&mut app, 2);

    let debug_frame = app.world().resource::<BoxdddDebugDrawFrame>();
    let handle = first_shape_handle(debug_frame).expect("expected a shape draw command");
    assert!(
        debug_frame.events().iter().any(|event| matches!(
            event,
            boxddd::DebugShapeEvent::Created(asset)
                if asset.handle == handle
                    && matches!(asset.geometry, boxddd::DebugShapeGeometry::Sphere { .. })
        )),
        "expected first frame to emit a sphere asset create event: {:?}",
        debug_frame.frame()
    );
    assert!(
        debug_frame.asset(handle).is_some(),
        "created shape asset should be cached before rendering"
    );

    let compatibility_alias = app.world().resource::<BoxdddDebugDrawCommands>();
    assert_eq!(
        compatibility_alias.commands().len(),
        debug_frame.commands().len()
    );
}

#[test]
fn disabling_debug_draw_clears_previous_frame_but_keeps_cached_assets() {
    let mut app = physics_app(BoxdddDebugDrawSettings {
        enabled: true,
        options: boxddd::DebugDrawOptions::default(),
    });

    dynamic_body(&mut app, Vec3::new(0.0, 1.0, 0.0));
    run_fixed_frames(&mut app, 2);
    assert!(
        !app.world()
            .resource::<BoxdddDebugDrawCommands>()
            .commands()
            .is_empty()
    );
    assert!(
        !app.world()
            .resource::<BoxdddDebugDrawFrame>()
            .assets()
            .is_empty()
    );

    app.world_mut()
        .resource_mut::<BoxdddDebugDrawSettings>()
        .enabled = false;
    run_fixed_frames(&mut app, 1);

    assert!(
        app.world()
            .resource::<BoxdddDebugDrawCommands>()
            .commands()
            .is_empty()
    );
    assert!(
        app.world()
            .resource::<BoxdddDebugDrawFrame>()
            .events()
            .is_empty()
    );
    assert!(
        !app.world()
            .resource::<BoxdddDebugDrawFrame>()
            .assets()
            .is_empty(),
        "turning off collection should not pretend shapes were destroyed"
    );
}

#[test]
fn debug_draw_failure_clears_previous_commands() {
    let mut app = physics_app(BoxdddDebugDrawSettings {
        enabled: true,
        options: boxddd::DebugDrawOptions::default(),
    });

    dynamic_body(&mut app, Vec3::new(0.0, 1.0, 0.0));
    run_fixed_frames(&mut app, 2);
    assert!(
        !app.world()
            .resource::<BoxdddDebugDrawCommands>()
            .commands()
            .is_empty()
    );

    app.world_mut()
        .resource_mut::<BoxdddDebugDrawSettings>()
        .options
        .force_scale = f32::NAN;
    run_fixed_frames(&mut app, 1);

    assert!(
        app.world()
            .resource::<BoxdddDebugDrawCommands>()
            .commands()
            .is_empty()
    );
    assert!(
        app.world()
            .resource::<BoxdddDebugDrawFrame>()
            .events()
            .is_empty()
    );

    let messages = app
        .world_mut()
        .resource_mut::<Messages<BoxdddErrorMessage>>()
        .drain()
        .collect::<Vec<_>>();
    assert!(messages.iter().any(|message| {
        message.operation == BoxdddOperation::DebugDraw
            && message.error == boxddd::Error::InvalidArgument
    }));
}

#[test]
fn debug_draw_destroy_events_remove_cached_assets() {
    let mut app = physics_app(BoxdddDebugDrawSettings {
        enabled: true,
        options: boxddd::DebugDrawOptions::default(),
    });

    let entity = dynamic_body(&mut app, Vec3::new(0.0, 1.0, 0.0));
    run_fixed_frames(&mut app, 2);
    let handle = first_shape_handle(app.world().resource::<BoxdddDebugDrawFrame>())
        .expect("expected shape handle before despawn");
    assert!(
        app.world()
            .resource::<BoxdddDebugDrawFrame>()
            .asset(handle)
            .is_some()
    );

    app.world_mut().entity_mut(entity).despawn();
    run_fixed_frames(&mut app, 1);

    let debug_frame = app.world().resource::<BoxdddDebugDrawFrame>();
    assert!(
        debug_frame.events().iter().any(|event| matches!(
            event,
            boxddd::DebugShapeEvent::Destroyed { handle: destroyed } if *destroyed == handle
        )),
        "expected destroy event for despawned shape: {:?}",
        debug_frame.frame()
    );
    assert!(
        debug_frame.asset(handle).is_none(),
        "destroy events should remove cached assets before rendering"
    );
    assert!(
        !debug_frame.commands().iter().any(|command| matches!(
            command,
            boxddd::DebugDrawCommand::Shape { handle: Some(active), .. } if *active == handle
        )),
        "current frame should not draw retired handles"
    );
}

#[test]
fn debug_draw_reports_missing_cached_shape_assets() {
    let mut app = physics_app(BoxdddDebugDrawSettings {
        enabled: true,
        options: boxddd::DebugDrawOptions::default(),
    });

    dynamic_body(&mut app, Vec3::new(0.0, 1.0, 0.0));
    run_fixed_frames(&mut app, 2);
    let handle = first_shape_handle(app.world().resource::<BoxdddDebugDrawFrame>())
        .expect("expected shape handle before cache clear");

    app.world_mut()
        .resource_mut::<BoxdddDebugDrawFrame>()
        .clear_cached_assets();
    run_fixed_frames(&mut app, 1);

    let debug_frame = app.world().resource::<BoxdddDebugDrawFrame>();
    assert!(debug_frame.asset(handle).is_none());
    assert!(
        debug_frame.diagnostics().iter().any(|diagnostic| {
            diagnostic.message.contains("debug shape asset missing")
                && diagnostic
                    .message
                    .contains(&format!("{}:{}", handle.index, handle.generation))
        }),
        "expected missing asset diagnostic: {:?}",
        debug_frame.frame()
    );
}

#[test]
fn debug_draw_collects_joint_commands_when_enabled() {
    let mut options = boxddd::DebugDrawOptions::default();
    options.draw_shapes = false;
    options.draw_joints = true;
    let mut app = physics_app(BoxdddDebugDrawSettings {
        enabled: true,
        options,
    });

    let body_a = dynamic_body(&mut app, Vec3::new(-0.5, 1.0, 0.0));
    let body_b = dynamic_body(&mut app, Vec3::new(0.5, 1.0, 0.0));
    app.world_mut()
        .spawn((JointTarget::new(body_a, body_b), Joint::distance(1.0)));

    run_fixed_frames(&mut app, 2);

    let commands = app.world().resource::<BoxdddDebugDrawCommands>();
    assert!(!commands.commands().is_empty());
    assert!(
        commands.commands().iter().any(|command| matches!(
            command,
            boxddd::DebugDrawCommand::Segment { .. }
                | boxddd::DebugDrawCommand::Point { .. }
                | boxddd::DebugDrawCommand::Sphere { .. }
        )),
        "expected joint debug drawing to emit visible primitives: {:?}",
        commands.commands()
    );
}
