use crate::control::{
    MAX_MATERIAL_FRICTION, MAX_MATERIAL_RESTITUTION, MAX_QUERY_AABB_HALF_EXTENT,
    MAX_QUERY_MOVER_CAST_LENGTH, MAX_QUERY_RAY_LENGTH, MAX_QUERY_SHAPE_CAST_LENGTH,
    MAX_QUERY_SHAPE_CAST_RADIUS, MIN_MATERIAL_FRICTION, MIN_MATERIAL_RESTITUTION,
    MIN_QUERY_AABB_HALF_EXTENT, MIN_QUERY_MOVER_CAST_LENGTH, MIN_QUERY_RAY_LENGTH,
    MIN_QUERY_SHAPE_CAST_LENGTH, MIN_QUERY_SHAPE_CAST_RADIUS, TestbedState,
};
use crate::scenes::{ALL_SCENES, MaterialLabTarget, TestbedScene};
use bevy::prelude::*;
use bevy_boxddd::prelude::*;

pub(crate) const QUERY_LAB_ORIGIN: Vec3 = Vec3::new(-4.0, 1.6, 0.0);
pub(crate) const QUERY_LAB_AABB_CENTER: Vec3 = Vec3::new(0.0, 1.55, 0.0);
pub(crate) const QUERY_LAB_SHAPE_CAST_ORIGIN: Vec3 = Vec3::new(-4.0, 1.6, 0.65);
pub(crate) const QUERY_LAB_MOVER_ORIGIN: Vec3 = Vec3::new(-4.0, 0.05, 0.0);
const QUERY_LAB_MOVER_POINT1: Vec3 = Vec3::new(0.0, 0.3, 0.0);
const QUERY_LAB_MOVER_POINT2: Vec3 = Vec3::new(0.0, 1.3, 0.0);
const QUERY_LAB_MOVER_RADIUS: f32 = 0.25;

