use bevy::prelude::*;
use bevy_boxddd::prelude::*;

fn main() {
    let mut options = boxddd::DebugDrawOptions::default();
    options.draw_joints = true;
    options.draw_bounds = true;

    App::new()
        .insert_resource(BoxdddDebugDrawSettings {
            enabled: true,
            options,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "boxddd Bevy Debug Draw Overlay".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BoxdddPhysicsPlugin::new(BoxdddPhysicsSettings::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, draw_debug_gizmos)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-5.5, 4.0, 8.0).looking_at(Vec3::new(0.0, 1.2, 0.0), Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 5_000_000.0,
            range: 60.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-4.0, 7.0, 5.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(10.0, 0.4, 8.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.18, 0.21, 0.22))),
        Transform::from_xyz(0.0, -0.2, 0.0),
        RigidBody::Static,
        Collider::cuboid(5.0, 0.2, 4.0),
    ));

    let anchor = commands
        .spawn((RigidBody::Static, Transform::from_xyz(-1.2, 2.8, 0.0)))
        .id();
    let body = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(0.7, 0.7, 0.7))),
            MeshMaterial3d(materials.add(Color::srgb(0.24, 0.48, 0.92))),
            Transform::from_xyz(0.0, 2.8, 0.0),
            RigidBody::Dynamic,
            Collider::cube(0.35),
            PhysicsMaterial {
                restitution: 0.1,
                ..default()
            },
        ))
        .id();
    commands.spawn((JointTarget::new(anchor, body), Joint::distance(1.2)));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.35).mesh().uv(32, 16))),
        MeshMaterial3d(materials.add(Color::srgb(0.92, 0.55, 0.22))),
        Transform::from_xyz(1.6, 3.4, 0.0),
        RigidBody::Dynamic,
        Collider::sphere(0.35),
        PhysicsMaterial {
            restitution: 0.25,
            ..default()
        },
    ));
}
