use crate::components::{
    AngularVelocity, BoxdddBody, BoxdddShape, Collider, ExternalForce, ExternalImpulse,
    LinearVelocity, PhysicsMaterial, RigidBody, TransformSyncMode,
};
use crate::errors::report_error;
use crate::messages::{
    BoxdddBodyMoveMessage, BoxdddContactBeginMessage, BoxdddContactEndMessage,
    BoxdddContactHitMessage, BoxdddErrorMessage, BoxdddOperation, BoxdddSensorBeginMessage,
    BoxdddSensorEndMessage,
};
use crate::resources::{BoxdddPhysicsContext, BoxdddPhysicsSettings};
use bevy_ecs::message::MessageWriter;
use bevy_ecs::prelude::{Commands, Entity, NonSendMut, Query, Res, With, Without};
use bevy_math::{Quat, Vec3};
use bevy_time::{Fixed, Time};
use bevy_transform::components::Transform;
use boxddd::{
    BodyDef, BodyId, BoxHull, Capsule as BoxdddCapsule, ShapeDef, ShapeId, Sphere as BoxdddSphere,
    WorldTransform as BoxdddWorldTransform,
};

pub fn create_missing_bodies(
    mut commands: Commands,
    mut context: NonSendMut<BoxdddPhysicsContext>,
    settings: Res<BoxdddPhysicsSettings>,
    mut errors: MessageWriter<BoxdddErrorMessage>,
    bodies: Query<
        (
            Entity,
            &RigidBody,
            Option<&Transform>,
            Option<&LinearVelocity>,
            Option<&AngularVelocity>,
        ),
        Without<BoxdddBody>,
    >,
) {
    if context.world().is_none() {
        return;
    }

    for (entity, rigid_body, transform, linear_velocity, angular_velocity) in &bodies {
        let mut def = BodyDef::builder().body_type((*rigid_body).into());

        if let Some(transform) = transform {
            def = def
                .position(to_boxddd_pos(transform.translation))
                .rotation(to_boxddd_quat(transform.rotation));
        }

        if let Some(linear_velocity) = linear_velocity {
            def = def.linear_velocity(to_boxddd_vec3(linear_velocity.0));
        }

        if let Some(angular_velocity) = angular_velocity {
            def = def.angular_velocity(to_boxddd_vec3(angular_velocity.0));
        }

        let result = context
            .world_mut()
            .expect("checked above")
            .try_create_body(def.build());

        match result {
            Ok(body_id) => {
                context.insert_body(entity, body_id);
                commands.entity(entity).insert(BoxdddBody(body_id));
            }
            Err(error) => report_error(
                &settings,
                &mut errors,
                BoxdddErrorMessage {
                    operation: BoxdddOperation::CreateBody,
                    entity: Some(entity),
                    error,
                },
            ),
        }
    }
}

pub fn create_missing_shapes(
    mut commands: Commands,
    mut context: NonSendMut<BoxdddPhysicsContext>,
    settings: Res<BoxdddPhysicsSettings>,
    mut errors: MessageWriter<BoxdddErrorMessage>,
    colliders: Query<
        (Entity, &BoxdddBody, &Collider, Option<&PhysicsMaterial>),
        Without<BoxdddShape>,
    >,
) {
    if context.world().is_none() {
        return;
    }

    for (entity, body, collider, material) in &colliders {
        let shape_def = material.copied().unwrap_or_default().shape_def();
        let result = collider.validate().and_then(|()| {
            create_shape(
                context.world_mut().expect("checked above"),
                body.0,
                *collider,
                &shape_def,
            )
        });

        match result {
            Ok(shape_id) => {
                context.insert_shape(entity, shape_id);
                commands.entity(entity).insert(BoxdddShape(shape_id));
            }
            Err(error) => report_error(
                &settings,
                &mut errors,
                BoxdddErrorMessage {
                    operation: BoxdddOperation::CreateShape,
                    entity: Some(entity),
                    error,
                },
            ),
        }
    }
}

pub fn cleanup_removed_colliders(
    mut commands: Commands,
    mut context: NonSendMut<BoxdddPhysicsContext>,
    settings: Res<BoxdddPhysicsSettings>,
    mut errors: MessageWriter<BoxdddErrorMessage>,
    shapes: Query<(Entity, &BoxdddShape, Option<&Collider>)>,
) {
    if context.world().is_none() {
        return;
    }

    for (entity, shape, collider) in &shapes {
        if collider.is_some() {
            continue;
        }

        let result = context
            .world_mut()
            .expect("checked above")
            .try_destroy_shape(shape.0, true);

        match result {
            Ok(()) => {
                context.remove_shape(entity, shape.0);
                commands.entity(entity).remove::<BoxdddShape>();
            }
            Err(error) => report_error(
                &settings,
                &mut errors,
                BoxdddErrorMessage {
                    operation: BoxdddOperation::DestroyShape,
                    entity: Some(entity),
                    error,
                },
            ),
        }
    }
}

