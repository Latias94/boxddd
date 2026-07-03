use bevy::prelude::*;
use bevy_boxddd::prelude::*;

#[derive(Component, Copy, Clone, Debug, Eq, PartialEq)]
pub struct TestbedEntity;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TestbedScene {
    FallingStack,
    CompoundTerrain,
    Joints,
    Contacts,
    RayPicking,
    DebugDraw,
}

pub const ALL_SCENES: [TestbedScene; 6] = [
    TestbedScene::FallingStack,
    TestbedScene::CompoundTerrain,
    TestbedScene::Joints,
    TestbedScene::Contacts,
    TestbedScene::RayPicking,
    TestbedScene::DebugDraw,
];

pub fn spawn_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    scene: TestbedScene,
) {
    spawn_ground(commands, meshes, materials);

    match scene {
        TestbedScene::FallingStack => spawn_falling_stack(commands, meshes, materials),
        TestbedScene::CompoundTerrain => spawn_compound_terrain(commands, meshes, materials),
        TestbedScene::Joints => spawn_joints(commands, meshes, materials),
        TestbedScene::Contacts => spawn_contacts(commands, meshes, materials),
        TestbedScene::RayPicking => spawn_ray_picking(commands, meshes, materials),
        TestbedScene::DebugDraw => spawn_debug_draw(commands, meshes, materials),
    }
}

fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(16.0, 0.4, 12.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.18, 0.22, 0.22))),
        Transform::from_xyz(0.0, -0.2, 0.0),
        RigidBody::Static,
        Collider::cuboid(8.0, 0.2, 6.0),
        TestbedEntity,
    ));
}

fn spawn_falling_stack(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mesh = meshes.add(Cuboid::new(0.75, 0.75, 0.75));
    let material = materials.add(Color::srgb(0.22, 0.48, 0.88));
    for layer in 0..5 {
        for column in 0..4 {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(column as f32 * 0.9 - 1.35, 0.55 + layer as f32 * 0.82, 0.0),
                RigidBody::Dynamic,
                Collider::cube(0.375),
                TestbedEntity,
            ));
        }
    }
}

fn spawn_compound_terrain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    commands.spawn((
        Transform::from_xyz(-2.5, 0.0, 0.0),
        RigidBody::Static,
        Collider::height_field_grid(5, 5, Vec3::new(0.6, 0.25, 0.6), false),
        TestbedEntity,
    ));
    commands.spawn((
        Transform::from_xyz(1.8, 0.25, 0.0),
        RigidBody::Static,
        Collider::compound_sphere(Vec3::ZERO, 0.65, boxddd::SurfaceMaterial::default()),
        TestbedEntity,
    ));

    let sphere_mesh = meshes.add(Sphere::new(0.35).mesh().uv(24, 12));
    let material = materials.add(Color::srgb(0.94, 0.58, 0.22));
    for index in 0..5 {
        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(index as f32 * 0.7 - 1.4, 3.0 + index as f32 * 0.25, 0.0),
            RigidBody::Dynamic,
            Collider::sphere(0.35),
            TestbedEntity,
        ));
    }
}

fn spawn_joints(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let body_mesh = meshes.add(Cuboid::new(0.65, 0.65, 0.65));
    let body_material = materials.add(Color::srgb(0.25, 0.50, 0.95));
    for index in 0..4 {
        let x = index as f32 * 1.6 - 2.4;
        let anchor = commands
            .spawn((
                Transform::from_xyz(x, 3.0, 0.0),
                RigidBody::Static,
                TestbedEntity,
            ))
            .id();
        let body = commands
            .spawn((
                Mesh3d(body_mesh.clone()),
                MeshMaterial3d(body_material.clone()),
                Transform::from_xyz(x + 0.8, 3.0, 0.0),
                RigidBody::Dynamic,
                Collider::cube(0.325),
                TestbedEntity,
            ))
            .id();
        let joint = match index {
            0 => Joint::distance(0.8),
            1 => Joint::revolute(),
            2 => Joint::weld(),
            _ => Joint::spherical(),
        };
        commands.spawn((JointTarget::new(anchor, body), joint, TestbedEntity));
    }
}

fn spawn_contacts(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    commands.spawn((
        Transform::from_xyz(0.0, 0.6, 0.0),
        RigidBody::Static,
        Collider::cuboid(2.0, 0.15, 2.0),
        PhysicsMaterial {
            is_sensor: true,
            enable_sensor_events: true,
            ..Default::default()
        },
        TestbedEntity,
    ));

    let mesh = meshes.add(Sphere::new(0.3).mesh().uv(24, 12));
    let material = materials.add(Color::srgb(0.34, 0.72, 0.36));
    for index in 0..6 {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(index as f32 * 0.55 - 1.4, 2.8 + index as f32 * 0.2, 0.0),
            RigidBody::Dynamic,
            Collider::sphere(0.3),
            PhysicsMaterial {
                enable_contact_events: true,
                enable_hit_events: true,
                ..Default::default()
            },
            TestbedEntity,
        ));
    }
}

fn spawn_ray_picking(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let cube_mesh = meshes.add(Cuboid::new(0.8, 0.8, 0.8));
    let sphere_mesh = meshes.add(Sphere::new(0.45).mesh().uv(24, 12));
    let blue = materials.add(Color::srgb(0.22, 0.46, 0.88));
    let orange = materials.add(Color::srgb(0.92, 0.55, 0.22));

    commands.spawn((
        Mesh3d(cube_mesh),
        MeshMaterial3d(blue),
        Transform::from_xyz(-1.0, 1.0, 0.0),
        RigidBody::Static,
        Collider::cube(0.4),
        TestbedEntity,
    ));
    commands.spawn((
        Mesh3d(sphere_mesh),
        MeshMaterial3d(orange),
        Transform::from_xyz(1.0, 1.1, 0.0),
        RigidBody::Static,
        Collider::sphere(0.45),
        TestbedEntity,
    ));
}

fn spawn_debug_draw(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_joints(commands, meshes, materials);
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.4).mesh().uv(24, 12))),
        MeshMaterial3d(materials.add(Color::srgb(0.92, 0.50, 0.25))),
        Transform::from_xyz(2.6, 3.6, 0.0),
        RigidBody::Dynamic,
        Collider::sphere(0.4),
        TestbedEntity,
    ));
}
