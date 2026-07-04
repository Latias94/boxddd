#[path = "testbed_3d/scenes.rs"]
#[allow(dead_code)]
mod scenes;
#[path = "support/mod.rs"]
mod support;

use bevy::color::palettes::css::{GOLD, LIME, TOMATO};
use bevy::prelude::*;
use bevy_boxddd::prelude::*;
use scenes::{TestbedScene, spawn_scene};

fn main() {
    App::new()
        .add_plugins(support::teaching_default_plugins(
            "boxddd Bevy Joint Gallery",
        ))
        .add_plugins(BoxdddPhysicsPlugin::new(BoxdddPhysicsSettings {
            gravity: Vec3::new(0.0, -6.0, 0.0),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, draw_joint_lines)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-8.0, 5.5, 11.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 6_000_000.0,
            range: 80.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-4.0, 9.0, 6.0),
    ));

    spawn_scene(
        &mut commands,
        &mut meshes,
        &mut materials,
        TestbedScene::Joints,
    );
}

fn draw_joint_lines(
    mut gizmos: Gizmos,
    joints: Query<(&JointTarget, Option<&BoxdddJoint>)>,
    transforms: Query<&GlobalTransform>,
) {
    for (target, joint_id) in &joints {
        let Ok(body_a_transform) = transforms.get(target.body_a) else {
            continue;
        };
        let Ok(body_b_transform) = transforms.get(target.body_b) else {
            continue;
        };

        let color = if joint_id.is_some() { LIME } else { TOMATO };
        gizmos.line(
            body_a_transform.translation(),
            body_b_transform.translation(),
            color,
        );
        gizmos.sphere(body_a_transform.translation(), 0.07, GOLD);
        gizmos.sphere(body_b_transform.translation(), 0.07, color);
    }
}
