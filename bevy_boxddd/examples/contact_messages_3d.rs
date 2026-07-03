#[path = "support/mod.rs"]
mod support;

use bevy::prelude::*;
use bevy_boxddd::prelude::*;

#[derive(Component)]
struct ContactTint;

#[derive(Resource, Clone)]
struct ContactMaterials {
    idle: Handle<StandardMaterial>,
    active: Handle<StandardMaterial>,
}

fn main() {
    App::new()
        .add_plugins(support::teaching_default_plugins(
            "boxddd Bevy Contact Messages",
        ))
        .add_plugins(BoxdddPhysicsPlugin::new(BoxdddPhysicsSettings::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, highlight_contacts)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-6.0, 5.0, 9.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 5_000_000.0,
            range: 60.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 5.0),
    ));

    let idle = materials.add(Color::srgb(0.25, 0.50, 0.88));
    let active = materials.add(Color::srgb(1.00, 0.24, 0.16));
    commands.insert_resource(ContactMaterials {
        idle: idle.clone(),
        active: active.clone(),
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(12.0, 0.5, 12.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.22, 0.24, 0.26))),
        Transform::from_xyz(0.0, -0.25, 0.0),
        RigidBody::Static,
        Collider::cuboid(6.0, 0.25, 6.0),
        PhysicsMaterial {
            enable_contact_events: true,
            ..default()
        },
    ));

    let cube_mesh = meshes.add(Cuboid::new(0.8, 0.8, 0.8));
    for index in 0..8 {
        let x = (index % 4) as f32 * 1.1 - 1.65;
        let y = 1.0 + (index / 4) as f32 * 1.25;
        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(idle.clone()),
            Transform::from_xyz(x, y, 0.0),
            RigidBody::Dynamic,
            Collider::cube(0.4),
            PhysicsMaterial {
                enable_contact_events: true,
                enable_hit_events: true,
                restitution: 0.1,
                ..default()
            },
            ContactTint,
        ));
    }
}

fn highlight_contacts(
    mut begin_messages: MessageReader<BoxdddContactBeginMessage>,
    mut end_messages: MessageReader<BoxdddContactEndMessage>,
    contact_materials: Res<ContactMaterials>,
    mut materials: Query<&mut MeshMaterial3d<StandardMaterial>, With<ContactTint>>,
) {
    for message in end_messages.read() {
        for entity in [message.entity_a, message.entity_b].into_iter().flatten() {
            if let Ok(mut material) = materials.get_mut(entity) {
                material.0 = contact_materials.idle.clone();
            }
        }
    }

    for message in begin_messages.read() {
        for entity in [message.entity_a, message.entity_b].into_iter().flatten() {
            if let Ok(mut material) = materials.get_mut(entity) {
                material.0 = contact_materials.active.clone();
            }
        }
    }
}
