#[path = "../examples/testbed_3d/scenes.rs"]
mod scenes;

use bevy::prelude::*;
use bevy_boxddd::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use bevy_time::{TimePlugin, TimeUpdateStrategy};
use scenes::{ALL_SCENES, TestbedEntity, TestbedScene, spawn_scene};

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

fn despawn_testbed_entities(app: &mut App) {
    for entity in testbed_entities(app) {
        app.world_mut().entity_mut(entity).despawn();
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
