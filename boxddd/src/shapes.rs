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
#[derive(Copy, Clone, Debug)]
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

impl PartialEq for Sphere {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        raw_vec3_eq(self.raw.center, other.raw.center) && self.raw.radius == other.raw.radius
    }
}

#[derive(Copy, Clone, Debug)]
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

impl PartialEq for BoxHull {
    fn eq(&self, other: &Self) -> bool {
        raw_hull_data_eq(&self.raw.base, &other.raw.base)
            && raw_hull_vertices_eq(&self.raw.boxVertices, &other.raw.boxVertices)
            && raw_vec3_array_eq(&self.raw.boxPoints, &other.raw.boxPoints)
            && raw_hull_edges_eq(&self.raw.boxEdges, &other.raw.boxEdges)
            && raw_hull_faces_eq(&self.raw.boxFaces, &other.raw.boxFaces)
            && raw_planes_eq(&self.raw.boxPlanes, &other.raw.boxPlanes)
    }
}

#[inline]
fn raw_vec3_eq(a: ffi::b3Vec3, b: ffi::b3Vec3) -> bool {
    a.x == b.x && a.y == b.y && a.z == b.z
}

#[inline]
fn raw_matrix3_eq(a: &ffi::b3Matrix3, b: &ffi::b3Matrix3) -> bool {
    raw_vec3_eq(a.cx, b.cx) && raw_vec3_eq(a.cy, b.cy) && raw_vec3_eq(a.cz, b.cz)
}

#[inline]
fn raw_aabb_eq(a: &ffi::b3AABB, b: &ffi::b3AABB) -> bool {
    raw_vec3_eq(a.lowerBound, b.lowerBound) && raw_vec3_eq(a.upperBound, b.upperBound)
}

fn raw_hull_data_eq(a: &ffi::b3HullData, b: &ffi::b3HullData) -> bool {
    a.version == b.version
        && a.byteCount == b.byteCount
        && a.hash == b.hash
        && raw_aabb_eq(&a.aabb, &b.aabb)
        && a.surfaceArea == b.surfaceArea
        && a.volume == b.volume
        && a.innerRadius == b.innerRadius
        && raw_vec3_eq(a.center, b.center)
        && raw_matrix3_eq(&a.centralInertia, &b.centralInertia)
        && a.vertexCount == b.vertexCount
        && a.vertexOffset == b.vertexOffset
        && a.pointOffset == b.pointOffset
        && a.edgeCount == b.edgeCount
        && a.edgeOffset == b.edgeOffset
        && a.faceCount == b.faceCount
        && a.faceOffset == b.faceOffset
        && a.planeOffset == b.planeOffset
}

fn raw_hull_vertices_eq(a: &[ffi::b3HullVertex; 8], b: &[ffi::b3HullVertex; 8]) -> bool {
    a.iter().zip(b).all(|(a, b)| a.edge == b.edge)
}

fn raw_vec3_array_eq(a: &[ffi::b3Vec3; 8], b: &[ffi::b3Vec3; 8]) -> bool {
    a.iter().zip(b).all(|(a, b)| raw_vec3_eq(*a, *b))
}

fn raw_hull_edges_eq(a: &[ffi::b3HullHalfEdge; 24], b: &[ffi::b3HullHalfEdge; 24]) -> bool {
    a.iter().zip(b).all(|(a, b)| {
        a.next == b.next && a.twin == b.twin && a.origin == b.origin && a.face == b.face
    })
}

fn raw_hull_faces_eq(a: &[ffi::b3HullFace; 6], b: &[ffi::b3HullFace; 6]) -> bool {
    a.iter().zip(b).all(|(a, b)| a.edge == b.edge)
}

fn raw_planes_eq(a: &[ffi::b3Plane; 6], b: &[ffi::b3Plane; 6]) -> bool {
    a.iter()
        .zip(b)
        .all(|(a, b)| raw_vec3_eq(a.normal, b.normal) && a.offset == b.offset)
}
