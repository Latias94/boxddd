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

fn dynamic_body(app: &mut App, position: Vec3) -> bevy_ecs::entity::Entity {
    app.world_mut()
        .spawn((RigidBody::Dynamic, Transform::from_translation(position)))
        .id()
}

#[test]
fn distance_joint_between_dynamic_bodies_creates_native_joint() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let body_a = dynamic_body(&mut app, Vec3::new(-0.5, 0.0, 0.0));
    let body_b = dynamic_body(&mut app, Vec3::new(0.5, 0.0, 0.0));
    let joint_entity = app
        .world_mut()
        .spawn((JointTarget::new(body_a, body_b), Joint::distance(1.0)))
        .id();

    run_fixed_frames(&mut app, 2);

    let joint_id = app
        .world()
        .entity(joint_entity)
        .get::<BoxdddJoint>()
        .unwrap()
        .id();
    let body_a_id = app.world().entity(body_a).get::<BoxdddBody>().unwrap().id();
    let body_b_id = app.world().entity(body_b).get::<BoxdddBody>().unwrap().id();
    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();
    let world = context.world().unwrap();

    assert!(joint_id.is_valid());
    assert_eq!(context.joint_entity(joint_id), Some(joint_entity));
    assert_eq!(
        world.try_joint_type(joint_id).unwrap(),
        boxddd::JointType::Distance
    );
    assert_eq!(world.try_joint_body_a(joint_id).unwrap(), body_a_id);
    assert_eq!(world.try_joint_body_b(joint_id).unwrap(), body_b_id);
}

#[test]
fn removing_joint_component_destroys_native_joint() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let body_a = dynamic_body(&mut app, Vec3::new(-0.5, 0.0, 0.0));
    let body_b = dynamic_body(&mut app, Vec3::new(0.5, 0.0, 0.0));
    let joint_entity = app
        .world_mut()
        .spawn((JointTarget::new(body_a, body_b), Joint::distance(1.0)))
        .id();

    run_fixed_frames(&mut app, 2);
    let joint_id = app
        .world()
        .entity(joint_entity)
        .get::<BoxdddJoint>()
        .unwrap()
        .id();

    app.world_mut().entity_mut(joint_entity).remove::<Joint>();
    run_fixed_frames(&mut app, 2);

    assert!(!joint_id.is_valid());
    assert!(
        app.world()
            .entity(joint_entity)
            .get::<BoxdddJoint>()
            .is_none()
    );
}

#[test]
fn despawning_body_destroys_attached_joint_before_body_teardown() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let body_a = dynamic_body(&mut app, Vec3::new(-0.5, 0.0, 0.0));
    let body_b = dynamic_body(&mut app, Vec3::new(0.5, 0.0, 0.0));
    let joint_entity = app
        .world_mut()
        .spawn((JointTarget::new(body_a, body_b), Joint::distance(1.0)))
        .id();

    run_fixed_frames(&mut app, 2);
    let joint_id = app
        .world()
        .entity(joint_entity)
        .get::<BoxdddJoint>()
        .unwrap()
        .id();
    let body_a_id = app.world().entity(body_a).get::<BoxdddBody>().unwrap().id();

    app.world_mut().entity_mut(body_a).despawn();
    run_fixed_frames(&mut app, 2);

    assert!(!joint_id.is_valid());
    assert!(!body_a_id.is_valid());
    assert!(
        app.world()
            .entity(joint_entity)
            .get::<BoxdddJoint>()
            .is_none()
    );
}

#[test]
fn invalid_joint_endpoint_emits_error_without_joint_id() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let body_a = dynamic_body(&mut app, Vec3::new(-0.5, 0.0, 0.0));
    let missing_body = app.world_mut().spawn_empty().id();
    let joint_entity = app
        .world_mut()
        .spawn((JointTarget::new(body_a, missing_body), Joint::distance(1.0)))
        .id();

    run_fixed_frames(&mut app, 2);

    assert!(
        app.world()
            .entity(joint_entity)
            .get::<BoxdddJoint>()
            .is_none()
    );

    let messages = app
        .world_mut()
        .resource_mut::<Messages<BoxdddErrorMessage>>()
        .drain()
        .collect::<Vec<_>>();

    assert!(messages.iter().any(|message| {
        message.operation == BoxdddOperation::CreateJoint
            && message.entity == Some(joint_entity)
            && message.error == boxddd::Error::InvalidBodyId
    }));
}

#[test]
fn changing_joint_target_recreates_native_joint() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let body_a = dynamic_body(&mut app, Vec3::new(-0.5, 0.0, 0.0));
    let body_b = dynamic_body(&mut app, Vec3::new(0.5, 0.0, 0.0));
    let body_c = dynamic_body(&mut app, Vec3::new(1.5, 0.0, 0.0));
    let joint_entity = app
        .world_mut()
        .spawn((JointTarget::new(body_a, body_b), Joint::distance(1.0)))
        .id();

    run_fixed_frames(&mut app, 2);
    let old_joint = app
        .world()
        .entity(joint_entity)
        .get::<BoxdddJoint>()
        .unwrap()
        .id();
    let body_c_id = app.world().entity(body_c).get::<BoxdddBody>().unwrap().id();

    app.world_mut()
        .entity_mut(joint_entity)
        .insert(JointTarget::new(body_a, body_c));
    run_fixed_frames(&mut app, 3);

    let new_joint = app
        .world()
        .entity(joint_entity)
        .get::<BoxdddJoint>()
        .unwrap()
        .id();
    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();
    let world = context.world().unwrap();

    assert!(!old_joint.is_valid());
    assert!(new_joint.is_valid());
    assert_ne!(old_joint, new_joint);
    assert_eq!(world.try_joint_body_b(new_joint).unwrap(), body_c_id);
}
