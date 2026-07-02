use crate::error::{Error, Result};
use crate::types::{Filter, Transform, Vec3};
use boxddd_sys::ffi;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

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

    pub fn validate(self) -> Result<()> {
        if self.friction.is_finite()
            && self.restitution.is_finite()
            && self.rolling_resistance.is_finite()
            && self.tangent_velocity.is_valid()
        {
            Ok(())
        } else {
            Err(Error::InvalidArgument)
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

    pub fn filter(&self) -> Filter {
        Filter::from_raw(self.raw.filter)
    }

    pub fn surface_material(&self) -> SurfaceMaterial {
        SurfaceMaterial::from_raw(self.raw.baseMaterial)
    }

    pub fn validate(&self) -> Result<()> {
        SurfaceMaterial::from_raw(self.raw.baseMaterial).validate()?;
        if self.raw.density.is_finite()
            && self.raw.density >= 0.0
            && self.raw.explosionScale.is_finite()
        {
            Ok(())
        } else {
            Err(Error::InvalidArgument)
        }
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
    pub fn filter(mut self, filter: Filter) -> Self {
        self.def.raw.filter = filter.into_raw();
        self
    }

    #[inline]
    pub fn surface_material(mut self, material: SurfaceMaterial) -> Self {
        self.def.raw.baseMaterial = material.into_raw();
        self
    }

    #[inline]
    pub fn user_material_id(mut self, user_material_id: u64) -> Self {
        self.def.raw.baseMaterial.userMaterialId = user_material_id;
        self
    }

    #[inline]
    pub fn sensor(mut self, is_sensor: bool) -> Self {
        self.def.raw.isSensor = is_sensor;
        self
    }

    #[inline]
    pub fn enable_sensor_events(mut self, enabled: bool) -> Self {
        self.def.raw.enableSensorEvents = enabled;
        self
    }

    #[inline]
    pub fn enable_contact_events(mut self, enabled: bool) -> Self {
        self.def.raw.enableContactEvents = enabled;
        self
    }

    #[inline]
    pub fn enable_hit_events(mut self, enabled: bool) -> Self {
        self.def.raw.enableHitEvents = enabled;
        self
    }

    #[inline]
    pub fn enable_pre_solve_events(mut self, enabled: bool) -> Self {
        self.def.raw.enablePreSolveEvents = enabled;
        self
    }

    #[inline]
    pub fn enable_custom_filtering(mut self, enabled: bool) -> Self {
        self.def.raw.enableCustomFiltering = enabled;
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

    pub fn validate(&self) -> Result<()> {
        if Vec3::from_raw(self.raw.center).is_valid()
            && self.raw.radius.is_finite()
            && self.raw.radius > 0.0
        {
            Ok(())
        } else {
            Err(Error::InvalidArgument)
        }
    }
}

impl PartialEq for Sphere {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        raw_vec3_eq(self.raw.center, other.raw.center) && self.raw.radius == other.raw.radius
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Capsule {
    raw: ffi::b3Capsule,
}

impl Capsule {
    #[inline]
    pub fn new(center1: impl Into<Vec3>, center2: impl Into<Vec3>, radius: f32) -> Self {
        Self {
            raw: ffi::b3Capsule {
                center1: center1.into().into_raw(),
                center2: center2.into().into_raw(),
                radius,
            },
        }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3Capsule) -> Self {
        Self { raw }
    }

    #[inline]
    pub const fn raw(&self) -> &ffi::b3Capsule {
        &self.raw
    }

    pub fn validate(&self) -> Result<()> {
        if Vec3::from_raw(self.raw.center1).is_valid()
            && Vec3::from_raw(self.raw.center2).is_valid()
            && self.raw.radius.is_finite()
            && self.raw.radius > 0.0
        {
            Ok(())
        } else {
            Err(Error::InvalidArgument)
        }
    }
}

impl PartialEq for Capsule {
    fn eq(&self, other: &Self) -> bool {
        raw_vec3_eq(self.raw.center1, other.raw.center1)
            && raw_vec3_eq(self.raw.center2, other.raw.center2)
            && self.raw.radius == other.raw.radius
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
    pub fn offset(hx: f32, hy: f32, hz: f32, offset: impl Into<Vec3>) -> Self {
        Self {
            raw: unsafe { ffi::b3MakeOffsetBoxHull(hx, hy, hz, offset.into().into_raw()) },
        }
    }

    #[inline]
    pub fn transformed(hx: f32, hy: f32, hz: f32, transform: Transform) -> Self {
        Self {
            raw: unsafe { ffi::b3MakeTransformedBoxHull(hx, hy, hz, transform.into_raw()) },
        }
    }

    #[inline]
    pub fn scaled(
        half_widths: impl Into<Vec3>,
        transform: Transform,
        post_scale: impl Into<Vec3>,
    ) -> Self {
        Self {
            raw: unsafe {
                ffi::b3MakeScaledBoxHull(
                    half_widths.into().into_raw(),
                    transform.into_raw(),
                    post_scale.into().into_raw(),
                )
            },
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

#[derive(Debug)]
pub struct Hull {
    raw: NonNull<ffi::b3HullData>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl Hull {
    pub fn from_points(points: impl AsRef<[Vec3]>, max_vertex_count: i32) -> Result<Self> {
        let points = points.as_ref();
        if points.len() < 4 || max_vertex_count <= 0 || points.iter().any(|point| !point.is_valid())
        {
            return Err(Error::InvalidArgument);
        }
        let raw_points: Vec<_> = points.iter().map(|point| point.into_raw()).collect();
        let ptr = unsafe {
            ffi::b3CreateHull(
                raw_points.as_ptr(),
                raw_points.len() as i32,
                max_vertex_count,
            )
        };
        Self::from_ptr(ptr)
    }

    pub fn cylinder(height: f32, radius: f32, y_offset: f32, sides: i32) -> Result<Self> {
        if !height.is_finite()
            || height <= 0.0
            || !radius.is_finite()
            || radius <= 0.0
            || !y_offset.is_finite()
            || sides < 3
        {
            return Err(Error::InvalidArgument);
        }
        Self::from_ptr(unsafe { ffi::b3CreateCylinder(height, radius, y_offset, sides) })
    }

    pub fn cone(height: f32, radius1: f32, radius2: f32, slices: i32) -> Result<Self> {
        if !height.is_finite()
            || height <= 0.0
            || !radius1.is_finite()
            || radius1 < 0.0
            || !radius2.is_finite()
            || radius2 < 0.0
            || slices < 3
        {
            return Err(Error::InvalidArgument);
        }
        Self::from_ptr(unsafe { ffi::b3CreateCone(height, radius1, radius2, slices) })
    }

    pub fn rock(radius: f32) -> Result<Self> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(Error::InvalidArgument);
        }
        Self::from_ptr(unsafe { ffi::b3CreateRock(radius) })
    }

    #[inline]
    pub fn as_hull_data(&self) -> &ffi::b3HullData {
        unsafe { self.raw.as_ref() }
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const ffi::b3HullData {
        self.raw.as_ptr()
    }

    fn from_ptr(ptr: *mut ffi::b3HullData) -> Result<Self> {
        NonNull::new(ptr)
            .map(|raw| Self {
                raw,
                _not_send_sync: PhantomData,
            })
            .ok_or(Error::InvalidArgument)
    }
}

impl Drop for Hull {
    fn drop(&mut self) {
        unsafe { ffi::b3DestroyHull(self.raw.as_ptr()) };
    }
}

#[derive(Debug)]
pub struct MeshData {
    raw: NonNull<ffi::b3MeshData>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl MeshData {
    pub fn box_mesh(
        center: impl Into<Vec3>,
        extent: impl Into<Vec3>,
        identify_edges: bool,
    ) -> Result<Self> {
        let center = center.into();
        let extent = extent.into();
        if !center.is_valid()
            || !extent.is_valid()
            || extent.x <= 0.0
            || extent.y <= 0.0
            || extent.z <= 0.0
        {
            return Err(Error::InvalidArgument);
        }
        Self::from_ptr(unsafe {
            ffi::b3CreateBoxMesh(center.into_raw(), extent.into_raw(), identify_edges)
        })
    }

    pub fn grid_mesh(
        x_count: i32,
        z_count: i32,
        cell_width: f32,
        material_count: i32,
        identify_edges: bool,
    ) -> Result<Self> {
        if x_count < 2
            || z_count < 2
            || !cell_width.is_finite()
            || cell_width <= 0.0
            || material_count <= 0
        {
            return Err(Error::InvalidArgument);
        }
        Self::from_ptr(unsafe {
            ffi::b3CreateGridMesh(x_count, z_count, cell_width, material_count, identify_edges)
        })
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const ffi::b3MeshData {
        self.raw.as_ptr()
    }

    fn from_ptr(ptr: *mut ffi::b3MeshData) -> Result<Self> {
        NonNull::new(ptr)
            .map(|raw| Self {
                raw,
                _not_send_sync: PhantomData,
            })
            .ok_or(Error::InvalidArgument)
    }
}

impl Drop for MeshData {
    fn drop(&mut self) {
        unsafe { ffi::b3DestroyMesh(self.raw.as_ptr()) };
    }
}

#[derive(Debug)]
pub struct HeightField {
    raw: NonNull<ffi::b3HeightFieldData>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl HeightField {
    pub fn grid(
        row_count: i32,
        column_count: i32,
        scale: impl Into<Vec3>,
        make_holes: bool,
    ) -> Result<Self> {
        let scale = scale.into();
        if row_count < 2
            || column_count < 2
            || !scale.is_valid()
            || scale.x <= 0.0
            || scale.y <= 0.0
            || scale.z <= 0.0
        {
            return Err(Error::InvalidArgument);
        }
        Self::from_ptr(unsafe {
            ffi::b3CreateGrid(row_count, column_count, scale.into_raw(), make_holes)
        })
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const ffi::b3HeightFieldData {
        self.raw.as_ptr()
    }

    fn from_ptr(ptr: *mut ffi::b3HeightFieldData) -> Result<Self> {
        NonNull::new(ptr)
            .map(|raw| Self {
                raw,
                _not_send_sync: PhantomData,
            })
            .ok_or(Error::InvalidArgument)
    }
}

impl Drop for HeightField {
    fn drop(&mut self) {
        unsafe { ffi::b3DestroyHeightField(self.raw.as_ptr()) };
    }
}

#[derive(Debug)]
pub struct Compound {
    raw: NonNull<ffi::b3CompoundData>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl Compound {
    pub fn single_sphere(sphere: Sphere, material: SurfaceMaterial) -> Result<Self> {
        sphere.validate()?;
        material.validate()?;
        let mut sphere_def = ffi::b3CompoundSphereDef {
            sphere: *sphere.raw(),
            material: material.into_raw(),
        };
        let mut def = ffi::b3CompoundDef {
            capsules: std::ptr::null_mut(),
            capsuleCount: 0,
            hulls: std::ptr::null_mut(),
            hullCount: 0,
            meshes: std::ptr::null_mut(),
            meshCount: 0,
            spheres: &mut sphere_def,
            sphereCount: 1,
        };
        Self::from_ptr(unsafe { ffi::b3CreateCompound(&mut def) })
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const ffi::b3CompoundData {
        self.raw.as_ptr()
    }

    fn from_ptr(ptr: *mut ffi::b3CompoundData) -> Result<Self> {
        NonNull::new(ptr)
            .map(|raw| Self {
                raw,
                _not_send_sync: PhantomData,
            })
            .ok_or(Error::InvalidArgument)
    }
}

impl Drop for Compound {
    fn drop(&mut self) {
        unsafe { ffi::b3DestroyCompound(self.raw.as_ptr()) };
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShapeType {
    Capsule,
    Compound,
    HeightField,
    Hull,
    Mesh,
    Sphere,
}

impl ShapeType {
    pub const fn from_raw(raw: ffi::b3ShapeType) -> Option<Self> {
        match raw {
            ffi::b3ShapeType_b3_capsuleShape => Some(Self::Capsule),
            ffi::b3ShapeType_b3_compoundShape => Some(Self::Compound),
            ffi::b3ShapeType_b3_heightShape => Some(Self::HeightField),
            ffi::b3ShapeType_b3_hullShape => Some(Self::Hull),
            ffi::b3ShapeType_b3_meshShape => Some(Self::Mesh),
            ffi::b3ShapeType_b3_sphereShape => Some(Self::Sphere),
            _ => None,
        }
    }
}
