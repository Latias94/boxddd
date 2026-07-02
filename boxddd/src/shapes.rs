use crate::types::Vec3;
use boxddd_sys::ffi;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SurfaceMaterial {
    pub friction: f32,
    pub restitution: f32,
    pub rolling_resistance: f32,
    pub tangent_velocity: Vec3,
    pub user_material_id: u64,
    pub custom_color: u32,
}

impl SurfaceMaterial {
    #[inline]
    pub fn from_raw(raw: ffi::b3SurfaceMaterial) -> Self {
        Self {
            friction: raw.friction,
            restitution: raw.restitution,
            rolling_resistance: raw.rollingResistance,
            tangent_velocity: Vec3::from_raw(raw.tangentVelocity),
            user_material_id: raw.userMaterialId,
            custom_color: raw.customColor,
        }
    }

    #[inline]
    pub fn into_raw(self) -> ffi::b3SurfaceMaterial {
        ffi::b3SurfaceMaterial {
            friction: self.friction,
            restitution: self.restitution,
            rollingResistance: self.rolling_resistance,
            tangentVelocity: self.tangent_velocity.into_raw(),
            userMaterialId: self.user_material_id,
            customColor: self.custom_color,
        }
    }
}

impl Default for ShapeDef {
    fn default() -> Self {
        Self {
            raw: unsafe { ffi::b3DefaultShapeDef() },
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShapeDef {
    raw: ffi::b3ShapeDef,
}

impl ShapeDef {
    #[inline]
    pub fn builder() -> ShapeDefBuilder {
        ShapeDefBuilder::new()
    }

    #[inline]
    pub fn raw(&self) -> &ffi::b3ShapeDef {
        &self.raw
    }
}

#[derive(Clone, Debug)]
pub struct ShapeDefBuilder {
    def: ShapeDef,
}

impl ShapeDefBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            def: ShapeDef::default(),
        }
    }

    #[inline]
    pub fn density(mut self, density: f32) -> Self {
        self.def.raw.density = density;
        self
    }

    #[inline]
    pub fn friction(mut self, friction: f32) -> Self {
        self.def.raw.baseMaterial.friction = friction;
        self
    }

    #[inline]
    pub fn restitution(mut self, restitution: f32) -> Self {
        self.def.raw.baseMaterial.restitution = restitution;
        self
    }

    #[inline]
    pub fn sensor(mut self, is_sensor: bool) -> Self {
        self.def.raw.isSensor = is_sensor;
        self
    }

    #[inline]
    pub fn build(self) -> ShapeDef {
        self.def
    }
}

impl Default for ShapeDefBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sphere {
    raw: ffi::b3Sphere,
}

impl Sphere {
    #[inline]
    pub fn new(center: impl Into<Vec3>, radius: f32) -> Self {
        Self {
            raw: ffi::b3Sphere {
                center: center.into().into_raw(),
                radius,
            },
        }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3Sphere) -> Self {
        Self { raw }
    }

    #[inline]
    pub const fn raw(&self) -> &ffi::b3Sphere {
        &self.raw
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoxHull {
    raw: ffi::b3BoxHull,
}

impl BoxHull {
    #[inline]
    pub fn cube(half_width: f32) -> Self {
        Self {
            raw: unsafe { ffi::b3MakeCubeHull(half_width) },
        }
    }

    #[inline]
    pub fn new(hx: f32, hy: f32, hz: f32) -> Self {
        Self {
            raw: unsafe { ffi::b3MakeBoxHull(hx, hy, hz) },
        }
    }

    #[inline]
    pub const fn raw(&self) -> &ffi::b3BoxHull {
        &self.raw
    }

    #[inline]
    pub const fn hull_data(&self) -> &ffi::b3HullData {
        &self.raw.base
    }
}
