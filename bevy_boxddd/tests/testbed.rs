#[path = "../examples/testbed_3d/scenes.rs"]
mod scenes;

use bevy::prelude::*;
use bevy_boxddd::prelude::*;
use bevy_ecs::message::Messages;
use bevy_ecs::system::RunSystemOnce;
use bevy_time::{TimePlugin, TimeUpdateStrategy};
use scenes::{ALL_SCENES, MoverProbe, SCENE_REGISTRY, TestbedEntity, TestbedScene, spawn_scene};

#[derive(Resource)]
struct SelectedScene(TestbedScene);

fn physics_app(scene: TestbedScene) -> App {
    let mut app = App::new();
    app.add_plugins(TimePlugin)
        .insert_resource(TimeUpdateStrategy::FixedTimesteps(1))
        .insert_resource(SelectedScene(scene))
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .add_plugins(BoxdddPhysicsPlugin::new(BoxdddPhysicsSettings::default()));
    app
}

fn spawn_selected_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scene: Res<SelectedScene>,
) {
    spawn_scene(&mut commands, &mut meshes, &mut materials, scene.0);
}

fn spawn_scene_once(app: &mut App, scene: TestbedScene) {
    app.insert_resource(SelectedScene(scene));
    app.world_mut()
        .run_system_once(spawn_selected_scene)
        .unwrap();
}

fn run_fixed_frames(app: &mut App, count: usize) {
    for _ in 0..count {
        app.update();
    }
}

fn testbed_entities(app: &mut App) -> Vec<Entity> {
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<TestbedEntity>>();
    query.iter(app.world()).collect()
}

fn testbed_body_ids(app: &mut App) -> Vec<boxddd::BodyId> {
    let mut query = app
        .world_mut()
        .query_filtered::<&BoxdddBody, With<TestbedEntity>>();
    query.iter(app.world()).map(|body| body.id()).collect()
}

fn testbed_shape_ids(app: &mut App) -> Vec<boxddd::ShapeId> {
    let mut query = app
        .world_mut()
        .query_filtered::<&BoxdddShape, With<TestbedEntity>>();
    query.iter(app.world()).map(|shape| shape.id()).collect()
}

fn testbed_joint_types(app: &mut App) -> Vec<boxddd::JointType> {
    let mut query = app
        .world_mut()
        .query_filtered::<&BoxdddJoint, With<TestbedEntity>>();
    let joint_ids = query
        .iter(app.world())
        .map(|joint| joint.id())
        .collect::<Vec<_>>();
    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();
    let world = context.world().unwrap();
    joint_ids
        .into_iter()
        .map(|joint_id| world.try_joint_type(joint_id).unwrap())
        .collect()
}

fn despawn_testbed_entities(app: &mut App) {
    for entity in testbed_entities(app) {
        app.world_mut().entity_mut(entity).despawn();
    }
}

fn to_boxddd_vec3(value: Vec3) -> boxddd::Vec3 {
    boxddd::Vec3::new(value.x, value.y, value.z)
}

fn to_boxddd_pos(value: Vec3) -> boxddd::Pos {
    boxddd::Pos::new(value.x.into(), value.y.into(), value.z.into())
}

#[test]
fn testbed_scene_registry_has_complete_unique_metadata() {
    assert_eq!(SCENE_REGISTRY.len(), ALL_SCENES.len());

    for (index, metadata) in SCENE_REGISTRY.iter().enumerate() {
        assert_eq!(metadata.scene, ALL_SCENES[index]);
        assert_eq!(metadata.scene.metadata().id, metadata.id);
        assert!(!metadata.id.is_empty());
        assert!(!metadata.category.is_empty());
        assert!(!metadata.name.is_empty());
        assert!(!metadata.description.is_empty());
        assert!(
            metadata.id.is_ascii() && !metadata.id.contains(' '),
            "scene id should be an ASCII slug: {}",
            metadata.id
        );
        assert_ne!(
            metadata.camera.position, metadata.camera.target,
            "camera position and target should differ for {}",
            metadata.id
        );
        assert!(
            metadata
                .camera
                .position
                .iter()
                .chain(metadata.camera.target.iter())
                .all(|value| value.is_finite()),
            "camera values should be finite for {}",
            metadata.id
        );
        assert!(
            metadata.camera.transform().translation.is_finite(),
            "camera transform should be finite for {}",
            metadata.id
        );
    }

    for (left_index, left) in SCENE_REGISTRY.iter().enumerate() {
        for right in SCENE_REGISTRY.iter().skip(left_index + 1) {
            assert_ne!(left.id, right.id, "duplicate scene id {}", left.id);
            assert_ne!(left.scene, right.scene, "duplicate scene {:?}", left.scene);
        }
    }
}

