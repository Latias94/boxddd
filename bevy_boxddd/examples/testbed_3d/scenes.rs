use bevy::prelude::*;
use bevy_boxddd::prelude::*;

#[derive(Component, Copy, Clone, Debug, Eq, PartialEq)]
pub struct TestbedEntity;

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct MoverProbe {
    pub origin: Vec3,
    pub point1: Vec3,
    pub point2: Vec3,
    pub radius: f32,
    pub delta: Vec3,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TestbedScene {
    FallingStack,
    AdvancedColliders,
    BodyControls,
    ContinuousCollision,
    CharacterMover,
    Materials,
    Joints,
    Contacts,
    RayPicking,
    DebugDraw,
}

pub const ALL_SCENES: [TestbedScene; 10] = [
    TestbedScene::FallingStack,
    TestbedScene::AdvancedColliders,
    TestbedScene::BodyControls,
    TestbedScene::ContinuousCollision,
    TestbedScene::CharacterMover,
    TestbedScene::Materials,
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
        TestbedScene::AdvancedColliders => spawn_advanced_colliders(commands, meshes, materials),
        TestbedScene::BodyControls => spawn_body_controls(commands, meshes, materials),
        TestbedScene::ContinuousCollision => {
            spawn_continuous_collision(commands, meshes, materials)
        }
        TestbedScene::CharacterMover => spawn_character_mover(commands, meshes, materials),
        TestbedScene::Materials => spawn_materials(commands, meshes, materials),
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

fn spawn_advanced_colliders(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mesh_platform = materials.add(Color::srgb(0.35, 0.38, 0.36));
    let height_field = materials.add(Color::srgb(0.34, 0.43, 0.34));
    let compound = materials.add(Color::srgb(0.45, 0.39, 0.28));
    let dynamic_cube = materials.add(Color::srgb(0.24, 0.48, 0.85));
    let dynamic_sphere = materials.add(Color::srgb(0.92, 0.56, 0.22));
    let dynamic_hull = materials.add(Color::srgb(0.36, 0.68, 0.42));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.5, 0.35, 3.5))),
        MeshMaterial3d(mesh_platform),
        Transform::from_xyz(-2.5, 0.0, 0.0),
        RigidBody::Static,
        Collider::mesh_box(Vec3::ZERO, Vec3::new(1.75, 0.175, 1.75), Vec3::ONE, true),
        TestbedEntity,
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.5, 0.25, 3.5))),
        MeshMaterial3d(height_field),
        Transform::from_xyz(1.5, 0.0, -1.4),
        RigidBody::Static,
        Collider::height_field_grid(5, 5, Vec3::new(0.75, 0.25, 0.75), false),
        TestbedEntity,
    ));
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.65).mesh().uv(32, 18))),
        MeshMaterial3d(compound),
        Transform::from_xyz(4.3, 0.65, 1.3),
        RigidBody::Static,
        Collider::compound_sphere(Vec3::ZERO, 0.65, boxddd::SurfaceMaterial::default()),
        TestbedEntity,
    ));

    let cube_mesh = meshes.add(Cuboid::new(0.7, 0.7, 0.7));
    for (index, position) in [
        Vec3::new(-3.4, 2.8, -0.5),
        Vec3::new(-2.4, 3.5, 0.4),
        Vec3::new(-1.4, 4.2, -0.2),
        Vec3::new(0.8, 3.0, -1.5),
        Vec3::new(1.8, 3.8, -1.3),
        Vec3::new(2.8, 4.6, -1.5),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(dynamic_cube.clone()),
            Transform::from_translation(position)
                .with_rotation(Quat::from_rotation_y(0.2 * index as f32)),
            RigidBody::Dynamic,
            Collider::cube(0.35),
            PhysicsMaterial {
                restitution: 0.08,
                ..default()
            },
            TestbedEntity,
        ));
    }

    let sphere_mesh = meshes.add(Sphere::new(0.35).mesh().uv(24, 12));
    for position in [Vec3::new(3.9, 3.4, 1.2), Vec3::new(4.8, 4.2, 1.6)] {
        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(dynamic_sphere.clone()),
            Transform::from_translation(position),
            RigidBody::Dynamic,
            Collider::sphere(0.35),
            PhysicsMaterial {
                restitution: 0.2,
                ..default()
            },
            TestbedEntity,
        ));
    }

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.55).mesh().uv(24, 12))),
        MeshMaterial3d(dynamic_hull),
        Transform::from_xyz(5.3, 5.1, 0.9),
        RigidBody::Dynamic,
        Collider::transformed_rock_hull(0.55, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
        TestbedEntity,
    ));
}

