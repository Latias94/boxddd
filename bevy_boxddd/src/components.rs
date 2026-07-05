//! Bevy ECS components used to author and observe Box3D physics objects.

use bevy_ecs::prelude::{Component, Entity};
use bevy_math::Vec3;
use boxddd::{BodyId, BodyType, Filter, JointId, MotionLocks, ShapeDef, ShapeId, SurfaceMaterial};

/// Body type to create for an entity that participates in the physics world.
#[derive(Component, Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum RigidBody {
    /// Immovable body used for terrain, walls, and other non-simulated geometry.
    Static,
    /// App-controlled body that can move but is not affected by forces.
    Kinematic,
    /// Fully simulated body affected by gravity, contacts, joints, and forces.
    #[default]
    Dynamic,
}

impl From<RigidBody> for BodyType {
    fn from(value: RigidBody) -> Self {
        match value {
            RigidBody::Static => BodyType::Static,
            RigidBody::Kinematic => BodyType::Kinematic,
            RigidBody::Dynamic => BodyType::Dynamic,
        }
    }
}

/// Runtime body tuning applied to the native Box3D body.
///
/// This component is optional. When it is present, the plugin validates it
/// before body creation and reapplies changed values before each physics step.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct BodySettings {
    /// Gravity multiplier applied to this body.
    pub gravity_scale: f32,
    /// Linear damping applied by Box3D.
    pub linear_damping: f32,
    /// Angular damping applied by Box3D.
    pub angular_damping: f32,
    /// Whether the body is allowed to sleep.
    pub sleep_enabled: bool,
    /// Whether the body uses bullet-style continuous collision handling.
    pub bullet: bool,
    /// Per-axis translation and rotation locks.
    pub motion_locks: MotionLocks,
}

impl BodySettings {
    /// Creates settings that mark a body as a bullet for continuous collision.
    pub fn bullet() -> Self {
        Self {
            bullet: true,
            ..Default::default()
        }
    }

    /// Validates finite tuning values before applying them.
    pub fn validate(self) -> boxddd::Result<()> {
        validate_scalar(self.gravity_scale)?;
        validate_nonnegative_scalar(self.linear_damping)?;
        validate_nonnegative_scalar(self.angular_damping)
    }
}

impl Default for BodySettings {
    fn default() -> Self {
        Self {
            gravity_scale: 1.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            sleep_enabled: true,
            bullet: false,
            motion_locks: MotionLocks::new(false, false, false, false, false, false),
        }
    }
}

