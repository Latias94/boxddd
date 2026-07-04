//! Bevy resources that own the native physics world and plugin settings.

use crate::components::{Collider, Joint, PhysicsMaterial};
use bevy_ecs::prelude::{Entity, Resource};
use bevy_math::{Quat, Vec3};
use boxddd::{BodyId, JointId, ShapeId, World, WorldDef};
use std::collections::HashMap;

/// How the plugin reports recoverable errors from fixed-update systems.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum BoxdddErrorPolicy {
    /// Emit [`crate::BoxdddErrorMessage`] only.
    #[default]
    MessageOnly,
    /// Emit [`crate::BoxdddErrorMessage`] and log the error.
    MessageAndLog,
    /// Panic immediately when a recoverable plugin error is observed.
    Panic,
}

/// Runtime settings used by [`crate::BoxdddPhysicsPlugin`].
#[derive(Resource, Clone, Debug)]
pub struct BoxdddPhysicsSettings {
    /// Gravity used when creating the native Box3D world.
    pub gravity: Vec3,
    /// Box3D sub-step count used for each fixed step.
    pub sub_step_count: i32,
    /// Optional Bevy fixed timestep override in seconds.
    pub fixed_timestep_seconds: Option<f64>,
    /// Error reporting policy for plugin systems.
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

/// Non-send resource that owns the native Box3D world and ECS id mappings.
///
/// `boxddd::World` is intentionally `!Send`/`!Sync`; Bevy apps should access
/// this resource from main-thread systems and move plain snapshots across
/// threads when needed.
#[derive(Debug)]
pub struct BoxdddPhysicsContext {
    world: Option<World>,
    pub(crate) entity_to_body: HashMap<Entity, BodyId>,
    pub(crate) body_to_entity: HashMap<BodyId, Entity>,
    pub(crate) entity_to_shape: HashMap<Entity, ShapeId>,
    pub(crate) shape_to_entity: HashMap<ShapeId, Entity>,
    pub(crate) shape_to_body_entity: HashMap<Entity, Entity>,
    pub(crate) shape_descriptors: HashMap<Entity, ShapeDescriptor>,
    pub(crate) entity_to_joint: HashMap<Entity, JointId>,
    pub(crate) joint_to_entity: HashMap<JointId, Entity>,
    pub(crate) joint_to_body_entities: HashMap<Entity, (Entity, Entity)>,
    pub(crate) joint_descriptors: HashMap<Entity, Joint>,
    pub(crate) last_step_failed: bool,
}

impl BoxdddPhysicsContext {
    /// Creates a context and native Box3D world from plugin settings.
    pub fn new(settings: &BoxdddPhysicsSettings) -> boxddd::Result<Self> {
        let gravity = boxddd::Vec3::new(settings.gravity.x, settings.gravity.y, settings.gravity.z);
        let world = World::new(WorldDef::builder().gravity(gravity).build())?;
        Ok(Self::from_world(world))
    }

    /// Creates a context without a native world.
    ///
    /// This is used after startup world creation fails so the app can keep
    /// running while reporting the failure through the configured error policy.
    pub fn disabled() -> Self {
        Self {
            world: None,
            entity_to_body: HashMap::new(),
            body_to_entity: HashMap::new(),
            entity_to_shape: HashMap::new(),
            shape_to_entity: HashMap::new(),
            shape_to_body_entity: HashMap::new(),
            shape_descriptors: HashMap::new(),
            entity_to_joint: HashMap::new(),
            joint_to_entity: HashMap::new(),
            joint_to_body_entities: HashMap::new(),
            joint_descriptors: HashMap::new(),
            last_step_failed: true,
        }
    }

