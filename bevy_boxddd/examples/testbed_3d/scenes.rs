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
    QueryLab,
    DebugDrawInspector,
    MaterialLab,
    DominoRun,
    ArchStack,
    WindField,
    RagdollChain,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TestbedCamera {
    pub position: [f32; 3],
    pub target: [f32; 3],
}

impl TestbedCamera {
    pub const fn new(position: [f32; 3], target: [f32; 3]) -> Self {
        Self { position, target }
    }

    pub fn transform(self) -> Transform {
        Transform::from_translation(vec3_from_array(self.position))
            .looking_at(vec3_from_array(self.target), Vec3::Y)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ParityMode {
    FaithfulPort,
    TeachingAdaptation,
}

impl ParityMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FaithfulPort => "FaithfulPort",
            Self::TeachingAdaptation => "TeachingAdaptation",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct UpstreamSampleRef {
    pub category: &'static str,
    pub name: &'static str,
    pub mode: ParityMode,
}

#[derive(Copy, Clone)]
pub struct TestbedSceneMetadata {
    pub scene: TestbedScene,
    pub id: &'static str,
    pub category: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub upstream: &'static [UpstreamSampleRef],
    pub showcase_lesson: Option<&'static str>,
    pub camera: TestbedCamera,
    spawn: fn(&mut Commands, &mut Assets<Mesh>, &mut Assets<StandardMaterial>),
}

impl TestbedSceneMetadata {
    pub const fn source_label(self) -> &'static str {
        if self.showcase_lesson.is_some() {
            "boxddd showcase"
        } else {
            "official Box3D sample"
        }
    }
}

impl TestbedScene {
    pub fn metadata(self) -> &'static TestbedSceneMetadata {
        SCENE_REGISTRY
            .iter()
            .find(|metadata| metadata.scene == self)
            .expect("testbed scene metadata missing")
    }

    pub fn from_id(id: &str) -> Option<Self> {
        SCENE_REGISTRY
            .iter()
            .find(|metadata| metadata.id == id)
            .map(|metadata| metadata.scene)
    }

    pub fn index(self) -> usize {
        ALL_SCENES
            .iter()
            .position(|scene| *scene == self)
            .expect("testbed scene missing from ALL_SCENES")
    }
}

pub const ALL_SCENES: [TestbedScene; 17] = [
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
    TestbedScene::QueryLab,
    TestbedScene::DebugDrawInspector,
    TestbedScene::MaterialLab,
    TestbedScene::DominoRun,
    TestbedScene::ArchStack,
    TestbedScene::WindField,
    TestbedScene::RagdollChain,
];

