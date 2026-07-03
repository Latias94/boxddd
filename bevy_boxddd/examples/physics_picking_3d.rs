use bevy::prelude::*;
use bevy_boxddd::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "boxddd Bevy Physics Picking".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BoxdddPhysicsPlugin::new(BoxdddPhysicsSettings::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, pick_with_physics)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-5.0, 4.0, 7.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(-2.0, 6.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(9.0, 0.4, 7.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.18, 0.22, 0.24))),
        Transform::from_xyz(0.0, -0.2, 0.0),
        RigidBody::Static,
        Collider::cuboid(4.5, 0.2, 3.5),
    ));

    let cube_mesh = meshes.add(Cuboid::new(0.8, 0.8, 0.8));
    let sphere_mesh = meshes.add(Sphere::new(0.45).mesh().uv(32, 18));
    let blue = materials.add(Color::srgb(0.22, 0.46, 0.88));
    let orange = materials.add(Color::srgb(0.94, 0.58, 0.22));

    commands.spawn((
        Mesh3d(cube_mesh),
        MeshMaterial3d(blue),
        Transform::from_xyz(-1.1, 1.2, 0.0),
        RigidBody::Static,
        Collider::cube(0.4),
    ));

    commands.spawn((
        Mesh3d(sphere_mesh),
        MeshMaterial3d(orange),
        Transform::from_xyz(1.1, 1.3, 0.0),
        RigidBody::Static,
        Collider::sphere(0.45),
    ));
}

fn pick_with_physics(
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    context: NonSend<BoxdddPhysicsContext>,
    mut gizmos: Gizmos,
) {
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform) = *camera;
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let translation = ray.direction * 100.0;
    gizmos.ray(ray.origin, translation, Color::srgb(0.45, 0.70, 1.0));

    let Ok(Some(hit)) = cast_ray_closest(
        &context,
        ray.origin,
        translation,
        boxddd::QueryFilter::default(),
    ) else {
        return;
    };

    gizmos.sphere(hit.point, 0.08, Color::srgb(1.0, 0.96, 0.25));
    gizmos.ray(hit.point, hit.normal * 0.45, Color::srgb(0.2, 1.0, 0.35));
}
