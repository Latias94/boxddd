use bevy_app::App;
use bevy_boxddd::prelude::*;
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

#[test]
fn debug_draw_collects_shape_commands_from_bevy_scene() {
    let mut options = boxddd::DebugDrawOptions::default();
    options.draw_joints = false;
    let mut app = physics_app(BoxdddDebugDrawSettings {
        enabled: true,
        options,
    });

    dynamic_body(&mut app, Vec3::new(0.0, 1.0, 0.0));
    run_fixed_frames(&mut app, 2);

    let commands = app.world().resource::<BoxdddDebugDrawCommands>();
    assert!(commands.commands().iter().any(|command| {
        matches!(
            command,
            boxddd::DebugDrawCommand::Shape { shape: Some(_), .. }
        )
    }));
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