pub fn cleanup_removed_bodies(
    mut commands: Commands,
    mut context: NonSendMut<BoxdddPhysicsContext>,
    settings: Res<BoxdddPhysicsSettings>,
    mut errors: MessageWriter<BoxdddErrorMessage>,
    bodies: Query<(), With<BoxdddBody>>,
    shapes: Query<Option<&BoxdddShape>>,
) {
    if context.world().is_none() {
        return;
    }

    let stale = context
        .entity_to_body
        .iter()
        .filter_map(|(entity, body_id)| bodies.get(*entity).is_err().then_some((*entity, *body_id)))
        .map(|(entity, body_id)| {
            let shape = shapes.get(entity).ok().flatten().copied();
            (entity, body_id, shape)
        })
        .collect::<Vec<_>>();

    for (entity, body_id, shape) in stale {
        let result = context
            .world_mut()
            .expect("checked above")
            .try_destroy_body(body_id);

        match result {
            Ok(()) => {
                context.remove_body(entity, body_id);
                if shape.is_some() {
                    commands.entity(entity).remove::<BoxdddShape>();
                }
            }
            Err(error) => report_error(
                &settings,
                &mut errors,
                BoxdddErrorMessage {
                    operation: BoxdddOperation::DestroyBody,
                    entity: Some(entity),
                    error,
                },
            ),
        }
    }
}

pub fn apply_body_controls(
    mut commands: Commands,
    mut context: NonSendMut<BoxdddPhysicsContext>,
    settings: Res<BoxdddPhysicsSettings>,
    mut errors: MessageWriter<BoxdddErrorMessage>,
    controls: Query<(
        Entity,
        &BoxdddBody,
        Option<&LinearVelocity>,
        Option<&AngularVelocity>,
        Option<&ExternalForce>,
        Option<&ExternalImpulse>,
    )>,
) {
    if context.world().is_none() {
        return;
    }

    for (entity, body, linear_velocity, angular_velocity, force, impulse) in &controls {
        if let Some(linear_velocity) = linear_velocity {
            apply_control_result(
                &settings,
                &mut errors,
                entity,
                context
                    .world_mut()
                    .expect("checked above")
                    .try_set_body_linear_velocity(body.0, to_boxddd_vec3(linear_velocity.0)),
            );
        }

        if let Some(angular_velocity) = angular_velocity {
            apply_control_result(
                &settings,
                &mut errors,
                entity,
                context
                    .world_mut()
                    .expect("checked above")
                    .try_set_body_angular_velocity(body.0, to_boxddd_vec3(angular_velocity.0)),
            );
        }

        if let Some(force) = force {
            let result = match force.point {
                Some(point) => context.world_mut().expect("checked above").try_apply_force(
                    body.0,
                    to_boxddd_vec3(force.force),
                    to_boxddd_pos(point),
                    force.wake,
                ),
                None => context
                    .world_mut()
                    .expect("checked above")
                    .try_apply_force_to_center(body.0, to_boxddd_vec3(force.force), force.wake),
            };
            apply_control_result(&settings, &mut errors, entity, result);
        }

        if let Some(impulse) = impulse {
            let result = match impulse.point {
                Some(point) => context
                    .world_mut()
                    .expect("checked above")
                    .try_apply_linear_impulse(
                        body.0,
                        to_boxddd_vec3(impulse.impulse),
                        to_boxddd_pos(point),
                        impulse.wake,
                    ),
                None => context
                    .world_mut()
                    .expect("checked above")
                    .try_apply_linear_impulse_to_center(
                        body.0,
                        to_boxddd_vec3(impulse.impulse),
                        impulse.wake,
                    ),
            };
            apply_control_result(&settings, &mut errors, entity, result);
            commands.entity(entity).remove::<ExternalImpulse>();
        }
    }
}

