#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
use crate::core::box3d_lock;
use crate::core::callback_state;
use crate::error::{Error, Result};
use crate::types::{Aabb, Filter, Transform, Vec3};
use boxddd_sys::ffi;
#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, forget};
#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::NonNull;
use std::rc::Rc;
use std::slice;

/// Height-field material marker that makes a cell behave as a hole.
pub const HEIGHT_FIELD_HOLE: u8 = ffi::B3_HEIGHT_FIELD_HOLE as u8;
/// Maximum number of material slots a compound mesh child can reference.
pub const MAX_COMPOUND_MESH_MATERIALS: usize = ffi::B3_MAX_COMPOUND_MESH_MATERIALS as usize;

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
/// Surface properties assigned to a shape or compound child.
pub struct SurfaceMaterial {
    /// Coulomb friction coefficient used by contacts.
    pub friction: f32,
    /// Restitution coefficient used to make contacts bounce.
    pub restitution: f32,
    /// Resistance applied to rolling contacts.
    pub rolling_resistance: f32,
    /// Tangential surface speed used for conveyor-style contacts.
    pub tangent_velocity: Vec3,
    /// User-defined material identifier carried through contact data.
    pub user_material_id: u64,
    /// Optional debug-render color in Box3D's packed color format.
    pub custom_color: u32,
}

impl Default for SurfaceMaterial {
    fn default() -> Self {
        Self::from_raw(unsafe { ffi::b3DefaultSurfaceMaterial() })
    }
}

impl SurfaceMaterial {
    #[inline]
    /// Converts a raw Box3D surface material into the safe wrapper.
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
    /// Converts this material into the raw Box3D representation.
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

    /// Validates that material coefficients are finite and non-negative.
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
/// Shared construction parameters used when attaching a shape to a body.
pub struct ShapeDef {
    raw: ffi::b3ShapeDef,
}

impl ShapeDef {
    #[inline]
    /// Starts a builder initialized with Box3D's default shape definition.
    pub fn builder() -> ShapeDefBuilder {
        ShapeDefBuilder::new()
    }

    #[inline]
    /// Returns the raw Box3D shape definition.
    pub fn raw(&self) -> &ffi::b3ShapeDef {
        &self.raw
    }

    /// Returns the collision filter configured for this shape.
    pub fn filter(&self) -> Filter {
        Filter::from_raw(self.raw.filter)
    }

    /// Returns the base surface material configured for this shape.
    pub fn surface_material(&self) -> SurfaceMaterial {
        SurfaceMaterial::from_raw(self.raw.baseMaterial)
    }

    /// Validates numeric fields and nested material data.
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
/// Builder for `ShapeDef`.
pub struct ShapeDefBuilder {
    def: ShapeDef,
}

impl ShapeDefBuilder {
    #[inline]
    /// Creates a builder using Box3D's default shape definition.
    pub fn new() -> Self {
        Self {
            def: ShapeDef::default(),
        }
    }

    #[inline]
    /// Sets the shape density used for mass properties.
    pub fn density(mut self, density: f32) -> Self {
        self.def.raw.density = density;
        self
    }

    #[inline]
    /// Sets the base material friction coefficient.
    pub fn friction(mut self, friction: f32) -> Self {
        self.def.raw.baseMaterial.friction = friction;
        self
    }

    #[inline]
    /// Sets the base material restitution coefficient.
    pub fn restitution(mut self, restitution: f32) -> Self {
        self.def.raw.baseMaterial.restitution = restitution;
        self
    }

    #[inline]
    /// Sets the collision filter.
    pub fn filter(mut self, filter: Filter) -> Self {
        self.def.raw.filter = filter.into_raw();
        self
    }

    #[inline]
    /// Replaces the complete base surface material.
    pub fn surface_material(mut self, material: SurfaceMaterial) -> Self {
        self.def.raw.baseMaterial = material.into_raw();
        self
    }

    #[inline]
    /// Sets the user material id on the base surface material.
    pub fn user_material_id(mut self, user_material_id: u64) -> Self {
        self.def.raw.baseMaterial.userMaterialId = user_material_id;
        self
    }

    #[inline]
    /// Marks the shape as a sensor instead of a solid collider.
    pub fn sensor(mut self, is_sensor: bool) -> Self {
        self.def.raw.isSensor = is_sensor;
        self
    }

    #[inline]
    /// Enables begin/end sensor overlap events for this shape.
    pub fn enable_sensor_events(mut self, enabled: bool) -> Self {
        self.def.raw.enableSensorEvents = enabled;
        self
    }

    #[inline]
    /// Enables contact begin/end events for this shape.
    pub fn enable_contact_events(mut self, enabled: bool) -> Self {
        self.def.raw.enableContactEvents = enabled;
        self
    }

    #[inline]
    /// Enables high-speed hit events for this shape.
    pub fn enable_hit_events(mut self, enabled: bool) -> Self {
        self.def.raw.enableHitEvents = enabled;
        self
    }

    #[inline]
    /// Enables pre-solve callbacks for this shape.
    pub fn enable_pre_solve_events(mut self, enabled: bool) -> Self {
        self.def.raw.enablePreSolveEvents = enabled;
        self
    }

    #[inline]
    /// Enables custom filtering callbacks for this shape.
    pub fn enable_custom_filtering(mut self, enabled: bool) -> Self {
        self.def.raw.enableCustomFiltering = enabled;
        self
    }

    #[inline]
    /// Finishes the builder and returns the shape definition.
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
/// Sphere shape geometry.
pub struct Sphere {
    raw: ffi::b3Sphere,
}

impl Sphere {
    #[inline]
    /// Creates a sphere from a center point and positive radius.
    pub fn new(center: impl Into<Vec3>, radius: f32) -> Self {
        Self {
            raw: ffi::b3Sphere {
                center: center.into().into_raw(),
                radius,
            },
        }
    }