/// Shape descriptor used to create a Box3D shape for a body entity.
///
/// A collider may live on the same entity as [`RigidBody`] or on a child entity.
/// Child collider transforms are interpreted as local offsets from the parent
/// body entity.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub enum Collider {
    /// Box hull defined by half extents in local Bevy units.
    Cuboid {
        /// Half size along the local X, Y, and Z axes.
        half_extents: Vec3,
    },
    /// Sphere collider.
    Sphere {
        /// Sphere radius in local Bevy units.
        radius: f32,
        /// Local-space center relative to the body.
        center: Vec3,
    },
    /// Capsule collider between two local-space endpoints.
    Capsule {
        /// First endpoint of the capsule segment in local space.
        point1: Vec3,
        /// Second endpoint of the capsule segment in local space.
        point2: Vec3,
        /// Capsule radius around the segment.
        radius: f32,
    },
    /// Static mesh box generated through `boxddd::MeshData`.
    MeshBox {
        /// Center of the generated mesh before scaling.
        center: Vec3,
        /// Positive mesh extents before scaling.
        extent: Vec3,
        /// Positive Box3D mesh scale applied at shape creation.
        scale: Vec3,
        /// Whether Box3D should identify hard edges for the generated mesh.
        identify_edges: bool,
    },
    /// Static grid mesh generated through `boxddd::MeshData`.
    MeshGrid {
        /// Number of grid vertices along the X axis.
        x_count: i32,
        /// Number of grid vertices along the Z axis.
        z_count: i32,
        /// Spacing between adjacent grid vertices.
        cell_width: f32,
        /// Number of surface-material slots in the generated mesh.
        material_count: i32,
        /// Positive Box3D mesh scale applied at shape creation.
        scale: Vec3,
        /// Whether Box3D should identify hard edges for the generated mesh.
        identify_edges: bool,
    },
    /// Static height-field grid generated through `boxddd::HeightField`.
    HeightFieldGrid {
        /// Number of height-field rows.
        row_count: i32,
        /// Number of height-field columns.
        column_count: i32,
        /// Positive scale applied to the generated height field.
        scale: Vec3,
        /// Whether the generated height field should contain sample holes.
        make_holes: bool,
    },
    /// Static compound shape containing one sphere child.
    CompoundSphere {
        /// Local-space center of the compound sphere.
        center: Vec3,
        /// Sphere radius.
        radius: f32,
        /// Surface material assigned to the compound child.
        material: SurfaceMaterial,
    },
    /// Hull created from a reusable descriptor and attached with the collider transform.
    CreatedHull {
        /// Hull recipe used to build the Box3D hull resource.
        hull: HullDescriptor,
    },
    /// Hull created from a descriptor and an extra local transform.
    TransformedHull {
        /// Hull recipe used to build the Box3D hull resource.
        hull: HullDescriptor,
        /// Additional local translation before shape creation.
        translation: Vec3,
        /// Additional local rotation before shape creation.
        rotation: bevy_math::Quat,
        /// Positive local scale passed to Box3D for the hull shape.
        scale: Vec3,
    },
}

/// Reusable hull recipe for collider descriptors.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HullDescriptor {
    /// Procedural rock hull with the given approximate radius.
    Rock {
        /// Approximate hull radius.
        radius: f32,
    },
    /// Procedural cylinder hull.
    Cylinder {
        /// Cylinder height along the Y axis.
        height: f32,
        /// Cylinder radius in the XZ plane.
        radius: f32,
        /// Local Y offset applied by Box3D when generating the hull.
        y_offset: f32,
        /// Number of sides in the generated hull.
        sides: i32,
    },
}

impl HullDescriptor {
    /// Creates a procedural rock hull descriptor.
    pub const fn rock(radius: f32) -> Self {
        Self::Rock { radius }
    }

    /// Creates a procedural cylinder hull descriptor.
    pub const fn cylinder(height: f32, radius: f32, sides: i32) -> Self {
        Self::Cylinder {
            height,
            radius,
            y_offset: 0.0,
            sides,
        }
    }

    /// Creates a procedural cylinder hull descriptor with a local Y offset.
    pub const fn offset_cylinder(height: f32, radius: f32, y_offset: f32, sides: i32) -> Self {
        Self::Cylinder {
            height,
            radius,
            y_offset,
            sides,
        }
    }

    /// Validates finite, positive descriptor parameters before creating native resources.
    pub fn validate(self) -> boxddd::Result<()> {
        match self {
            Self::Rock { radius } => validate_positive_scalar(radius),
            Self::Cylinder {
                height,
                radius,
                y_offset,
                sides,
            } => {
                validate_positive_scalar(height)?;
                validate_positive_scalar(radius)?;
                validate_scalar(y_offset)?;
                if (3..=32).contains(&sides) {
                    Ok(())
                } else {
                    Err(boxddd::Error::InvalidArgument)
                }
            }
        }
    }
}

impl Collider {
    /// Creates a cuboid collider from half extents.
    pub fn cuboid(x: f32, y: f32, z: f32) -> Self {
        Self::Cuboid {
            half_extents: Vec3::new(x, y, z),
        }
    }

    /// Creates a cube collider from one half extent.
    pub fn cube(half_extent: f32) -> Self {
        Self::Cuboid {
            half_extents: Vec3::splat(half_extent),
        }
    }

    /// Creates a sphere collider centered on the body origin.
    pub fn sphere(radius: f32) -> Self {
        Self::Sphere {
            radius,
            center: Vec3::ZERO,
        }
    }

