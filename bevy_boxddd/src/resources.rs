use bevy_ecs::prelude::{Entity, Resource};
use bevy_math::Vec3;
use boxddd::{BodyId, ShapeId, World, WorldDef};
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

    pub(crate) fn insert_body(&mut self, entity: Entity, body_id: BodyId) {
        self.entity_to_body.insert(entity, body_id);
        self.body_to_entity.insert(body_id, entity);
    }

    pub(crate) fn remove_body(&mut self, entity: Entity, body_id: BodyId) {
        self.entity_to_body.remove(&entity);
        self.body_to_entity.remove(&body_id);
        let shapes = self
            .entity_to_shape
            .iter()
            .filter_map(|(shape_entity, shape_id)| (*shape_entity == entity).then_some(*shape_id))
            .collect::<Vec<_>>();
        for shape_id in shapes {
            self.remove_shape(entity, shape_id);
        }
    }

    pub(crate) fn insert_shape(&mut self, entity: Entity, shape_id: ShapeId) {
        self.entity_to_shape.insert(entity, shape_id);
        self.shape_to_entity.insert(shape_id, entity);
    }

    pub(crate) fn remove_shape(&mut self, entity: Entity, shape_id: ShapeId) {
        self.entity_to_shape.remove(&entity);
        self.shape_to_entity.remove(&shape_id);
    }
}
