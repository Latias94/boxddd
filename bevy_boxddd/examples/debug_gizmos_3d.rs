use bevy::color::palettes::css::{LIME, ORANGE, TURQUOISE, WHITE};
use bevy::prelude::*;
use bevy_boxddd::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "boxddd Bevy Debug Gizmos".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BoxdddPhysicsPlugin::new(BoxdddPhysicsSettings::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, draw_collider_gizmos)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-7.0, 5.0, 10.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 5_000_000.0,
            range: 60.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 8.0, 4.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(14.0, 0.4, 14.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.20, 0.23, 0.22))),
        Transform::from_xyz(0.0, -0.2, 0.0),
        RigidBody::Static,
        Collider::cuboid(7.0, 0.2, 7.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.9, 0.9, 0.9))),
        MeshMaterial3d(materials.add(Color::srgb(0.24, 0.45, 0.85))),
        Transform::from_xyz(-1.4, 3.0, 0.0).with_rotation(Quat::from_rotation_z(0.35)),
        RigidBody::Dynamic,
        Collider::cube(0.45),
        PhysicsMaterial {
            restitution: 0.1,
            ..default()
        },
    ));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.45).mesh().uv(32, 18))),
        MeshMaterial3d(materials.add(Color::srgb(0.94, 0.58, 0.23))),
        Transform::from_xyz(0.0, 4.6, 0.0),
        RigidBody::Dynamic,
        Collider::sphere(0.45),
        PhysicsMaterial {
            restitution: 0.25,
            ..default()
        },
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.28, 0.9))),
        MeshMaterial3d(materials.add(Color::srgb(0.34, 0.68, 0.40))),
        Transform::from_xyz(1.5, 6.2, 0.0).with_rotation(Quat::from_rotation_x(0.4)),
        RigidBody::Dynamic,
        Collider::capsule_y(0.45, 0.28),
        PhysicsMaterial {
            restitution: 0.2,
            ..default()
        },
    ));
}

fn draw_collider_gizmos(mut gizmos: Gizmos, colliders: Query<(&Transform, &Collider)>) {
    for (transform, collider) in &colliders {
        match *collider {
            Collider::Cuboid { half_extents } => {
                gizmos.cube(
                    transform.with_scale(half_extents * 2.0),
                    if half_extents.y < 0.3 {
                        WHITE
                    } else {
                        TURQUOISE
                    },
                );
            }
            Collider::Sphere { radius, center } => {
                gizmos.sphere(transform.transform_point(center), radius, ORANGE);
            }
            Collider::Capsule {
                point1,
                point2,
                radius,
            } => {
                let start = transform.transform_point(point1);
                let end = transform.transform_point(point2);
                gizmos.line(start, end, LIME);
                gizmos.sphere(start, radius, LIME);
                gizmos.sphere(end, radius, LIME);
            }
            _ => {}
        }
    }
}
