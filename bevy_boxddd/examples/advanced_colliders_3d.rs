use bevy::prelude::*;
use bevy_boxddd::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "boxddd Bevy Advanced Colliders".into(),
                ..default()
            }),
            ..default()
        }))
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
    let accent_material = materials.add(Color::srgb(0.82, 0.46, 0.22));
    let body_material = materials.add(Color::srgb(0.24, 0.48, 0.85));

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
        MeshMaterial3d(accent_material.clone()),
        Transform::from_xyz(2.5, 0.65, 1.6),
        RigidBody::Static,
        Collider::compound_sphere(Vec3::ZERO, 0.65, boxddd::SurfaceMaterial::default()),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.55).mesh().uv(24, 12))),
        MeshMaterial3d(accent_material),
        Transform::from_xyz(4.5, 0.55, 1.6),
        RigidBody::Static,
        Collider::transformed_rock_hull(0.55, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
    ));

    let cube_mesh = meshes.add(Cuboid::new(0.7, 0.7, 0.7));
    for index in 0..8 {
        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(body_material.clone()),
            Transform::from_xyz(-2.5 + index as f32 * 0.6, 2.5 + index as f32 * 0.5, 0.0),
            RigidBody::Dynamic,
            Collider::cube(0.35),
        ));
    }
}
