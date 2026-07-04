use crate::error::{Error, Result};
use crate::types::{Pos, Quat, Vec3};
use boxddd_sys::ffi;
use std::ffi::CString;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// The simulation behavior assigned to a body.
pub enum BodyType {
    /// A non-moving body with infinite mass.
    Static,
    /// A body moved explicitly by the user rather than by forces.
    Kinematic,
    /// A fully simulated body affected by forces, contacts, and joints.
    Dynamic,
}

impl BodyType {
    /// Converts this body type to the raw Box3D enum value.
    #[inline]
    pub const fn into_raw(self) -> ffi::b3BodyType {
        match self {
            Self::Static => ffi::b3BodyType_b3_staticBody,
            Self::Kinematic => ffi::b3BodyType_b3_kinematicBody,
            Self::Dynamic => ffi::b3BodyType_b3_dynamicBody,
        }
    }

    /// Converts a raw Box3D body type into the safe Rust enum.
    #[inline]
    pub const fn from_raw(raw: ffi::b3BodyType) -> Option<Self> {
        match raw {
            ffi::b3BodyType_b3_staticBody => Some(Self::Static),
            ffi::b3BodyType_b3_kinematicBody => Some(Self::Kinematic),
            ffi::b3BodyType_b3_dynamicBody => Some(Self::Dynamic),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
/// Parameters used when creating a body in a world.
pub struct BodyDef {
    raw: ffi::b3BodyDef,
    name: Option<CString>,
}

impl Default for BodyDef {
    fn default() -> Self {
        let raw = unsafe { ffi::b3DefaultBodyDef() };
        Self { raw, name: None }
    }
}

impl BodyDef {
    /// Starts a builder initialized with Box3D default body settings.
    #[inline]
    pub fn builder() -> BodyDefBuilder {
        BodyDefBuilder::new()
    }

    /// Returns the raw Box3D body definition backing this value.
    #[inline]
    pub fn raw(&self) -> &ffi::b3BodyDef {
        &self.raw
    }

    /// Checks that all vectors, rotations, and scalar tuning values are finite and valid.
    pub fn validate(&self) -> Result<()> {
        Pos::from_raw(self.raw.position).validate()?;
        Quat::from_raw(self.raw.rotation).validate()?;
        Vec3::from_raw(self.raw.linearVelocity).validate()?;
        Vec3::from_raw(self.raw.angularVelocity).validate()?;
        if is_valid_scalar(self.raw.linearDamping)
            && is_valid_scalar(self.raw.angularDamping)
            && is_valid_scalar(self.raw.gravityScale)
            && is_valid_scalar(self.raw.sleepThreshold)
        {
            Ok(())
        } else {
            Err(Error::InvalidArgument)
        }
    }
}

#[derive(Clone, Debug)]
/// Builder for a Box3D body definition.
pub struct BodyDefBuilder {
    def: BodyDef,
}

impl BodyDefBuilder {
    /// Creates a builder initialized with Box3D defaults.
    #[inline]
    pub fn new() -> Self {
        Self {
            def: BodyDef::default(),
        }
    }

    /// Sets whether the body is static, kinematic, or dynamic.
    #[inline]
    pub fn body_type(mut self, body_type: BodyType) -> Self {
        self.def.raw.type_ = body_type.into_raw();
        self
    }

    /// Sets the initial world-space body position.
    #[inline]
    pub fn position(mut self, position: impl Into<Pos>) -> Self {
        self.def.raw.position = position.into().into_raw();
        self
    }

    /// Sets the initial world-space body rotation.
    #[inline]
    pub fn rotation(mut self, rotation: Quat) -> Self {
        self.def.raw.rotation = rotation.into_raw();
        self
    }

    /// Sets the initial linear velocity.
    #[inline]
    pub fn linear_velocity(mut self, velocity: impl Into<Vec3>) -> Self {
        self.def.raw.linearVelocity = velocity.into().into_raw();
        self
    }

    /// Sets the initial angular velocity.
    #[inline]
    pub fn angular_velocity(mut self, velocity: impl Into<Vec3>) -> Self {
        self.def.raw.angularVelocity = velocity.into().into_raw();
        self
    }

    /// Scales gravity applied to this body.
    #[inline]
    pub fn gravity_scale(mut self, gravity_scale: f32) -> Self {
        self.def.raw.gravityScale = gravity_scale;
        self
    }

    /// Enables or disables continuous collision handling for fast-moving bodies.
    #[inline]
    pub fn bullet(mut self, is_bullet: bool) -> Self {
        self.def.raw.isBullet = is_bullet;
        self
    }

    /// Sets the optional body name.
    ///
    /// Panics if the provided bytes contain an interior NUL byte. Use
    /// pre-validation if the name comes from untrusted input.
    pub fn name(mut self, name: impl Into<Vec<u8>>) -> Self {
        let c_name = CString::new(name).expect("body name must not contain interior NUL bytes");
        self.def.raw.name = c_name.as_ptr();
        self.def.name = Some(c_name);
        self
    }

    /// Finishes the builder and returns the body definition.
    #[inline]
    pub fn build(self) -> BodyDef {
        self.def
    }
}

impl Default for BodyDefBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn is_valid_scalar(value: f32) -> bool {
    value.is_finite()
}
