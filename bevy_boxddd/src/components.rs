use bevy_ecs::prelude::Component;
use bevy_math::Vec3;
use boxddd::{BodyId, BodyType, Filter, ShapeDef, ShapeId, SurfaceMaterial};

#[derive(Component, Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum RigidBody {
    Static,
    Kinematic,
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

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub enum Collider {
    Cuboid {
        half_extents: Vec3,
    },
    Sphere {
        radius: f32,
        center: Vec3,
    },
    Capsule {
        point1: Vec3,
        point2: Vec3,
        radius: f32,
    },
    MeshBox {
        center: Vec3,
        extent: Vec3,
        scale: Vec3,
        identify_edges: bool,
    },
    MeshGrid {
        x_count: i32,
        z_count: i32,
        cell_width: f32,
        material_count: i32,
        scale: Vec3,
        identify_edges: bool,
    },
    HeightFieldGrid {
        row_count: i32,
        column_count: i32,
        scale: Vec3,
        make_holes: bool,
    },
    CompoundSphere {
        center: Vec3,
        radius: f32,
        material: SurfaceMaterial,
    },
    CreatedRockHull {
        radius: f32,
    },
    TransformedRockHull {
        radius: f32,
        translation: Vec3,
        rotation: bevy_math::Quat,
        scale: Vec3,
    },
}

impl Collider {
    pub fn cuboid(x: f32, y: f32, z: f32) -> Self {
        Self::Cuboid {
            half_extents: Vec3::new(x, y, z),
        }
    }

    pub fn cube(half_extent: f32) -> Self {
        Self::Cuboid {
            half_extents: Vec3::splat(half_extent),
        }
    }

    pub fn sphere(radius: f32) -> Self {
        Self::Sphere {
            radius,
            center: Vec3::ZERO,
        }
    }

    pub fn capsule_y(half_height: f32, radius: f32) -> Self {
        Self::Capsule {
            point1: Vec3::new(0.0, -half_height, 0.0),
            point2: Vec3::new(0.0, half_height, 0.0),
            radius,
        }
    }

    pub fn mesh_box(center: Vec3, extent: Vec3, scale: Vec3, identify_edges: bool) -> Self {
        Self::MeshBox {
            center,
            extent,
            scale,
            identify_edges,
        }
    }

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

    pub fn compound_sphere(center: Vec3, radius: f32, material: SurfaceMaterial) -> Self {
        Self::CompoundSphere {
            center,
            radius,
            material,
        }
    }

    pub fn created_rock_hull(radius: f32) -> Self {
        Self::CreatedRockHull { radius }
    }

    pub fn transformed_rock_hull(
        radius: f32,
        translation: Vec3,
        rotation: bevy_math::Quat,
        scale: Vec3,
    ) -> Self {
        Self::TransformedRockHull {
            radius,
            translation,
            rotation,
            scale,
        }
    }

    pub const fn requires_static_body(self) -> bool {
        matches!(
            self,
            Self::MeshBox { .. }
                | Self::MeshGrid { .. }
                | Self::HeightFieldGrid { .. }
                | Self::CompoundSphere { .. }
                | Self::CreatedRockHull { .. }
                | Self::TransformedRockHull { .. }
        )
    }

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
            Self::CreatedRockHull { radius } => validate_positive_scalar(radius),
            Self::TransformedRockHull {
                radius,
                translation,
                rotation,
                scale,
            } => {
                validate_positive_scalar(radius)?;
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

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct PhysicsMaterial {
    pub density: f32,
    pub friction: f32,
    pub restitution: f32,
    pub is_sensor: bool,
    pub enable_contact_events: bool,
    pub enable_sensor_events: bool,
    pub enable_hit_events: bool,
    pub filter: Filter,
}

impl PhysicsMaterial {
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

#[derive(Component, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct BoxdddBody(pub BodyId);

impl BoxdddBody {
    pub const fn id(self) -> BodyId {
        self.0
    }
}

#[derive(Component, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct BoxdddShape(pub ShapeId);

impl BoxdddShape {
    pub const fn id(self) -> ShapeId {
        self.0
    }
}

#[derive(Component, Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum TransformSyncMode {
    #[default]
    PhysicsToBevy,
    BevyToPhysics,
    None,
}

#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub struct LinearVelocity(pub Vec3);

#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub struct AngularVelocity(pub Vec3);

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct ExternalForce {
    pub force: Vec3,
    pub point: Option<Vec3>,
    pub wake: bool,
}

impl ExternalForce {
    pub fn at_center(force: Vec3) -> Self {
        Self {
            force,
            point: None,
            wake: true,
        }
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct ExternalImpulse {
    pub impulse: Vec3,
    pub point: Option<Vec3>,
    pub wake: bool,
}

impl ExternalImpulse {
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

fn validate_positive_scalar(value: f32) -> boxddd::Result<()> {
    if value.is_finite() && value > 0.0 {
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