#[test]
fn every_testbed_scene_factory_spawns_without_panic() {
    for scene in ALL_SCENES {
        let mut app = physics_app(scene);
        spawn_scene_once(&mut app, scene);
    }
}

#[test]
fn every_testbed_scene_creates_native_shapes_after_fixed_updates() {
    for scene in ALL_SCENES {
        let mut app = physics_app(scene);
        spawn_scene_once(&mut app, scene);
        run_fixed_frames(&mut app, 3);

        let body_ids = testbed_body_ids(&mut app);
        let shape_ids = testbed_shape_ids(&mut app);
        assert!(
            body_ids.iter().any(|body_id| body_id.is_valid()),
            "{scene:?} should create at least one native body"
        );
        assert!(
            shape_ids.iter().any(|shape_id| shape_id.is_valid()),
            "{scene:?} should create at least one native shape"
        );
    }
}

#[test]
fn advanced_collider_scene_separates_static_resources_from_dynamic_bodies() {
    let mut app = physics_app(TestbedScene::AdvancedColliders);
    spawn_scene_once(&mut app, TestbedScene::AdvancedColliders);

    let mut resources = app
        .world_mut()
        .query_filtered::<(&RigidBody, &Collider), With<TestbedEntity>>();
    let static_resource_colliders = resources
        .iter(app.world())
        .filter(|(body, collider)| **body == RigidBody::Static && collider.requires_static_body())
        .count();
    assert!(static_resource_colliders >= 3);

    let mut dynamic_query = app
        .world_mut()
        .query_filtered::<(Entity, &RigidBody, &Transform), With<TestbedEntity>>();
    let initial_dynamic_y = dynamic_query
        .iter(app.world())
        .filter(|(_, body, _)| **body == RigidBody::Dynamic)
        .map(|(entity, _, transform)| (entity, transform.translation.y))
        .collect::<Vec<_>>();
    assert!(initial_dynamic_y.len() >= 6);

    run_fixed_frames(&mut app, 12);

    let mut transform_query = app.world_mut().query::<&Transform>();
    assert!(initial_dynamic_y.iter().any(|(entity, initial_y)| {
        transform_query
            .get(app.world(), *entity)
            .is_ok_and(|transform| transform.translation.y < *initial_y)
    }));
}

#[test]
fn materials_scene_contains_friction_and_restitution_variants() {
    let mut app = physics_app(TestbedScene::Materials);
    spawn_scene_once(&mut app, TestbedScene::Materials);

    let mut query = app
        .world_mut()
        .query_filtered::<&PhysicsMaterial, With<TestbedEntity>>();
    let materials = query.iter(app.world()).copied().collect::<Vec<_>>();

    assert!(materials.iter().any(|material| material.friction <= 0.05));
    assert!(materials.iter().any(|material| material.friction >= 1.0));
    assert!(materials.iter().any(|material| material.restitution >= 0.8));
}

#[test]
fn body_controls_scene_applies_body_settings_and_controls() {
    let mut app = physics_app(TestbedScene::BodyControls);
    spawn_scene_once(&mut app, TestbedScene::BodyControls);
    run_fixed_frames(&mut app, 3);

    let mut query = app
        .world_mut()
        .query_filtered::<(Entity, &BodySettings, Option<&ExternalForce>), With<TestbedEntity>>();
    let controlled = query
        .iter(app.world())
        .map(|(entity, settings, force)| (entity, *settings, force.is_some()))
        .collect::<Vec<_>>();

    assert!(
        controlled
            .iter()
            .any(|(_, settings, has_force)| settings.motion_locks.linear_z && *has_force)
    );
    assert!(
        controlled
            .iter()
            .any(|(_, settings, _)| (settings.gravity_scale - 0.25).abs() < f32::EPSILON)
    );

    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();
    let world = context.world().unwrap();
    for (entity, settings, _) in controlled {
        let body_id = app.world().entity(entity).get::<BoxdddBody>().unwrap().id();
        assert_eq!(
            world.try_body_motion_locks(body_id).unwrap(),
            settings.motion_locks
        );
        assert_eq!(
            world.try_body_gravity_scale(body_id).unwrap(),
            settings.gravity_scale
        );
    }
}

#[test]
fn continuous_collision_scene_creates_bullet_bodies() {
    let mut app = physics_app(TestbedScene::ContinuousCollision);
    spawn_scene_once(&mut app, TestbedScene::ContinuousCollision);
    run_fixed_frames(&mut app, 3);

    let mut query = app
        .world_mut()
        .query_filtered::<(Entity, &BodySettings), With<TestbedEntity>>();
    let bullet_entities = query
        .iter(app.world())
        .filter_map(|(entity, settings)| settings.bullet.then_some(entity))
        .collect::<Vec<_>>();
    assert!(bullet_entities.len() >= 3);

    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();
    let world = context.world().unwrap();
    for entity in bullet_entities {
        let body_id = app.world().entity(entity).get::<BoxdddBody>().unwrap().id();
        assert!(world.try_body_bullet(body_id).unwrap());
        assert_eq!(world.try_body_gravity_scale(body_id).unwrap(), 0.0);
    }
}