    #[inline]
    /// Wraps a raw Box3D sphere.
    pub const fn from_raw(raw: ffi::b3Sphere) -> Self {
        Self { raw }
    }

    #[inline]
    /// Returns the raw Box3D sphere.
    pub const fn raw(&self) -> &ffi::b3Sphere {
        &self.raw
    }

    /// Validates that the center is finite and the radius is positive.
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
/// Capsule shape geometry defined by two segment endpoints and a radius.
pub struct Capsule {
    raw: ffi::b3Capsule,
}

impl Capsule {
    #[inline]
    /// Creates a capsule from two centerline endpoints and a positive radius.
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
    /// Wraps a raw Box3D capsule.
    pub const fn from_raw(raw: ffi::b3Capsule) -> Self {
        Self { raw }
    }

    #[inline]
    /// Returns the raw Box3D capsule.
    pub const fn raw(&self) -> &ffi::b3Capsule {
        &self.raw
    }

    /// Validates that endpoints are finite and the radius is positive.
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
/// Convex hull data for box-shaped hulls generated by Box3D.
pub struct BoxHull {
    raw: ffi::b3BoxHull,
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// Result of scaling box half-widths and transform together.
pub struct ScaledBox {
    /// Adjusted box half-widths after scaling and clamping.
    pub half_widths: Vec3,
    /// Adjusted transform that keeps the scaled box representation stable.
    pub transform: Transform,
}

impl BoxHull {
    #[inline]
    /// Creates an axis-aligned cube hull from a positive half-width.
    pub fn cube(half_width: f32) -> Self {
        Self {
            raw: unsafe { ffi::b3MakeCubeHull(half_width) },
        }
    }

    #[inline]
    /// Creates an axis-aligned box hull from positive half-widths.
    pub fn new(hx: f32, hy: f32, hz: f32) -> Self {
        Self {
            raw: unsafe { ffi::b3MakeBoxHull(hx, hy, hz) },
        }
    }

    #[inline]
    /// Creates an axis-aligned box hull offset from the local origin.
    pub fn offset(hx: f32, hy: f32, hz: f32, offset: impl Into<Vec3>) -> Self {
        Self {
            raw: unsafe { ffi::b3MakeOffsetBoxHull(hx, hy, hz, offset.into().into_raw()) },
        }
    }

    #[inline]
    /// Creates a box hull with a local transform baked into the hull data.
    pub fn transformed(hx: f32, hy: f32, hz: f32, transform: Transform) -> Self {
        Self {
            raw: unsafe { ffi::b3MakeTransformedBoxHull(hx, hy, hz, transform.into_raw()) },
        }
    }

    #[inline]
    /// Creates a box hull after applying a post scale.
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

    /// Scales box half-widths and transform while preserving a minimum half-width.
    pub fn scale_box(
        half_widths: impl Into<Vec3>,
        transform: Transform,
        post_scale: impl Into<Vec3>,
        min_half_width: f32,
    ) -> Result<ScaledBox> {
        let mut half_widths = validate_box_half_widths(half_widths.into())?.into_raw();
        let mut transform = transform.validate()?.into_raw();
        let post_scale = post_scale.into().validate()?;
        if !min_half_width.is_finite() || min_half_width <= 0.0 {
            return Err(Error::InvalidArgument);
        }

        unsafe {
            ffi::b3ScaleBox(
                &mut half_widths,
                &mut transform,
                post_scale.into_raw(),
                min_half_width,
            );
        }

        Ok(ScaledBox {
            half_widths: Vec3::from_raw(half_widths).validate()?,
            transform: Transform::from_raw(transform).validate()?,
        })
    }

    #[inline]
    /// Returns the raw Box3D box hull.
    pub const fn raw(&self) -> &ffi::b3BoxHull {
        &self.raw
    }

