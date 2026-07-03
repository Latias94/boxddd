use crate::error::{Error, Result};
use crate::types::{Aabb, Filter, Transform, Vec3};
use boxddd_sys::ffi;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

pub const HEIGHT_FIELD_HOLE: u8 = ffi::B3_HEIGHT_FIELD_HOLE as u8;
pub const MAX_COMPOUND_MESH_MATERIALS: usize = ffi::B3_MAX_COMPOUND_MESH_MATERIALS as usize;

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SurfaceMaterial {
    pub friction: f32,
    pub restitution: f32,
    pub rolling_resistance: f32,
    pub tangent_velocity: Vec3,
    pub user_material_id: u64,
    pub custom_color: u32,
}

impl Default for SurfaceMaterial {
    fn default() -> Self {
        Self::from_raw(unsafe { ffi::b3DefaultSurfaceMaterial() })
    }
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
            && self.friction >= 0.0
            && self.restitution.is_finite()
            && self.restitution >= 0.0
            && self.rolling_resistance.is_finite()
            && self.rolling_resistance >= 0.0
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

fn triangle_area_squared(a: Vec3, b: Vec3, c: Vec3) -> f32 {
    let ab = Vec3::new(b.x - a.x, b.y - a.y, b.z - a.z);
    let ac = Vec3::new(c.x - a.x, c.y - a.y, c.z - a.z);
    let cross = Vec3::new(
        ab.y * ac.z - ab.z * ac.y,
        ab.z * ac.x - ab.x * ac.z,
        ab.x * ac.y - ab.y * ac.x,
    );
    cross.x * cross.x + cross.y * cross.y + cross.z * cross.z
}

fn min_max_finite(values: &[f32]) -> Option<(f32, f32)> {
    let mut iter = values.iter().copied();
    let first = iter.next()?;
    if !first.is_finite() {
        return None;
    }
    let mut min = first;
    let mut max = first;
    for value in iter {
        if !value.is_finite() {
            return None;
        }
        min = min.min(value);
        max = max.max(value);
    }
    Some((min, max))
}

pub(crate) fn validate_mesh_scale(scale: Vec3) -> Result<Vec3> {
    if scale.is_valid()
        && scale.x.abs() > f32::EPSILON
        && scale.y.abs() > f32::EPSILON
        && scale.z.abs() > f32::EPSILON
    {
        Ok(scale)
    } else {
        Err(Error::InvalidArgument)
    }
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
        let ptr = unsafe {
            ffi::b3CreateHull(
                points.as_ptr().cast(),
                points.len() as i32,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MeshDataOptions {
    pub weld_tolerance: f32,
    pub weld_vertices: bool,
    pub use_median_split: bool,
    pub identify_edges: bool,
}

impl MeshDataOptions {
    #[inline]
    pub const fn new() -> Self {
        Self {
            weld_tolerance: 0.0,
            weld_vertices: false,
            use_median_split: false,
            identify_edges: true,
        }
    }

    #[inline]
    pub const fn weld_tolerance(mut self, weld_tolerance: f32) -> Self {
        self.weld_tolerance = weld_tolerance;
        self
    }

    #[inline]
    pub const fn weld_vertices(mut self, weld_vertices: bool) -> Self {
        self.weld_vertices = weld_vertices;
        self
    }

    #[inline]
    pub const fn use_median_split(mut self, use_median_split: bool) -> Self {
        self.use_median_split = use_median_split;
        self
    }

    #[inline]
    pub const fn identify_edges(mut self, identify_edges: bool) -> Self {
        self.identify_edges = identify_edges;
        self
    }

    fn validate(self) -> Result<()> {
        if self.weld_tolerance.is_finite() && self.weld_tolerance >= 0.0 {
            Ok(())
        } else {
            Err(Error::InvalidArgument)
        }
    }
}

impl Default for MeshDataOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct MeshDataBuilder {
    vertices: Vec<Vec3>,
    indices: Vec<i32>,
    material_indices: Option<Vec<u8>>,
    options: MeshDataOptions,
}

impl MeshDataBuilder {
    #[inline]
    pub fn new(vertices: impl Into<Vec<Vec3>>, indices: impl Into<Vec<i32>>) -> Self {
        Self {
            vertices: vertices.into(),
            indices: indices.into(),
            material_indices: None,
            options: MeshDataOptions::default(),
        }
    }

    #[inline]
    pub fn material_indices(mut self, material_indices: impl Into<Vec<u8>>) -> Self {
        self.material_indices = Some(material_indices.into());
        self
    }

    #[inline]
    pub fn weld_tolerance(mut self, weld_tolerance: f32) -> Self {
        self.options.weld_tolerance = weld_tolerance;
        self
    }

    #[inline]
    pub fn weld_vertices(mut self, weld_vertices: bool) -> Self {
        self.options.weld_vertices = weld_vertices;
        self
    }

    #[inline]
    pub fn use_median_split(mut self, use_median_split: bool) -> Self {
        self.options.use_median_split = use_median_split;
        self
    }

    #[inline]
    pub fn identify_edges(mut self, identify_edges: bool) -> Self {
        self.options.identify_edges = identify_edges;
        self
    }

    #[inline]
    pub fn build(self) -> Result<MeshData> {
        MeshData::from_triangles(
            &self.vertices,
            &self.indices,
            self.material_indices.as_deref(),
            self.options,
        )
    }
}

#[derive(Debug)]
pub struct MeshData {
    raw: NonNull<ffi::b3MeshData>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl MeshData {
    #[inline]
    pub fn builder(
        vertices: impl Into<Vec<Vec3>>,
        indices: impl Into<Vec<i32>>,
    ) -> MeshDataBuilder {
        MeshDataBuilder::new(vertices, indices)
    }

    pub fn from_triangles(
        vertices: impl AsRef<[Vec3]>,
        indices: impl AsRef<[i32]>,
        material_indices: Option<&[u8]>,
        options: MeshDataOptions,
    ) -> Result<Self> {
        options.validate()?;
        let vertices = vertices.as_ref();
        let indices = indices.as_ref();
        let triangle_count = indices.len() / 3;
        if vertices.len() < 3
            || vertices.len() > i32::MAX as usize
            || indices.is_empty()
            || indices.len() % 3 != 0
            || triangle_count > i32::MAX as usize
            || vertices.iter().any(|vertex| !vertex.is_valid())
        {
            return Err(Error::InvalidArgument);
        }
        if let Some(material_indices) = material_indices {
            if material_indices.len() != triangle_count {
                return Err(Error::InvalidArgument);
            }
        }

        for triangle in indices.chunks_exact(3) {
            let [a, b, c]: [i32; 3] = triangle.try_into().expect("chunk size is fixed");
            if a < 0
                || b < 0
                || c < 0
                || a as usize >= vertices.len()
                || b as usize >= vertices.len()
                || c as usize >= vertices.len()
                || triangle_area_squared(
                    vertices[a as usize],
                    vertices[b as usize],
                    vertices[c as usize],
                ) <= f32::MIN_POSITIVE
            {
                return Err(Error::InvalidArgument);
            }
        }

        let mut vertices: Vec<ffi::b3Vec3> =
            vertices.iter().map(|vertex| vertex.into_raw()).collect();
        let mut indices = indices.to_vec();
        let mut materials = material_indices.map(<[u8]>::to_vec);
        let mut def = ffi::b3MeshDef {
            vertices: vertices.as_mut_ptr(),
            indices: indices.as_mut_ptr(),
            materialIndices: materials
                .as_mut()
                .map_or(std::ptr::null_mut(), |materials| materials.as_mut_ptr()),
            weldTolerance: options.weld_tolerance,
            vertexCount: vertices.len() as i32,
            triangleCount: triangle_count as i32,
            weldVertices: options.weld_vertices,
            useMedianSplit: options.use_median_split,
            identifyEdges: options.identify_edges,
        };

        Self::from_ptr(unsafe { ffi::b3CreateMesh(&mut def, std::ptr::null_mut(), 0) })
    }

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

    pub fn wave_mesh(
        x_count: i32,
        z_count: i32,
        cell_width: f32,
        amplitude: f32,
        row_frequency: f32,
        column_frequency: f32,
    ) -> Result<Self> {
        if x_count < 2
            || z_count < 2
            || !cell_width.is_finite()
            || cell_width <= 0.0
            || !amplitude.is_finite()
            || !row_frequency.is_finite()
            || !column_frequency.is_finite()
        {
            return Err(Error::InvalidArgument);
        }
        Self::from_ptr(unsafe {
            ffi::b3CreateWaveMesh(
                x_count,
                z_count,
                cell_width,
                amplitude,
                row_frequency,
                column_frequency,
            )
        })
    }

    pub fn torus_mesh(
        radial_resolution: i32,
        tubular_resolution: i32,
        radius: f32,
        thickness: f32,
    ) -> Result<Self> {
        if radial_resolution < 3
            || tubular_resolution < 3
            || !radius.is_finite()
            || radius <= 0.0
            || !thickness.is_finite()
            || thickness <= 0.0
        {
            return Err(Error::InvalidArgument);
        }
        Self::from_ptr(unsafe {
            ffi::b3CreateTorusMesh(radial_resolution, tubular_resolution, radius, thickness)
        })
    }

    pub fn hollow_box_mesh(center: impl Into<Vec3>, extent: impl Into<Vec3>) -> Result<Self> {
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
        Self::from_ptr(unsafe { ffi::b3CreateHollowBoxMesh(center.into_raw(), extent.into_raw()) })
    }

    pub fn platform_mesh(
        center: impl Into<Vec3>,
        height: f32,
        top_width: f32,
        bottom_width: f32,
    ) -> Result<Self> {
        let center = center.into();
        if !center.is_valid()
            || !height.is_finite()
            || height <= 0.0
            || !top_width.is_finite()
            || top_width <= 0.0
            || !bottom_width.is_finite()
            || bottom_width <= 0.0
        {
            return Err(Error::InvalidArgument);
        }
        Self::from_ptr(unsafe {
            ffi::b3CreatePlatformMesh(center.into_raw(), height, top_width, bottom_width)
        })
    }

    #[inline]
    pub fn byte_count(&self) -> i32 {
        unsafe { self.raw.as_ref().byteCount }
    }

    #[inline]
    pub fn tree_height(&self) -> i32 {
        unsafe { ffi::b3GetHeight(self.raw.as_ptr()) }
    }

    #[inline]
    pub fn vertex_count(&self) -> i32 {
        unsafe { self.raw.as_ref().vertexCount }
    }

    #[inline]
    pub fn triangle_count(&self) -> i32 {
        unsafe { self.raw.as_ref().triangleCount }
    }

    #[inline]
    pub fn material_count(&self) -> i32 {
        unsafe { self.raw.as_ref().materialCount }
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

#[derive(Clone, Debug)]
pub struct HeightFieldBuilder {
    row_count: i32,
    column_count: i32,
    heights: Vec<f32>,
    material_indices: Option<Vec<u8>>,
    clockwise_winding: bool,
}

impl HeightFieldBuilder {
    #[inline]
    pub fn new(row_count: i32, column_count: i32, heights: impl Into<Vec<f32>>) -> Self {
        Self {
            row_count,
            column_count,
            heights: heights.into(),
            material_indices: None,
            clockwise_winding: false,
        }
    }

    #[inline]
    pub fn material_indices(mut self, material_indices: impl Into<Vec<u8>>) -> Self {
        self.material_indices = Some(material_indices.into());
        self
    }

    #[inline]
    pub fn clockwise_winding(mut self, clockwise_winding: bool) -> Self {
        self.clockwise_winding = clockwise_winding;
        self
    }

    #[inline]
    pub fn build(self, scale: impl Into<Vec3>) -> Result<HeightField> {
        HeightField::from_samples(
            self.row_count,
            self.column_count,
            &self.heights,
            scale,
            self.material_indices.as_deref(),
            self.clockwise_winding,
        )
    }
}

#[derive(Debug)]
pub struct HeightField {
    raw: NonNull<ffi::b3HeightFieldData>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl HeightField {
    #[inline]
    pub fn builder(
        row_count: i32,
        column_count: i32,
        heights: impl Into<Vec<f32>>,
    ) -> HeightFieldBuilder {
        HeightFieldBuilder::new(row_count, column_count, heights)
    }

    pub fn from_samples(
        row_count: i32,
        column_count: i32,
        heights: impl AsRef<[f32]>,
        scale: impl Into<Vec3>,
        material_indices: Option<&[u8]>,
        clockwise_winding: bool,
    ) -> Result<Self> {
        let scale = scale.into();
        let heights = heights.as_ref();
        if row_count < 2
            || column_count < 2
            || row_count as usize > i32::MAX as usize / column_count as usize
            || heights.len() != row_count as usize * column_count as usize
            || !scale.is_valid()
            || scale.x <= 0.0
            || scale.y <= 0.0
            || scale.z <= 0.0
            || heights.iter().any(|height| !height.is_finite())
        {
            return Err(Error::InvalidArgument);
        }

        let cell_count = (row_count as usize - 1) * (column_count as usize - 1);
        if let Some(material_indices) = material_indices {
            if material_indices.len() != cell_count {
                return Err(Error::InvalidArgument);
            }
        }

        let mut heights = heights.to_vec();
        let mut materials = material_indices.map(<[u8]>::to_vec);
        let (global_minimum_height, global_maximum_height) =
            min_max_finite(&heights).ok_or(Error::InvalidArgument)?;
        let mut def = ffi::b3HeightFieldDef {
            heights: heights.as_mut_ptr(),
            materialIndices: materials
                .as_mut()
                .map_or(std::ptr::null_mut(), |materials| materials.as_mut_ptr()),
            scale: scale.into_raw(),
            countX: column_count,
            countZ: row_count,
            globalMinimumHeight: global_minimum_height,
            globalMaximumHeight: global_maximum_height,
            clockwiseWinding: clockwise_winding,
        };

        Self::from_ptr(unsafe { ffi::b3CreateHeightField(&mut def) })
    }

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

    pub fn wave(
        row_count: i32,
        column_count: i32,
        scale: impl Into<Vec3>,
        row_frequency: f32,
        column_frequency: f32,
        make_holes: bool,
    ) -> Result<Self> {
        let scale = scale.into();
        if row_count < 2
            || column_count < 2
            || !scale.is_valid()
            || scale.x <= 0.0
            || scale.y <= 0.0
            || scale.z <= 0.0
            || !row_frequency.is_finite()
            || !column_frequency.is_finite()
        {
            return Err(Error::InvalidArgument);
        }
        Self::from_ptr(unsafe {
            ffi::b3CreateWave(
                row_count,
                column_count,
                scale.into_raw(),
                row_frequency,
                column_frequency,
                make_holes,
            )
        })
    }

    #[inline]
    pub fn byte_count(&self) -> i32 {
        unsafe { self.raw.as_ref().byteCount }
    }

    #[inline]
    pub fn row_count(&self) -> i32 {
        unsafe { self.raw.as_ref().rowCount }
    }

    #[inline]
    pub fn column_count(&self) -> i32 {
        unsafe { self.raw.as_ref().columnCount }
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

#[derive(Copy, Clone, Debug)]
pub struct ShapeHull<'a> {
    raw: &'a ffi::b3HullData,
}

impl<'a> ShapeHull<'a> {
    #[inline]
    pub(crate) const fn from_raw(raw: &'a ffi::b3HullData) -> Self {
        Self { raw }
    }

    #[inline]
    pub const fn byte_count(&self) -> i32 {
        self.raw.byteCount
    }

    #[inline]
    pub const fn hash(&self) -> u32 {
        self.raw.hash
    }

    #[inline]
    pub const fn aabb(&self) -> Aabb {
        Aabb::from_raw(self.raw.aabb)
    }

    #[inline]
    pub const fn surface_area(&self) -> f32 {
        self.raw.surfaceArea
    }

    #[inline]
    pub const fn volume(&self) -> f32 {
        self.raw.volume
    }

    #[inline]
    pub const fn inner_radius(&self) -> f32 {
        self.raw.innerRadius
    }

    #[inline]
    pub const fn center(&self) -> Vec3 {
        Vec3::from_raw(self.raw.center)
    }

    #[inline]
    pub const fn vertex_count(&self) -> i32 {
        self.raw.vertexCount
    }

    #[inline]
    pub const fn edge_count(&self) -> i32 {
        self.raw.edgeCount
    }

    #[inline]
    pub const fn face_count(&self) -> i32 {
        self.raw.faceCount
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ShapeMesh<'a> {
    data: &'a ffi::b3MeshData,
    scale: Vec3,
}

impl<'a> ShapeMesh<'a> {
    #[inline]
    pub(crate) fn from_raw(raw: ffi::b3Mesh) -> Option<Self> {
        unsafe { raw.data.as_ref() }.map(|data| Self {
            data,
            scale: Vec3::from_raw(raw.scale),
        })
    }

    #[inline]
    pub const fn scale(&self) -> Vec3 {
        self.scale
    }

    #[inline]
    pub const fn byte_count(&self) -> i32 {
        self.data.byteCount
    }

    #[inline]
    pub const fn hash(&self) -> u32 {
        self.data.hash
    }

    #[inline]
    pub const fn bounds(&self) -> Aabb {
        Aabb::from_raw(self.data.bounds)
    }

    #[inline]
    pub const fn surface_area(&self) -> f32 {
        self.data.surfaceArea
    }

    #[inline]
    pub const fn tree_height(&self) -> i32 {
        self.data.treeHeight
    }

    #[inline]
    pub const fn degenerate_count(&self) -> i32 {
        self.data.degenerateCount
    }

    #[inline]
    pub const fn vertex_count(&self) -> i32 {
        self.data.vertexCount
    }

    #[inline]
    pub const fn triangle_count(&self) -> i32 {
        self.data.triangleCount
    }

    #[inline]
    pub const fn material_count(&self) -> i32 {
        self.data.materialCount
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ShapeHeightField<'a> {
    raw: &'a ffi::b3HeightFieldData,
}

impl<'a> ShapeHeightField<'a> {
    #[inline]
    pub(crate) const fn from_raw(raw: &'a ffi::b3HeightFieldData) -> Self {
        Self { raw }
    }

    #[inline]
    pub const fn byte_count(&self) -> i32 {
        self.raw.byteCount
    }

    #[inline]
    pub const fn hash(&self) -> u32 {
        self.raw.hash
    }

    #[inline]
    pub const fn aabb(&self) -> Aabb {
        Aabb::from_raw(self.raw.aabb)
    }

    #[inline]
    pub const fn min_height(&self) -> f32 {
        self.raw.minHeight
    }

    #[inline]
    pub const fn max_height(&self) -> f32 {
        self.raw.maxHeight
    }

    #[inline]
    pub const fn scale(&self) -> Vec3 {
        Vec3::from_raw(self.raw.scale)
    }

    #[inline]
    pub const fn column_count(&self) -> i32 {
        self.raw.columnCount
    }

    #[inline]
    pub const fn row_count(&self) -> i32 {
        self.raw.rowCount
    }

    #[inline]
    pub const fn clockwise(&self) -> bool {
        self.raw.clockwise
    }
}

#[derive(Debug)]
pub struct Compound {
    raw: NonNull<ffi::b3CompoundData>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl Compound {
    #[inline]
    pub fn builder<'a>() -> CompoundBuilder<'a> {
        CompoundBuilder::new()
    }

    pub fn single_sphere(sphere: Sphere, material: SurfaceMaterial) -> Result<Self> {
        Self::builder().with_sphere(sphere, material)?.build()
    }

    #[inline]
    pub fn byte_count(&self) -> i32 {
        unsafe { self.raw.as_ref().byteCount }
    }

    #[inline]
    pub fn child_count(&self) -> i32 {
        unsafe {
            let raw = self.raw.as_ref();
            raw.capsuleCount + raw.hullCount + raw.meshCount + raw.sphereCount
        }
    }

    #[inline]
    pub fn capsule_count(&self) -> i32 {
        unsafe { self.raw.as_ref().capsuleCount }
    }

    #[inline]
    pub fn hull_count(&self) -> i32 {
        unsafe { self.raw.as_ref().hullCount }
    }

    #[inline]
    pub fn mesh_count(&self) -> i32 {
        unsafe { self.raw.as_ref().meshCount }
    }

    #[inline]
    pub fn sphere_count(&self) -> i32 {
        unsafe { self.raw.as_ref().sphereCount }
    }

    #[inline]
    pub fn material_count(&self) -> i32 {
        unsafe { self.raw.as_ref().materialCount }
    }

    #[inline]
    pub fn shared_hull_count(&self) -> i32 {
        unsafe { self.raw.as_ref().sharedHullCount }
    }

    #[inline]
    pub fn shared_mesh_count(&self) -> i32 {
        unsafe { self.raw.as_ref().sharedMeshCount }
    }

    pub fn material(&self, index: i32) -> Result<SurfaceMaterial> {
        if index < 0 || index >= self.material_count() {
            return Err(Error::IndexOutOfRange);
        }
        let materials = unsafe { ffi::b3GetCompoundMaterials(self.raw.as_ptr()) };
        unsafe { materials.add(index as usize).as_ref() }
            .copied()
            .map(SurfaceMaterial::from_raw)
            .ok_or(Error::InvalidArgument)
    }

    pub fn child(&self, index: i32) -> Result<CompoundChild<'_>> {
        if index < 0 || index >= self.child_count() {
            return Err(Error::IndexOutOfRange);
        }
        CompoundChild::from_raw(unsafe { ffi::b3GetCompoundChild(self.raw.as_ptr(), index) })
    }

    pub fn capsule_child(&self, index: i32) -> Result<CompoundCapsule> {
        if index < 0 || index >= self.capsule_count() {
            return Err(Error::IndexOutOfRange);
        }
        let raw = unsafe { ffi::b3GetCompoundCapsule(self.raw.as_ptr(), index) };
        Ok(CompoundCapsule::from_raw(raw))
    }

    pub fn hull_child(&self, index: i32) -> Result<CompoundHull<'_>> {
        if index < 0 || index >= self.hull_count() {
            return Err(Error::IndexOutOfRange);
        }
        let raw = unsafe { ffi::b3GetCompoundHull(self.raw.as_ptr(), index) };
        CompoundHull::from_raw(raw)
    }

    pub fn mesh_child(&self, index: i32) -> Result<CompoundMesh<'_>> {
        if index < 0 || index >= self.mesh_count() {
            return Err(Error::IndexOutOfRange);
        }
        let raw = unsafe { ffi::b3GetCompoundMesh(self.raw.as_ptr(), index) };
        CompoundMesh::from_raw(raw)
    }

    pub fn sphere_child(&self, index: i32) -> Result<CompoundSphere> {
        if index < 0 || index >= self.sphere_count() {
            return Err(Error::IndexOutOfRange);
        }
        let raw = unsafe { ffi::b3GetCompoundSphere(self.raw.as_ptr(), index) };
        Ok(CompoundSphere::from_raw(raw))
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

#[derive(Debug)]
pub struct CompoundBuilder<'a> {
    capsules: Vec<ffi::b3CompoundCapsuleDef>,
    hulls: Vec<ffi::b3CompoundHullDef>,
    meshes: Vec<ffi::b3CompoundMeshDef>,
    mesh_materials: Vec<Box<[ffi::b3SurfaceMaterial]>>,
    spheres: Vec<ffi::b3CompoundSphereDef>,
    error: Option<Error>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> CompoundBuilder<'a> {
    #[inline]
    pub fn new() -> Self {
        Self {
            capsules: Vec::new(),
            hulls: Vec::new(),
            meshes: Vec::new(),
            mesh_materials: Vec::new(),
            spheres: Vec::new(),
            error: None,
            _lifetime: PhantomData,
        }
    }

    pub fn sphere(mut self, sphere: Sphere, material: SurfaceMaterial) -> Self {
        if let Err(error) = self.add_sphere(sphere, material) {
            self.error = Some(error);
        }
        self
    }

    pub fn with_sphere(mut self, sphere: Sphere, material: SurfaceMaterial) -> Result<Self> {
        self.add_sphere(sphere, material)?;
        Ok(self)
    }

    pub fn add_sphere(&mut self, sphere: Sphere, material: SurfaceMaterial) -> Result<&mut Self> {
        sphere.validate()?;
        material.validate()?;
        self.spheres.push(ffi::b3CompoundSphereDef {
            sphere: *sphere.raw(),
            material: material.into_raw(),
        });
        self.validate_child_capacity()?;
        Ok(self)
    }

    pub fn capsule(mut self, capsule: Capsule, material: SurfaceMaterial) -> Self {
        if let Err(error) = self.add_capsule(capsule, material) {
            self.error = Some(error);
        }
        self
    }

    pub fn with_capsule(mut self, capsule: Capsule, material: SurfaceMaterial) -> Result<Self> {
        self.add_capsule(capsule, material)?;
        Ok(self)
    }

    pub fn add_capsule(
        &mut self,
        capsule: Capsule,
        material: SurfaceMaterial,
    ) -> Result<&mut Self> {
        capsule.validate()?;
        material.validate()?;
        self.capsules.push(ffi::b3CompoundCapsuleDef {
            capsule: *capsule.raw(),
            material: material.into_raw(),
        });
        self.validate_child_capacity()?;
        Ok(self)
    }

    pub fn hull(
        mut self,
        hull: &'a Hull,
        transform: impl Into<Transform>,
        material: SurfaceMaterial,
    ) -> Self {
        if let Err(error) = self.add_hull(hull, transform, material) {
            self.error = Some(error);
        }
        self
    }

    pub fn with_hull(
        mut self,
        hull: &'a Hull,
        transform: impl Into<Transform>,
        material: SurfaceMaterial,
    ) -> Result<Self> {
        self.add_hull(hull, transform, material)?;
        Ok(self)
    }

    pub fn add_hull(
        &mut self,
        hull: &'a Hull,
        transform: impl Into<Transform>,
        material: SurfaceMaterial,
    ) -> Result<&mut Self> {
        self.add_hull_ptr(hull.as_ptr(), transform.into(), material)
    }

    pub fn box_hull(
        mut self,
        hull: &'a BoxHull,
        transform: impl Into<Transform>,
        material: SurfaceMaterial,
    ) -> Self {
        if let Err(error) = self.add_box_hull(hull, transform, material) {
            self.error = Some(error);
        }
        self
    }

    pub fn with_box_hull(
        mut self,
        hull: &'a BoxHull,
        transform: impl Into<Transform>,
        material: SurfaceMaterial,
    ) -> Result<Self> {
        self.add_box_hull(hull, transform, material)?;
        Ok(self)
    }

    pub fn add_box_hull(
        &mut self,
        hull: &'a BoxHull,
        transform: impl Into<Transform>,
        material: SurfaceMaterial,
    ) -> Result<&mut Self> {
        self.add_hull_ptr(hull.hull_data(), transform.into(), material)
    }

    fn add_hull_ptr(
        &mut self,
        hull: *const ffi::b3HullData,
        transform: Transform,
        material: SurfaceMaterial,
    ) -> Result<&mut Self> {
        transform.validate()?;
        material.validate()?;
        if hull.is_null() {
            return Err(Error::InvalidArgument);
        }
        self.hulls.push(ffi::b3CompoundHullDef {
            hull,
            transform: transform.into_raw(),
            material: material.into_raw(),
        });
        self.validate_child_capacity()?;
        Ok(self)
    }

    pub fn mesh(
        mut self,
        mesh: &'a MeshData,
        transform: impl Into<Transform>,
        scale: impl Into<Vec3>,
        materials: impl AsRef<[SurfaceMaterial]>,
    ) -> Self {
        if let Err(error) = self.add_mesh(mesh, transform, scale, materials) {
            self.error = Some(error);
        }
        self
    }

    pub fn with_mesh(
        mut self,
        mesh: &'a MeshData,
        transform: impl Into<Transform>,
        scale: impl Into<Vec3>,
        materials: impl AsRef<[SurfaceMaterial]>,
    ) -> Result<Self> {
        self.add_mesh(mesh, transform, scale, materials)?;
        Ok(self)
    }

    pub fn add_mesh(
        &mut self,
        mesh: &'a MeshData,
        transform: impl Into<Transform>,
        scale: impl Into<Vec3>,
        materials: impl AsRef<[SurfaceMaterial]>,
    ) -> Result<&mut Self> {
        let transform = transform.into();
        let scale = validate_mesh_scale(scale.into())?;
        transform.validate()?;
        let materials = materials.as_ref();
        if materials.is_empty()
            || materials.len() > MAX_COMPOUND_MESH_MATERIALS
            || materials.len() != mesh.material_count() as usize
        {
            return Err(Error::InvalidArgument);
        }
        let raw_materials: Vec<_> = materials
            .iter()
            .copied()
            .map(|material| {
                material.validate()?;
                Ok(material.into_raw())
            })
            .collect::<Result<_>>()?;
        self.mesh_materials.push(raw_materials.into_boxed_slice());
        let material_ptr = self
            .mesh_materials
            .last()
            .expect("just pushed mesh materials")
            .as_ptr();
        self.meshes.push(ffi::b3CompoundMeshDef {
            meshData: mesh.as_ptr(),
            transform: transform.into_raw(),
            scale: scale.into_raw(),
            materials: material_ptr,
            materialCount: materials.len() as i32,
        });
        self.validate_child_capacity()?;
        Ok(self)
    }

    pub fn build(mut self) -> Result<Compound> {
        if let Some(error) = self.error {
            return Err(error);
        }
        self.validate_child_capacity()?;
        if self.child_count() == 0 {
            return Err(Error::InvalidArgument);
        }
        let mut def = ffi::b3CompoundDef {
            capsules: self.capsules.as_mut_ptr(),
            capsuleCount: self.capsules.len() as i32,
            hulls: self.hulls.as_mut_ptr(),
            hullCount: self.hulls.len() as i32,
            meshes: self.meshes.as_mut_ptr(),
            meshCount: self.meshes.len() as i32,
            spheres: self.spheres.as_mut_ptr(),
            sphereCount: self.spheres.len() as i32,
        };
        Compound::from_ptr(unsafe { ffi::b3CreateCompound(&mut def) })
    }

    fn child_count(&self) -> usize {
        self.capsules.len() + self.hulls.len() + self.meshes.len() + self.spheres.len()
    }

    fn validate_child_capacity(&self) -> Result<()> {
        if self.child_count() < ffi::B3_MAX_CHILD_SHAPES as usize {
            Ok(())
        } else {
            Err(Error::InvalidArgument)
        }
    }
}

impl<'a> Default for CompoundBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CompoundCapsule {
    pub capsule: Capsule,
    pub material_index: i32,
}

impl CompoundCapsule {
    #[inline]
    const fn from_raw(raw: ffi::b3CompoundCapsule) -> Self {
        Self {
            capsule: Capsule::from_raw(raw.capsule),
            material_index: raw.materialIndex,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CompoundHull<'a> {
    pub hull: ShapeHull<'a>,
    pub transform: Transform,
    pub material_index: i32,
}

impl<'a> CompoundHull<'a> {
    fn from_raw(raw: ffi::b3CompoundHull) -> Result<Self> {
        unsafe { raw.hull.as_ref() }
            .map(|hull| Self {
                hull: ShapeHull::from_raw(hull),
                transform: Transform::from_raw(raw.transform),
                material_index: raw.materialIndex,
            })
            .ok_or(Error::InvalidArgument)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CompoundMesh<'a> {
    pub mesh: ShapeMesh<'a>,
    pub transform: Transform,
    pub material_indices: [i32; MAX_COMPOUND_MESH_MATERIALS],
}

impl<'a> CompoundMesh<'a> {
    fn from_raw(raw: ffi::b3CompoundMesh) -> Result<Self> {
        ShapeMesh::from_raw(ffi::b3Mesh {
            data: raw.meshData,
            scale: raw.scale,
        })
        .map(|mesh| Self {
            mesh,
            transform: Transform::from_raw(raw.transform),
            material_indices: raw.materialIndices,
        })
        .ok_or(Error::InvalidArgument)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CompoundSphere {
    pub sphere: Sphere,
    pub material_index: i32,
}

impl CompoundSphere {
    #[inline]
    const fn from_raw(raw: ffi::b3CompoundSphere) -> Self {
        Self {
            sphere: Sphere::from_raw(raw.sphere),
            material_index: raw.materialIndex,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CompoundChild<'a> {
    pub shape: CompoundChildShape<'a>,
    pub transform: Transform,
    pub material_indices: [i32; MAX_COMPOUND_MESH_MATERIALS],
}

impl<'a> CompoundChild<'a> {
    fn from_raw(raw: ffi::b3ChildShape) -> Result<Self> {
        let shape = match ShapeType::from_raw(raw.type_) {
            Some(ShapeType::Capsule) => CompoundChildShape::Capsule(Capsule::from_raw(unsafe {
                raw.__bindgen_anon_1.capsule
            })),
            Some(ShapeType::Hull) => {
                let hull = unsafe { raw.__bindgen_anon_1.hull.as_ref() }
                    .map(ShapeHull::from_raw)
                    .ok_or(Error::InvalidArgument)?;
                CompoundChildShape::Hull(hull)
            }
            Some(ShapeType::Mesh) => {
                let mesh = ShapeMesh::from_raw(unsafe { raw.__bindgen_anon_1.mesh })
                    .ok_or(Error::InvalidArgument)?;
                CompoundChildShape::Mesh(mesh)
            }
            Some(ShapeType::Sphere) => {
                CompoundChildShape::Sphere(Sphere::from_raw(unsafe { raw.__bindgen_anon_1.sphere }))
            }
            _ => return Err(Error::InvalidArgument),
        };
        Ok(Self {
            shape,
            transform: Transform::from_raw(raw.transform),
            material_indices: raw.materialIndices,
        })
    }

    #[inline]
    pub const fn shape_type(&self) -> ShapeType {
        self.shape.shape_type()
    }

    #[inline]
    pub const fn primary_material_index(&self) -> i32 {
        self.material_indices[0]
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CompoundChildShape<'a> {
    Capsule(Capsule),
    Hull(ShapeHull<'a>),
    Mesh(ShapeMesh<'a>),
    Sphere(Sphere),
}

impl<'a> CompoundChildShape<'a> {
    #[inline]
    pub const fn shape_type(&self) -> ShapeType {
        match self {
            Self::Capsule(_) => ShapeType::Capsule,
            Self::Hull(_) => ShapeType::Hull,
            Self::Mesh(_) => ShapeType::Mesh,
            Self::Sphere(_) => ShapeType::Sphere,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