pub const SCENE_REGISTRY: [TestbedSceneMetadata; 17] = [
    TestbedSceneMetadata {
        scene: TestbedScene::FallingStack,
        id: "falling-stack",
        category: "Basics",
        name: "Falling Stack",
        description: "Box, sphere, capsule, and cylinder stacks falling onto a static floor.",
        upstream: &[
            UpstreamSampleRef {
                category: "Stacking",
                name: "Single Box",
                mode: ParityMode::FaithfulPort,
            },
            UpstreamSampleRef {
                category: "Stacking",
                name: "Box Stack",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Stacking",
                name: "Pyramid2D",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Stacking",
                name: "Sphere Stack",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Stacking",
                name: "Capsule Stack",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Stacking",
                name: "Cylinder",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Stacking",
                name: "Cylinder Stack",
                mode: ParityMode::TeachingAdaptation,
            },
        ],
        showcase_lesson: None,
        camera: TestbedCamera::new([-8.2, 5.7, 9.6], [0.0, 1.5, 0.0]),
        spawn: spawn_falling_stack,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::AdvancedColliders,
        id: "advanced-colliders",
        category: "Colliders",
        name: "Advanced Colliders",
        description: "Mesh, height-field, compound, sphere, and hull colliders.",
        upstream: &[
            UpstreamSampleRef {
                category: "Compound",
                name: "Simple",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Compound",
                name: "Mesh Tile",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Mesh",
                name: "Grid",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Mesh",
                name: "Height Field",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Geometry",
                name: "Hull",
                mode: ParityMode::TeachingAdaptation,
            },
        ],
        showcase_lesson: None,
        camera: TestbedCamera::new([-7.0, 5.0, 9.0], [1.0, 1.2, 0.0]),
        spawn: spawn_advanced_colliders,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::BodyControls,
        id: "body-controls",
        category: "Bodies",
        name: "Body Controls",
        description: "Body settings, force, impulse, kinematic motion, and gravity scale.",
        upstream: &[
            UpstreamSampleRef {
                category: "Bodies",
                name: "Body Type",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Bodies",
                name: "Disable",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Bodies",
                name: "Kinematic",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Bodies",
                name: "Fixed Rotation",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Joints",
                name: "Motion Locks",
                mode: ParityMode::TeachingAdaptation,
            },
        ],
        showcase_lesson: None,
        camera: TestbedCamera::new([-7.0, 5.0, 9.0], [0.0, 1.2, 0.0]),
        spawn: spawn_body_controls,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::ContinuousCollision,
        id: "continuous-collision",
        category: "Collision",
        name: "Continuous Collision",
        description: "Bullet-style fast bodies colliding with thin obstacles.",
        upstream: &[
            UpstreamSampleRef {
                category: "Continuous",
                name: "Thin Wall",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Continuous",
                name: "Bullet vs Stack",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Collision",
                name: "Time of Impact",
                mode: ParityMode::TeachingAdaptation,
            },
        ],
        showcase_lesson: None,
        camera: TestbedCamera::new([-7.0, 5.0, 9.0], [0.0, 1.2, 0.0]),
        spawn: spawn_continuous_collision,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::CharacterMover,
        id: "character-mover",
        category: "Character",
        name: "Character Mover",
        description: "Capsule mover casts and obstacle probes.",
        upstream: &[
            UpstreamSampleRef {
                category: "Character",
                name: "Mover",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Character",
                name: "Rigid Body",
                mode: ParityMode::TeachingAdaptation,
            },
        ],
        showcase_lesson: None,
        camera: TestbedCamera::new([-7.0, 5.0, 9.0], [0.0, 1.2, 0.0]),
        spawn: spawn_character_mover,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::Materials,
        id: "materials",
        category: "Materials",
        name: "Materials",
        description: "Friction and restitution variants shown with dynamic shapes.",
        upstream: &[
            UpstreamSampleRef {
                category: "Shapes",
                name: "Inclined Plane",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Shapes",
                name: "Rolling Resistance",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Shapes",
                name: "High Resistance",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Shapes",
                name: "Restitution",
                mode: ParityMode::TeachingAdaptation,
            },
        ],
        showcase_lesson: None,
        camera: TestbedCamera::new([-7.0, 5.0, 9.0], [0.0, 1.2, 0.0]),
        spawn: spawn_materials,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::Joints,
        id: "joints",
        category: "Joints",
        name: "Joints",
        description: "Public joint variants authored as Bevy entities.",
        upstream: &[
            UpstreamSampleRef {
                category: "Joints",
                name: "Distance Joint",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Joints",
                name: "Prismatic",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Joints",
                name: "Spherical",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Joints",
                name: "Revolute",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Joints",
                name: "Weld",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Joints",
                name: "Wheel",
                mode: ParityMode::TeachingAdaptation,
            },
        ],
        showcase_lesson: None,
        camera: TestbedCamera::new([-7.0, 5.0, 9.0], [0.0, 1.2, 0.0]),
        spawn: spawn_joints,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::Contacts,
        id: "contacts",
        category: "Events",
        name: "Contacts And Sensors",
        description: "Contact and sensor messages emitted from the physics step.",
        upstream: &[
            UpstreamSampleRef {
                category: "Events",
                name: "Hit",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Events",
                name: "Persistent Contact",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Events",
                name: "Sensor Hits",
                mode: ParityMode::TeachingAdaptation,
            },
        ],
        showcase_lesson: None,
        camera: TestbedCamera::new([-7.0, 5.0, 9.0], [0.0, 1.2, 0.0]),
        spawn: spawn_contacts,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::RayPicking,
        id: "ray-picking",
        category: "Queries",
        name: "Ray Picking",
        description: "Camera rays resolved through Box3D world queries.",
        upstream: &[
            UpstreamSampleRef {
                category: "Collision",
                name: "Ray Curtain",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Collision",
                name: "Cast World",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Collision",
                name: "Overlap World",
                mode: ParityMode::TeachingAdaptation,
            },
        ],
        showcase_lesson: None,
        camera: TestbedCamera::new([-7.0, 5.0, 9.0], [0.0, 1.2, 0.0]),
        spawn: spawn_ray_picking,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::DebugDraw,
        id: "debug-draw",
        category: "Debug",
        name: "Debug Draw",
        description: "Native Box3D debug draw commands rendered through Bevy gizmos.",
        upstream: &[
            UpstreamSampleRef {
                category: "Collision",
                name: "Shape Cast Debug",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Collision",
                name: "Distance Debug",
                mode: ParityMode::TeachingAdaptation,
            },
        ],
        showcase_lesson: None,
        camera: TestbedCamera::new([-7.0, 5.0, 9.0], [0.0, 1.2, 0.0]),
        spawn: spawn_debug_draw,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::QueryLab,
        id: "query-lab",
        category: "Showcase",
        name: "Query Lab",
        description: "Editor-style picking bodies driven by Box3D world queries.",
        upstream: &[],
        showcase_lesson: Some("Use Box3D query results as the authority for Bevy tool selection."),
        camera: TestbedCamera::new([-7.0, 5.0, 9.0], [0.0, 1.2, 0.0]),
        spawn: spawn_query_lab,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::DebugDrawInspector,
        id: "debug-draw-inspector",
        category: "Showcase",
        name: "Debug Draw Inspector",
        description: "Debug draw frame assets rendered through the Bevy testbed overlay.",
        upstream: &[],
        showcase_lesson: Some("Inspect persistent debug assets without borrowing native memory."),
        camera: TestbedCamera::new([-7.0, 5.0, 9.0], [0.0, 1.2, 0.0]),
        spawn: spawn_debug_draw_inspector,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::MaterialLab,
        id: "material-lab",
        category: "Showcase",
        name: "Material Lab",
        description: "Friction and restitution variants arranged for side-by-side comparison.",
        upstream: &[],
        showcase_lesson: Some(
            "Compare material coefficients in a Bevy scene before building custom tooling.",
        ),
        camera: TestbedCamera::new([-7.0, 5.0, 9.0], [0.0, 1.2, 0.0]),
        spawn: spawn_material_lab,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::DominoRun,
        id: "domino-run",
        category: "Stacking",
        name: "Domino Run",
        description: "A curved line of dynamic dominoes started by a moving sphere.",
        upstream: &[
            UpstreamSampleRef {
                category: "Stacking",
                name: "Dominoes",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Stacking",
                name: "Double Domino",
                mode: ParityMode::TeachingAdaptation,
            },
        ],
        showcase_lesson: None,
        camera: TestbedCamera::new([-6.5, 5.3, 8.4], [0.3, 0.9, 0.1]),
        spawn: spawn_domino_run,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::ArchStack,
        id: "arch-stack",
        category: "Stacking",
        name: "Arch Stack",
        description: "Dynamic blocks arranged as a simple arch over static pillars.",
        upstream: &[
            UpstreamSampleRef {
                category: "Stacking",
                name: "Arch",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Stacking",
                name: "Wedge",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Stacking",
                name: "Card House",
                mode: ParityMode::TeachingAdaptation,
            },
        ],
        showcase_lesson: None,
        camera: TestbedCamera::new([-7.2, 5.4, 8.8], [0.0, 1.6, 0.0]),
        spawn: spawn_arch_stack,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::WindField,
        id: "wind-field",
        category: "Forces",
        name: "Wind Field",
        description: "Continuous external forces push light bodies through obstacles.",
        upstream: &[
            UpstreamSampleRef {
                category: "Shapes",
                name: "Wind",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Shapes",
                name: "Wind Drop",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Shapes",
                name: "Wind Flap",
                mode: ParityMode::TeachingAdaptation,
            },
        ],
        showcase_lesson: None,
        camera: TestbedCamera::new([-7.0, 4.8, 8.6], [0.3, 1.2, 0.0]),
        spawn: spawn_wind_field,
    },
    TestbedSceneMetadata {
        scene: TestbedScene::RagdollChain,
        id: "ragdoll-chain",
        category: "Joints",
        name: "Ragdoll Chain",
        description: "A lightweight joint chain made from capsule bodies.",
        upstream: &[
            UpstreamSampleRef {
                category: "Ragdoll",
                name: "Box",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Ragdoll",
                name: "Pile",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Ragdoll",
                name: "Incline",
                mode: ParityMode::TeachingAdaptation,
            },
            UpstreamSampleRef {
                category: "Joints",
                name: "Ball and Chain",
                mode: ParityMode::TeachingAdaptation,
            },
        ],
        showcase_lesson: None,
        camera: TestbedCamera::new([-6.4, 5.6, 8.0], [0.0, 1.9, 0.0]),
        spawn: spawn_ragdoll_chain,
    },
];