    #[inline]
    /// Returns the hull portion of the box hull data.
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

#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
fn clamp_height_field_query_bounds(a: Aabb, b: ffi::b3AABB) -> Result<Option<Aabb>> {
    let b = Aabb::from_raw(b).validate()?;
    if a.upper_bound.x < b.lower_bound.x
        || a.lower_bound.x > b.upper_bound.x
        || a.upper_bound.y < b.lower_bound.y
        || a.lower_bound.y > b.upper_bound.y
        || a.upper_bound.z < b.lower_bound.z
        || a.lower_bound.z > b.upper_bound.z
    {
        Ok(None)
    } else {
        Aabb {
            lower_bound: Vec3::new(
                a.lower_bound.x.max(b.lower_bound.x),
                a.lower_bound.y,
                a.lower_bound.z.max(b.lower_bound.z),
            ),
            upper_bound: Vec3::new(
                a.upper_bound.x.min(b.upper_bound.x),
                a.upper_bound.y,
                a.upper_bound.z.min(b.upper_bound.z),
            ),
        }
        .validate()
        .map(Some)
    }
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

fn validate_box_half_widths(half_widths: Vec3) -> Result<Vec3> {
    if half_widths.x.is_finite()
        && half_widths.x > 0.0
        && half_widths.y.is_finite()
        && half_widths.y > 0.0
        && half_widths.z.is_finite()
        && half_widths.z > 0.0
    {
        Ok(half_widths)
    } else {
        Err(Error::InvalidArgument)
    }
}

#[derive(Debug)]
/// Owned convex hull data allocated by Box3D.
///
/// Hull values own native memory and are intentionally not `Send` or `Sync`.
pub struct Hull {
    raw: NonNull<ffi::b3HullData>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl Hull {
    /// Builds a convex hull from points with an upper bound on generated vertices.
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

    /// Creates a cylinder hull.
    pub fn cylinder(height: f32, radius: f32, y_offset: f32, sides: i32) -> Result<Self> {
        if !height.is_finite()
            || height <= 0.0
            || !radius.is_finite()
            || radius <= 0.0
            || !y_offset.is_finite()
            || sides < 3
            || sides > 32
        {
            return Err(Error::InvalidArgument);
        }
        Self::from_ptr(unsafe { ffi::b3CreateCylinder(height, radius, y_offset, sides) })
    }

    /// Creates a cone or truncated cone hull.
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

    /// Creates an irregular rock-like convex hull.
    pub fn rock(radius: f32) -> Result<Self> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(Error::InvalidArgument);
        }
        Self::from_ptr(unsafe { ffi::b3CreateRock(radius) })
    }

    /// Clones this hull into a new owned Box3D allocation.
    pub fn try_clone(&self) -> Result<Self> {
        Self::from_ptr(unsafe { ffi::b3CloneHull(self.as_ptr()) })
    }

    /// Clones this hull after applying a transform and non-zero scale.
    pub fn try_clone_transformed(
        &self,
        transform: Transform,
        scale: impl Into<Vec3>,
    ) -> Result<Self> {
        let transform = transform.validate()?;
        let scale = validate_mesh_scale(scale.into())?;
        Self::from_ptr(unsafe {
            ffi::b3CloneAndTransformHull(self.as_ptr(), transform.into_raw(), scale.into_raw())
        })
    }

    #[inline]
    /// Returns the raw Box3D hull data.
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
/// Options controlling mesh cooking.
pub struct MeshDataOptions {
    /// Vertex welding tolerance used when `weld_vertices` is enabled.
    pub weld_tolerance: f32,
    /// Whether mesh cooking should merge nearby vertices.
    pub weld_vertices: bool,
    /// Whether the cooked mesh BVH should use median splitting.
    pub use_median_split: bool,
    /// Whether Box3D should identify internal and boundary edges.
    pub identify_edges: bool,
}

impl MeshDataOptions {
    #[inline]
    /// Creates default mesh cooking options.
    pub const fn new() -> Self {
        Self {
            weld_tolerance: 0.0,
            weld_vertices: false,
            use_median_split: false,
            identify_edges: true,
        }
    }

    #[inline]
    /// Sets the vertex welding tolerance.
    pub const fn weld_tolerance(mut self, weld_tolerance: f32) -> Self {
        self.weld_tolerance = weld_tolerance;
        self
    }

    #[inline]
    /// Enables or disables vertex welding.
    pub const fn weld_vertices(mut self, weld_vertices: bool) -> Self {
        self.weld_vertices = weld_vertices;
        self
    }

    #[inline]
    /// Enables or disables median splitting for the mesh tree.
    pub const fn use_median_split(mut self, use_median_split: bool) -> Self {
        self.use_median_split = use_median_split;
        self
    }

    #[inline]
    /// Enables or disables edge identification during mesh cooking.
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
/// Builder for `MeshData` triangle meshes.
pub struct MeshDataBuilder {
    vertices: Vec<Vec3>,
    indices: Vec<i32>,
    material_indices: Option<Vec<u8>>,
    options: MeshDataOptions,
}

impl MeshDataBuilder {
    #[inline]
    /// Starts a mesh builder from vertices and triangle indices.
    pub fn new(vertices: impl Into<Vec<Vec3>>, indices: impl Into<Vec<i32>>) -> Self {
        Self {
            vertices: vertices.into(),
            indices: indices.into(),
            material_indices: None,
            options: MeshDataOptions::default(),
        }
    }

    #[inline]
    /// Sets one material index per triangle.
    pub fn material_indices(mut self, material_indices: impl Into<Vec<u8>>) -> Self {
        self.material_indices = Some(material_indices.into());
        self
    }

    #[inline]
    /// Sets the vertex welding tolerance.
    pub fn weld_tolerance(mut self, weld_tolerance: f32) -> Self {
        self.options.weld_tolerance = weld_tolerance;
        self
    }

    #[inline]
    /// Enables or disables vertex welding.
    pub fn weld_vertices(mut self, weld_vertices: bool) -> Self {
        self.options.weld_vertices = weld_vertices;
        self
    }

    #[inline]
    /// Enables or disables median splitting for the mesh tree.
    pub fn use_median_split(mut self, use_median_split: bool) -> Self {
        self.options.use_median_split = use_median_split;
        self
    }

    #[inline]
    /// Enables or disables edge identification.
    pub fn identify_edges(mut self, identify_edges: bool) -> Self {
        self.options.identify_edges = identify_edges;
        self
    }

    #[inline]
    /// Cooks the configured mesh into owned Box3D mesh data.
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
/// Owned cooked triangle mesh data allocated by Box3D.
///
/// Mesh values own native memory and are intentionally not `Send` or `Sync`.
pub struct MeshData {
    raw: NonNull<ffi::b3MeshData>,
    _not_send_sync: PhantomData<Rc<()>>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// Triangle returned by a mesh or height-field overlap query.
pub struct MeshTriangleHit {
    /// First triangle vertex in local shape space.
    pub a: Vec3,
    /// Second triangle vertex in local shape space.
    pub b: Vec3,
    /// Third triangle vertex in local shape space.
    pub c: Vec3,
    /// Index of the triangle in the source mesh or height field.
    pub triangle_index: i32,
}

impl MeshData {
    #[inline]
    /// Starts a mesh builder from vertices and triangle indices.
    pub fn builder(
        vertices: impl Into<Vec<Vec3>>,
        indices: impl Into<Vec<i32>>,
    ) -> MeshDataBuilder {
        MeshDataBuilder::new(vertices, indices)
    }

    /// Cooks triangle vertices and indices into owned mesh data.
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

    /// Creates a generated box mesh.
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

    /// Creates a generated grid mesh.
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

    /// Creates a generated wave mesh.
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

    /// Creates a generated torus mesh.
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

    /// Creates a generated hollow box mesh.
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

    /// Creates a generated platform mesh.
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
    /// Returns the native byte count of the cooked mesh.
    pub fn byte_count(&self) -> i32 {
        unsafe { self.raw.as_ref().byteCount }
    }

    #[inline]
    /// Returns the height of the cooked mesh acceleration tree.
    pub fn tree_height(&self) -> i32 {
        unsafe { ffi::b3GetHeight(self.raw.as_ptr()) }
    }

    #[inline]
    /// Returns the number of cooked mesh vertices.
    pub fn vertex_count(&self) -> i32 {
        unsafe { self.raw.as_ref().vertexCount }
    }

    #[inline]
    /// Returns the number of cooked mesh triangles.
    pub fn triangle_count(&self) -> i32 {
        unsafe { self.raw.as_ref().triangleCount }
    }

    #[inline]
    /// Returns the number of material slots referenced by the cooked mesh.
    pub fn material_count(&self) -> i32 {
        unsafe { self.raw.as_ref().materialCount }
    }

    /// Collects triangles whose bounds overlap an AABB at the given scale.
    pub fn query_triangles(
        &self,
        bounds: Aabb,
        scale: impl Into<Vec3>,
    ) -> Result<Vec<MeshTriangleHit>> {
        let mut out = Vec::new();
        self.query_triangles_into(bounds, scale, &mut out)?;
        Ok(out)
    }

    /// Writes triangles whose bounds overlap an AABB into `out`.
    pub fn query_triangles_into(
        &self,
        bounds: Aabb,
        scale: impl Into<Vec3>,
        out: &mut Vec<MeshTriangleHit>,
    ) -> Result<()> {
        out.clear();
        self.visit_triangles(bounds, scale, |hit| {
            out.push(hit);
            true
        })
    }

    /// Visits triangles whose bounds overlap an AABB at the given scale.
    ///
    /// Return `false` from the visitor to stop traversal early.
    pub fn visit_triangles<F>(&self, bounds: Aabb, scale: impl Into<Vec3>, visitor: F) -> Result<()>
    where
        F: FnMut(MeshTriangleHit) -> bool,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = (bounds, scale, visitor);
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let bounds = bounds.validate()?;
            let scale = validate_mesh_scale(scale.into())?;
            let raw_mesh = ffi::b3Mesh {
                data: self.as_ptr(),
                scale: scale.into_raw(),
            };
            let mut ctx = MeshTriangleQueryContext {
                visitor,
                panicked: false,
            };
            let _guard = box3d_lock::lock();
            unsafe {
                ffi::b3QueryMesh(
                    &raw_mesh,
                    bounds.into_raw(),
                    Some(mesh_triangle_query_trampoline::<F>),
                    (&mut ctx as *mut MeshTriangleQueryContext<_>).cast(),
                );
            }
            if ctx.panicked {
                Err(Error::CallbackPanicked)
            } else {
                Ok(())
            }
        }
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
/// Builder for `HeightField`.
pub struct HeightFieldBuilder {
    row_count: i32,
    column_count: i32,
    heights: Vec<f32>,
    material_indices: Option<Vec<u8>>,
    clockwise_winding: bool,
}

impl HeightFieldBuilder {
    #[inline]
    /// Starts a height-field builder from row count, column count, and samples.
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
    /// Sets one material index per height-field cell.
    pub fn material_indices(mut self, material_indices: impl Into<Vec<u8>>) -> Self {
        self.material_indices = Some(material_indices.into());
        self
    }

    #[inline]
    /// Sets whether cells use clockwise triangle winding.
    pub fn clockwise_winding(mut self, clockwise_winding: bool) -> Self {
        self.clockwise_winding = clockwise_winding;
        self
    }

    #[inline]
    /// Builds owned height-field data using the supplied sample scale.
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
/// Owned height-field data allocated by Box3D.
///
/// Height fields own native memory and are intentionally not `Send` or `Sync`.
pub struct HeightField {
    raw: NonNull<ffi::b3HeightFieldData>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl HeightField {
    #[inline]
    /// Starts a height-field builder from row count, column count, and samples.
    pub fn builder(
        row_count: i32,
        column_count: i32,
        heights: impl Into<Vec<f32>>,
    ) -> HeightFieldBuilder {
        HeightFieldBuilder::new(row_count, column_count, heights)
    }

    /// Creates a height field from explicit samples.
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

    /// Creates a generated grid height field.
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

    /// Creates a generated wave height field.
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
    /// Returns the native byte count of the height-field data.
    pub fn byte_count(&self) -> i32 {
        unsafe { self.raw.as_ref().byteCount }
    }

    #[inline]
    /// Returns the number of sample rows.
    pub fn row_count(&self) -> i32 {
        unsafe { self.raw.as_ref().rowCount }
    }

    #[inline]
    /// Returns the number of sample columns.
    pub fn column_count(&self) -> i32 {
        unsafe { self.raw.as_ref().columnCount }
    }

    /// Collects height-field triangles whose bounds overlap an AABB.
    pub fn query_triangles(&self, bounds: Aabb) -> Result<Vec<MeshTriangleHit>> {
        let mut out = Vec::new();
        self.query_triangles_into(bounds, &mut out)?;
        Ok(out)
    }

    /// Writes height-field triangles whose bounds overlap an AABB into `out`.
    pub fn query_triangles_into(&self, bounds: Aabb, out: &mut Vec<MeshTriangleHit>) -> Result<()> {
        out.clear();
        self.visit_triangles(bounds, |hit| out.push(hit))
    }

    /// Visits height-field triangles whose bounds overlap an AABB.
    pub fn visit_triangles<F>(&self, bounds: Aabb, visitor: F) -> Result<()>
    where
        F: FnMut(MeshTriangleHit),
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = (bounds, visitor);
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let Some(bounds) = clamp_height_field_query_bounds(bounds.validate()?, unsafe {
                self.raw.as_ref().aabb
            })?
            else {
                return Ok(());
            };
            let mut ctx = HeightFieldTriangleQueryContext {
                visitor,
                panicked: false,
            };
            let _guard = box3d_lock::lock();
            unsafe {
                ffi::b3QueryHeightField(
                    self.as_ptr(),
                    bounds.into_raw(),
                    Some(height_field_triangle_query_trampoline::<F>),
                    (&mut ctx as *mut HeightFieldTriangleQueryContext<_>).cast(),
                );
            }
            if ctx.panicked {
                Err(Error::CallbackPanicked)
            } else {
                Ok(())
            }
        }
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

#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
struct MeshTriangleQueryContext<F> {
    visitor: F,
    panicked: bool,
}

#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
unsafe extern "C" fn mesh_triangle_query_trampoline<F>(
    a: ffi::b3Vec3,
    b: ffi::b3Vec3,
    c: ffi::b3Vec3,
    triangle_index: i32,
    context: *mut c_void,
) -> bool
where
    F: FnMut(MeshTriangleHit) -> bool,
{
    let _guard = callback_state::CallbackGuard::enter();
    let ctx = unsafe { &mut *context.cast::<MeshTriangleQueryContext<F>>() };
    if ctx.panicked {
        return false;
    }
    let hit = MeshTriangleHit {
        a: Vec3::from_raw(a),
        b: Vec3::from_raw(b),
        c: Vec3::from_raw(c),
        triangle_index,
    };
    match catch_unwind(AssertUnwindSafe(|| (ctx.visitor)(hit))) {
        Ok(keep_going) => keep_going,
        Err(_) => {
            ctx.panicked = true;
            false
        }
    }
}

#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
struct HeightFieldTriangleQueryContext<F> {
    visitor: F,
    panicked: bool,
}

#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
unsafe extern "C" fn height_field_triangle_query_trampoline<F>(
    a: ffi::b3Vec3,
    b: ffi::b3Vec3,
    c: ffi::b3Vec3,
    triangle_index: i32,
    context: *mut c_void,
) -> bool
where
    F: FnMut(MeshTriangleHit),
{
    let _guard = callback_state::CallbackGuard::enter();
    let ctx = unsafe { &mut *context.cast::<HeightFieldTriangleQueryContext<F>>() };
    if ctx.panicked {
        return false;
    }
    let hit = MeshTriangleHit {
        a: Vec3::from_raw(a),
        b: Vec3::from_raw(b),
        c: Vec3::from_raw(c),
        triangle_index,
    };
    match catch_unwind(AssertUnwindSafe(|| (ctx.visitor)(hit))) {
        Ok(()) => true,
        Err(_) => {
            ctx.panicked = true;
            false
        }
    }
}

#[derive(Copy, Clone, Debug)]
/// Borrowed view of hull data owned by another Box3D shape resource.
///
/// This type does not own native storage. It is tied to the `World` or
/// `Compound` borrow that produced it and becomes invalid if the owning shape,
/// compound, or resource-backed geometry is destroyed or replaced.
pub struct ShapeHull<'a> {
    raw: &'a ffi::b3HullData,
}

impl<'a> ShapeHull<'a> {
    #[inline]
    pub(crate) const fn from_raw(raw: &'a ffi::b3HullData) -> Self {
        Self { raw }
    }

    #[inline]
    /// Returns the native byte count of the hull data.
    pub const fn byte_count(&self) -> i32 {
        self.raw.byteCount
    }

    #[inline]
    /// Returns Box3D's stable hash for the hull data.
    pub const fn hash(&self) -> u32 {
        self.raw.hash
    }

    #[inline]
    /// Returns the hull's local-space AABB.
    pub const fn aabb(&self) -> Aabb {
        Aabb::from_raw(self.raw.aabb)
    }

    #[inline]
    /// Returns the hull surface area.
    pub const fn surface_area(&self) -> f32 {
        self.raw.surfaceArea
    }

    #[inline]
    /// Returns the hull volume.
    pub const fn volume(&self) -> f32 {
        self.raw.volume
    }

    #[inline]
    /// Returns the hull inner radius.
    pub const fn inner_radius(&self) -> f32 {
        self.raw.innerRadius
    }

    #[inline]
    /// Returns the hull center of mass.
    pub const fn center(&self) -> Vec3 {
        Vec3::from_raw(self.raw.center)
    }

    #[inline]
    /// Returns the number of hull vertices.
    pub const fn vertex_count(&self) -> i32 {
        self.raw.vertexCount
    }

    #[inline]
    /// Returns the number of hull half-edges.
    pub const fn edge_count(&self) -> i32 {
        self.raw.edgeCount
    }

    #[inline]
    /// Returns the number of hull faces.
    pub const fn face_count(&self) -> i32 {
        self.raw.faceCount
    }
}

#[derive(Copy, Clone, Debug)]
/// Borrowed view of mesh data owned by another Box3D shape resource.
///
/// This type does not own native storage. It is tied to the `World` or
/// `Compound` borrow that produced it and becomes invalid if the owning shape,
/// compound, or resource-backed geometry is destroyed or replaced.
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
    /// Returns the scale applied to this mesh instance.
    pub const fn scale(&self) -> Vec3 {
        self.scale
    }

    #[inline]
    /// Returns the native byte count of the mesh data.
    pub const fn byte_count(&self) -> i32 {
        self.data.byteCount
    }

    #[inline]
    /// Returns Box3D's stable hash for the mesh data.
    pub const fn hash(&self) -> u32 {
        self.data.hash
    }

    #[inline]
    /// Returns the mesh local-space bounds.
    pub const fn bounds(&self) -> Aabb {
        Aabb::from_raw(self.data.bounds)
    }

    #[inline]
    /// Returns the mesh surface area.
    pub const fn surface_area(&self) -> f32 {
        self.data.surfaceArea
    }

    #[inline]
    /// Returns the height of the mesh acceleration tree.
    pub const fn tree_height(&self) -> i32 {
        self.data.treeHeight
    }

    #[inline]
    /// Returns the number of degenerate triangles found during cooking.
    pub const fn degenerate_count(&self) -> i32 {
        self.data.degenerateCount
    }

    #[inline]
    /// Returns the number of mesh vertices.
    pub const fn vertex_count(&self) -> i32 {
        self.data.vertexCount
    }

    #[inline]
    /// Returns the number of mesh triangles.
    pub const fn triangle_count(&self) -> i32 {
        self.data.triangleCount
    }

    #[inline]
    /// Returns the number of material slots referenced by the mesh.
    pub const fn material_count(&self) -> i32 {
        self.data.materialCount
    }
}

#[derive(Copy, Clone, Debug)]
/// Borrowed view of height-field data owned by another Box3D shape resource.
///
/// This type does not own native storage. It is tied to the `World` borrow that
/// produced it and becomes invalid if the owning shape or resource-backed
/// geometry is destroyed or replaced.
pub struct ShapeHeightField<'a> {
    raw: &'a ffi::b3HeightFieldData,
}

impl<'a> ShapeHeightField<'a> {
    #[inline]
    pub(crate) const fn from_raw(raw: &'a ffi::b3HeightFieldData) -> Self {
        Self { raw }
    }

    #[inline]
    /// Returns the native byte count of the height-field data.
    pub const fn byte_count(&self) -> i32 {
        self.raw.byteCount
    }

    #[inline]
    /// Returns Box3D's stable hash for the height-field data.
    pub const fn hash(&self) -> u32 {
        self.raw.hash
    }

    #[inline]
    /// Returns the height-field local-space AABB.
    pub const fn aabb(&self) -> Aabb {
        Aabb::from_raw(self.raw.aabb)
    }

    #[inline]
    /// Returns the minimum sample height.
    pub const fn min_height(&self) -> f32 {
        self.raw.minHeight
    }

    #[inline]
    /// Returns the maximum sample height.
    pub const fn max_height(&self) -> f32 {
        self.raw.maxHeight
    }

    #[inline]
    /// Returns the sample scale used by the height field.
    pub const fn scale(&self) -> Vec3 {
        Vec3::from_raw(self.raw.scale)
    }

    #[inline]
    /// Returns the number of sample columns.
    pub const fn column_count(&self) -> i32 {
        self.raw.columnCount
    }

    #[inline]
    /// Returns the number of sample rows.
    pub const fn row_count(&self) -> i32 {
        self.raw.rowCount
    }

    #[inline]
    /// Returns whether cells use clockwise triangle winding.
    pub const fn clockwise(&self) -> bool {
        self.raw.clockwise
    }
}

#[derive(Debug)]
/// Owned compound shape data allocated by Box3D.
///
/// A compound stores multiple primitive children and shared geometry resources
/// in a single native allocation.
pub struct Compound {
    raw: NonNull<ffi::b3CompoundData>,
    _not_send_sync: PhantomData<Rc<()>>,
}

#[derive(Debug)]
/// Serialized bytes for a `Compound`.
///
/// The byte buffer is still owned by Box3D and can be converted back into an
/// owned `Compound`.
pub struct CompoundBytes {
    raw: NonNull<u8>,
    byte_count: i32,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl Compound {
    #[inline]
    /// Starts a compound builder.
    pub fn builder<'a>() -> CompoundBuilder<'a> {
        CompoundBuilder::new()
    }

    /// Creates a compound containing exactly one sphere child.
    pub fn single_sphere(sphere: Sphere, material: SurfaceMaterial) -> Result<Self> {
        Self::builder().with_sphere(sphere, material)?.build()
    }

    #[inline]
    /// Returns the native byte count of the compound data.
    pub fn byte_count(&self) -> i32 {
        unsafe { self.raw.as_ref().byteCount }
    }

    #[inline]
    /// Returns the total number of child shapes.
    pub fn child_count(&self) -> i32 {
        unsafe {
            let raw = self.raw.as_ref();
            raw.capsuleCount + raw.hullCount + raw.meshCount + raw.sphereCount
        }
    }

    #[inline]
    /// Returns the number of capsule children.
    pub fn capsule_count(&self) -> i32 {
        unsafe { self.raw.as_ref().capsuleCount }
    }

    #[inline]
    /// Returns the number of hull children.
    pub fn hull_count(&self) -> i32 {
        unsafe { self.raw.as_ref().hullCount }
    }

    #[inline]
    /// Returns the number of mesh children.
    pub fn mesh_count(&self) -> i32 {
        unsafe { self.raw.as_ref().meshCount }
    }

    #[inline]
    /// Returns the number of sphere children.
    pub fn sphere_count(&self) -> i32 {
        unsafe { self.raw.as_ref().sphereCount }
    }

    #[inline]
    /// Returns the number of material records stored by the compound.
    pub fn material_count(&self) -> i32 {
        unsafe { self.raw.as_ref().materialCount }
    }

    #[inline]
    /// Returns the number of shared hull resources stored by the compound.
    pub fn shared_hull_count(&self) -> i32 {
        unsafe { self.raw.as_ref().sharedHullCount }
    }

    #[inline]
    /// Returns the number of shared mesh resources stored by the compound.
    pub fn shared_mesh_count(&self) -> i32 {
        unsafe { self.raw.as_ref().sharedMeshCount }
    }

    /// Returns a material by compound material index.
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

    /// Returns a child by flattened child index.
    pub fn child(&self, index: i32) -> Result<CompoundChild<'_>> {
        if index < 0 || index >= self.child_count() {
            return Err(Error::IndexOutOfRange);
        }
        CompoundChild::from_raw(unsafe { ffi::b3GetCompoundChild(self.raw.as_ptr(), index) })
    }

    /// Returns a capsule child by capsule-child index.
    pub fn capsule_child(&self, index: i32) -> Result<CompoundCapsule> {
        if index < 0 || index >= self.capsule_count() {
            return Err(Error::IndexOutOfRange);
        }
        let raw = unsafe { ffi::b3GetCompoundCapsule(self.raw.as_ptr(), index) };
        Ok(CompoundCapsule::from_raw(raw))
    }

    /// Returns a hull child by hull-child index.
    pub fn hull_child(&self, index: i32) -> Result<CompoundHull<'_>> {
        if index < 0 || index >= self.hull_count() {
            return Err(Error::IndexOutOfRange);
        }
        let raw = unsafe { ffi::b3GetCompoundHull(self.raw.as_ptr(), index) };
        CompoundHull::from_raw(raw)
    }

    /// Returns a mesh child by mesh-child index.
    pub fn mesh_child(&self, index: i32) -> Result<CompoundMesh<'_>> {
        if index < 0 || index >= self.mesh_count() {
            return Err(Error::IndexOutOfRange);
        }
        let raw = unsafe { ffi::b3GetCompoundMesh(self.raw.as_ptr(), index) };
        CompoundMesh::from_raw(raw)
    }

    /// Returns a sphere child by sphere-child index.
    pub fn sphere_child(&self, index: i32) -> Result<CompoundSphere> {
        if index < 0 || index >= self.sphere_count() {
            return Err(Error::IndexOutOfRange);
        }
        let raw = unsafe { ffi::b3GetCompoundSphere(self.raw.as_ptr(), index) };
        Ok(CompoundSphere::from_raw(raw))
    }

    /// Collects compound children whose bounds overlap an AABB.
    pub fn query_aabb(&self, aabb: Aabb) -> Result<Vec<CompoundQueryHit<'_>>> {
        let mut out = Vec::new();
        self.query_aabb_into(aabb, &mut out)?;
        Ok(out)
    }

    /// Writes compound children whose bounds overlap an AABB into `out`.
    pub fn query_aabb_into<'a>(
        &'a self,
        aabb: Aabb,
        out: &mut Vec<CompoundQueryHit<'a>>,
    ) -> Result<()> {
        out.clear();
        self.visit_query_aabb(aabb, |hit| {
            out.push(hit);
            true
        })
    }

