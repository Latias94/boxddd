use bevy::prelude::*;
use bevy::time::Real;
use bevy_boxddd::prelude::*;
use bevy_egui::input::EguiWantsInput;

const MAX_PICK_DISTANCE: f32 = 100.0;
const MIN_DRAG_DEPTH: f32 = 0.35;
const MAX_THROW_SPEED: f32 = 35.0;

#[derive(Resource, Default, Debug)]
pub(crate) struct PhysicsDragState {
    active: Option<DragTarget>,
}

#[derive(Copy, Clone, Debug)]
struct DragTarget {
    entity: Entity,
    body_id: boxddd::BodyId,
    previous_type: boxddd::BodyType,
    rotation: boxddd::Quat,
    depth: f32,
    grab_offset: Vec3,
    last_position: Vec3,
    last_velocity: Vec3,
}

pub(crate) fn update_physics_drag(
    buttons: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    egui_input: Option<Res<EguiWantsInput>>,
    real_time: Res<Time<Real>>,
    mut context: NonSendMut<BoxdddPhysicsContext>,
    mut drag: ResMut<PhysicsDragState>,
    mut bodies: Query<(&BoxdddBody, &mut Transform)>,
) {
    if buttons.just_released(MouseButton::Left) || !buttons.pressed(MouseButton::Left) {
        if let Some(target) = drag.active.take() {
            finish_drag(target, &mut context);
        }
        return;
    }

    let Some(ray) = cursor_ray(*camera, *window) else {
        return;
    };

    if let Some(target) = drag.active {
        drag.active = update_active_drag(target, ray, &real_time, &mut context, &mut bodies);
        return;
    }

    if !buttons.just_pressed(MouseButton::Left)
        || egui_input
            .as_ref()
            .is_some_and(|input| input.wants_any_pointer_input())
    {
        return;
    }

    let translation = ray.direction * MAX_PICK_DISTANCE;
    let Ok(Some(hit)) = cast_ray_closest(
        &context,
        ray.origin,
        translation,
        boxddd::QueryFilter::default(),
    ) else {
        return;
    };
    let Some(entity) = hit.entity else {
        return;
    };
    let Ok((body, _)) = bodies.get_mut(entity) else {
        return;
    };
    let body_id = body.id();

    let Some(world) = context.world_mut() else {
        return;
    };
    let Ok(previous_type) = world.try_body_type(body_id) else {
        return;
    };
    if previous_type != boxddd::BodyType::Dynamic {
        return;
    }
    let Ok(transform) = world.try_body_transform(body_id) else {
        return;
    };

    let body_position = transform.p.to_bevy_vec3();
    let direction = *ray.direction;
    let depth = (body_position - ray.origin)
        .dot(direction)
        .max(MIN_DRAG_DEPTH);
    let projected_position = point_on_ray(ray.origin, direction, depth);
    let _ = world.try_set_body_type(body_id, boxddd::BodyType::Kinematic);
    let _ = world.try_set_body_linear_velocity(body_id, boxddd::Vec3::ZERO);

    drag.active = Some(DragTarget {
        entity,
        body_id,
        previous_type,
        rotation: transform.q,
        depth,
        grab_offset: body_position - projected_position,
        last_position: body_position,
        last_velocity: Vec3::ZERO,
    });
}

pub(crate) fn draw_physics_pick(
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    context: NonSend<BoxdddPhysicsContext>,
    drag: Res<PhysicsDragState>,
    mut gizmos: Gizmos,
) {
    let Some(ray) = cursor_ray(*camera, *window) else {
        return;
    };

    if let Some(target) = drag.active {
        let cursor_anchor = point_on_ray(ray.origin, *ray.direction, target.depth);
        gizmos.ray(
            ray.origin,
            *ray.direction * target.depth,
            Color::srgb(0.35, 0.70, 1.0),
        );
        gizmos.line(
            cursor_anchor,
            target.last_position,
            Color::srgb(1.0, 0.78, 0.22),
        );
        gizmos.sphere(target.last_position, 0.16, Color::srgb(1.0, 0.78, 0.22));
        return;
    }

    let translation = ray.direction * MAX_PICK_DISTANCE;
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

fn update_active_drag(
    target: DragTarget,
    ray: Ray3d,
    real_time: &Time<Real>,
    context: &mut BoxdddPhysicsContext,
    bodies: &mut Query<(&BoxdddBody, &mut Transform)>,
) -> Option<DragTarget> {
    let dt = real_time.delta_secs().max(1.0 / 240.0);
    let target_position =
        point_on_ray(ray.origin, *ray.direction, target.depth) + target.grab_offset;
    let velocity = clamp_throw_velocity((target_position - target.last_position) / dt);
    let world_transform =
        boxddd::WorldTransform::new(target_position.to_boxddd_pos(), target.rotation);

    let Some(world) = context.world_mut() else {
        return None;
    };
    if world
        .try_set_body_target_transform(target.body_id, world_transform, dt, true)
        .or_else(|_| {
            world.try_set_body_transform(
                target.body_id,
                target_position.to_boxddd_pos(),
                target.rotation,
            )
        })
        .is_err()
    {
        return None;
    }
    let _ = world.try_set_body_linear_velocity(target.body_id, velocity.to_boxddd_vec3());

    if let Ok((_, mut transform)) = bodies.get_mut(target.entity) {
        transform.translation = target_position;
    } else {
        finish_drag(target, context);
        return None;
    }

    Some(DragTarget {
        last_position: target_position,
        last_velocity: velocity,
        ..target
    })
}

fn finish_drag(target: DragTarget, context: &mut BoxdddPhysicsContext) {
    let Some(world) = context.world_mut() else {
        return;
    };
    let _ = world.try_set_body_type(target.body_id, target.previous_type);
    if target.previous_type == boxddd::BodyType::Dynamic {
        let _ = world.try_set_body_linear_velocity(
            target.body_id,
            clamp_throw_velocity(target.last_velocity).to_boxddd_vec3(),
        );
    }
}

fn cursor_ray(
    (camera, camera_transform): (&Camera, &GlobalTransform),
    window: &Window,
) -> Option<Ray3d> {
    let cursor_position = window.cursor_position()?;
    camera
        .viewport_to_world(camera_transform, cursor_position)
        .ok()
}

pub(crate) fn point_on_ray(origin: Vec3, direction: Vec3, depth: f32) -> Vec3 {
    origin + direction.normalize_or_zero() * depth.max(MIN_DRAG_DEPTH)
}

pub(crate) fn clamp_throw_velocity(velocity: Vec3) -> Vec3 {
    velocity.clamp_length_max(MAX_THROW_SPEED)
}
