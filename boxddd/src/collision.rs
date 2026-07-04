#![cfg_attr(all(target_arch = "wasm32", boxddd_wasm_provider), allow(dead_code))]

use crate::core::box3d_lock;
use crate::error::{Error, Result};
use crate::shapes::{Capsule, Compound, HeightField, Hull, MeshData, Sphere, validate_mesh_scale};
use crate::types::{Aabb, MassData, Plane, Quat, Transform, Vec3};
use boxddd_sys::ffi;
use std::mem::MaybeUninit;

/// Maximum point count accepted by a shape proxy.
pub const MAX_SHAPE_PROXY_POINTS: usize = ffi::B3_MAX_SHAPE_CAST_POINTS as usize;
/// Maximum contact point count returned by local manifold helpers.
pub const MAX_LOCAL_MANIFOLD_POINTS: usize = ffi::B3_MAX_MANIFOLD_POINTS as usize;

#[derive(Clone, Debug, PartialEq)]
/// Convex point cloud proxy used by distance, overlap, and shape-cast helpers.
pub struct ShapeProxy {
    points: Vec<Vec3>,
    radius: f32,
}

impl ShapeProxy {
    /// Creates a proxy from support points and a non-negative radius.
    pub fn new(points: impl Into<Vec<Vec3>>, radius: f32) -> Result<Self> {
        let points = points.into();
        if points.is_empty()
            || points.len() > MAX_SHAPE_PROXY_POINTS
            || !radius.is_finite()
            || radius < 0.0
            || points.iter().any(|point| !point.is_valid())
        {
            return Err(Error::InvalidArgument);
        }
        Ok(Self { points, radius })
    }

    /// Creates a sphere proxy centered at the origin.
    pub fn sphere(radius: f32) -> Result<Self> {
        Self::new(vec![Vec3::ZERO], radius)
    }

    /// Creates a capsule proxy from two centerline points and a radius.
    pub fn capsule(
        center1: impl Into<Vec3>,
        center2: impl Into<Vec3>,
        radius: f32,
    ) -> Result<Self> {
        Self::new(vec![center1.into(), center2.into()], radius)
    }

    #[inline]
    /// Returns the convex support points passed to GJK-style queries and casts.
    pub fn points(&self) -> &[Vec3] {
        &self.points
    }

    #[inline]
    /// Returns the convex radius applied around each proxy support point.
    pub fn radius(&self) -> f32 {
        self.radius
    }