pub fn sync_bevy_transforms_to_boxddd(
    mut context: NonSendMut<BoxdddPhysicsContext>,
    settings: Res<BoxdddPhysicsSettings>,
    mut errors: MessageWriter<BoxdddErrorMessage>,
    bodies: Query<(
        Entity,
        &BoxdddBody,
        &Transform,
        Option<&TransformSyncMode>,
        Option<&RigidBody>,
    )>,
) {
    if context.world().is_none() {
        return;
    }

    for (entity, body, transform, sync_mode, rigid_body) in &bodies {
        if effective_sync_mode(sync_mode, rigid_body) != TransformSyncMode::BevyToPhysics {
            continue;
        }

        let result = context
            .world_mut()
            .expect("checked above")
            .try_set_body_transform(
                body.0,
                to_boxddd_pos(transform.translation),
                to_boxddd_quat(transform.rotation),
            );

        if let Err(error) = result {
            report_error(
                &settings,
                &mut errors,
                BoxdddErrorMessage {
                    operation: BoxdddOperation::SyncTransform,
                    entity: Some(entity),
                    error,
                },
            );
        }
    }
}

pub fn step_world(
    mut context: NonSendMut<BoxdddPhysicsContext>,
    settings: Res<BoxdddPhysicsSettings>,
    time: Res<Time<Fixed>>,
    mut errors: MessageWriter<BoxdddErrorMessage>,
) {
    let Some(world) = context.world_mut() else {
        return;
    };

    match world.try_step(time.delta_secs(), settings.sub_step_count) {
        Ok(()) => {
            context.last_step_failed = false;
        }
        Err(error) => {
            context.last_step_failed = true;
            report_error(
                &settings,
                &mut errors,
                BoxdddErrorMessage {
                    operation: BoxdddOperation::StepWorld,
                    entity: None,
                    error,
                },
            );
        }
    }
}

pub fn publish_physics_messages(
    context: NonSendMut<BoxdddPhysicsContext>,
    settings: Res<BoxdddPhysicsSettings>,
    mut errors: MessageWriter<BoxdddErrorMessage>,
    mut body_moves: MessageWriter<BoxdddBodyMoveMessage>,
    mut contact_begin: MessageWriter<BoxdddContactBeginMessage>,
    mut contact_end: MessageWriter<BoxdddContactEndMessage>,
    mut contact_hit: MessageWriter<BoxdddContactHitMessage>,
    mut sensor_begin: MessageWriter<BoxdddSensorBeginMessage>,
    mut sensor_end: MessageWriter<BoxdddSensorEndMessage>,
) {
    if context.last_step_failed {
        return;
    }

    let Some(world) = context.world() else {
        return;
    };

    match world.try_body_events() {
        Ok(events) => {
            for event in events {
                body_moves.write(BoxdddBodyMoveMessage {
                    body_id: event.body_id,
                    entity: context.body_entity(event.body_id),
                    transform: event.transform,
                    fell_asleep: event.fell_asleep,
                });
            }
        }
        Err(error) => report_error(
            &settings,
            &mut errors,
            BoxdddErrorMessage {
                operation: BoxdddOperation::ReadEvents,
                entity: None,
                error,
            },
        ),
    }

    match world.try_contact_events() {
        Ok(events) => {
            for event in events.begin {
                contact_begin.write(BoxdddContactBeginMessage {
                    shape_a: event.shape_a,
                    shape_b: event.shape_b,
                    entity_a: context.shape_entity(event.shape_a),
                    entity_b: context.shape_entity(event.shape_b),
                    contact_id: event.contact_id,
                });
            }

            for event in events.end {
                contact_end.write(BoxdddContactEndMessage {
                    shape_a: event.shape_a,
                    shape_b: event.shape_b,
                    entity_a: context.shape_entity(event.shape_a),
                    entity_b: context.shape_entity(event.shape_b),
                    contact_id: event.contact_id,
                });
            }

            for event in events.hit {
                contact_hit.write(BoxdddContactHitMessage {
                    shape_a: event.shape_a,
                    shape_b: event.shape_b,
                    entity_a: context.shape_entity(event.shape_a),
                    entity_b: context.shape_entity(event.shape_b),
                    contact_id: event.contact_id,
                    point: event.point,
                    normal: event.normal,
                    approach_speed: event.approach_speed,
                    user_material_id_a: event.user_material_id_a,
                    user_material_id_b: event.user_material_id_b,
                });
            }
        }
        Err(error) => report_error(
            &settings,
            &mut errors,
            BoxdddErrorMessage {
                operation: BoxdddOperation::ReadEvents,
                entity: None,
                error,
            },
        ),
    }

    match world.try_sensor_events() {
        Ok(events) => {
            for event in events.begin {
                sensor_begin.write(BoxdddSensorBeginMessage {
                    sensor_shape: event.sensor_shape,
                    visitor_shape: event.visitor_shape,
                    sensor_entity: context.shape_entity(event.sensor_shape),
                    visitor_entity: context.shape_entity(event.visitor_shape),
                });
            }

            for event in events.end {
                sensor_end.write(BoxdddSensorEndMessage {
                    sensor_shape: event.sensor_shape,
                    visitor_shape: event.visitor_shape,
                    sensor_entity: context.shape_entity(event.sensor_shape),
                    visitor_entity: context.shape_entity(event.visitor_shape),
                });
            }
        }
        Err(error) => report_error(
            &settings,
            &mut errors,
            BoxdddErrorMessage {
                operation: BoxdddOperation::ReadEvents,
                entity: None,
                error,
            },
        ),
    }
}