#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub(crate) struct LabDiagnostics {
    pub query_ray_supported: bool,
    pub query_ray_hit_count: usize,
    pub query_overlap_supported: bool,
    pub query_overlap_hit_count: usize,
    pub query_closest_fraction: Option<f32>,
    pub query_shape_cast_supported: bool,
    pub query_shape_cast_hit_count: usize,
    pub query_shape_cast_closest_fraction: Option<f32>,
    pub query_mover_supported: bool,
    pub query_mover_fraction: Option<f32>,
    pub query_mover_planes_supported: bool,
    pub query_mover_plane_count: usize,
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
            match cast_ray(
                &context,
                QUERY_LAB_ORIGIN,
                translation,
                boxddd::QueryFilter::default(),
            ) {
                Ok(hits) => {
                    diagnostics.query_ray_supported = true;
                    diagnostics.query_ray_hit_count = hits.len();
                }
                Err(boxddd::Error::UnsupportedOnWasm) => {
                    diagnostics.query_ray_supported = false;
                    diagnostics.query_ray_hit_count = 0;
                }
                Err(_) => {
                    diagnostics.query_ray_supported = true;
                    diagnostics.query_ray_hit_count = 0;
                }
            }
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
            match overlap_aabb(
                &context,
                lower_bound,
                upper_bound,
                boxddd::QueryFilter::default(),
            ) {
                Ok(hits) => {
                    diagnostics.query_overlap_supported = true;
                    diagnostics.query_overlap_hit_count = hits.len();
                }
                Err(boxddd::Error::UnsupportedOnWasm) => {
                    diagnostics.query_overlap_supported = false;
                    diagnostics.query_overlap_hit_count = 0;
                }
                Err(_) => {
                    diagnostics.query_overlap_supported = true;
                    diagnostics.query_overlap_hit_count = 0;
                }
            };

            match run_shape_cast(&context, &state) {
                Ok(hits) => {
                    diagnostics.query_shape_cast_supported = true;
                    diagnostics.query_shape_cast_hit_count = hits.len();
                    diagnostics.query_shape_cast_closest_fraction =
                        hits.iter().map(|hit| hit.fraction).reduce(f32::min);
                }
                Err(boxddd::Error::UnsupportedOnWasm) => {
                    diagnostics.query_shape_cast_supported = false;
                    diagnostics.query_shape_cast_hit_count = 0;
                    diagnostics.query_shape_cast_closest_fraction = None;
                }
                Err(_) => {
                    diagnostics.query_shape_cast_supported = true;
                    diagnostics.query_shape_cast_hit_count = 0;
                    diagnostics.query_shape_cast_closest_fraction = None;
                }
            }

            match run_mover_cast(&context, &state) {
                Ok(result) => {
                    diagnostics.query_mover_supported = true;
                    diagnostics.query_mover_planes_supported = result.planes_supported;
                    diagnostics.query_mover_fraction = Some(result.fraction);
                    diagnostics.query_mover_plane_count = result.planes.len();
                }
                Err(boxddd::Error::UnsupportedOnWasm) => {
                    diagnostics.query_mover_supported = false;
                    diagnostics.query_mover_planes_supported = false;
                    diagnostics.query_mover_fraction = None;
                    diagnostics.query_mover_plane_count = 0;
                }
                Err(_) => {
                    diagnostics.query_mover_supported = true;
                    diagnostics.query_mover_planes_supported = true;
                    diagnostics.query_mover_fraction = None;
                    diagnostics.query_mover_plane_count = 0;
                }
            }
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

    let shape_translation = query_lab_shape_cast_translation(&state);
    let shape_radius = query_lab_shape_cast_radius(&state);
    draw_sphere_cast(
        &mut gizmos,
        QUERY_LAB_SHAPE_CAST_ORIGIN,
        shape_translation,
        shape_radius,
    );

    if let Ok(hits) = run_shape_cast(&context, &state) {
        if let Some(hit) = hits.iter().min_by(|left, right| {
            left.fraction
                .partial_cmp(&right.fraction)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            let hit_center = QUERY_LAB_SHAPE_CAST_ORIGIN + shape_translation * hit.fraction;
            gizmos.sphere(hit_center, shape_radius, Color::srgb(0.98, 0.52, 0.18));
            gizmos.sphere(
                hit.point.to_bevy_vec3(),
                0.08,
                Color::srgb(0.98, 0.52, 0.18),
            );
            gizmos.line(
                hit.point.to_bevy_vec3(),
                hit.point.to_bevy_vec3() + hit.normal.to_bevy_vec3() * 0.35,
                Color::srgb(0.98, 0.52, 0.18),
            );
        }
    }

    draw_mover_cast(&mut gizmos, &context, &state);
}

