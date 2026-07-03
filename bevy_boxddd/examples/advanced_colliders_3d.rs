#[path = "support/mod.rs"]
mod support;

use bevy::prelude::*;
use bevy_boxddd::prelude::*;

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

    let terrain_material = materials.add(Color::srgb(0.22, 0.28, 0.24));
    let static_obstacle_material = materials.add(Color::srgb(0.45, 0.43, 0.34));
    let cube_material = materials.add(Color::srgb(0.24, 0.48, 0.85));
    let sphere_material = materials.add(Color::srgb(0.92, 0.56, 0.22));
    let hull_material = materials.add(Color::srgb(0.36, 0.68, 0.42));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(8.0, 0.35, 8.0))),
        MeshMaterial3d(terrain_material.clone()),
        Transform::from_xyz(-4.0, -0.2, 0.0),
        RigidBody::Static,
        Collider::mesh_box(Vec3::ZERO, Vec3::new(4.0, 0.175, 4.0), Vec3::ONE, true),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.5, 0.25, 3.5))),
        MeshMaterial3d(terrain_material.clone()),
        Transform::from_xyz(2.5, -0.125, -1.5),
        RigidBody::Static,
        Collider::height_field_grid(5, 5, Vec3::new(1.0, 0.2, 1.0), false),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.65).mesh().uv(32, 18))),
        MeshMaterial3d(static_obstacle_material.clone()),
        Transform::from_xyz(2.5, 0.65, 1.6),
        RigidBody::Static,
        Collider::compound_sphere(Vec3::ZERO, 0.65, boxddd::SurfaceMaterial::default()),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.55).mesh().uv(24, 12))),
        MeshMaterial3d(hull_material),
        Transform::from_xyz(4.5, 4.2, 1.6),
        RigidBody::Dynamic,
        Collider::transformed_rock_hull(0.55, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
    ));

    let cube_mesh = meshes.add(Cuboid::new(0.7, 0.7, 0.7));
    let sphere_mesh = meshes.add(Sphere::new(0.35).mesh().uv(24, 12));

    for (index, position) in [
        Vec3::new(-5.6, 2.6, -0.8),
        Vec3::new(-4.8, 3.2, 0.4),
        Vec3::new(-4.0, 3.8, -0.2),
        Vec3::new(-3.2, 4.4, 0.8),
        Vec3::new(-2.4, 5.0, 0.0),
        Vec3::new(1.6, 2.9, -1.4),
        Vec3::new(3.2, 3.4, -1.5),
        Vec3::new(4.3, 3.9, -1.2),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(cube_material.clone()),
            Transform::from_translation(position)
                .with_rotation(Quat::from_rotation_y(0.18 * index as f32)),
            RigidBody::Dynamic,
            Collider::cube(0.35),
            PhysicsMaterial {
                restitution: 0.08,
                ..default()
            },
        ));
    }

    for position in [Vec3::new(2.5, 3.2, 1.6), Vec3::new(3.7, 4.4, 1.8)] {
        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(sphere_material.clone()),
            Transform::from_translation(position),
            RigidBody::Dynamic,
            Collider::sphere(0.35),
            PhysicsMaterial {
                restitution: 0.2,
                ..default()
            },
        ));
    }
}