pub fn spawn_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    scene: TestbedScene,
) {
    spawn_ground(commands, meshes, materials);

    (scene.metadata().spawn)(commands, meshes, materials);
}

fn vec3_from_array(value: [f32; 3]) -> Vec3 {
    Vec3::new(value[0], value[1], value[2])
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
    let box_mesh = meshes.add(Cuboid::new(0.72, 0.72, 0.72));
    let box_material = materials.add(Color::srgb(0.22, 0.48, 0.88));
    for layer in 0..5 {
        for column in 0..(5 - layer) {
            let x = -5.0 + column as f32 * 0.82 + layer as f32 * 0.41;
            commands.spawn((
                Mesh3d(box_mesh.clone()),
                MeshMaterial3d(box_material.clone()),
                Transform::from_xyz(x, 0.52 + layer as f32 * 0.78, -0.7),
                RigidBody::Dynamic,
                Collider::cube(0.36),
                TestbedEntity,
            ));
        }
    }

    let sphere_mesh = meshes.add(Sphere::new(0.34).mesh().uv(24, 12));
    let sphere_material = materials.add(Color::srgb(0.88, 0.56, 0.24));
    for layer in 0..5 {
        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(sphere_material.clone()),
            Transform::from_xyz(
                -1.2 + (layer % 2) as f32 * 0.06,
                0.48 + layer as f32 * 0.7,
                0.0,
            ),
            RigidBody::Dynamic,
            Collider::sphere(0.34),
            PhysicsMaterial {
                restitution: 0.12,
                ..default()
            },
            TestbedEntity,
        ));
    }

    let capsule_mesh = meshes.add(Capsule3d::new(0.22, 0.68));
    let capsule_material = materials.add(Color::srgb(0.38, 0.68, 0.42));
    for layer in 0..4 {
        commands.spawn((
            Mesh3d(capsule_mesh.clone()),
            MeshMaterial3d(capsule_material.clone()),
            Transform::from_xyz(1.3, 0.62 + layer as f32 * 0.78, -0.1)
                .with_rotation(Quat::from_rotation_z(0.18 * (layer as f32 - 1.5))),
            RigidBody::Dynamic,
            Collider::capsule_y(0.34, 0.22),
            PhysicsMaterial {
                friction: 0.75,
                ..default()
            },
            TestbedEntity,
        ));
    }

    let cylinder_mesh = meshes.add(Cylinder::new(0.32, 0.72).mesh().resolution(24));
    let cylinder_material = materials.add(Color::srgb(0.72, 0.42, 0.82));
    for layer in 0..4 {
        commands.spawn((
            Mesh3d(cylinder_mesh.clone()),
            MeshMaterial3d(cylinder_material.clone()),
            Transform::from_xyz(3.6, 0.54 + layer as f32 * 0.76, 0.15)
                .with_rotation(Quat::from_rotation_y(0.32 * layer as f32)),
            RigidBody::Dynamic,
            Collider::cylinder_hull(0.72, 0.32, 16),
            PhysicsMaterial {
                friction: 0.65,
                ..default()
            },
            TestbedEntity,
        ));
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
        Transform::from_xyz(-1.0, 1.6, 0.0),
        RigidBody::Dynamic,
        Collider::cube(0.4),
        BodySettings {
            linear_damping: 0.15,
            angular_damping: 0.25,
            ..default()
        },
        TestbedEntity,
    ));
    commands.spawn((
        Mesh3d(sphere_mesh),
        MeshMaterial3d(orange),
        Transform::from_xyz(1.0, 1.8, 0.0),
        RigidBody::Dynamic,
        Collider::sphere(0.45),
        BodySettings {
            linear_damping: 0.12,
            angular_damping: 0.2,
            ..default()
        },
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

fn spawn_query_lab(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_ray_picking(commands, meshes, materials);
}

fn spawn_debug_draw_inspector(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_debug_draw(commands, meshes, materials);
}

fn spawn_material_lab(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_materials(commands, meshes, materials);
}

fn spawn_domino_run(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let domino_mesh = meshes.add(Cuboid::new(0.16, 1.2, 0.52));
    let domino_material = materials.add(Color::srgb(0.18, 0.45, 0.86));
    let accent_material = materials.add(Color::srgb(0.95, 0.55, 0.24));

    for index in 0..18 {
        let t = index as f32 / 17.0;
        let angle = -0.72 + t * 1.44;
        let position = Vec3::new(angle.sin() * 3.2, 0.6, angle.cos() * 1.8);
        commands.spawn((
            Mesh3d(domino_mesh.clone()),
            MeshMaterial3d(if index % 5 == 0 {
                accent_material.clone()
            } else {
                domino_material.clone()
            }),
            Transform::from_translation(position)
                .with_rotation(Quat::from_rotation_y(-angle + 0.25)),
            RigidBody::Dynamic,
            Collider::cuboid(0.08, 0.6, 0.26),
            PhysicsMaterial {
                friction: 0.8,
                restitution: 0.05,
                ..default()
            },
            TestbedEntity,
        ));
    }

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.28).mesh().uv(24, 12))),
        MeshMaterial3d(materials.add(Color::srgb(0.38, 0.78, 0.38))),
        Transform::from_xyz(-3.45, 0.45, 1.55),
        RigidBody::Dynamic,
        Collider::sphere(0.28),
        LinearVelocity(Vec3::new(4.4, 0.0, -0.4)),
        PhysicsMaterial {
            restitution: 0.25,
            ..default()
        },
        TestbedEntity,
    ));
}

