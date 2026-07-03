#[path = "../examples/testbed_3d/scenes.rs"]
mod scenes;

use bevy::prelude::*;
use bevy_boxddd::prelude::*;
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

fn run_fixed_frames(app: &mut App, count: usize) {
    for _ in 0..count {
        app.update();
    }
}

#[test]
fn every_testbed_scene_factory_spawns_without_panic() {
    for scene in ALL_SCENES {
        let mut app = physics_app(scene);
        app.add_systems(Update, spawn_selected_scene);
        app.update();
    }
}

#[test]
fn despawning_testbed_scene_releases_native_body_ids() {
    let mut app = physics_app(TestbedScene::FallingStack);
    app.add_systems(Update, spawn_selected_scene);
    app.update();
    run_fixed_frames(&mut app, 2);

    let body_ids = {
        let mut query = app
            .world_mut()
            .query_filtered::<&BoxdddBody, With<TestbedEntity>>();
        query
            .iter(app.world())
            .map(|body| body.id())
            .collect::<Vec<_>>()
    };
    assert!(!body_ids.is_empty());

    let entities = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<TestbedEntity>>();
        query.iter(app.world()).collect::<Vec<_>>()
    };
    for entity in entities {
        app.world_mut().entity_mut(entity).despawn();
    }

    run_fixed_frames(&mut app, 2);

    for body_id in body_ids {
        assert!(!body_id.is_valid());
    }
}
