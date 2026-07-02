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

#[test]
fn contact_messages_include_boxddd_ids_and_bevy_entities() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let ground = app
        .world_mut()
        .spawn((
            RigidBody::Static,
            Collider::cuboid(10.0, 0.5, 10.0),
            Transform::from_xyz(0.0, -0.5, 0.0),
        ))
        .id();
    let sphere = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            PhysicsMaterial {
                enable_contact_events: true,
                enable_hit_events: true,
                ..Default::default()
            },
            Transform::from_xyz(0.0, 4.0, 0.0),
        ))
        .id();

    let mut seen_contact = false;
    for _ in 0..180 {
        app.update();
        let contacts = app
            .world_mut()
            .resource_mut::<Messages<BoxdddContactBeginMessage>>()
            .drain()
            .collect::<Vec<_>>();
        seen_contact |= contacts.iter().any(|message| {
            [message.entity_a, message.entity_b].contains(&Some(ground))
                && [message.entity_a, message.entity_b].contains(&Some(sphere))
                && message.contact_id.is_valid()
        });
        if seen_contact {
            break;
        }
    }

    assert!(seen_contact);
}

#[test]
fn sensor_messages_include_shape_entity_mapping() {
    let settings = BoxdddPhysicsSettings {
        gravity: Vec3::ZERO,
        ..Default::default()
    };
    let mut app = physics_app(settings);
    let wall = app
        .world_mut()
        .spawn((
            RigidBody::Static,
            Collider::cuboid(0.5, 10.0, 1.0),
            PhysicsMaterial {
                enable_sensor_events: true,
                ..Default::default()
            },
            Transform::from_xyz(1.5, 0.0, 0.0),
        ))
        .id();
    let bullet = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.1),
            PhysicsMaterial {
                is_sensor: true,
                enable_sensor_events: true,
                ..Default::default()
            },
            LinearVelocity(Vec3::new(-20.0, 0.0, 0.0)),
            Transform::from_xyz(7.39814, 0.0, 0.0),
        ))
        .id();

    let mut seen_begin = false;
    for _ in 0..120 {
        app.update();
        let messages = app
            .world_mut()
            .resource_mut::<Messages<BoxdddSensorBeginMessage>>()
            .drain()
            .collect::<Vec<_>>();
        seen_begin |= messages.iter().any(|message| {
            [message.sensor_entity, message.visitor_entity].contains(&Some(wall))
                && [message.sensor_entity, message.visitor_entity].contains(&Some(bullet))
        });
        if seen_begin {
            break;
        }
    }

    assert!(seen_begin);
}