fn spawn_arch_stack(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let pillar_material = materials.add(Color::srgb(0.34, 0.36, 0.40));
    let block_material = materials.add(Color::srgb(0.76, 0.52, 0.32));
    let block_mesh = meshes.add(Cuboid::new(0.78, 0.28, 0.55));

    for x in [-2.2, 2.2] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.55, 1.7, 0.75))),
            MeshMaterial3d(pillar_material.clone()),
            Transform::from_xyz(x, 0.85, 0.0),
            RigidBody::Static,
            Collider::cuboid(0.275, 0.85, 0.375),
            TestbedEntity,
        ));
    }

    for index in 0..11 {
        let t = index as f32 / 10.0;
        let angle = std::f32::consts::PI * (0.12 + t * 0.76);
        let radius = 2.25;
        let position = Vec3::new(angle.cos() * radius, 0.65 + angle.sin() * 1.7, 0.0);
        commands.spawn((
            Mesh3d(block_mesh.clone()),
            MeshMaterial3d(block_material.clone()),
            Transform::from_translation(position)
                .with_rotation(Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2)),
            RigidBody::Dynamic,
            Collider::cuboid(0.39, 0.14, 0.275),
            PhysicsMaterial {
                friction: 0.9,
                restitution: 0.02,
                ..default()
            },
            TestbedEntity,
        ));
    }
}

