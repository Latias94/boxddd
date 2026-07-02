use bevy_app::App;
use bevy_boxddd::prelude::*;
use bevy_ecs::message::Messages;
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

fn run_fixed_frames(app: &mut App, count: usize) {
    for _ in 0..count {
        app.update();
    }
}

#[test]
fn plugin_inserts_non_send_context_and_uses_settings() {
    let settings = BoxdddPhysicsSettings {
        gravity: Vec3::new(0.0, -3.0, 0.0),
        ..Default::default()
    };
    let app = physics_app(settings);

    let context = app
        .world()
        .get_non_send::<BoxdddPhysicsContext>()
        .expect("plugin should insert the non-send physics context");
    let gravity = context
        .world()
        .expect("world should be initialized")
        .gravity();

    assert_eq!(gravity, boxddd::Vec3::new(0.0, -3.0, 0.0));
}

#[test]
fn invalid_world_settings_emit_error_message() {
    let settings = BoxdddPhysicsSettings {
        gravity: Vec3::new(f32::NAN, 0.0, 0.0),
        ..Default::default()
    };
    let mut app = physics_app(settings);

    let messages = app
        .world_mut()
        .resource_mut::<Messages<BoxdddErrorMessage>>()
        .drain()
        .collect::<Vec<_>>();
    assert!(messages.iter().any(|message| {
        message.operation == BoxdddOperation::CreateWorld
            && message.error == boxddd::Error::InvalidArgument
    }));

    let context = app
        .world()
        .get_non_send::<BoxdddPhysicsContext>()
        .expect("disabled context should still be inserted");
    assert!(context.world().is_none());
}

#[test]
fn invalid_fixed_timestep_settings_emit_error_message() {
    let settings = BoxdddPhysicsSettings {
        fixed_timestep_seconds: Some(0.0),
        ..Default::default()
    };
    let mut app = physics_app(settings);

    let messages = app
        .world_mut()
        .resource_mut::<Messages<BoxdddErrorMessage>>()
        .drain()
        .collect::<Vec<_>>();
    assert!(messages.iter().any(|message| {
        message.operation == BoxdddOperation::ConfigureFixedTimestep
            && message.error == boxddd::Error::InvalidArgument
    }));

    let context = app
        .world()
        .get_non_send::<BoxdddPhysicsContext>()
        .expect("valid world context should still be inserted");
    assert!(context.world().is_some());
}

#[test]
fn bodies_and_simple_colliders_are_created_and_cleaned_up() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let entity = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::capsule_y(0.5, 0.2),
            Transform::from_xyz(0.0, 2.0, 0.0),
        ))
        .id();

    run_fixed_frames(&mut app, 2);

    let body = *app
        .world()
        .entity(entity)
        .get::<BoxdddBody>()
        .expect("plugin should insert BoxdddBody");
    let shape = *app
        .world()
        .entity(entity)
        .get::<BoxdddShape>()
        .expect("plugin should insert BoxdddShape");
    assert!(body.id().is_valid());
    assert!(shape.id().is_valid());

    app.world_mut().entity_mut(entity).despawn();
    run_fixed_frames(&mut app, 2);

    assert!(!body.id().is_valid());
}

#[test]
fn removing_body_component_recreates_body_and_shape_without_stale_ids() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let entity = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            Transform::from_xyz(0.0, 2.0, 0.0),
        ))
        .id();

    run_fixed_frames(&mut app, 2);

    let old_body = *app.world().entity(entity).get::<BoxdddBody>().unwrap();
    let old_shape = *app.world().entity(entity).get::<BoxdddShape>().unwrap();

    app.world_mut().entity_mut(entity).remove::<BoxdddBody>();
    run_fixed_frames(&mut app, 2);

    let new_body = *app
        .world()
        .entity(entity)
        .get::<BoxdddBody>()
        .expect("body should be recreated when the authoring component remains");
    let new_shape = *app
        .world()
        .entity(entity)
        .get::<BoxdddShape>()
        .expect("shape should be recreated against the replacement body");

    assert!(!old_body.id().is_valid());
    assert!(!old_shape.id().is_valid());
    assert!(new_body.id().is_valid());
    assert!(new_shape.id().is_valid());
}

#[test]
fn removing_collider_component_destroys_shape_but_keeps_body() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let entity = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            Transform::from_xyz(0.0, 2.0, 0.0),
        ))
        .id();

    run_fixed_frames(&mut app, 2);

    let body = *app.world().entity(entity).get::<BoxdddBody>().unwrap();
    let shape = *app.world().entity(entity).get::<BoxdddShape>().unwrap();

    app.world_mut().entity_mut(entity).remove::<Collider>();
    run_fixed_frames(&mut app, 2);

    assert!(body.id().is_valid());
    assert!(!shape.id().is_valid());
    assert!(app.world().entity(entity).get::<BoxdddBody>().is_some());
    assert!(app.world().entity(entity).get::<BoxdddShape>().is_none());
}

#[test]
fn invalid_collider_dimensions_emit_error_message() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let entity = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(-1.0),
            Transform::from_xyz(0.0, 2.0, 0.0),
        ))
        .id();

    run_fixed_frames(&mut app, 2);

    assert!(app.world().entity(entity).get::<BoxdddBody>().is_some());
    assert!(app.world().entity(entity).get::<BoxdddShape>().is_none());

    let messages = app
        .world_mut()
        .resource_mut::<Messages<BoxdddErrorMessage>>()
        .drain()
        .collect::<Vec<_>>();
    assert!(messages.iter().any(|message| {
        message.operation == BoxdddOperation::CreateShape
            && message.entity == Some(entity)
            && message.error == boxddd::Error::InvalidArgument
    }));
}
