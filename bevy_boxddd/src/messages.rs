use bevy_ecs::prelude::{Entity, Message};
use boxddd::{
    BodyId, ContactId, Error, ShapeId, Vec3 as BoxdddVec3, WorldTransform as BoxdddWorldTransform,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BoxdddOperation {
    CreateWorld,
    CreateBody,
    CreateShape,
    CreateJoint,
    DestroyBody,
    DestroyShape,
    DestroyJoint,
    ApplyBodyControl,
    ConfigureFixedTimestep,
    SyncTransform,
    StepWorld,
    ReadEvents,
    DebugDraw,
}

#[derive(Message, Copy, Clone, Debug, Eq, PartialEq)]
pub struct BoxdddErrorMessage {
    pub operation: BoxdddOperation,
    pub entity: Option<Entity>,
    pub error: Error,
}

#[derive(Message, Clone, Debug, PartialEq)]
pub struct BoxdddBodyMoveMessage {
    pub body_id: BodyId,
    pub entity: Option<Entity>,
    pub transform: BoxdddWorldTransform,
    pub fell_asleep: bool,
}

#[derive(Message, Copy, Clone, Debug, Eq, PartialEq)]
pub struct BoxdddContactBeginMessage {
    pub shape_a: ShapeId,
    pub shape_b: ShapeId,
    pub entity_a: Option<Entity>,
    pub entity_b: Option<Entity>,
    pub contact_id: ContactId,
}

#[derive(Message, Copy, Clone, Debug, Eq, PartialEq)]
pub struct BoxdddContactEndMessage {
    pub shape_a: ShapeId,
    pub shape_b: ShapeId,
    pub entity_a: Option<Entity>,
    pub entity_b: Option<Entity>,
    pub contact_id: ContactId,
}

#[derive(Message, Copy, Clone, Debug, PartialEq)]
pub struct BoxdddContactHitMessage {
    pub shape_a: ShapeId,
    pub shape_b: ShapeId,
    pub entity_a: Option<Entity>,
    pub entity_b: Option<Entity>,
    pub contact_id: ContactId,
    pub point: boxddd::Pos,
    pub normal: BoxdddVec3,
    pub approach_speed: f32,
    pub user_material_id_a: u64,
    pub user_material_id_b: u64,
}

#[derive(Message, Copy, Clone, Debug, Eq, PartialEq)]
pub struct BoxdddSensorBeginMessage {
    pub sensor_shape: ShapeId,
    pub visitor_shape: ShapeId,
    pub sensor_entity: Option<Entity>,
    pub visitor_entity: Option<Entity>,
}

#[derive(Message, Copy, Clone, Debug, Eq, PartialEq)]
pub struct BoxdddSensorEndMessage {
    pub sensor_shape: ShapeId,
    pub visitor_shape: ShapeId,
    pub sensor_entity: Option<Entity>,
    pub visitor_entity: Option<Entity>,
}
