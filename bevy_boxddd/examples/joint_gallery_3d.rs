#[path = "support/mod.rs"]
mod support;

use bevy::color::palettes::css::{GOLD, LIME, TOMATO};
use bevy::prelude::*;
use bevy_boxddd::prelude::*;

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

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(14.0, 0.4, 10.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.19, 0.22, 0.23))),
        Transform::from_xyz(0.0, -0.2, 0.0),
        RigidBody::Static,
        Collider::cuboid(7.0, 0.2, 5.0),
    ));

    let anchor_mesh = meshes.add(Sphere::new(0.18).mesh().uv(24, 12));
    let body_mesh = meshes.add(Cuboid::new(0.6, 0.6, 0.6));
    let anchor_material = materials.add(Color::srgb(0.95, 0.76, 0.22));
    let body_material = materials.add(Color::srgb(0.22, 0.48, 0.88));
    let accent_material = materials.add(Color::srgb(0.90, 0.33, 0.24));

    spawn_pair(
        &mut commands,
        anchor_mesh.clone(),
        body_mesh.clone(),
        anchor_material.clone(),
        body_material.clone(),
        Vec3::new(-4.5, 3.2, 0.0),
        Vec3::new(-3.4, 3.2, 0.0),
        Joint::distance(1.1),
    );

    spawn_pair(
        &mut commands,
        anchor_mesh.clone(),
        body_mesh.clone(),
        anchor_material.clone(),
        accent_material.clone(),
        Vec3::new(-1.5, 3.4, 0.0),
        Vec3::new(-0.5, 3.4, 0.0),
        Joint::revolute(),
    );

    spawn_pair(
        &mut commands,
        anchor_mesh.clone(),
        body_mesh.clone(),
        anchor_material.clone(),
        body_material.clone(),
        Vec3::new(1.4, 3.6, 0.0),
        Vec3::new(2.4, 3.6, 0.0),
        Joint::weld(),
    );

    spawn_pair(
        &mut commands,
        anchor_mesh,
        body_mesh,
        anchor_material,
        accent_material,
        Vec3::new(4.0, 3.8, 0.0),
        Vec3::new(5.0, 3.8, 0.0),
        Joint::spherical(),
    );
}

fn spawn_pair(
    commands: &mut Commands,
    anchor_mesh: Handle<Mesh>,
    body_mesh: Handle<Mesh>,
    anchor_material: Handle<StandardMaterial>,
    body_material: Handle<StandardMaterial>,
    anchor_position: Vec3,
    body_position: Vec3,
    joint: Joint,
) {
    let anchor = commands
        .spawn((
            Mesh3d(anchor_mesh),
            MeshMaterial3d(anchor_material),
            Transform::from_translation(anchor_position),
            RigidBody::Static,
        ))
        .id();

    let body = commands
        .spawn((
            Mesh3d(body_mesh),
            MeshMaterial3d(body_material),
            Transform::from_translation(body_position),
            RigidBody::Dynamic,
            Collider::cube(0.3),
            PhysicsMaterial {
                restitution: 0.1,
                ..default()
            },
        ))
        .id();

    commands.spawn((JointTarget::new(anchor, body), joint));
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