fn spawn_body_controls(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let cube_mesh = meshes.add(Cuboid::new(0.75, 0.75, 0.75));
    let sphere_mesh = meshes.add(Sphere::new(0.38).mesh().uv(24, 12));
    let force_material = materials.add(Color::srgb(0.18, 0.48, 0.86));
    let impulse_material = materials.add(Color::srgb(0.92, 0.48, 0.24));
    let kinematic_material = materials.add(Color::srgb(0.34, 0.72, 0.42));
    let low_gravity_material = materials.add(Color::srgb(0.82, 0.68, 0.24));

    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(force_material),
        Transform::from_xyz(-4.0, 1.4, 0.0),
        RigidBody::Dynamic,
        Collider::cube(0.375),
        BodySettings {
            motion_locks: boxddd::MotionLocks::new(false, false, true, true, true, false),
            ..default()
        },
        ExternalForce::at_center(Vec3::new(18.0, 0.0, 0.0)),
        TestbedEntity,
    ));

    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(impulse_material),
        Transform::from_xyz(-1.4, 1.4, 0.0),
        RigidBody::Dynamic,
        Collider::cube(0.375),
        ExternalImpulse::at_center(Vec3::new(4.0, 6.0, 0.0)),
        TestbedEntity,
    ));

    commands.spawn((
        Mesh3d(sphere_mesh.clone()),
        MeshMaterial3d(kinematic_material),
        Transform::from_xyz(1.4, 1.2, 0.0),
        RigidBody::Kinematic,
        Collider::sphere(0.38),
        BodySettings {
            gravity_scale: 0.0,
            ..default()
        },
        LinearVelocity(Vec3::new(0.85, 0.0, 0.0)),
        TransformSyncMode::PhysicsToBevy,
        TestbedEntity,
    ));

    commands.spawn((
        Mesh3d(sphere_mesh),
        MeshMaterial3d(low_gravity_material),
        Transform::from_xyz(4.0, 3.6, 0.0),
        RigidBody::Dynamic,
        Collider::sphere(0.38),
        BodySettings {
            gravity_scale: 0.25,
            linear_damping: 0.05,
            ..default()
        },
        TestbedEntity,
    ));
}

fn spawn_continuous_collision(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let wall_material = materials.add(Color::srgb(0.42, 0.47, 0.54));
    let bullet_material = materials.add(Color::srgb(0.95, 0.56, 0.20));
    let stack_material = materials.add(Color::srgb(0.28, 0.58, 0.88));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 3.6, 4.5))),
        MeshMaterial3d(wall_material),
        Transform::from_xyz(0.0, 1.6, 0.0),
        RigidBody::Static,
        Collider::cuboid(0.05, 1.8, 2.25),
        TestbedEntity,
    ));

    let sphere_mesh = meshes.add(Sphere::new(0.18).mesh().uv(24, 12));
    for (index, z) in [-1.2, 0.0, 1.2].into_iter().enumerate() {
        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(bullet_material.clone()),
            Transform::from_xyz(-5.0, 1.0 + index as f32 * 0.45, z),
            RigidBody::Dynamic,
            Collider::sphere(0.18),
            BodySettings {
                gravity_scale: 0.0,
                bullet: true,
                ..default()
            },
            LinearVelocity(Vec3::new(32.0, 0.0, 0.0)),
            TransformSyncMode::PhysicsToBevy,
            PhysicsMaterial {
                restitution: 0.25,
                enable_contact_events: true,
                enable_hit_events: true,
                ..default()
            },
            TestbedEntity,
        ));
    }

    let cube_mesh = meshes.add(Cuboid::new(0.45, 0.45, 0.45));
    for layer in 0..4 {
        for column in 0..3 {
            commands.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(stack_material.clone()),
                Transform::from_xyz(2.8 + column as f32 * 0.5, 0.45 + layer as f32 * 0.5, 0.0),
                RigidBody::Dynamic,
                Collider::cube(0.225),
                TestbedEntity,
            ));
        }
    }
}