fn spawn_wind_field(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let field_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.30, 0.70, 0.95, 0.16),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(5.6, 2.0, 3.2))),
        MeshMaterial3d(field_material),
        Transform::from_xyz(0.0, 1.25, 0.0),
        TestbedEntity,
    ));

    let obstacle_material = materials.add(Color::srgb(0.38, 0.40, 0.46));
    for position in [
        Vec3::new(-0.8, 0.8, -0.7),
        Vec3::new(0.7, 1.1, 0.55),
        Vec3::new(2.0, 0.7, -0.35),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.42, 1.6, 0.42))),
            MeshMaterial3d(obstacle_material.clone()),
            Transform::from_translation(position),
            RigidBody::Static,
            Collider::cuboid(0.21, 0.8, 0.21),
            TestbedEntity,
        ));
    }

    let sphere_mesh = meshes.add(Sphere::new(0.24).mesh().uv(20, 10));
    let colors = [
        Color::srgb(0.20, 0.52, 0.90),
        Color::srgb(0.90, 0.56, 0.22),
        Color::srgb(0.35, 0.74, 0.45),
    ];
    for index in 0..12 {
        let row = index / 4;
        let column = index % 4;
        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(materials.add(colors[index % colors.len()])),
            Transform::from_xyz(
                -3.2 - row as f32 * 0.2,
                0.65 + column as f32 * 0.42,
                -1.2 + row as f32 * 0.85,
            ),
            RigidBody::Dynamic,
            Collider::sphere(0.24),
            BodySettings {
                gravity_scale: 0.35,
                linear_damping: 0.04,
                ..default()
            },
            ExternalForce::at_center(Vec3::new(17.0, 1.2, 2.4)),
            PhysicsMaterial {
                restitution: 0.35,
                ..default()
            },
            TestbedEntity,
        ));
    }
}

