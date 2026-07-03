mod scenes;

use bevy::prelude::*;
use bevy_boxddd::prelude::*;
use scenes::{ALL_SCENES, TestbedEntity, TestbedScene, spawn_scene};

#[derive(Resource, Debug)]
struct TestbedState {
    scene_index: usize,
    paused: bool,
    debug_draw: bool,
    gravity_enabled: bool,
}

impl Default for TestbedState {
    fn default() -> Self {
        Self {
            scene_index: 0,
            paused: false,
            debug_draw: false,
            gravity_enabled: true,
        }
    }
}

impl TestbedState {
    fn scene(&self) -> TestbedScene {
        ALL_SCENES[self.scene_index]
    }
}

fn main() {
    App::new()
        .insert_resource(TestbedState::default())
        .insert_resource(BoxdddDebugDrawSettings {
            enabled: false,
            options: boxddd::DebugDrawOptions::default(),
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "boxddd Bevy Testbed".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BoxdddPhysicsPlugin::new(BoxdddPhysicsSettings::default()))
        .add_systems(Startup, (setup_view, spawn_initial_scene).chain())
        .add_systems(
            Update,
            (
                handle_input,
                apply_debug_draw_toggle,
                apply_gravity_to_world,
                draw_debug_gizmos,
                draw_physics_pick,
            ),
        )
        .run();
}

fn setup_view(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-7.0, 5.0, 9.0).looking_at(Vec3::new(0.0, 1.2, 0.0), Vec3::Y),
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
}

fn spawn_initial_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<TestbedState>,
) {
    spawn_scene(&mut commands, &mut meshes, &mut materials, state.scene());
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<TestbedState>,
    mut physics_settings: ResMut<BoxdddPhysicsSettings>,
    mut time: ResMut<Time<Virtual>>,
    mut commands: Commands,
    entities: Query<Entity, With<TestbedEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut requested_scene = None;
    for (index, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
    ]
    .into_iter()
    .enumerate()
    {
        if keys.just_pressed(key) {
            requested_scene = Some(index);
        }
    }

    if keys.just_pressed(KeyCode::KeyR) {
        requested_scene = Some(state.scene_index);
    }
    if keys.just_pressed(KeyCode::KeyD) {
        state.debug_draw = !state.debug_draw;
    }
    if keys.just_pressed(KeyCode::KeyG) {
        state.gravity_enabled = !state.gravity_enabled;
        physics_settings.gravity = if state.gravity_enabled {
            Vec3::new(0.0, -10.0, 0.0)
        } else {
            Vec3::ZERO
        };
    }
    if keys.just_pressed(KeyCode::Space) {
        state.paused = !state.paused;
        if state.paused {
            time.pause();
        } else {
            time.unpause();
        }
    }

    let Some(scene_index) = requested_scene else {
        return;
    };

    for entity in &entities {
        commands.entity(entity).despawn();
    }
    state.scene_index = scene_index.min(ALL_SCENES.len() - 1);
    spawn_scene(&mut commands, &mut meshes, &mut materials, state.scene());
}

fn apply_debug_draw_toggle(
    state: Res<TestbedState>,
    mut debug_settings: ResMut<BoxdddDebugDrawSettings>,
) {
    debug_settings.enabled = state.debug_draw;
    debug_settings.options.draw_joints = true;
    debug_settings.options.draw_bounds = state.debug_draw;
}

fn apply_gravity_to_world(state: Res<TestbedState>, mut context: NonSendMut<BoxdddPhysicsContext>) {
    if !state.is_changed() {
        return;
    }

    let gravity = if state.gravity_enabled {
        boxddd::Vec3::new(0.0, -10.0, 0.0)
    } else {
        boxddd::Vec3::ZERO
    };
    if let Some(world) = context.world_mut() {
        let _ = world.try_set_gravity(gravity);
    }
}

fn draw_physics_pick(
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
    let Ok(Some(hit)) = cast_ray_closest(
        &context,
        ray.origin,
        translation,
        boxddd::QueryFilter::default(),
    ) else {
        return;
    };

    gizmos.sphere(hit.point, 0.08, Color::srgb(1.0, 0.95, 0.2));
    gizmos.ray(hit.point, hit.normal * 0.45, Color::srgb(0.2, 1.0, 0.35));
}
