use bevy_app::App;
use bevy_boxddd::prelude::*;
use bevy_math::Vec3;
use bevy_time::{TimePlugin, TimeUpdateStrategy};
use bevy_transform::components::Transform;

fn physics_app(settings: BoxdddPhysicsSettings) -> App {
    let mut app = App::new();
    app.add_plugins(TimePlugin)
        .insert_resource(TimeUpdateStrategy::FixedTimesteps(1))
        .add_plugins(BoxdddPhysicsPlugin::new(settings));
    app
}

#[test]
fn fixed_step_moves_dynamic_body_and_syncs_transform() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());

    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(10.0, 0.5, 10.0),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));
    let falling = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            Transform::from_xyz(0.0, 4.0, 0.0),
        ))
        .id();

    for _ in 0..8 {
        app.update();
    }

    let transform = app
        .world()
        .entity(falling)
        .get::<Transform>()
        .expect("falling body should still have a Transform");
    assert!(transform.translation.y < 4.0);
}

#[test]
fn static_body_uses_bevy_authored_transform() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let ground = app
        .world_mut()
        .spawn((
            RigidBody::Static,
            Collider::cuboid(10.0, 0.5, 10.0),
            Transform::from_xyz(0.0, -1.25, 0.0),
        ))
        .id();

    for _ in 0..4 {
        app.update();
    }

    let body = app.world().entity(ground).get::<BoxdddBody>().unwrap().id();
    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();
    let position = context.world().unwrap().body_position(body);

    assert_eq!(position, boxddd::Pos::new(0.0, -1.25, 0.0));
}

#[test]
fn linear_velocity_component_controls_body_before_sync() {
    let settings = BoxdddPhysicsSettings {
        gravity: Vec3::ZERO,
        ..Default::default()
    };
    let mut app = physics_app(settings);
    let entity = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            LinearVelocity(Vec3::new(2.0, 0.0, 0.0)),
            Transform::from_xyz(0.0, 1.0, 0.0),
        ))
        .id();

    for _ in 0..4 {
        app.update();
    }

    let transform = app.world().entity(entity).get::<Transform>().unwrap();
    assert!(transform.translation.x > 0.0);
}

#[test]
fn external_impulse_is_applied_once_and_then_removed() {
    let settings = BoxdddPhysicsSettings {
        gravity: Vec3::ZERO,
        ..Default::default()
    };
    let mut app = physics_app(settings);
    let entity = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            ExternalImpulse::at_center(Vec3::new(6.0, 0.0, 0.0)),
            Transform::from_xyz(0.0, 1.0, 0.0),
        ))
        .id();

    for _ in 0..4 {
        app.update();
    }

    let transform = app.world().entity(entity).get::<Transform>().unwrap();
    assert!(transform.translation.x > 0.0);
    assert!(
        app.world()
            .entity(entity)
            .get::<ExternalImpulse>()
            .is_none()
    );
}
