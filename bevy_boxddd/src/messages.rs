//! Bevy messages emitted by the physics plugin.

use bevy_ecs::prelude::{Entity, Message};
use boxddd::{
    BodyId, ContactId, Error, ShapeId, Vec3 as BoxdddVec3, WorldTransform as BoxdddWorldTransform,
};

/// Plugin operation associated with a recoverable error message.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BoxdddOperation {
    /// Creating the native Box3D world.
    CreateWorld,
    /// Creating a native body from a [`crate::RigidBody`] entity.
    CreateBody,
    /// Creating a native shape from a [`crate::Collider`] entity.
    CreateShape,
    /// Creating a native joint from a [`crate::Joint`] entity.
    CreateJoint,
    /// Destroying a native body after ECS removal or descriptor invalidation.
    DestroyBody,
    /// Destroying a native shape after ECS removal or descriptor invalidation.
    DestroyShape,
    /// Destroying a native joint after ECS removal or descriptor invalidation.
    DestroyJoint,
    /// Applying velocity, force, or impulse components.
    ApplyBodyControl,
    /// Configuring Bevy's fixed timestep resource.
    ConfigureFixedTimestep,
    /// Synchronizing transforms between Bevy and Box3D.
    SyncTransform,
    /// Stepping the native Box3D world.
    StepWorld,
    /// Reading body, contact, or sensor events after a step.
    ReadEvents,
    /// Collecting Box3D debug draw commands.
    DebugDraw,
}

/// Recoverable plugin error routed through Bevy messages.
#[derive(Message, Copy, Clone, Debug, Eq, PartialEq)]
pub struct BoxdddErrorMessage {
    /// Operation that produced the error.
    pub operation: BoxdddOperation,
    /// Entity associated with the operation, when one exists.
    pub entity: Option<Entity>,
    /// The underlying `boxddd` error.
    pub error: Error,
}

/// Body transform notification emitted after a successful physics step.
#[derive(Message, Clone, Debug, PartialEq)]
pub struct BoxdddBodyMoveMessage {
    /// Native body id that moved.
    pub body_id: BodyId,
    /// Bevy entity mapped to the body id, if owned by this plugin.
    pub entity: Option<Entity>,
    /// Current Box3D world transform.
    pub transform: BoxdddWorldTransform,
    /// Whether the body fell asleep during the step.
    pub fell_asleep: bool,
}

/// Contact begin notification emitted after a successful physics step.
#[derive(Message, Copy, Clone, Debug, Eq, PartialEq)]
pub struct BoxdddContactBeginMessage {
    /// First native shape in the contact pair.
    pub shape_a: ShapeId,
    /// Second native shape in the contact pair.
    pub shape_b: ShapeId,
    /// Bevy entity mapped to `shape_a`, if owned by this plugin.
    pub entity_a: Option<Entity>,
    /// Bevy entity mapped to `shape_b`, if owned by this plugin.
    pub entity_b: Option<Entity>,
    /// Native contact id.
    pub contact_id: ContactId,
}

/// Contact end notification emitted after a successful physics step.
#[derive(Message, Copy, Clone, Debug, Eq, PartialEq)]
pub struct BoxdddContactEndMessage {
    /// First native shape in the contact pair.
    pub shape_a: ShapeId,
    /// Second native shape in the contact pair.
    pub shape_b: ShapeId,
    /// Bevy entity mapped to `shape_a`, if owned by this plugin.
    pub entity_a: Option<Entity>,
    /// Bevy entity mapped to `shape_b`, if owned by this plugin.
    pub entity_b: Option<Entity>,
    /// Native contact id.
    pub contact_id: ContactId,
}

/// High-speed contact hit notification emitted after a successful physics step.
#[derive(Message, Copy, Clone, Debug, PartialEq)]
pub struct BoxdddContactHitMessage {
    /// First native shape in the contact pair.
    pub shape_a: ShapeId,
    /// Second native shape in the contact pair.
    pub shape_b: ShapeId,
    /// Bevy entity mapped to `shape_a`, if owned by this plugin.
    pub entity_a: Option<Entity>,
    /// Bevy entity mapped to `shape_b`, if owned by this plugin.
    pub entity_b: Option<Entity>,
    /// Native contact id.
    pub contact_id: ContactId,
    /// Contact point reported by Box3D.
    pub point: boxddd::Pos,
    /// Contact normal reported by Box3D.
    pub normal: BoxdddVec3,
    /// Relative approach speed for the hit.
    pub approach_speed: f32,
    /// Surface material id for `shape_a`.
    pub user_material_id_a: u64,
    /// Surface material id for `shape_b`.
    pub user_material_id_b: u64,
}

/// Sensor overlap begin notification emitted after a successful physics step.
#[derive(Message, Copy, Clone, Debug, Eq, PartialEq)]
pub struct BoxdddSensorBeginMessage {
    /// Native sensor shape.
    pub sensor_shape: ShapeId,
    /// Native shape entering the sensor.
    pub visitor_shape: ShapeId,
    /// Bevy entity mapped to the sensor shape, if owned by this plugin.
    pub sensor_entity: Option<Entity>,
    /// Bevy entity mapped to the visitor shape, if owned by this plugin.
    pub visitor_entity: Option<Entity>,
}

/// Sensor overlap end notification emitted after a successful physics step.
#[derive(Message, Copy, Clone, Debug, Eq, PartialEq)]
pub struct BoxdddSensorEndMessage {
    /// Native sensor shape.
    pub sensor_shape: ShapeId,
    /// Native shape leaving the sensor.
    pub visitor_shape: ShapeId,
    /// Bevy entity mapped to the sensor shape, if owned by this plugin.
    pub sensor_entity: Option<Entity>,
    /// Bevy entity mapped to the visitor shape, if owned by this plugin.
    pub visitor_entity: Option<Entity>,
}
