mod control;
mod picking;
mod scenes;
#[path = "../support/mod.rs"]
mod support;
mod ui;

use bevy::prelude::*;
use bevy::time::Fixed;
use bevy_boxddd::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use control::TestbedState;
use scenes::{ALL_SCENES, MoverProbe, TestbedEntity, TestbedScene, spawn_scene};

#[derive(Component, Debug)]
pub(crate) struct TestbedCamera;

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
        .insert_resource(picking::PhysicsDragState::default())
        .add_plugins(support::teaching_default_plugins("boxddd Bevy Testbed"))
        .add_plugins(EguiPlugin::default())
        .add_plugins(BoxdddPhysicsPlugin::new(BoxdddPhysicsSettings::default()))
        .add_systems(First, prepare_time_control)
        .add_systems(Startup, (setup_view, spawn_initial_scene).chain())
        .add_systems(EguiPrimaryContextPass, ui::draw_testbed_ui)
        .add_systems(
            Update,
            (
                handle_input,
                apply_testbed_settings,
                draw_debug_gizmos,
                draw_mover_probe,
                picking::update_physics_drag,
                picking::draw_physics_pick,
            ),
        )
        .add_systems(PostUpdate, finish_single_step)
        .run();
}

fn setup_view(mut commands: Commands, state: Res<TestbedState>) {
    commands.spawn((
        TestbedCamera,
        Camera3d::default(),
        state.scene().metadata().camera.transform(),
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
    log_scene_selection(state.scene());
    spawn_scene(&mut commands, &mut meshes, &mut materials, state.scene());
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<TestbedState>,
    mut commands: Commands,
    entities: Query<Entity, With<TestbedEntity>>,
    mut camera: Query<&mut Transform, With<TestbedCamera>>,
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
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::Digit0,
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
        state.debug_preset = state.debug_preset.toggled();
    }
    if keys.just_pressed(KeyCode::KeyG) {
        state.gravity_enabled = !state.gravity_enabled;
    }
    if keys.just_pressed(KeyCode::Space) {
        state.paused = !state.paused;
        if !state.paused {
            state.cancel_single_step();
        }
    }
    if keys.just_pressed(KeyCode::Enter) {
        state.request_single_step();
    }

    let Some(scene_index) = requested_scene else {
        return;
    };

    switch_scene(
        scene_index,
        &mut state,
        &mut commands,
        &entities,
        &mut camera,
        &mut meshes,
        &mut materials,
    );
}

pub(crate) fn switch_scene(
    scene_index: usize,
    state: &mut TestbedState,
    commands: &mut Commands,
    entities: &Query<Entity, With<TestbedEntity>>,
    camera: &mut Query<&mut Transform, With<TestbedCamera>>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for entity in entities {
        commands.entity(entity).despawn();
    }
    state.scene_index = scene_index.min(ALL_SCENES.len() - 1);
    if let Ok(mut transform) = camera.single_mut() {
        *transform = state.scene().metadata().camera.transform();
    }
    log_scene_selection(state.scene());
    spawn_scene(commands, meshes, materials, state.scene());
}

fn log_scene_selection(scene: TestbedScene) {
    let metadata = scene.metadata();
    bevy::log::info!(
        "Testbed scene [{}] {} ({}) - {}",
        metadata.category,
        metadata.name,
        metadata.id,
        metadata.description
    );
}

fn prepare_time_control(mut state: ResMut<TestbedState>, mut time: ResMut<Time<Virtual>>) {
    state.clamp_controls();
    if !state.paused {
        state.cancel_single_step();
        time.unpause();
        return;
    }

    if state.single_step_pending {
        state.single_step_pending = false;
        state.single_step_active = true;
        time.unpause();
    } else {
        time.pause();
    }
}

fn finish_single_step(mut state: ResMut<TestbedState>, mut time: ResMut<Time<Virtual>>) {
    if state.single_step_active {
        state.single_step_active = false;
        time.pause();
    }
}

fn apply_testbed_settings(
    mut state: ResMut<TestbedState>,
    mut physics_settings: ResMut<BoxdddPhysicsSettings>,
    mut fixed_time: ResMut<Time<Fixed>>,
    mut debug_settings: ResMut<BoxdddDebugDrawSettings>,
    mut context: NonSendMut<BoxdddPhysicsContext>,
) {
    state.clamp_controls();

    let gravity = if state.gravity_enabled {
        Vec3::new(0.0, -10.0, 0.0)
    } else {
        Vec3::ZERO
    };
    physics_settings.gravity = gravity;
    physics_settings.sub_step_count = state.sub_step_count;
    physics_settings.fixed_timestep_seconds = Some(state.fixed_timestep_seconds());
    fixed_time.set_timestep_hz(state.hertz);

    debug_settings.enabled = state.debug_preset.is_enabled();
    debug_settings.options = state.debug_preset.options();

    if let Some(world) = context.world_mut() {
        let _ = world.try_set_gravity(boxddd::Vec3::new(gravity.x, gravity.y, gravity.z));
        let _ = world.try_enable_sleeping(state.sleeping_enabled);
        let _ = world.try_enable_warm_starting(state.warm_starting_enabled);
        let _ = world.try_enable_continuous(state.continuous_enabled);
    }
}

fn draw_mover_probe(
    probes: Query<&MoverProbe>,
    context: NonSend<BoxdddPhysicsContext>,
    mut gizmos: Gizmos,
) {
    let Some(world) = context.world() else {
        return;
    };

    for probe in &probes {
        let mover = boxddd::Capsule::new(
            to_boxddd_vec3(probe.point1),
            to_boxddd_vec3(probe.point2),
            probe.radius,
        );
        let Ok(fraction) = world.cast_mover(
            to_boxddd_pos(probe.origin),
            &mover,
            to_boxddd_vec3(probe.delta),
            boxddd::QueryFilter::default(),
        ) else {
            continue;
        };
        let safe_delta = probe.delta * fraction;
        let start = probe.origin;
        let requested_end = start + probe.delta;
        let safe_end = start + safe_delta;

        gizmos.line(start, requested_end, Color::srgb(0.45, 0.50, 0.58));
        gizmos.line(start, safe_end, Color::srgb(0.2, 0.9, 0.45));
        gizmos.sphere(
            safe_end + Vec3::new(0.0, 0.8, 0.0),
            probe.radius,
            Color::srgb(0.2, 0.9, 0.45),
        );
    }
}

fn to_boxddd_vec3(value: Vec3) -> boxddd::Vec3 {
    boxddd::Vec3::new(value.x, value.y, value.z)
}

fn to_boxddd_pos(value: Vec3) -> boxddd::Pos {
    boxddd::Pos::new(value.x.into(), value.y.into(), value.z.into())
}