    /// Visits compound children whose bounds overlap an AABB.
    ///
    /// Return `false` from the visitor to stop traversal early.
    pub fn visit_query_aabb<'a, F>(&'a self, aabb: Aabb, visitor: F) -> Result<()>
    where
        F: FnMut(CompoundQueryHit<'a>) -> bool,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = (aabb, visitor);
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let aabb = aabb.validate()?;
            let mut ctx = CompoundQueryContext {
                visitor,
                error: None,
                panicked: false,
                _lifetime: PhantomData,
            };
            let _guard = box3d_lock::lock();
            unsafe {
                ffi::b3QueryCompound(
                    self.raw.as_ptr(),
                    aabb.into_raw(),
                    Some(compound_query_trampoline::<F>),
                    (&mut ctx as *mut CompoundQueryContext<'a, F>).cast(),
                );
            }
            if ctx.panicked {
                Err(Error::CallbackPanicked)
            } else if let Some(error) = ctx.error {
                Err(error)
            } else {
                Ok(())
            }
        }
    }

    /// Converts this compound into Box3D-owned serialized bytes.
    pub fn into_bytes(self) -> CompoundBytes {
        let compound = ManuallyDrop::new(self);
        let byte_count = compound.byte_count();
        let raw = unsafe { ffi::b3ConvertCompoundToBytes(compound.raw.as_ptr()) };
        CompoundBytes {
            raw: NonNull::new(raw).expect("valid Compound converted to null bytes"),
            byte_count,
            _not_send_sync: PhantomData,
        }
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

impl CompoundBytes {
    #[inline]
    /// Returns the number of serialized bytes.
    pub const fn byte_count(&self) -> i32 {
        self.byte_count
    }

    #[inline]
    /// Borrows the serialized byte buffer.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.raw.as_ptr(), self.byte_count as usize) }
    }

    /// Converts the serialized bytes back into an owned compound.
    pub fn into_compound(self) -> Result<Compound> {
        let raw = unsafe { ffi::b3ConvertBytesToCompound(self.raw.as_ptr(), self.byte_count) };
        let compound = Compound::from_ptr(raw)?;
        forget(self);
        Ok(compound)
    }
}

impl Drop for CompoundBytes {
    fn drop(&mut self) {
        unsafe { ffi::b3DestroyCompound(self.raw.as_ptr().cast()) };
    }
}

#[derive(Debug)]
/// Builder for `Compound`.
///
/// Borrowed hull and mesh inputs must outlive the builder until `build` is
/// called because Box3D reads them during compound creation.
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
    /// Creates an empty compound builder.
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

