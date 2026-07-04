#[path = "testbed_3d/scenes.rs"]
#[allow(dead_code)]
mod scenes;
#[path = "support/mod.rs"]
mod support;

use bevy::prelude::*;
use bevy_boxddd::prelude::*;
use scenes::{TestbedScene, spawn_scene};

fn main() {
    App::new()
        .add_plugins(support::teaching_default_plugins(
            "boxddd Bevy Advanced Colliders",
        ))
        .add_plugins(BoxdddPhysicsPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-8.0, 6.0, 11.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 5_000_000.0,
            range: 70.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-3.0, 8.0, 5.0),
    ));

    spawn_scene(
        &mut commands,
        &mut meshes,
        &mut materials,
        TestbedScene::AdvancedColliders,
    );
}
