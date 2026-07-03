use bevy_ecs::prelude::{Entity, Resource};
use bevy_math::Vec3;
use boxddd::{BodyId, JointId, ShapeId, World, WorldDef};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum BoxdddErrorPolicy {
    #[default]
    MessageOnly,
    MessageAndLog,
    Panic,
}

#[derive(Resource, Clone, Debug)]
pub struct BoxdddPhysicsSettings {
    pub gravity: Vec3,
    pub sub_step_count: i32,
    pub fixed_timestep_seconds: Option<f64>,
    pub error_policy: BoxdddErrorPolicy,
}

impl Default for BoxdddPhysicsSettings {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -10.0, 0.0),
            sub_step_count: 4,
            fixed_timestep_seconds: Some(1.0 / 60.0),
            error_policy: BoxdddErrorPolicy::MessageOnly,
        }
    }
}

#[derive(Debug)]
pub struct BoxdddPhysicsContext {
    world: Option<World>,
    pub(crate) entity_to_body: HashMap<Entity, BodyId>,
    pub(crate) body_to_entity: HashMap<BodyId, Entity>,
    pub(crate) entity_to_shape: HashMap<Entity, ShapeId>,
    pub(crate) shape_to_entity: HashMap<ShapeId, Entity>,
    pub(crate) shape_to_body_entity: HashMap<Entity, Entity>,
    pub(crate) entity_to_joint: HashMap<Entity, JointId>,
    pub(crate) joint_to_entity: HashMap<JointId, Entity>,
    pub(crate) joint_to_body_entities: HashMap<Entity, (Entity, Entity)>,
    pub(crate) last_step_failed: bool,
}

impl BoxdddPhysicsContext {
    pub fn new(settings: &BoxdddPhysicsSettings) -> boxddd::Result<Self> {
        let gravity = boxddd::Vec3::new(settings.gravity.x, settings.gravity.y, settings.gravity.z);
        let world = World::new(WorldDef::builder().gravity(gravity).build())?;
        Ok(Self::from_world(world))
    }

    pub fn disabled() -> Self {
        Self {
            world: None,
            entity_to_body: HashMap::new(),
            body_to_entity: HashMap::new(),
            entity_to_shape: HashMap::new(),
            shape_to_entity: HashMap::new(),
            shape_to_body_entity: HashMap::new(),
            entity_to_joint: HashMap::new(),
            joint_to_entity: HashMap::new(),
            joint_to_body_entities: HashMap::new(),
            last_step_failed: true,
        }
    }

    pub fn from_world(world: World) -> Self {
        Self {
            world: Some(world),
            entity_to_body: HashMap::new(),
            body_to_entity: HashMap::new(),
            entity_to_shape: HashMap::new(),
            shape_to_entity: HashMap::new(),
            shape_to_body_entity: HashMap::new(),
            entity_to_joint: HashMap::new(),
            joint_to_entity: HashMap::new(),
            joint_to_body_entities: HashMap::new(),
            last_step_failed: false,
        }
    }

    pub fn world(&self) -> Option<&World> {
        self.world.as_ref()
    }

    pub fn world_mut(&mut self) -> Option<&mut World> {
        self.world.as_mut()
    }

    pub fn body_entity(&self, body_id: BodyId) -> Option<Entity> {
        self.body_to_entity.get(&body_id).copied()
    }

    pub fn shape_entity(&self, shape_id: ShapeId) -> Option<Entity> {
        self.shape_to_entity.get(&shape_id).copied()
    }

    pub fn joint_entity(&self, joint_id: JointId) -> Option<Entity> {
        self.joint_to_entity.get(&joint_id).copied()
    }

    pub(crate) fn insert_body(&mut self, entity: Entity, body_id: BodyId) {
        self.entity_to_body.insert(entity, body_id);
        self.body_to_entity.insert(body_id, entity);
    }

    pub(crate) fn remove_body(&mut self, entity: Entity, body_id: BodyId) {
        self.entity_to_body.remove(&entity);
        self.body_to_entity.remove(&body_id);
        let shapes = self
            .shape_to_body_entity
            .iter()
            .filter_map(|(shape_entity, body_entity)| {
                (*body_entity == entity).then_some(*shape_entity)
            })
            .collect::<Vec<_>>();
        for shape_entity in shapes {
            if let Some(shape_id) = self.entity_to_shape.get(&shape_entity).copied() {
                self.remove_shape(shape_entity, shape_id);
            }
        }

        let joints = self
            .joint_to_body_entities
            .iter()
            .filter_map(|(joint_entity, (body_a, body_b))| {
                (*body_a == entity || *body_b == entity).then_some(*joint_entity)
            })
            .collect::<Vec<_>>();
        for joint_entity in joints {
            if let Some(joint_id) = self.entity_to_joint.get(&joint_entity).copied() {
                self.remove_joint(joint_entity, joint_id);
            }
        }
    }

    pub(crate) fn insert_shape(&mut self, entity: Entity, body_entity: Entity, shape_id: ShapeId) {
        self.entity_to_shape.insert(entity, shape_id);
        self.shape_to_entity.insert(shape_id, entity);
        self.shape_to_body_entity.insert(entity, body_entity);
    }

    pub(crate) fn remove_shape(&mut self, entity: Entity, shape_id: ShapeId) {
        self.entity_to_shape.remove(&entity);
        self.shape_to_entity.remove(&shape_id);
        self.shape_to_body_entity.remove(&entity);
    }

    pub(crate) fn shape_body_entity(&self, shape_entity: Entity) -> Option<Entity> {
        self.shape_to_body_entity.get(&shape_entity).copied()
    }

    pub(crate) fn insert_joint(
        &mut self,
        entity: Entity,
        body_a: Entity,
        body_b: Entity,
        joint_id: JointId,
    ) {
        self.entity_to_joint.insert(entity, joint_id);
        self.joint_to_entity.insert(joint_id, entity);
        self.joint_to_body_entities.insert(entity, (body_a, body_b));
    }

    pub(crate) fn remove_joint(&mut self, entity: Entity, joint_id: JointId) {
        self.entity_to_joint.remove(&entity);
        self.joint_to_entity.remove(&joint_id);
        self.joint_to_body_entities.remove(&entity);
    }

    pub(crate) fn joint_body_entities(&self, joint_entity: Entity) -> Option<(Entity, Entity)> {
        self.joint_to_body_entities.get(&joint_entity).copied()
    }
}