    /// Adds a sphere child, storing any validation error until `build`.
    pub fn sphere(mut self, sphere: Sphere, material: SurfaceMaterial) -> Self {
        if let Err(error) = self.add_sphere(sphere, material) {
            self.error = Some(error);
        }
        self
    }

    /// Adds a sphere child and returns validation errors immediately.
    pub fn with_sphere(mut self, sphere: Sphere, material: SurfaceMaterial) -> Result<Self> {
        self.add_sphere(sphere, material)?;
        Ok(self)
    }

    /// Adds a sphere child to the builder in place.
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

    /// Adds a capsule child, storing any validation error until `build`.
    pub fn capsule(mut self, capsule: Capsule, material: SurfaceMaterial) -> Self {
        if let Err(error) = self.add_capsule(capsule, material) {
            self.error = Some(error);
        }
        self
    }

    /// Adds a capsule child and returns validation errors immediately.
    pub fn with_capsule(mut self, capsule: Capsule, material: SurfaceMaterial) -> Result<Self> {
        self.add_capsule(capsule, material)?;
        Ok(self)
    }

    /// Adds a capsule child to the builder in place.
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

    /// Adds a hull child, storing any validation error until `build`.
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

    /// Adds a hull child and returns validation errors immediately.
    pub fn with_hull(
        mut self,
        hull: &'a Hull,
        transform: impl Into<Transform>,
        material: SurfaceMaterial,
    ) -> Result<Self> {
        self.add_hull(hull, transform, material)?;
        Ok(self)
    }