    /// Creates a context from an existing native world.
    pub fn from_world(world: World) -> Self {
        Self {
            world: Some(world),
            entity_to_body: HashMap::new(),
            body_to_entity: HashMap::new(),
            entity_to_shape: HashMap::new(),
            shape_to_entity: HashMap::new(),
            shape_to_body_entity: HashMap::new(),
            shape_descriptors: HashMap::new(),
            entity_to_joint: HashMap::new(),
            joint_to_entity: HashMap::new(),
            joint_to_body_entities: HashMap::new(),
            joint_descriptors: HashMap::new(),
            last_step_failed: false,
        }
    }

    /// Returns the native world, if startup succeeded.
    pub fn world(&self) -> Option<&World> {
        self.world.as_ref()
    }

    /// Returns the native world mutably, if startup succeeded.
    pub fn world_mut(&mut self) -> Option<&mut World> {
        self.world.as_mut()
    }

    /// Returns the Bevy entity mapped to a native body id.
    pub fn body_entity(&self, body_id: BodyId) -> Option<Entity> {
        self.body_to_entity.get(&body_id).copied()
    }

    /// Returns the Bevy entity mapped to a native shape id.
    pub fn shape_entity(&self, shape_id: ShapeId) -> Option<Entity> {
        self.shape_to_entity.get(&shape_id).copied()
    }

    /// Returns the Bevy entity mapped to a native joint id.
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

    pub(crate) fn insert_shape(
        &mut self,
        entity: Entity,
        body_entity: Entity,
        descriptor: ShapeDescriptor,
        shape_id: ShapeId,
    ) {
        self.entity_to_shape.insert(entity, shape_id);
        self.shape_to_entity.insert(shape_id, entity);
        self.shape_to_body_entity.insert(entity, body_entity);
        self.shape_descriptors.insert(entity, descriptor);
    }

    pub(crate) fn remove_shape(&mut self, entity: Entity, shape_id: ShapeId) {
        self.entity_to_shape.remove(&entity);
        self.shape_to_entity.remove(&shape_id);
        self.shape_to_body_entity.remove(&entity);
        self.shape_descriptors.remove(&entity);
    }

    pub(crate) fn shape_body_entity(&self, shape_entity: Entity) -> Option<Entity> {
        self.shape_to_body_entity.get(&shape_entity).copied()
    }

    pub(crate) fn shape_descriptor(&self, shape_entity: Entity) -> Option<ShapeDescriptor> {
        self.shape_descriptors.get(&shape_entity).copied()
    }

    pub(crate) fn insert_joint(
        &mut self,
        entity: Entity,
        body_a: Entity,
        body_b: Entity,
        joint: Joint,
        joint_id: JointId,
    ) {
        self.entity_to_joint.insert(entity, joint_id);
        self.joint_to_entity.insert(joint_id, entity);
        self.joint_to_body_entities.insert(entity, (body_a, body_b));
        self.joint_descriptors.insert(entity, joint);
    }

    pub(crate) fn remove_joint(&mut self, entity: Entity, joint_id: JointId) {
        self.entity_to_joint.remove(&entity);
        self.joint_to_entity.remove(&joint_id);
        self.joint_to_body_entities.remove(&entity);
        self.joint_descriptors.remove(&entity);
    }

    pub(crate) fn joint_body_entities(&self, joint_entity: Entity) -> Option<(Entity, Entity)> {
        self.joint_to_body_entities.get(&joint_entity).copied()
    }

    pub(crate) fn joint_descriptor(&self, joint_entity: Entity) -> Option<Joint> {
        self.joint_descriptors.get(&joint_entity).copied()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct ShapeDescriptor {
    pub collider: Collider,
    pub material: PhysicsMaterial,
    pub local_transform: ShapeLocalTransform,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct ShapeLocalTransform {
    pub translation: Vec3,
    pub rotation: Quat,
}

impl ShapeLocalTransform {
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
    };

    pub fn from_transform(transform: Option<&bevy_transform::components::Transform>) -> Self {
        transform.map_or(Self::IDENTITY, |transform| Self {
            translation: transform.translation,
            rotation: transform.rotation,
        })
    }
}