impl LabDiagnostics {
    fn clear_query_counts(&mut self) {
        self.query_ray_supported = false;
        self.query_ray_hit_count = 0;
        self.query_overlap_supported = false;
        self.query_overlap_hit_count = 0;
        self.query_closest_fraction = None;
        self.query_shape_cast_supported = false;
        self.query_shape_cast_hit_count = 0;
        self.query_shape_cast_closest_fraction = None;
        self.query_mover_supported = false;
        self.query_mover_fraction = None;
        self.query_mover_planes_supported = false;
        self.query_mover_plane_count = 0;
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

pub(crate) fn query_lab_shape_cast_translation(state: &TestbedState) -> Vec3 {
    Vec3::X
        * state
            .query_lab_shape_cast_length
            .clamp(MIN_QUERY_SHAPE_CAST_LENGTH, MAX_QUERY_SHAPE_CAST_LENGTH)
}

pub(crate) fn query_lab_shape_cast_radius(state: &TestbedState) -> f32 {
    state
        .query_lab_shape_cast_radius
        .clamp(MIN_QUERY_SHAPE_CAST_RADIUS, MAX_QUERY_SHAPE_CAST_RADIUS)
}

pub(crate) fn query_lab_mover_cast_translation(state: &TestbedState) -> Vec3 {
    Vec3::X
        * state
            .query_lab_mover_cast_length
            .clamp(MIN_QUERY_MOVER_CAST_LENGTH, MAX_QUERY_MOVER_CAST_LENGTH)
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

fn run_shape_cast(
    context: &BoxdddPhysicsContext,
    state: &TestbedState,
) -> boxddd::Result<Vec<boxddd::RayHit>> {
    let world = context.world().ok_or(boxddd::Error::InvalidWorldId)?;
    let proxy = boxddd::ShapeProxy::sphere(query_lab_shape_cast_radius(state))?;
    let input = boxddd::ShapeCastInput::new(
        proxy,
        query_lab_shape_cast_translation(state).to_boxddd_vec3(),
    )?;
    world.cast_shape(
        QUERY_LAB_SHAPE_CAST_ORIGIN.to_boxddd_pos(),
        input,
        boxddd::QueryFilter::default(),
    )
}

fn run_mover_cast(
    context: &BoxdddPhysicsContext,
    state: &TestbedState,
) -> boxddd::Result<MoverCastResult> {
    let world = context.world().ok_or(boxddd::Error::InvalidWorldId)?;
    let mover = query_lab_mover();
    let translation = query_lab_mover_cast_translation(state);
    let fraction = world.cast_mover(
        QUERY_LAB_MOVER_ORIGIN.to_boxddd_pos(),
        &mover,
        translation.to_boxddd_vec3(),
        boxddd::QueryFilter::default(),
    )?;
    let final_origin = QUERY_LAB_MOVER_ORIGIN + translation * fraction;
    let (planes, planes_supported) = match world.collide_mover(
        final_origin.to_boxddd_pos(),
        &mover,
        boxddd::QueryFilter::default(),
    ) {
        Ok(planes) => (planes, true),
        Err(boxddd::Error::UnsupportedOnWasm) => (Vec::new(), false),
        Err(error) => return Err(error),
    };
    Ok(MoverCastResult {
        fraction,
        planes,
        planes_supported,
    })
}

struct MoverCastResult {
    fraction: f32,
    planes: Vec<boxddd::MoverPlane>,
    planes_supported: bool,
}

fn query_lab_mover() -> boxddd::Capsule {
    boxddd::Capsule::new(
        QUERY_LAB_MOVER_POINT1.to_boxddd_vec3(),
        QUERY_LAB_MOVER_POINT2.to_boxddd_vec3(),
        QUERY_LAB_MOVER_RADIUS,
    )
}

fn draw_sphere_cast(gizmos: &mut Gizmos, origin: Vec3, translation: Vec3, radius: f32) {
    let requested_end = origin + translation;
    gizmos.line(origin, requested_end, Color::srgb(0.78, 0.44, 0.95));
    gizmos.sphere(origin, radius, Color::srgb(0.78, 0.44, 0.95));
    gizmos.sphere(requested_end, radius, Color::srgb(0.45, 0.36, 0.55));
}

fn draw_mover_cast(gizmos: &mut Gizmos, context: &BoxdddPhysicsContext, state: &TestbedState) {
    let translation = query_lab_mover_cast_translation(state);
    let requested_end = QUERY_LAB_MOVER_ORIGIN + translation;
    draw_capsule(
        gizmos,
        QUERY_LAB_MOVER_ORIGIN,
        Color::srgb(0.24, 0.78, 0.72),
    );
    draw_capsule(gizmos, requested_end, Color::srgb(0.25, 0.44, 0.46));
    gizmos.line(
        QUERY_LAB_MOVER_ORIGIN + Vec3::Y * 0.8,
        requested_end + Vec3::Y * 0.8,
        Color::srgb(0.24, 0.78, 0.72),
    );

    let Ok(result) = run_mover_cast(context, state) else {
        return;
    };
    let safe_end = QUERY_LAB_MOVER_ORIGIN + translation * result.fraction;
    draw_capsule(gizmos, safe_end, Color::srgb(0.98, 0.88, 0.20));

    for plane in result.planes {
        let point = plane.point.to_bevy_vec3();
        let normal = plane.plane.normal.to_bevy_vec3();
        gizmos.line(point, point + normal * 0.45, Color::srgb(0.98, 0.88, 0.20));
    }
}

fn draw_capsule(gizmos: &mut Gizmos, origin: Vec3, color: Color) {
    let point1 = origin + QUERY_LAB_MOVER_POINT1;
    let point2 = origin + QUERY_LAB_MOVER_POINT2;
    gizmos.line(point1, point2, color);
    gizmos.sphere(point1, QUERY_LAB_MOVER_RADIUS, color);
    gizmos.sphere(point2, QUERY_LAB_MOVER_RADIUS, color);
}