    /// Adds a hull child to the builder in place.
    pub fn add_hull(
        &mut self,
        hull: &'a Hull,
        transform: impl Into<Transform>,
        material: SurfaceMaterial,
    ) -> Result<&mut Self> {
        self.add_hull_ptr(hull.as_ptr(), transform.into(), material)
    }

    /// Adds a generated box hull child, storing any validation error until `build`.
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

    /// Adds a generated box hull child and returns validation errors immediately.
    pub fn with_box_hull(
        mut self,
        hull: &'a BoxHull,
        transform: impl Into<Transform>,
        material: SurfaceMaterial,
    ) -> Result<Self> {
        self.add_box_hull(hull, transform, material)?;
        Ok(self)
    }

    /// Adds a generated box hull child to the builder in place.
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

    /// Adds a mesh child, storing any validation error until `build`.
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

    /// Adds a mesh child and returns validation errors immediately.
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

    /// Adds a mesh child to the builder in place.
    ///
    /// `materials` must contain exactly `mesh.material_count()` entries and no
    /// more than `MAX_COMPOUND_MESH_MATERIALS` entries.
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

    /// Builds the compound data.
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
/// Capsule child returned from a compound.
pub struct CompoundCapsule {
    /// Capsule geometry.
    pub capsule: Capsule,
    /// Index into the compound material table.
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
/// Hull child returned from a compound.
pub struct CompoundHull<'a> {
    /// Borrowed hull geometry.
    pub hull: ShapeHull<'a>,
    /// Child transform in compound-local space.
    pub transform: Transform,
    /// Index into the compound material table.
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
/// Mesh child returned from a compound.
pub struct CompoundMesh<'a> {
    /// Borrowed mesh geometry and per-child scale.
    pub mesh: ShapeMesh<'a>,
    /// Child transform in compound-local space.
    pub transform: Transform,
    /// Material indices used by the mesh child's material slots.
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
/// Sphere child returned from a compound.
pub struct CompoundSphere {
    /// Sphere geometry.
    pub sphere: Sphere,
    /// Index into the compound material table.
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
/// Flattened child returned from generic compound indexing and queries.
///
/// Hull and mesh children borrow native storage owned by the parent
/// [`Compound`]. Keep this value scoped to the compound borrow that produced it.
pub struct CompoundChild<'a> {
    /// Child geometry stored by a compound shape.
    pub shape: CompoundChildShape<'a>,
    /// Child transform in compound-local space.
    pub transform: Transform,
    /// Material indices used by the child.
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
    /// Returns the child shape type.
    pub const fn shape_type(&self) -> ShapeType {
        self.shape.shape_type()
    }