    /// Creates a Y-axis capsule centered on the body origin.
    pub fn capsule_y(half_height: f32, radius: f32) -> Self {
        Self::Capsule {
            point1: Vec3::new(0.0, -half_height, 0.0),
            point2: Vec3::new(0.0, half_height, 0.0),
            radius,
        }
    }

    /// Creates a static mesh-box collider descriptor.
    pub fn mesh_box(center: Vec3, extent: Vec3, scale: Vec3, identify_edges: bool) -> Self {
        Self::MeshBox {
            center,
            extent,
            scale,
            identify_edges,
        }
    }

    /// Creates a static grid-mesh collider descriptor.
    pub fn mesh_grid(
        x_count: i32,
        z_count: i32,
        cell_width: f32,
        material_count: i32,
        scale: Vec3,
        identify_edges: bool,
    ) -> Self {
        Self::MeshGrid {
            x_count,
            z_count,
            cell_width,
            material_count,
            scale,
            identify_edges,
        }
    }

    /// Creates a static height-field collider descriptor.
    pub fn height_field_grid(
        row_count: i32,
        column_count: i32,
        scale: Vec3,
        make_holes: bool,
    ) -> Self {
        Self::HeightFieldGrid {
            row_count,
            column_count,
            scale,
            make_holes,
        }
    }

    /// Creates a static single-sphere compound collider descriptor.
    pub fn compound_sphere(center: Vec3, radius: f32, material: SurfaceMaterial) -> Self {
        Self::CompoundSphere {
            center,
            radius,
            material,
        }
    }

    /// Creates a hull collider from a descriptor.
    pub fn created_hull(hull: HullDescriptor) -> Self {
        Self::CreatedHull { hull }
    }

    /// Creates a hull collider with an additional local transform and scale.
    pub fn transformed_hull(
        hull: HullDescriptor,
        translation: Vec3,
        rotation: bevy_math::Quat,
        scale: Vec3,
    ) -> Self {
        Self::TransformedHull {
            hull,
            translation,
            rotation,
            scale,
        }
    }

    /// Creates a procedural rock hull collider.
    pub fn created_rock_hull(radius: f32) -> Self {
        Self::created_hull(HullDescriptor::rock(radius))
    }

    /// Creates a procedural cylinder hull collider.
    pub fn cylinder_hull(height: f32, radius: f32, sides: i32) -> Self {
        Self::created_hull(HullDescriptor::cylinder(height, radius, sides))
    }

    /// Creates a transformed procedural rock hull collider.
    pub fn transformed_rock_hull(
        radius: f32,
        translation: Vec3,
        rotation: bevy_math::Quat,
        scale: Vec3,
    ) -> Self {
        Self::transformed_hull(HullDescriptor::rock(radius), translation, rotation, scale)
    }

    /// Returns true for native resource shapes that Box3D only supports on static bodies here.
    pub const fn requires_static_body(self) -> bool {
        matches!(
            self,
            Self::MeshBox { .. }
                | Self::MeshGrid { .. }
                | Self::HeightFieldGrid { .. }
                | Self::CompoundSphere { .. }
        )
    }