fn spawn_ragdoll_chain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let anchor = commands
        .spawn((
            Mesh3d(meshes.add(Sphere::new(0.16).mesh().uv(20, 10))),
            MeshMaterial3d(materials.add(Color::srgb(0.95, 0.72, 0.22))),
            Transform::from_xyz(-2.0, 3.4, 0.0),
            RigidBody::Static,
            TestbedEntity,
        ))
        .id();

    let capsule_mesh = meshes.add(Capsule3d::new(0.16, 0.68));
    let capsule_material = materials.add(Color::srgb(0.32, 0.60, 0.90));
    let mut previous = anchor;
    for index in 0..7 {
        let body = commands
            .spawn((
                Mesh3d(capsule_mesh.clone()),
                MeshMaterial3d(capsule_material.clone()),
                Transform::from_xyz(-1.45 + index as f32 * 0.48, 3.05 - index as f32 * 0.24, 0.0)
                    .with_rotation(Quat::from_rotation_z(0.28)),
                RigidBody::Dynamic,
                Collider::capsule_y(0.34, 0.16),
                BodySettings {
                    angular_damping: 0.18,
                    ..default()
                },
                TestbedEntity,
            ))
            .id();
        commands.spawn((
            JointTarget::new(previous, body),
            Joint::distance(0.55),
            TestbedEntity,
        ));
        previous = body;
    }
}
