#[path = "support/mod.rs"]
mod support;

use bevy::prelude::*;
use bevy_boxddd::prelude::*;

fn main() {
    App::new()
        .add_plugins(support::teaching_default_plugins(
            "boxddd Bevy Falling Stack",
        ))
        .add_plugins(BoxdddPhysicsPlugin::new(BoxdddPhysicsSettings::default()))
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
        Transform::from_xyz(-7.5, 6.0, 11.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 6_000_000.0,
            range: 80.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-4.0, 10.0, 6.0),
    ));

    let ground_mesh = meshes.add(Cuboid::new(16.0, 0.5, 16.0));
    let ground_material = materials.add(Color::srgb(0.22, 0.26, 0.24));
    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_material),
        Transform::from_xyz(0.0, -0.25, 0.0),
        RigidBody::Static,
        Collider::cuboid(8.0, 0.25, 8.0),
    ));

    let cube_mesh = meshes.add(Cuboid::new(0.9, 0.9, 0.9));
    let sphere_mesh = meshes.add(Sphere::new(0.45).mesh().uv(32, 18));
    let cube_material = materials.add(Color::srgb(0.20, 0.48, 0.90));
    let sphere_material = materials.add(Color::srgb(0.95, 0.63, 0.25));

    for layer in 0..5 {
        for column in 0..4 {
            let x = column as f32 * 1.15 - 1.725;
            let z = if layer % 2 == 0 { -0.35 } else { 0.35 };
            let y = 0.75 + layer as f32 * 1.05;
            let is_sphere = (layer + column) % 3 == 0;

            if is_sphere {
                commands.spawn((
                    Mesh3d(sphere_mesh.clone()),
                    MeshMaterial3d(sphere_material.clone()),
                    Transform::from_xyz(x, y, z),
                    RigidBody::Dynamic,
                    Collider::sphere(0.45),
                    PhysicsMaterial {
                        restitution: 0.15,
                        ..default()
                    },
                ));
            } else {
                commands.spawn((
                    Mesh3d(cube_mesh.clone()),
                    MeshMaterial3d(cube_material.clone()),
                    Transform::from_xyz(x, y, z)
                        .with_rotation(Quat::from_rotation_y(0.08 * (layer + column) as f32)),
                    RigidBody::Dynamic,
                    Collider::cube(0.45),
                    PhysicsMaterial {
                        restitution: 0.05,
                        ..default()
                    },
                ));
            }
        }
    }
}