    /// Validates finite, positive collider parameters before native shape creation.
    pub fn validate(self) -> boxddd::Result<()> {
        match self {
            Self::Cuboid { half_extents } => validate_positive_vec3(half_extents),
            Self::Sphere { radius, center } => {
                validate_vec3(center)?;
                validate_positive_scalar(radius)
            }
            Self::Capsule {
                point1,
                point2,
                radius,
            } => {
                validate_vec3(point1)?;
                validate_vec3(point2)?;
                validate_positive_scalar(radius)
            }
            Self::MeshBox {
                center,
                extent,
                scale,
                ..
            } => {
                validate_vec3(center)?;
                validate_positive_vec3(extent)?;
                validate_positive_vec3(scale)
            }
            Self::MeshGrid {
                x_count,
                z_count,
                cell_width,
                material_count,
                scale,
                ..
            } => {
                if x_count < 2 || z_count < 2 || material_count <= 0 {
                    return Err(boxddd::Error::InvalidArgument);
                }
                validate_positive_scalar(cell_width)?;
                validate_positive_vec3(scale)
            }
            Self::HeightFieldGrid {
                row_count,
                column_count,
                scale,
                ..
            } => {
                if row_count < 2 || column_count < 2 {
                    return Err(boxddd::Error::InvalidArgument);
                }
                validate_positive_vec3(scale)
            }
            Self::CompoundSphere {
                center,
                radius,
                material,
            } => {
                validate_vec3(center)?;
                validate_positive_scalar(radius)?;
                material.validate()
            }
            Self::CreatedHull { hull } => hull.validate(),
            Self::TransformedHull {
                hull,
                translation,
                rotation,
                scale,
            } => {
                hull.validate()?;
                validate_vec3(translation)?;
                if rotation.is_finite() {
                    validate_positive_vec3(scale)
                } else {
                    Err(boxddd::Error::InvalidArgument)
                }
            }
        }
    }
}

/// Shape material and event flags used when creating a collider shape.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct PhysicsMaterial {
    /// Shape density passed to Box3D.
    pub density: f32,
    /// Shape friction passed to Box3D.
    pub friction: f32,
    /// Shape restitution passed to Box3D.
    pub restitution: f32,
    /// Whether the shape is a sensor.
    pub is_sensor: bool,
    /// Enables contact begin/end messages for this shape.
    pub enable_contact_events: bool,
    /// Enables sensor begin/end messages for this shape.
    pub enable_sensor_events: bool,
    /// Enables contact hit messages for this shape.
    pub enable_hit_events: bool,
    /// Box3D collision filter data.
    pub filter: Filter,
}

impl PhysicsMaterial {
    /// Converts the component into the `boxddd` shape definition used by the plugin.
    pub fn shape_def(self) -> ShapeDef {
        ShapeDef::builder()
            .density(self.density)
            .friction(self.friction)
            .restitution(self.restitution)
            .sensor(self.is_sensor)
            .enable_contact_events(self.enable_contact_events)
            .enable_sensor_events(self.enable_sensor_events)
            .enable_hit_events(self.enable_hit_events)
            .filter(self.filter)
            .build()
    }
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            density: 1.0,
            friction: 0.6,
            restitution: 0.0,
            is_sensor: false,
            enable_contact_events: false,
            enable_sensor_events: false,
            enable_hit_events: false,
            filter: Filter::default(),
        }
    }
}

/// Native Box3D body id inserted after the plugin creates a body.
#[derive(Component, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct BoxdddBody(pub BodyId);

impl BoxdddBody {
    /// Returns the native Box3D body id.
    pub const fn id(self) -> BodyId {
        self.0
    }
}

/// Native Box3D shape id inserted after the plugin creates a shape.
#[derive(Component, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct BoxdddShape(pub ShapeId);

impl BoxdddShape {
    /// Returns the native Box3D shape id.
    pub const fn id(self) -> ShapeId {
        self.0
    }
}

/// Bevy body entities connected by a declarative joint component.
#[derive(Component, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct JointTarget {
    /// First Bevy entity containing a [`BoxdddBody`].
    pub body_a: Entity,
    /// Second Bevy entity containing a [`BoxdddBody`].
    pub body_b: Entity,
}

impl JointTarget {
    /// Creates a joint target from two body entities.
    pub const fn new(body_a: Entity, body_b: Entity) -> Self {
        Self { body_a, body_b }
    }
}

/// Declarative joint descriptor created between the entities in [`JointTarget`].
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub enum Joint {
    /// Distance joint with a target length in physics units.
    Distance {
        /// Target distance between the two connected bodies.
        length: f32,
    },
    /// Revolute joint around Box3D's default local axes.
    Revolute,
    /// Spherical joint around Box3D's default local anchors.
    Spherical,
    /// Weld joint using Box3D defaults.
    Weld,
    /// Prismatic joint using Box3D defaults.
    Prismatic,
    /// Wheel joint using Box3D defaults.
    Wheel,
}