#[test]
fn character_mover_scene_probe_hits_obstacle() {
    let mut app = physics_app(TestbedScene::CharacterMover);
    spawn_scene_once(&mut app, TestbedScene::CharacterMover);
    run_fixed_frames(&mut app, 3);

    let mut query = app
        .world_mut()
        .query_filtered::<&MoverProbe, With<TestbedEntity>>();
    let probe = *query
        .iter(app.world())
        .next()
        .expect("character mover scene should contain a probe");

    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();
    let world = context.world().unwrap();
    let mover = boxddd::Capsule::new(
        to_boxddd_vec3(probe.point1),
        to_boxddd_vec3(probe.point2),
        probe.radius,
    );
    let fraction = world
        .cast_mover(
            to_boxddd_pos(probe.origin),
            &mover,
            to_boxddd_vec3(probe.delta),
            boxddd::QueryFilter::default(),
        )
        .unwrap();

    assert!(
        (0.0..1.0).contains(&fraction),
        "expected character mover probe to stop before an obstacle, got {fraction}"
    );
}

#[test]
fn joint_scene_creates_every_public_joint_variant() {
    let mut app = physics_app(TestbedScene::Joints);
    spawn_scene_once(&mut app, TestbedScene::Joints);
    run_fixed_frames(&mut app, 3);

    let joint_types = testbed_joint_types(&mut app);
    for expected_type in [
        boxddd::JointType::Distance,
        boxddd::JointType::Revolute,
        boxddd::JointType::Spherical,
        boxddd::JointType::Weld,
        boxddd::JointType::Prismatic,
        boxddd::JointType::Wheel,
    ] {
        assert!(
            joint_types.contains(&expected_type),
            "missing {expected_type:?}; got {joint_types:?}"
        );
    }
    assert_eq!(joint_types.len(), 6);
}

#[test]
fn contact_scene_emits_physics_messages() {
    let mut app = physics_app(TestbedScene::Contacts);
    spawn_scene_once(&mut app, TestbedScene::Contacts);

    let mut saw_contact = false;
    let mut saw_sensor = false;
    for _ in 0..180 {
        app.update();
        let contacts = app
            .world_mut()
            .resource_mut::<Messages<BoxdddContactBeginMessage>>()
            .drain()
            .collect::<Vec<_>>();
        let sensors = app
            .world_mut()
            .resource_mut::<Messages<BoxdddSensorBeginMessage>>()
            .drain()
            .collect::<Vec<_>>();
        saw_contact |= contacts.iter().any(|message| message.contact_id.is_valid());
        saw_sensor |= sensors
            .iter()
            .any(|message| message.sensor_shape.is_valid() && message.visitor_shape.is_valid());

        if saw_contact && saw_sensor {
            break;
        }
    }

    assert!(saw_contact || saw_sensor);
}

#[test]
fn despawning_testbed_scene_releases_native_body_ids() {
    let mut app = physics_app(TestbedScene::FallingStack);
    spawn_scene_once(&mut app, TestbedScene::FallingStack);
    run_fixed_frames(&mut app, 2);

    let body_ids = testbed_body_ids(&mut app);
    assert!(!body_ids.is_empty());

    despawn_testbed_entities(&mut app);
    run_fixed_frames(&mut app, 2);

    for body_id in body_ids {
        assert!(!body_id.is_valid());
    }
}

#[test]
fn switching_testbed_scenes_releases_old_ids_and_creates_new_scene() {
    let mut app = physics_app(TestbedScene::FallingStack);
    spawn_scene_once(&mut app, TestbedScene::FallingStack);
    run_fixed_frames(&mut app, 2);

    let old_body_ids = testbed_body_ids(&mut app);
    let old_shape_ids = testbed_shape_ids(&mut app);
    assert!(!old_body_ids.is_empty());
    assert!(!old_shape_ids.is_empty());

    despawn_testbed_entities(&mut app);
    spawn_scene_once(&mut app, TestbedScene::Joints);
    run_fixed_frames(&mut app, 3);

    for body_id in old_body_ids {
        assert!(!body_id.is_valid());
    }
    for shape_id in old_shape_ids {
        assert!(!shape_id.is_valid());
    }

    let new_body_ids = testbed_body_ids(&mut app);
    assert!(new_body_ids.iter().any(|body_id| body_id.is_valid()));

    let mut joints = app
        .world_mut()
        .query_filtered::<&BoxdddJoint, With<TestbedEntity>>();
    assert!(joints.iter(app.world()).any(|joint| joint.id().is_valid()));
}