    #[inline]
    pub(crate) fn raw(&self) -> ffi::b3ShapeProxy {
        ffi::b3ShapeProxy {
            points: self.points.as_ptr().cast(),
            count: self.points.len() as i32,
            radius: self.radius,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
/// Ray-cast input in local shape space.
pub struct RayCastInput {
    /// Ray origin.
    pub origin: Vec3,
    /// Ray translation vector.
    pub translation: Vec3,
    /// Maximum fraction of the translation to consider.
    pub max_fraction: f32,
}

impl RayCastInput {
    /// Creates a ray-cast input with `max_fraction` set to `1.0`.
    pub fn new(origin: impl Into<Vec3>, translation: impl Into<Vec3>) -> Result<Self> {
        Self::with_max_fraction(origin, translation, 1.0)
    }

    /// Creates a ray-cast input with an explicit maximum fraction.
    pub fn with_max_fraction(
        origin: impl Into<Vec3>,
        translation: impl Into<Vec3>,
        max_fraction: f32,
    ) -> Result<Self> {
        let input = Self {
            origin: origin.into(),
            translation: translation.into(),
            max_fraction,
        };
        if input.origin.is_valid()
            && input.translation.is_valid()
            && input.max_fraction.is_finite()
            && input.max_fraction >= 0.0
        {
            Ok(input)
        } else {
            Err(Error::InvalidArgument)
        }
    }

    #[inline]
    pub(crate) fn validate(self) -> Result<Self> {
        if self.origin.is_valid()
            && self.translation.is_valid()
            && self.max_fraction.is_finite()
            && self.max_fraction >= 0.0
        {
            Ok(self)
        } else {
            Err(Error::InvalidArgument)
        }
    }

    #[inline]
    pub(crate) fn raw(&self) -> ffi::b3RayCastInput {
        ffi::b3RayCastInput {
            origin: self.origin.into_raw(),
            translation: self.translation.into_raw(),
            maxFraction: self.max_fraction,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
/// Box-cast input for sweeping an AABB.
pub struct BoxCastInput {
    /// Box bounds at the start of the cast.
    pub aabb: Aabb,
    /// Cast translation.
    pub translation: Vec3,
    /// Maximum fraction of the translation to consider.
    pub max_fraction: f32,
}

impl BoxCastInput {
    /// Creates a box-cast input with `max_fraction` set to `1.0`.
    pub fn new(aabb: Aabb, translation: impl Into<Vec3>) -> Result<Self> {
        Self::with_max_fraction(aabb, translation, 1.0)
    }

    /// Creates a box-cast input with an explicit maximum fraction.
    pub fn with_max_fraction(
        aabb: Aabb,
        translation: impl Into<Vec3>,
        max_fraction: f32,
    ) -> Result<Self> {
        let input = Self {
            aabb,
            translation: translation.into(),
            max_fraction,
        };
        if input.aabb.is_valid()
            && input.translation.is_valid()
            && input.max_fraction.is_finite()
            && input.max_fraction >= 0.0
        {
            Ok(input)
        } else {
            Err(Error::InvalidArgument)
        }
    }

    #[inline]
    pub(crate) fn validate(self) -> Result<Self> {
        if self.aabb.is_valid()
            && self.translation.is_valid()
            && self.max_fraction.is_finite()
            && self.max_fraction >= 0.0
        {
            Ok(self)
        } else {
            Err(Error::InvalidArgument)
        }
    }

    #[inline]
    pub(crate) fn raw(&self) -> ffi::b3BoxCastInput {
        ffi::b3BoxCastInput {
            box_: self.aabb.into_raw(),
            translation: self.translation.into_raw(),
            maxFraction: self.max_fraction,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Shape-cast input for sweeping a convex proxy.
pub struct ShapeCastInput {
    /// Convex proxy to sweep.
    pub proxy: ShapeProxy,
    /// Cast translation.
    pub translation: Vec3,
    /// Maximum fraction of the translation to consider.
    pub max_fraction: f32,
    /// Whether the cast may start slightly encroached on the target.
    pub can_encroach: bool,
}

impl ShapeCastInput {
    /// Creates a shape-cast input with default options.
    pub fn new(proxy: ShapeProxy, translation: impl Into<Vec3>) -> Result<Self> {
        Self::with_options(proxy, translation, 1.0, false)
    }

    /// Creates a shape-cast input with explicit options.
    pub fn with_options(
        proxy: ShapeProxy,
        translation: impl Into<Vec3>,
        max_fraction: f32,
        can_encroach: bool,
    ) -> Result<Self> {
        let input = Self {
            proxy,
            translation: translation.into(),
            max_fraction,
            can_encroach,
        };
        if input.translation.is_valid()
            && input.max_fraction.is_finite()
            && input.max_fraction >= 0.0
        {
            Ok(input)
        } else {
            Err(Error::InvalidArgument)
        }
    }

    #[inline]
    pub(crate) fn validate(self) -> Result<Self> {
        if self.translation.is_valid() && self.max_fraction.is_finite() && self.max_fraction >= 0.0
        {
            Ok(self)
        } else {
            Err(Error::InvalidArgument)
        }
    }

    #[inline]
    pub(crate) fn raw(&self) -> ffi::b3ShapeCastInput {
        ffi::b3ShapeCastInput {
            proxy: self.proxy.raw(),
            translation: self.translation.into_raw(),
            maxFraction: self.max_fraction,
            canEncroach: self.can_encroach,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Input for computing distance between two convex proxies.
pub struct DistanceInput {
    /// First proxy, expressed in frame A.
    pub proxy_a: ShapeProxy,
    /// Second proxy, expressed in frame B.
    pub proxy_b: ShapeProxy,
    /// Transform that moves points from frame B into frame A.
    pub transform_b_to_a: Transform,
    /// Whether proxy radii should be included in the distance result.
    pub use_radii: bool,
}

impl DistanceInput {
    /// Creates distance input that includes proxy radii.
    pub fn new(
        proxy_a: ShapeProxy,
        proxy_b: ShapeProxy,
        transform_b_to_a: Transform,
    ) -> Result<Self> {
        Self::with_options(proxy_a, proxy_b, transform_b_to_a, true)
    }

    /// Creates distance input with explicit radius handling.
    pub fn with_options(
        proxy_a: ShapeProxy,
        proxy_b: ShapeProxy,
        transform_b_to_a: Transform,
        use_radii: bool,
    ) -> Result<Self> {
        transform_b_to_a.validate()?;
        Ok(Self {
            proxy_a,
            proxy_b,
            transform_b_to_a,
            use_radii,
        })
    }

    #[inline]
    fn raw(&self) -> ffi::b3DistanceInput {
        ffi::b3DistanceInput {
            proxyA: self.proxy_a.raw(),
            proxyB: self.proxy_b.raw(),
            transform: self.transform_b_to_a.into_raw(),
            useRadii: self.use_radii,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Input for sweeping one convex proxy against another.
pub struct ShapeCastPairInput {
    /// First proxy, expressed in frame A.
    pub proxy_a: ShapeProxy,
    /// Second proxy, expressed in frame B.
    pub proxy_b: ShapeProxy,
    /// Transform that moves points from frame B into frame A.
    pub transform_b_to_a: Transform,
    /// Translation applied to proxy B.
    pub translation_b: Vec3,
    /// Maximum fraction of the translation to consider.
    pub max_fraction: f32,
    /// Whether the cast may start slightly encroached.
    pub can_encroach: bool,
}

impl ShapeCastPairInput {
    /// Creates pair shape-cast input with default options.
    pub fn new(
        proxy_a: ShapeProxy,
        proxy_b: ShapeProxy,
        transform_b_to_a: Transform,
        translation_b: impl Into<Vec3>,
    ) -> Result<Self> {
        Self::with_options(
            proxy_a,
            proxy_b,
            transform_b_to_a,
            translation_b,
            1.0,
            false,
        )
    }

    /// Creates pair shape-cast input with explicit options.
    pub fn with_options(
        proxy_a: ShapeProxy,
        proxy_b: ShapeProxy,
        transform_b_to_a: Transform,
        translation_b: impl Into<Vec3>,
        max_fraction: f32,
        can_encroach: bool,
    ) -> Result<Self> {
        let translation_b = translation_b.into();
        if translation_b.is_valid() && max_fraction.is_finite() && max_fraction >= 0.0 {
            transform_b_to_a.validate()?;
            Ok(Self {
                proxy_a,
                proxy_b,
                transform_b_to_a,
                translation_b,
                max_fraction,
                can_encroach,
            })
        } else {
            Err(Error::InvalidArgument)
        }
    }

    #[inline]
    fn raw(&self) -> ffi::b3ShapeCastPairInput {
        ffi::b3ShapeCastPairInput {
            proxyA: self.proxy_a.raw(),
            proxyB: self.proxy_b.raw(),
            transform: self.transform_b_to_a.into_raw(),
            translationB: self.translation_b.into_raw(),
            maxFraction: self.max_fraction,
            canEncroach: self.can_encroach,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
/// Result returned by ray-cast and shape-cast helpers.
pub struct CastOutput {
    /// Hit normal, if any.
    pub normal: Vec3,
    /// Hit point, if any.
    pub point: Vec3,
    /// Fraction along the cast where the hit occurred.
    pub fraction: f32,
    /// Iteration count used by the cast algorithm.
    pub iterations: i32,
    /// Triangle index hit by mesh or height-field casts.
    pub triangle_index: i32,
    /// Child index hit by compound casts.
    pub child_index: i32,
    /// Material index hit by mesh, height-field, or compound casts.
    pub material_index: i32,
    /// Whether the cast hit.
    pub hit: bool,
}

impl CastOutput {
    #[inline]
    /// Converts a raw Box3D cast output into the safe value type.
    pub fn from_raw(raw: ffi::b3CastOutput) -> Self {
        Self {
            normal: Vec3::from_raw(raw.normal),
            point: Vec3::from_raw(raw.point),
            fraction: raw.fraction,
            iterations: raw.iterations,
            triangle_index: raw.triangleIndex,
            child_index: raw.childIndex,
            material_index: raw.materialIndex,
            hit: raw.hit,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
/// Result returned by the proxy distance helper.
pub struct DistanceOutput {
    /// Closest point on proxy A.
    pub point_a: Vec3,
    /// Closest point on proxy B.
    pub point_b: Vec3,
    /// Normal pointing from A toward B.
    pub normal: Vec3,
    /// Distance between the proxies.
    pub distance: f32,
    /// Iteration count used by the distance algorithm.
    pub iterations: i32,
    /// Number of simplex points in the final cache.
    pub simplex_count: i32,
}

impl DistanceOutput {
    #[inline]
    fn from_raw(raw: ffi::b3DistanceOutput) -> Self {
        Self {
            point_a: Vec3::from_raw(raw.pointA),
            point_b: Vec3::from_raw(raw.pointB),
            normal: Vec3::from_raw(raw.normal),
            distance: raw.distance,
            iterations: raw.iterations,
            simplex_count: raw.simplexCount,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// Swept transform endpoints used by time-of-impact queries.
pub struct Sweep {
    /// Local center of mass.
    pub local_center: Vec3,
    /// Center position at the start of the sweep.
    pub c1: Vec3,
    /// Center position at the end of the sweep.
    pub c2: Vec3,
    /// Rotation at the start of the sweep.
    pub q1: Quat,
    /// Rotation at the end of the sweep.
    pub q2: Quat,
}

impl Sweep {
    /// Creates a validated sweep.
    pub fn new(local_center: Vec3, c1: Vec3, c2: Vec3, q1: Quat, q2: Quat) -> Result<Self> {
        let sweep = Self {
            local_center,
            c1,
            c2,
            q1,
            q2,
        };
        sweep.validate()
    }

    #[inline]
    fn validate(self) -> Result<Self> {
        if self.local_center.is_valid()
            && self.c1.is_valid()
            && self.c2.is_valid()
            && self.q1.is_valid()
            && self.q2.is_valid()
        {
            Ok(self)
        } else {
            Err(Error::InvalidArgument)
        }
    }

    #[inline]
    fn raw(self) -> ffi::b3Sweep {
        ffi::b3Sweep {
            localCenter: self.local_center.into_raw(),
            c1: self.c1.into_raw(),
            c2: self.c2.into_raw(),
            q1: self.q1.into_raw(),
            q2: self.q2.into_raw(),
        }
    }
}

impl Default for Sweep {
    fn default() -> Self {
        Self {
            local_center: Vec3::ZERO,
            c1: Vec3::ZERO,
            c2: Vec3::ZERO,
            q1: Quat::IDENTITY,
            q2: Quat::IDENTITY,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Input for continuous collision detection between two swept proxies.
pub struct TimeOfImpactInput {
    /// First proxy.
    pub proxy_a: ShapeProxy,
    /// Second proxy.
    pub proxy_b: ShapeProxy,
    /// Sweep for proxy A.
    pub sweep_a: Sweep,
    /// Sweep for proxy B.
    pub sweep_b: Sweep,
    /// Maximum sweep fraction to consider.
    pub max_fraction: f32,
}

impl TimeOfImpactInput {
    /// Creates time-of-impact input with `max_fraction` set to `1.0`.
    pub fn new(
        proxy_a: ShapeProxy,
        proxy_b: ShapeProxy,
        sweep_a: Sweep,
        sweep_b: Sweep,
    ) -> Result<Self> {
        Self::with_max_fraction(proxy_a, proxy_b, sweep_a, sweep_b, 1.0)
    }

    /// Creates time-of-impact input with an explicit maximum fraction.
    pub fn with_max_fraction(
        proxy_a: ShapeProxy,
        proxy_b: ShapeProxy,
        sweep_a: Sweep,
        sweep_b: Sweep,
        max_fraction: f32,
    ) -> Result<Self> {
        sweep_a.validate()?;
        sweep_b.validate()?;
        if max_fraction.is_finite() && max_fraction >= 0.0 {
            Ok(Self {
                proxy_a,
                proxy_b,
                sweep_a,
                sweep_b,
                max_fraction,
            })
        } else {
            Err(Error::InvalidArgument)
        }
    }

    #[inline]
    fn raw(&self) -> ffi::b3TOIInput {
        ffi::b3TOIInput {
            proxyA: self.proxy_a.raw(),
            proxyB: self.proxy_b.raw(),
            sweepA: self.sweep_a.raw(),
            sweepB: self.sweep_b.raw(),
            maxFraction: self.max_fraction,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// State reported by a time-of-impact query.
pub enum TimeOfImpactState {
    /// The raw state was not recognized.
    Unknown,
    /// The solver failed to produce a reliable result.
    Failed,
    /// The sweeps start overlapped.
    Overlapped,
    /// The sweeps touch within the requested fraction.
    Hit,
    /// The sweeps remain separated within the requested fraction.
    Separated,
}

impl TimeOfImpactState {
    #[inline]
    fn from_raw(raw: ffi::b3TOIState) -> Self {
        match raw {
            ffi::b3TOIState_b3_toiStateFailed => Self::Failed,
            ffi::b3TOIState_b3_toiStateOverlapped => Self::Overlapped,
            ffi::b3TOIState_b3_toiStateHit => Self::Hit,
            ffi::b3TOIState_b3_toiStateSeparated => Self::Separated,
            _ => Self::Unknown,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// Output from a time-of-impact query.
pub struct TimeOfImpactOutput {
    /// Solver state.
    pub state: TimeOfImpactState,
    /// Contact point at the time of impact.
    pub point: Vec3,
    /// Contact normal at the time of impact.
    pub normal: Vec3,
    /// Fraction at which the reported state occurred.
    pub fraction: f32,
    /// Distance at the reported state.
    pub distance: f32,
    /// Distance solver iteration count.
    pub distance_iterations: i32,
    /// Push-back solver iteration count.
    pub push_back_iterations: i32,
    /// Root solver iteration count.
    pub root_iterations: i32,
    /// Whether Box3D used a fallback path.
    pub used_fallback: bool,
}

impl TimeOfImpactOutput {
    #[inline]
    fn from_raw(raw: ffi::b3TOIOutput) -> Self {
        Self {
            state: TimeOfImpactState::from_raw(raw.state),
            point: Vec3::from_raw(raw.point),
            normal: Vec3::from_raw(raw.normal),
            fraction: raw.fraction,
            distance: raw.distance,
            distance_iterations: raw.distanceIterations,
            push_back_iterations: raw.pushBackIterations,
            root_iterations: raw.rootIterations,
            used_fallback: raw.usedFallback,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// Plane constraint used by the collision plane solver.
pub struct CollisionPlane {
    /// Plane equation.
    pub plane: Plane,
    /// Maximum push distance allowed for this plane.
    pub push_limit: f32,
    /// Push distance accumulated by the solver.
    pub push: f32,
    /// Whether velocity should be clipped by this plane.
    pub clip_velocity: bool,
}

impl CollisionPlane {
    /// Creates a collision plane with zero initial push.
    pub fn new(plane: Plane, push_limit: f32, clip_velocity: bool) -> Result<Self> {
        Self::with_options(plane, push_limit, 0.0, clip_velocity)
    }

    /// Creates a collision plane with explicit solver state.
    pub fn with_options(
        plane: Plane,
        push_limit: f32,
        push: f32,
        clip_velocity: bool,
    ) -> Result<Self> {
        let input = Self {
            plane,
            push_limit,
            push,
            clip_velocity,
        };
        input.validate()?;
        Ok(input)
    }

    fn validate(self) -> Result<()> {
        if self.plane.is_valid()
            && self.push_limit.is_finite()
            && self.push_limit >= 0.0
            && self.push.is_finite()
        {
            Ok(())
        } else {
            Err(Error::InvalidArgument)
        }
    }

    #[inline]
    fn raw(self) -> ffi::b3CollisionPlane {
        ffi::b3CollisionPlane {
            plane: self.plane.into_raw(),
            pushLimit: self.push_limit,
            push: self.push,
            clipVelocity: self.clip_velocity,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
/// Result returned by `solve_planes`.
pub struct PlaneSolverResult {
    /// Solver displacement.
    pub delta: Vec3,
    /// Number of solver iterations used.
    pub iteration_count: i32,
}

impl PlaneSolverResult {
    #[inline]
    fn from_raw(raw: ffi::b3PlaneSolverResult) -> Self {
        Self {
            delta: Vec3::from_raw(raw.delta),
            iteration_count: raw.iterationCount,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
/// Contact point returned by local manifold helpers.
pub struct LocalManifoldPoint {
    /// Contact point in local space.
    pub point: Vec3,
    /// Contact separation.
    pub separation: f32,
    /// Triangle index associated with the point, when applicable.
    pub triangle_index: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Local-space manifold returned by narrow-phase collision helpers.
pub struct LocalManifold {
    /// Manifold normal.
    pub normal: Vec3,
    /// Triangle normal, when the manifold comes from triangle collision.
    pub triangle_normal: Vec3,
    /// Contact points in the manifold.
    pub points: Vec<LocalManifoldPoint>,
}

impl LocalManifold {
    unsafe fn from_raw(raw: ffi::b3LocalManifold, points: &[ffi::b3LocalManifoldPoint]) -> Self {
        Self {
            normal: Vec3::from_raw(raw.normal),
            triangle_normal: Vec3::from_raw(raw.triangleNormal),
            points: points
                .iter()
                .map(|point| LocalManifoldPoint {
                    point: Vec3::from_raw(point.point),
                    separation: point.separation,
                    triangle_index: point.triangleIndex,
                })
                .collect(),
        }
    }
}

/// Computes mass properties for a sphere at the given density.
pub fn compute_sphere_mass(sphere: &Sphere, density: f32) -> Result<MassData> {
    sphere.validate()?;
    validate_density(density)?;
    let _guard = box3d_lock::lock();
    Ok(MassData::from_raw(unsafe {
        ffi::b3ComputeSphereMass(sphere.raw(), density)
    }))
}

/// Computes mass properties for a capsule at the given density.
pub fn compute_capsule_mass(capsule: &Capsule, density: f32) -> Result<MassData> {
    capsule.validate()?;
    validate_density(density)?;
    let _guard = box3d_lock::lock();
    Ok(MassData::from_raw(unsafe {
        ffi::b3ComputeCapsuleMass(capsule.raw(), density)
    }))
}

/// Computes mass properties for a convex hull at the given density.
pub fn compute_hull_mass(hull: &Hull, density: f32) -> Result<MassData> {
    validate_density(density)?;
    let _guard = box3d_lock::lock();
    Ok(MassData::from_raw(unsafe {
        ffi::b3ComputeHullMass(hull.as_ptr(), density)
    }))
}

/// Computes the world-space AABB for a transformed sphere.
pub fn compute_sphere_aabb(sphere: &Sphere, transform: Transform) -> Result<Aabb> {
    sphere.validate()?;
    transform.validate()?;
    let _guard = box3d_lock::lock();
    Ok(Aabb::from_raw(unsafe {
        ffi::b3ComputeSphereAABB(sphere.raw(), transform.into_raw())
    }))
}

/// Computes the world-space AABB for a transformed capsule.
pub fn compute_capsule_aabb(capsule: &Capsule, transform: Transform) -> Result<Aabb> {
    capsule.validate()?;
    transform.validate()?;
    let _guard = box3d_lock::lock();
    Ok(Aabb::from_raw(unsafe {
        ffi::b3ComputeCapsuleAABB(capsule.raw(), transform.into_raw())
    }))
}

/// Computes the world-space AABB for a transformed hull.
pub fn compute_hull_aabb(hull: &Hull, transform: Transform) -> Result<Aabb> {
    transform.validate()?;
    let _guard = box3d_lock::lock();
    Ok(Aabb::from_raw(unsafe {
        ffi::b3ComputeHullAABB(hull.as_ptr(), transform.into_raw())
    }))
}

/// Computes the world-space AABB for a transformed and scaled mesh.
pub fn compute_mesh_aabb(
    mesh: &MeshData,
    transform: Transform,
    scale: impl Into<Vec3>,
) -> Result<Aabb> {
    transform.validate()?;
    let scale = validate_mesh_scale(scale.into())?;
    let _guard = box3d_lock::lock();
    Ok(Aabb::from_raw(unsafe {
        ffi::b3ComputeMeshAABB(mesh.as_ptr(), transform.into_raw(), scale.into_raw())
    }))
}

/// Computes the world-space AABB for a transformed height field.
pub fn compute_height_field_aabb(height_field: &HeightField, transform: Transform) -> Result<Aabb> {
    transform.validate()?;
    let _guard = box3d_lock::lock();
    Ok(Aabb::from_raw(unsafe {
        ffi::b3ComputeHeightFieldAABB(height_field.as_ptr(), transform.into_raw())
    }))
}

/// Computes the world-space AABB for a transformed compound.
pub fn compute_compound_aabb(compound: &Compound, transform: Transform) -> Result<Aabb> {
    transform.validate()?;
    let _guard = box3d_lock::lock();
    Ok(Aabb::from_raw(unsafe {
        ffi::b3ComputeCompoundAABB(compound.as_ptr(), transform.into_raw())
    }))
}

/// Computes closest points and distance between two shape proxies.
pub fn shape_distance(input: DistanceInput) -> Result<DistanceOutput> {
    input.transform_b_to_a.validate()?;
    let raw = input.raw();
    let mut cache: ffi::b3SimplexCache = unsafe { std::mem::zeroed() };
    let _guard = box3d_lock::lock();
    Ok(DistanceOutput::from_raw(unsafe {
        ffi::b3ShapeDistance(&raw, &mut cache, std::ptr::null_mut(), 0)
    }))
}

/// Sweeps one shape proxy against another.
pub fn shape_cast_pair(input: ShapeCastPairInput) -> Result<CastOutput> {
    input.transform_b_to_a.validate()?;
    if !input.translation_b.is_valid()
        || !input.max_fraction.is_finite()
        || input.max_fraction < 0.0
    {
        return Err(Error::InvalidArgument);
    }
    let raw = input.raw();
    let _guard = box3d_lock::lock();
    Ok(CastOutput::from_raw(unsafe { ffi::b3ShapeCast(&raw) }))
}

/// Interpolates a sweep at a time fraction in `[0, 1]`.
pub fn sweep_transform(sweep: Sweep, time: f32) -> Result<Transform> {
    let sweep = sweep.validate()?;
    if !time.is_finite() || !(0.0..=1.0).contains(&time) {
        return Err(Error::InvalidArgument);
    }
    let raw = sweep.raw();
    let _guard = box3d_lock::lock();
    Ok(Transform::from_raw(unsafe {
        ffi::b3GetSweepTransform(&raw, time)
    }))
}

/// Interpolates a borrowed sweep at a time fraction in `[0, 1]`.
pub fn get_sweep_transform(sweep: &Sweep, time: f32) -> Result<Transform> {
    sweep_transform(*sweep, time)
}

/// Computes the time of impact between two swept proxies.
pub fn time_of_impact(input: TimeOfImpactInput) -> Result<TimeOfImpactOutput> {
    input.sweep_a.validate()?;
    input.sweep_b.validate()?;
    if !input.max_fraction.is_finite() || input.max_fraction < 0.0 {
        return Err(Error::InvalidArgument);
    }
    let raw = input.raw();
    let _guard = box3d_lock::lock();
    Ok(TimeOfImpactOutput::from_raw(unsafe {
        ffi::b3TimeOfImpact(&raw)
    }))
}

/// Solves displacement against a mutable set of collision planes.
pub fn solve_planes(
    target_delta: impl Into<Vec3>,
    planes: &mut [CollisionPlane],
) -> Result<PlaneSolverResult> {
    let target_delta = target_delta.into().validate()?;
    if planes.is_empty() || planes.len() > i32::MAX as usize {
        return Err(Error::InvalidArgument);
    }
    let mut raw_planes = planes
        .iter()
        .copied()
        .map(|plane| {
            plane.validate()?;
            Ok(plane.raw())
        })
        .collect::<Result<Vec<_>>>()?;
    let _guard = box3d_lock::lock();
    let raw = unsafe {
        ffi::b3SolvePlanes(
            target_delta.into_raw(),
            raw_planes.as_mut_ptr(),
            raw_planes.len() as i32,
        )
    };
    for (plane, raw) in planes.iter_mut().zip(raw_planes.iter()) {
        plane.push = raw.push;
    }
    Ok(PlaneSolverResult::from_raw(raw))
}

/// Clips a vector against collision planes.
pub fn clip_vector(vector: impl Into<Vec3>, planes: &[CollisionPlane]) -> Result<Vec3> {
    let vector = vector.into().validate()?;
    if planes.is_empty() || planes.len() > i32::MAX as usize {
        return Err(Error::InvalidArgument);
    }
    let raw_planes = planes
        .iter()
        .copied()
        .map(|plane| {
            plane.validate()?;
            Ok(plane.raw())
        })
        .collect::<Result<Vec<_>>>()?;
    let _guard = box3d_lock::lock();
    Ok(Vec3::from_raw(unsafe {
        ffi::b3ClipVector(
            vector.into_raw(),
            raw_planes.as_ptr(),
            raw_planes.len() as i32,
        )
    }))
}

/// Tests whether a transformed sphere overlaps a shape proxy.
pub fn overlap_sphere(sphere: &Sphere, transform: Transform, proxy: &ShapeProxy) -> Result<bool> {
    sphere.validate()?;
    overlap(proxy, transform, |proxy, transform| unsafe {
        ffi::b3OverlapSphere(sphere.raw(), transform, proxy)
    })
}

/// Tests whether a transformed capsule overlaps a shape proxy.
pub fn overlap_capsule(
    capsule: &Capsule,
    transform: Transform,
    proxy: &ShapeProxy,
) -> Result<bool> {
    capsule.validate()?;
    overlap(proxy, transform, |proxy, transform| unsafe {
        ffi::b3OverlapCapsule(capsule.raw(), transform, proxy)
    })
}

/// Tests whether a transformed hull overlaps a shape proxy.
pub fn overlap_hull(hull: &Hull, transform: Transform, proxy: &ShapeProxy) -> Result<bool> {
    overlap(proxy, transform, |proxy, transform| unsafe {
        ffi::b3OverlapHull(hull.as_ptr(), transform, proxy)
    })
}

/// Tests whether a transformed and scaled mesh overlaps a shape proxy.
pub fn overlap_mesh(
    mesh: &MeshData,
    scale: impl Into<Vec3>,
    transform: Transform,
    proxy: &ShapeProxy,
) -> Result<bool> {
    let raw_mesh = ffi::b3Mesh {
        data: mesh.as_ptr(),
        scale: validate_mesh_scale(scale.into())?.into_raw(),
    };
    overlap(proxy, transform, |proxy, transform| unsafe {
        ffi::b3OverlapMesh(&raw_mesh, transform, proxy)
    })
}

/// Tests whether a transformed height field overlaps a shape proxy.
pub fn overlap_height_field(
    height_field: &HeightField,
    transform: Transform,
    proxy: &ShapeProxy,
) -> Result<bool> {
    overlap(proxy, transform, |proxy, transform| unsafe {
        ffi::b3OverlapHeightField(height_field.as_ptr(), transform, proxy)
    })
}

/// Tests whether a transformed compound overlaps a shape proxy.
pub fn overlap_compound(
    compound: &Compound,
    transform: Transform,
    proxy: &ShapeProxy,
) -> Result<bool> {
    overlap(proxy, transform, |proxy, transform| unsafe {
        ffi::b3OverlapCompound(compound.as_ptr(), transform, proxy)
    })
}

/// Casts a ray against a sphere.
pub fn ray_cast_sphere(sphere: &Sphere, input: RayCastInput) -> Result<CastOutput> {
    sphere.validate()?;
    ray_cast(input, |input| unsafe {
        ffi::b3RayCastSphere(sphere.raw(), input)
    })
}

/// Casts a ray against a hollow sphere shell.
pub fn ray_cast_hollow_sphere(sphere: &Sphere, input: RayCastInput) -> Result<CastOutput> {
    sphere.validate()?;
    ray_cast(input, |input| unsafe {
        ffi::b3RayCastHollowSphere(sphere.raw(), input)
    })
}

/// Casts a ray against a capsule.
pub fn ray_cast_capsule(capsule: &Capsule, input: RayCastInput) -> Result<CastOutput> {
    capsule.validate()?;
    ray_cast(input, |input| unsafe {
        ffi::b3RayCastCapsule(capsule.raw(), input)
    })
}

/// Casts a ray against a convex hull.
pub fn ray_cast_hull(hull: &Hull, input: RayCastInput) -> Result<CastOutput> {
    ray_cast(input, |input| unsafe {
        ffi::b3RayCastHull(hull.as_ptr(), input)
    })
}

/// Casts a ray against a scaled mesh.
pub fn ray_cast_mesh(
    mesh: &MeshData,
    scale: impl Into<Vec3>,
    input: RayCastInput,
) -> Result<CastOutput> {
    let raw_mesh = ffi::b3Mesh {
        data: mesh.as_ptr(),
        scale: validate_mesh_scale(scale.into())?.into_raw(),
    };
    ray_cast(input, |input| unsafe {
        ffi::b3RayCastMesh(&raw_mesh, input)
    })
}

/// Casts a ray against a height field.
pub fn ray_cast_height_field(
    height_field: &HeightField,
    input: RayCastInput,
) -> Result<CastOutput> {
    ray_cast(input, |input| unsafe {
        ffi::b3RayCastHeightField(height_field.as_ptr(), input)
    })
}

/// Casts a ray against a compound.
pub fn ray_cast_compound(compound: &Compound, input: RayCastInput) -> Result<CastOutput> {
    ray_cast(input, |input| unsafe {
        ffi::b3RayCastCompound(compound.as_ptr(), input)
    })
}

/// Sweeps a proxy against a sphere.
pub fn shape_cast_sphere(sphere: &Sphere, input: ShapeCastInput) -> Result<CastOutput> {
    sphere.validate()?;
    shape_cast(input, |input| unsafe {
        ffi::b3ShapeCastSphere(sphere.raw(), input)
    })
}

/// Sweeps a proxy against a capsule.
pub fn shape_cast_capsule(capsule: &Capsule, input: ShapeCastInput) -> Result<CastOutput> {
    capsule.validate()?;
    shape_cast(input, |input| unsafe {
        ffi::b3ShapeCastCapsule(capsule.raw(), input)
    })
}

/// Sweeps a proxy against a convex hull.
pub fn shape_cast_hull(hull: &Hull, input: ShapeCastInput) -> Result<CastOutput> {
    shape_cast(input, |input| unsafe {
        ffi::b3ShapeCastHull(hull.as_ptr(), input)
    })
}

/// Sweeps a proxy against a scaled mesh.
pub fn shape_cast_mesh(
    mesh: &MeshData,
    scale: impl Into<Vec3>,
    input: ShapeCastInput,
) -> Result<CastOutput> {
    let raw_mesh = ffi::b3Mesh {
        data: mesh.as_ptr(),
        scale: validate_mesh_scale(scale.into())?.into_raw(),
    };
    shape_cast(input, |input| unsafe {
        ffi::b3ShapeCastMesh(&raw_mesh, input)
    })
}

/// Sweeps a proxy against a height field.
pub fn shape_cast_height_field(
    height_field: &HeightField,
    input: ShapeCastInput,
) -> Result<CastOutput> {
    shape_cast(input, |input| unsafe {
        ffi::b3ShapeCastHeightField(height_field.as_ptr(), input)
    })
}

/// Sweeps a proxy against a compound.
pub fn shape_cast_compound(compound: &Compound, input: ShapeCastInput) -> Result<CastOutput> {
    shape_cast(input, |input| unsafe {
        ffi::b3ShapeCastCompound(compound.as_ptr(), input)
    })
}

/// Computes a local manifold for two spheres.
pub fn collide_spheres(
    a: &Sphere,
    b: &Sphere,
    transform_b_to_a: Transform,
) -> Result<LocalManifold> {
    a.validate()?;
    b.validate()?;
    collide(transform_b_to_a, |manifold, capacity| unsafe {
        ffi::b3CollideSpheres(
            manifold,
            capacity,
            a.raw(),
            b.raw(),
            transform_b_to_a.into_raw(),
        )
    })
}

/// Computes a local manifold for a capsule and a sphere.
pub fn collide_capsule_and_sphere(
    capsule_a: &Capsule,
    sphere_b: &Sphere,
    transform_b_to_a: Transform,
) -> Result<LocalManifold> {
    capsule_a.validate()?;
    sphere_b.validate()?;
    collide(transform_b_to_a, |manifold, capacity| unsafe {
        ffi::b3CollideCapsuleAndSphere(
            manifold,
            capacity,
            capsule_a.raw(),
            sphere_b.raw(),
            transform_b_to_a.into_raw(),
        )
    })
}

/// Computes a local manifold for a hull and a sphere.
pub fn collide_hull_and_sphere(
    hull_a: &Hull,
    sphere_b: &Sphere,
    transform_b_to_a: Transform,
) -> Result<LocalManifold> {
    sphere_b.validate()?;
    let mut cache: ffi::b3SimplexCache = unsafe { std::mem::zeroed() };
    collide(transform_b_to_a, |manifold, capacity| unsafe {
        ffi::b3CollideHullAndSphere(
            manifold,
            capacity,
            hull_a.as_ptr(),
            sphere_b.raw(),
            transform_b_to_a.into_raw(),
            &mut cache,
        )
    })
}

/// Computes a local manifold for two capsules.
pub fn collide_capsules(
    a: &Capsule,
    b: &Capsule,
    transform_b_to_a: Transform,
) -> Result<LocalManifold> {
    a.validate()?;
    b.validate()?;
    collide(transform_b_to_a, |manifold, capacity| unsafe {
        ffi::b3CollideCapsules(
            manifold,
            capacity,
            a.raw(),
            b.raw(),
            transform_b_to_a.into_raw(),
        )
    })
}

/// Computes a local manifold for a hull and a capsule.
pub fn collide_hull_and_capsule(
    hull_a: &Hull,
    capsule_b: &Capsule,
    transform_b_to_a: Transform,
) -> Result<LocalManifold> {
    capsule_b.validate()?;
    let mut cache: ffi::b3SimplexCache = unsafe { std::mem::zeroed() };
    collide(transform_b_to_a, |manifold, capacity| unsafe {
        ffi::b3CollideHullAndCapsule(
            manifold,
            capacity,
            hull_a.as_ptr(),
            capsule_b.raw(),
            transform_b_to_a.into_raw(),
            &mut cache,
        )
    })
}

/// Computes a local manifold for two hulls.
pub fn collide_hulls(
    hull_a: &Hull,
    hull_b: &Hull,
    transform_b_to_a: Transform,
) -> Result<LocalManifold> {
    let mut cache: ffi::b3SATCache = unsafe { std::mem::zeroed() };
    collide(transform_b_to_a, |manifold, capacity| unsafe {
        ffi::b3CollideHulls(
            manifold,
            capacity,
            hull_a.as_ptr(),
            hull_b.as_ptr(),
            transform_b_to_a.into_raw(),
            &mut cache,
        )
    })
}

/// Computes a local manifold for a capsule and a triangle.
pub fn collide_capsule_and_triangle(
    capsule_a: &Capsule,
    triangle_b: [Vec3; 3],
) -> Result<LocalManifold> {
    capsule_a.validate()?;
    validate_triangle(triangle_b)?;
    let raw_triangle = triangle_b.map(Vec3::into_raw);
    let mut cache: ffi::b3SimplexCache = unsafe { std::mem::zeroed() };
    collide(Transform::IDENTITY, |manifold, capacity| unsafe {
        ffi::b3CollideCapsuleAndTriangle(
            manifold,
            capacity,
            capsule_a.raw(),
            raw_triangle.as_ptr(),
            &mut cache,
        )
    })
}

/// Computes a local manifold for a hull and a triangle.
pub fn collide_hull_and_triangle(
    hull_a: &Hull,
    triangle_b: [Vec3; 3],
    triangle_flags: i32,
) -> Result<LocalManifold> {
    validate_triangle(triangle_b)?;
    if triangle_flags < 0 {
        return Err(Error::InvalidArgument);
    }
    let mut cache: ffi::b3SATCache = unsafe { std::mem::zeroed() };
    collide(Transform::IDENTITY, |manifold, capacity| unsafe {
        ffi::b3CollideHullAndTriangle(
            manifold,
            capacity,
            hull_a.as_ptr(),
            triangle_b[0].into_raw(),
            triangle_b[1].into_raw(),
            triangle_b[2].into_raw(),
            triangle_flags,
            &mut cache,
        )
    })
}

/// Computes a local manifold for a sphere and a triangle.
pub fn collide_sphere_and_triangle(
    sphere_a: &Sphere,
    triangle_b: [Vec3; 3],
) -> Result<LocalManifold> {
    sphere_a.validate()?;
    validate_triangle(triangle_b)?;
    let raw_triangle = triangle_b.map(Vec3::into_raw);
    collide(Transform::IDENTITY, |manifold, capacity| unsafe {
        ffi::b3CollideSphereAndTriangle(manifold, capacity, sphere_a.raw(), raw_triangle.as_ptr())
    })
}

fn overlap(
    proxy: &ShapeProxy,
    transform: Transform,
    f: impl FnOnce(*const ffi::b3ShapeProxy, ffi::b3Transform) -> bool,
) -> Result<bool> {
    transform.validate()?;
    let raw_proxy = proxy.raw();
    let _guard = box3d_lock::lock();
    Ok(f(&raw_proxy, transform.into_raw()))
}

fn ray_cast(
    input: RayCastInput,
    f: impl FnOnce(*const ffi::b3RayCastInput) -> ffi::b3CastOutput,
) -> Result<CastOutput> {
    let raw = input.raw();
    let _guard = box3d_lock::lock();
    if !unsafe { ffi::b3IsValidRay(&raw) } {
        return Err(Error::InvalidArgument);
    }
    Ok(CastOutput::from_raw(f(&raw)))
}

fn shape_cast(
    input: ShapeCastInput,
    f: impl FnOnce(*const ffi::b3ShapeCastInput) -> ffi::b3CastOutput,
) -> Result<CastOutput> {
    let input = input.validate()?;
    let raw = input.raw();
    let _guard = box3d_lock::lock();
    Ok(CastOutput::from_raw(f(&raw)))
}

fn collide(
    transform_b_to_a: Transform,
    f: impl FnOnce(*mut ffi::b3LocalManifold, i32),
) -> Result<LocalManifold> {
    transform_b_to_a.validate()?;
    let _guard = box3d_lock::lock();
    let mut points: [MaybeUninit<ffi::b3LocalManifoldPoint>; MAX_LOCAL_MANIFOLD_POINTS] =
        [MaybeUninit::uninit(); MAX_LOCAL_MANIFOLD_POINTS];
    let mut raw: ffi::b3LocalManifold = unsafe { std::mem::zeroed() };
    raw.points = points.as_mut_ptr().cast();
    f(&mut raw, ffi::B3_MAX_MANIFOLD_POINTS as i32);
    let count = raw.pointCount.clamp(0, ffi::B3_MAX_MANIFOLD_POINTS as i32) as usize;
    let initialized = unsafe { std::slice::from_raw_parts(points.as_ptr().cast(), count) };
    Ok(unsafe { LocalManifold::from_raw(raw, &initialized) })
}

fn validate_density(density: f32) -> Result<()> {
    if density.is_finite() && density >= 0.0 {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}

fn validate_triangle(triangle: [Vec3; 3]) -> Result<()> {
    if triangle.iter().all(|point| point.is_valid())
        && triangle_area_squared(triangle[0], triangle[1], triangle[2]) > f32::EPSILON
    {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
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