impl Joint {
    /// Creates a distance joint descriptor.
    pub const fn distance(length: f32) -> Self {
        Self::Distance { length }
    }

    /// Creates a revolute joint descriptor.
    pub const fn revolute() -> Self {
        Self::Revolute
    }

    /// Creates a spherical joint descriptor.
    pub const fn spherical() -> Self {
        Self::Spherical
    }

    /// Creates a weld joint descriptor.
    pub const fn weld() -> Self {
        Self::Weld
    }

    /// Creates a prismatic joint descriptor.
    pub const fn prismatic() -> Self {
        Self::Prismatic
    }

    /// Creates a wheel joint descriptor.
    pub const fn wheel() -> Self {
        Self::Wheel
    }

    /// Validates descriptor parameters before creating the native joint.
    pub fn validate(self) -> boxddd::Result<()> {
        match self {
            Self::Distance { length } => validate_nonnegative_scalar(length),
            Self::Revolute | Self::Spherical | Self::Weld | Self::Prismatic | Self::Wheel => Ok(()),
        }
    }
}

impl Default for Joint {
    fn default() -> Self {
        Self::distance(1.0)
    }
}

/// Native Box3D joint id inserted after the plugin creates a joint.
#[derive(Component, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct BoxdddJoint(pub JointId);

impl BoxdddJoint {
    /// Returns the native Box3D joint id.
    pub const fn id(self) -> JointId {
        self.0
    }
}

/// Direction used when synchronizing Bevy and Box3D transforms.
#[derive(Component, Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum TransformSyncMode {
    /// Read the Box3D transform after stepping and write it into Bevy.
    #[default]
    PhysicsToBevy,
    /// Read the Bevy transform before stepping and write it into Box3D.
    BevyToPhysics,
    /// Disable automatic transform synchronization for this entity.
    None,
}

/// Linear velocity command applied to a body before each physics step.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub struct LinearVelocity(pub Vec3);

/// Angular velocity command applied to a body before each physics step.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub struct AngularVelocity(pub Vec3);

/// Continuous force command applied to a body before each physics step.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct ExternalForce {
    /// Force vector in physics units.
    pub force: Vec3,
    /// Optional world-space application point. `None` applies at the center of mass.
    pub point: Option<Vec3>,
    /// Whether applying the force should wake a sleeping body.
    pub wake: bool,
}

impl ExternalForce {
    /// Creates a force command applied at the body center of mass.
    pub fn at_center(force: Vec3) -> Self {
        Self {
            force,
            point: None,
            wake: true,
        }
    }
}

/// One-shot impulse command applied to a body before the next physics step.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct ExternalImpulse {
    /// Impulse vector in physics units.
    pub impulse: Vec3,
    /// Optional world-space application point. `None` applies at the center of mass.
    pub point: Option<Vec3>,
    /// Whether applying the impulse should wake a sleeping body.
    pub wake: bool,
}

impl ExternalImpulse {
    /// Creates an impulse command applied at the body center of mass.
    pub fn at_center(impulse: Vec3) -> Self {
        Self {
            impulse,
            point: None,
            wake: true,
        }
    }
}

fn validate_vec3(value: Vec3) -> boxddd::Result<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(boxddd::Error::InvalidArgument)
    }
}

fn validate_scalar(value: f32) -> boxddd::Result<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(boxddd::Error::InvalidArgument)
    }
}

fn validate_positive_scalar(value: f32) -> boxddd::Result<()> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(boxddd::Error::InvalidArgument)
    }
}

fn validate_nonnegative_scalar(value: f32) -> boxddd::Result<()> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(boxddd::Error::InvalidArgument)
    }
}

fn validate_positive_vec3(value: Vec3) -> boxddd::Result<()> {
    if value.is_finite() && value.x > 0.0 && value.y > 0.0 && value.z > 0.0 {
        Ok(())
    } else {
        Err(boxddd::Error::InvalidArgument)
    }
}