    #[inline]
    /// Returns the first material index for the child.
    pub const fn primary_material_index(&self) -> i32 {
        self.material_indices[0]
    }
}

#[derive(Copy, Clone, Debug)]
/// Hit returned by a compound AABB query.
///
/// The contained child borrows native storage owned by the queried
/// [`Compound`]. Do not retain it beyond the compound borrow.
pub struct CompoundQueryHit<'a> {
    /// Flattened child index hit by the query.
    pub child_index: i32,
    /// Borrowed child data for the hit.
    pub child: CompoundChild<'a>,
}

#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
struct CompoundQueryContext<'a, F> {
    visitor: F,
    error: Option<Error>,
    panicked: bool,
    _lifetime: PhantomData<&'a Compound>,
}

#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
unsafe extern "C" fn compound_query_trampoline<'a, F>(
    compound: *const ffi::b3CompoundData,
    child_index: i32,
    context: *mut c_void,
) -> bool
where
    F: FnMut(CompoundQueryHit<'a>) -> bool,
{
    let _guard = callback_state::CallbackGuard::enter();
    let ctx = unsafe { &mut *context.cast::<CompoundQueryContext<'a, F>>() };
    if ctx.panicked || ctx.error.is_some() {
        return false;
    }

    if compound.is_null() || child_index < 0 {
        ctx.error = Some(Error::InvalidArgument);
        return false;
    }

    let raw_child = unsafe { ffi::b3GetCompoundChild(compound, child_index) };
    let child = match CompoundChild::from_raw(raw_child) {
        Ok(child) => child,
        Err(error) => {
            ctx.error = Some(error);
            return false;
        }
    };
    let hit = CompoundQueryHit { child_index, child };

    match catch_unwind(AssertUnwindSafe(|| (ctx.visitor)(hit))) {
        Ok(keep_going) => keep_going,
        Err(_) => {
            ctx.panicked = true;
            false
        }
    }
}

#[derive(Copy, Clone, Debug)]
/// Shape variants that can appear inside a compound child.
pub enum CompoundChildShape<'a> {
    /// Capsule child geometry.
    Capsule(Capsule),
    /// Hull child geometry.
    Hull(ShapeHull<'a>),
    /// Mesh child geometry.
    Mesh(ShapeMesh<'a>),
    /// Sphere child geometry.
    Sphere(Sphere),
}

impl<'a> CompoundChildShape<'a> {
    #[inline]
    /// Returns the shape type represented by this variant.
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
/// Box3D shape type tag.
pub enum ShapeType {
    /// Capsule shape.
    Capsule,
    /// Compound shape.
    Compound,
    /// Height-field shape.
    HeightField,
    /// Convex hull shape.
    Hull,
    /// Triangle mesh shape.
    Mesh,
    /// Sphere shape.
    Sphere,
}

impl ShapeType {
    /// Converts a raw Box3D shape type into the safe enum.
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