pub fn sync_boxddd_transforms_to_bevy(
    context: NonSendMut<BoxdddPhysicsContext>,
    settings: Res<BoxdddPhysicsSettings>,
    mut errors: MessageWriter<BoxdddErrorMessage>,
    mut bodies: Query<(
        Entity,
        &BoxdddBody,
        &mut Transform,
        Option<&TransformSyncMode>,
        Option<&RigidBody>,
    )>,
) {
    if context.last_step_failed || context.world().is_none() {
        return;
    }

    for (entity, body, mut transform, sync_mode, rigid_body) in &mut bodies {
        if effective_sync_mode(sync_mode, rigid_body) != TransformSyncMode::PhysicsToBevy {
            continue;
        }

        let result = context
            .world()
            .expect("checked above")
            .try_body_transform(body.0);

        match result {
            Ok(boxddd_transform) => apply_boxddd_transform(&mut transform, boxddd_transform),
            Err(error) => report_error(
                &settings,
                &mut errors,
                BoxdddErrorMessage {
                    operation: BoxdddOperation::SyncTransform,
                    entity: Some(entity),
                    error,
                },
            ),
        }
    }
}

fn create_shape(
    world: &mut boxddd::World,
    body_id: BodyId,
    collider: Collider,
    shape_def: &ShapeDef,
) -> boxddd::Result<ShapeId> {
    match collider {
        Collider::Cuboid { half_extents } => world.try_create_hull_shape(
            body_id,
            shape_def,
            &BoxHull::new(half_extents.x, half_extents.y, half_extents.z),
        ),
        Collider::Sphere { radius, center } => world.try_create_sphere_shape(
            body_id,
            shape_def,
            &BoxdddSphere::new(to_boxddd_vec3(center), radius),
        ),
        Collider::Capsule {
            point1,
            point2,
            radius,
        } => world.try_create_capsule_shape(
            body_id,
            shape_def,
            &BoxdddCapsule::new(to_boxddd_vec3(point1), to_boxddd_vec3(point2), radius),
        ),
    }
}

fn apply_control_result(
    settings: &BoxdddPhysicsSettings,
    errors: &mut MessageWriter<'_, BoxdddErrorMessage>,
    entity: Entity,
    result: boxddd::Result<()>,
) {
    if let Err(error) = result {
        report_error(
            settings,
            errors,
            BoxdddErrorMessage {
                operation: BoxdddOperation::ApplyBodyControl,
                entity: Some(entity),
                error,
            },
        );
    }
}

fn effective_sync_mode(
    mode: Option<&TransformSyncMode>,
    rigid_body: Option<&RigidBody>,
) -> TransformSyncMode {
    mode.copied().unwrap_or(match rigid_body.copied() {
        Some(RigidBody::Static | RigidBody::Kinematic) => TransformSyncMode::BevyToPhysics,
        Some(RigidBody::Dynamic) | None => TransformSyncMode::PhysicsToBevy,
    })
}

fn to_boxddd_vec3(value: Vec3) -> boxddd::Vec3 {
    boxddd::Vec3::new(value.x, value.y, value.z)
}

fn to_boxddd_pos(value: Vec3) -> boxddd::Pos {
    boxddd::Pos::new(value.x.into(), value.y.into(), value.z.into())
}

fn to_boxddd_quat(value: Quat) -> boxddd::Quat {
    boxddd::Quat::new(boxddd::Vec3::new(value.x, value.y, value.z), value.w)
}

fn to_bevy_vec3(value: boxddd::Pos) -> Vec3 {
    Vec3::new(value.x as f32, value.y as f32, value.z as f32)
}

fn to_bevy_quat(value: boxddd::Quat) -> Quat {
    Quat::from_xyzw(value.v.x, value.v.y, value.v.z, value.s)
}

fn apply_boxddd_transform(transform: &mut Transform, boxddd_transform: BoxdddWorldTransform) {
    transform.translation = to_bevy_vec3(boxddd_transform.p);
    transform.rotation = to_bevy_quat(boxddd_transform.q);
}
