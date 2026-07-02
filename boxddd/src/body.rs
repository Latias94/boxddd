use crate::error::{Error, Result};
use crate::types::{Pos, Quat, Vec3};
use boxddd_sys::ffi;
use std::ffi::CString;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BodyType {
    Static,
    Kinematic,
    Dynamic,
}

impl BodyType {
    #[inline]
    pub const fn into_raw(self) -> ffi::b3BodyType {
        match self {
            Self::Static => ffi::b3BodyType_b3_staticBody,
            Self::Kinematic => ffi::b3BodyType_b3_kinematicBody,
            Self::Dynamic => ffi::b3BodyType_b3_dynamicBody,
        }
    }

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
    #[inline]
    pub fn builder() -> BodyDefBuilder {
        BodyDefBuilder::new()
    }

    #[inline]
    pub fn raw(&self) -> &ffi::b3BodyDef {
        &self.raw
    }

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
pub struct BodyDefBuilder {
    def: BodyDef,
}

impl BodyDefBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            def: BodyDef::default(),
        }
    }

    #[inline]
    pub fn body_type(mut self, body_type: BodyType) -> Self {
        self.def.raw.type_ = body_type.into_raw();
        self
    }

    #[inline]
    pub fn position(mut self, position: impl Into<Pos>) -> Self {
        self.def.raw.position = position.into().into_raw();
        self
    }

    #[inline]
    pub fn rotation(mut self, rotation: Quat) -> Self {
        self.def.raw.rotation = rotation.into_raw();
        self
    }

    #[inline]
    pub fn linear_velocity(mut self, velocity: impl Into<Vec3>) -> Self {
        self.def.raw.linearVelocity = velocity.into().into_raw();
        self
    }

    #[inline]
    pub fn angular_velocity(mut self, velocity: impl Into<Vec3>) -> Self {
        self.def.raw.angularVelocity = velocity.into().into_raw();
        self
    }

    #[inline]
    pub fn gravity_scale(mut self, gravity_scale: f32) -> Self {
        self.def.raw.gravityScale = gravity_scale;
        self
    }

    #[inline]
    pub fn bullet(mut self, is_bullet: bool) -> Self {
        self.def.raw.isBullet = is_bullet;
        self
    }

    pub fn name(mut self, name: impl Into<Vec<u8>>) -> Self {
        let c_name = CString::new(name).expect("body name must not contain interior NUL bytes");
        self.def.raw.name = c_name.as_ptr();
        self.def.name = Some(c_name);
        self
    }

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
