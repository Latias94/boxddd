use crate::control::{
    MAX_MATERIAL_FRICTION, MAX_MATERIAL_RESTITUTION, MAX_QUERY_AABB_HALF_EXTENT,
    MAX_QUERY_RAY_LENGTH, MIN_MATERIAL_FRICTION, MIN_MATERIAL_RESTITUTION,
    MIN_QUERY_AABB_HALF_EXTENT, MIN_QUERY_RAY_LENGTH, TestbedState,
};
use crate::scenes::{ALL_SCENES, MaterialLabTarget, TestbedScene};
use bevy::prelude::*;
use bevy_boxddd::prelude::*;

pub(crate) const QUERY_LAB_ORIGIN: Vec3 = Vec3::new(-4.0, 1.6, 0.0);
pub(crate) const QUERY_LAB_AABB_CENTER: Vec3 = Vec3::new(0.0, 1.55, 0.0);

#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub(crate) struct LabDiagnostics {
    pub query_ray_hit_count: usize,
    pub query_overlap_hit_count: usize,
    pub query_closest_fraction: Option<f32>,
    pub debug_command_count: usize,
    pub debug_event_count: usize,
    pub debug_diagnostic_count: usize,
    pub material_shape_count: usize,
}

pub(crate) fn apply_material_lab_controls(
    mut context: NonSendMut<BoxdddPhysicsContext>,
    state: Res<TestbedState>,
    shapes: Query<&BoxdddShape, With<MaterialLabTarget>>,
    mut diagnostics: ResMut<LabDiagnostics>,
) {
    if current_scene(&state) != TestbedScene::MaterialLab {
        diagnostics.material_shape_count = 0;
        return;
    }

    let Some(world) = context.world_mut() else {
        diagnostics.material_shape_count = 0;
        return;
    };

    let friction = material_friction(&state);
    let restitution = material_restitution(&state);
    let mut applied = 0;

    for shape in &shapes {
        let shape_id = shape.id();
        let friction_result = world.try_set_shape_friction(shape_id, friction);
        let restitution_result = world.try_set_shape_restitution(shape_id, restitution);
        if friction_result.is_ok() && restitution_result.is_ok() {
            applied += 1;
        }
    }

    diagnostics.material_shape_count = applied;
}

pub(crate) fn update_lab_diagnostics(
    state: Res<TestbedState>,
    context: NonSend<BoxdddPhysicsContext>,
    debug_frame: Res<BoxdddDebugDrawFrame>,
    mut diagnostics: ResMut<LabDiagnostics>,
) {
    match current_scene(&state) {
        TestbedScene::QueryLab => {
            diagnostics.clear_debug_counts();
            let translation = query_lab_ray_translation(&state);
            diagnostics.query_ray_hit_count = cast_ray(
                &context,
                QUERY_LAB_ORIGIN,
                translation,
                boxddd::QueryFilter::default(),
            )
            .map(|hits| hits.len())
            .unwrap_or(0);
            diagnostics.query_closest_fraction = cast_ray_closest(
                &context,
                QUERY_LAB_ORIGIN,
                translation,
                boxddd::QueryFilter::default(),
            )
            .ok()
            .flatten()
            .map(|hit| hit.fraction);

            let (lower_bound, upper_bound) = query_lab_aabb(&state);
            diagnostics.query_overlap_hit_count = overlap_aabb(
                &context,
                lower_bound,
                upper_bound,
                boxddd::QueryFilter::default(),
            )
            .map(|hits| hits.len())
            .unwrap_or(0);
        }
        TestbedScene::DebugDrawInspector => {
            diagnostics.clear_query_counts();
            diagnostics.debug_command_count = debug_frame.commands().len();
            diagnostics.debug_event_count = debug_frame.events().len();
            diagnostics.debug_diagnostic_count = debug_frame.diagnostics().len();
        }
        _ => {
            diagnostics.clear_query_counts();
            diagnostics.clear_debug_counts();
        }
    }
}

pub(crate) fn draw_lab_overlays(
    state: Res<TestbedState>,
    context: NonSend<BoxdddPhysicsContext>,
    mut gizmos: Gizmos,
) {
    if current_scene(&state) != TestbedScene::QueryLab {
        return;
    }

    let translation = query_lab_ray_translation(&state);
    let ray_end = QUERY_LAB_ORIGIN + translation;
    gizmos.line(QUERY_LAB_ORIGIN, ray_end, Color::srgb(0.20, 0.72, 0.95));

    if let Ok(hits) = cast_ray(
        &context,
        QUERY_LAB_ORIGIN,
        translation,
        boxddd::QueryFilter::default(),
    ) {
        for hit in hits {
            gizmos.sphere(hit.point, 0.08, Color::srgb(0.98, 0.72, 0.24));
            gizmos.line(
                hit.point,
                hit.point + hit.normal * 0.35,
                Color::srgb(0.98, 0.72, 0.24),
            );
        }
    }

    let (lower_bound, upper_bound) = query_lab_aabb(&state);
    gizmos.cube(
        Transform::from_translation((lower_bound + upper_bound) * 0.5)
            .with_scale(upper_bound - lower_bound),
        Color::srgb(0.36, 0.86, 0.48),
    );
}

impl LabDiagnostics {
    fn clear_query_counts(&mut self) {
        self.query_ray_hit_count = 0;
        self.query_overlap_hit_count = 0;
        self.query_closest_fraction = None;
    }

    fn clear_debug_counts(&mut self) {
        self.debug_command_count = 0;
        self.debug_event_count = 0;
        self.debug_diagnostic_count = 0;
    }
}

pub(crate) fn current_scene(state: &TestbedState) -> TestbedScene {
    ALL_SCENES[state.scene_index.min(ALL_SCENES.len() - 1)]
}

pub(crate) fn query_lab_ray_translation(state: &TestbedState) -> Vec3 {
    Vec3::X
        * state
            .query_lab_ray_length
            .clamp(MIN_QUERY_RAY_LENGTH, MAX_QUERY_RAY_LENGTH)
}

pub(crate) fn query_lab_aabb(state: &TestbedState) -> (Vec3, Vec3) {
    let half_extent = state
        .query_lab_aabb_half_extent
        .clamp(MIN_QUERY_AABB_HALF_EXTENT, MAX_QUERY_AABB_HALF_EXTENT);
    let half_extents = Vec3::splat(half_extent);
    (
        QUERY_LAB_AABB_CENTER - half_extents,
        QUERY_LAB_AABB_CENTER + half_extents,
    )
}

pub(crate) fn material_friction(state: &TestbedState) -> f32 {
    state
        .material_lab_friction
        .clamp(MIN_MATERIAL_FRICTION, MAX_MATERIAL_FRICTION)
}

pub(crate) fn material_restitution(state: &TestbedState) -> f32 {
    state
        .material_lab_restitution
        .clamp(MIN_MATERIAL_RESTITUTION, MAX_MATERIAL_RESTITUTION)
}