fn spawn_character_mover(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mover_material = materials.add(Color::srgb(0.30, 0.68, 0.44));
    let obstacle_material = materials.add(Color::srgb(0.70, 0.36, 0.28));
    let guide_material = materials.add(Color::srgb(0.28, 0.42, 0.74));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.25, 1.0))),
        MeshMaterial3d(mover_material),
        Transform::from_xyz(-2.6, 0.8, 0.0),
        RigidBody::Kinematic,
        Collider::capsule_y(0.5, 0.25),
        BodySettings {
            gravity_scale: 0.0,
            ..default()
        },
        LinearVelocity(Vec3::new(0.75, 0.0, 0.0)),
        TransformSyncMode::PhysicsToBevy,
        TestbedEntity,
    ));

    for position in [
        Vec3::new(0.1, 0.7, 0.0),
        Vec3::new(1.3, 1.1, -0.8),
        Vec3::new(1.9, 0.6, 0.9),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.8, 1.4, 0.8))),
            MeshMaterial3d(obstacle_material.clone()),
            Transform::from_translation(position),
            RigidBody::Static,
            Collider::cuboid(0.4, 0.7, 0.4),
            TestbedEntity,
        ));
    }

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.0, 0.05, 0.05))),
        MeshMaterial3d(guide_material),
        Transform::from_xyz(-0.4, 1.55, 0.0),
        TestbedEntity,
        MoverProbe {
            origin: Vec3::new(-2.4, 0.05, 0.0),
            point1: Vec3::new(0.0, 0.3, 0.0),
            point2: Vec3::new(0.0, 1.3, 0.0),
            radius: 0.25,
            delta: Vec3::new(4.0, 0.0, 0.0),
        },
    ));
}

fn spawn_materials(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let ramp_mesh = meshes.add(Cuboid::new(3.5, 0.25, 1.1));
    let low_friction = materials.add(Color::srgb(0.18, 0.55, 0.85));
    let high_friction = materials.add(Color::srgb(0.32, 0.68, 0.38));
    let bouncy = materials.add(Color::srgb(0.94, 0.48, 0.18));
    let ramp_material = materials.add(Color::srgb(0.38, 0.39, 0.36));

    for (z, friction, material) in [(-2.8, 0.05, low_friction), (0.0, 1.2, high_friction)] {
        commands.spawn((
            Mesh3d(ramp_mesh.clone()),
            MeshMaterial3d(ramp_material.clone()),
            Transform::from_xyz(-2.2, 0.85, z).with_rotation(Quat::from_rotation_z(-0.24)),
            RigidBody::Static,
            Collider::cuboid(1.75, 0.125, 0.55),
            PhysicsMaterial {
                friction,
                ..default()
            },
            TestbedEntity,
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.55, 0.55, 0.55))),
            MeshMaterial3d(material),
            Transform::from_xyz(-3.25, 2.0, z),
            RigidBody::Dynamic,
            Collider::cube(0.275),
            PhysicsMaterial {
                friction,
                restitution: 0.02,
                ..default()
            },
            TestbedEntity,
        ));
    }

    let sphere_mesh = meshes.add(Sphere::new(0.35).mesh().uv(24, 12));
    for index in 0..4 {
        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(bouncy.clone()),
            Transform::from_xyz(2.2 + index as f32 * 0.45, 2.8 + index as f32 * 0.7, 0.8),
            RigidBody::Dynamic,
            Collider::sphere(0.35),
            PhysicsMaterial {
                restitution: 0.85,
                friction: 0.2,
                enable_contact_events: true,
                enable_hit_events: true,
                ..default()
            },
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
    for (index, joint) in [
        Joint::distance(0.8),
        Joint::revolute(),
        Joint::weld(),
        Joint::spherical(),
        Joint::prismatic(),
        Joint::wheel(),
    ]
    .into_iter()
    .enumerate()
    {
        let x = index as f32 * 1.25 - 3.2;
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
        commands.spawn((JointTarget::new(anchor, body), joint, TestbedEntity));
    }
}

fn spawn_contacts(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let sensor_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.20, 0.75, 0.95, 0.24),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.0, 0.3, 4.0))),
        MeshMaterial3d(sensor_material),
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
