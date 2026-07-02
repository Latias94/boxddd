use crate::core::box3d_lock;
use crate::error::{Error, Result};
use crate::shapes::{Capsule, Compound, HeightField, Hull, MeshData, Sphere};
use crate::types::{Aabb, MassData, Transform, Vec3};
use boxddd_sys::ffi;
use std::mem::MaybeUninit;

pub const MAX_SHAPE_PROXY_POINTS: usize = ffi::B3_MAX_SHAPE_CAST_POINTS as usize;
pub const MAX_LOCAL_MANIFOLD_POINTS: usize = ffi::B3_MAX_MANIFOLD_POINTS as usize;

#[derive(Clone, Debug, PartialEq)]
pub struct ShapeProxy {
    points: Vec<Vec3>,
    radius: f32,
}

impl ShapeProxy {
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

    pub fn sphere(radius: f32) -> Result<Self> {
        Self::new(vec![Vec3::ZERO], radius)
    }

    pub fn capsule(
        center1: impl Into<Vec3>,
        center2: impl Into<Vec3>,
        radius: f32,
    ) -> Result<Self> {
        Self::new(vec![center1.into(), center2.into()], radius)
    }

    #[inline]
    pub fn points(&self) -> &[Vec3] {
        &self.points
    }

    #[inline]
    pub fn radius(&self) -> f32 {
        self.radius
    }

    #[inline]
    fn raw(&self) -> ffi::b3ShapeProxy {
        ffi::b3ShapeProxy {
            points: self.points.as_ptr().cast(),
            count: self.points.len() as i32,
            radius: self.radius,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RayCastInput {
    pub origin: Vec3,
    pub translation: Vec3,
    pub max_fraction: f32,
}

impl RayCastInput {
    pub fn new(origin: impl Into<Vec3>, translation: impl Into<Vec3>) -> Result<Self> {
        Self::with_max_fraction(origin, translation, 1.0)
    }

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
    fn raw(&self) -> ffi::b3RayCastInput {
        ffi::b3RayCastInput {
            origin: self.origin.into_raw(),
            translation: self.translation.into_raw(),
            maxFraction: self.max_fraction,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShapeCastInput {
    pub proxy: ShapeProxy,
    pub translation: Vec3,
    pub max_fraction: f32,
    pub can_encroach: bool,
}

impl ShapeCastInput {
    pub fn new(proxy: ShapeProxy, translation: impl Into<Vec3>) -> Result<Self> {
        Self::with_options(proxy, translation, 1.0, false)
    }

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
    fn raw(&self) -> ffi::b3ShapeCastInput {
        ffi::b3ShapeCastInput {
            proxy: self.proxy.raw(),
            translation: self.translation.into_raw(),
            maxFraction: self.max_fraction,
            canEncroach: self.can_encroach,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct CastOutput {
    pub normal: Vec3,
    pub point: Vec3,
    pub fraction: f32,
    pub iterations: i32,
    pub triangle_index: i32,
    pub child_index: i32,
    pub material_index: i32,
    pub hit: bool,
}

impl CastOutput {
    #[inline]
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
pub struct LocalManifoldPoint {
    pub point: Vec3,
    pub separation: f32,
    pub triangle_index: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LocalManifold {
    pub normal: Vec3,
    pub triangle_normal: Vec3,
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

pub fn compute_sphere_mass(sphere: &Sphere, density: f32) -> Result<MassData> {
    sphere.validate()?;
    validate_density(density)?;
    let _guard = box3d_lock::lock();
    Ok(MassData::from_raw(unsafe {
        ffi::b3ComputeSphereMass(sphere.raw(), density)
    }))
}

pub fn compute_capsule_mass(capsule: &Capsule, density: f32) -> Result<MassData> {
    capsule.validate()?;
    validate_density(density)?;
    let _guard = box3d_lock::lock();
    Ok(MassData::from_raw(unsafe {
        ffi::b3ComputeCapsuleMass(capsule.raw(), density)
    }))
}

pub fn compute_hull_mass(hull: &Hull, density: f32) -> Result<MassData> {
    validate_density(density)?;
    let _guard = box3d_lock::lock();
    Ok(MassData::from_raw(unsafe {
        ffi::b3ComputeHullMass(hull.as_ptr(), density)
    }))
}

pub fn compute_sphere_aabb(sphere: &Sphere, transform: Transform) -> Result<Aabb> {
    sphere.validate()?;
    transform.validate()?;
    let _guard = box3d_lock::lock();
    Ok(Aabb::from_raw(unsafe {
        ffi::b3ComputeSphereAABB(sphere.raw(), transform.into_raw())
    }))
}

pub fn compute_capsule_aabb(capsule: &Capsule, transform: Transform) -> Result<Aabb> {
    capsule.validate()?;
    transform.validate()?;
    let _guard = box3d_lock::lock();
    Ok(Aabb::from_raw(unsafe {
        ffi::b3ComputeCapsuleAABB(capsule.raw(), transform.into_raw())
    }))
}

pub fn compute_hull_aabb(hull: &Hull, transform: Transform) -> Result<Aabb> {
    transform.validate()?;
    let _guard = box3d_lock::lock();
    Ok(Aabb::from_raw(unsafe {
        ffi::b3ComputeHullAABB(hull.as_ptr(), transform.into_raw())
    }))
}

pub fn compute_mesh_aabb(
    mesh: &MeshData,
    transform: Transform,
    scale: impl Into<Vec3>,
) -> Result<Aabb> {
    transform.validate()?;
    let scale = scale.into().validate()?;
    let _guard = box3d_lock::lock();
    Ok(Aabb::from_raw(unsafe {
        ffi::b3ComputeMeshAABB(mesh.as_ptr(), transform.into_raw(), scale.into_raw())
    }))
}

pub fn compute_height_field_aabb(height_field: &HeightField, transform: Transform) -> Result<Aabb> {
    transform.validate()?;
    let _guard = box3d_lock::lock();
    Ok(Aabb::from_raw(unsafe {
        ffi::b3ComputeHeightFieldAABB(height_field.as_ptr(), transform.into_raw())
    }))
}

pub fn compute_compound_aabb(compound: &Compound, transform: Transform) -> Result<Aabb> {
    transform.validate()?;
    let _guard = box3d_lock::lock();
    Ok(Aabb::from_raw(unsafe {
        ffi::b3ComputeCompoundAABB(compound.as_ptr(), transform.into_raw())
    }))
}

pub fn overlap_sphere(sphere: &Sphere, transform: Transform, proxy: &ShapeProxy) -> Result<bool> {
    sphere.validate()?;
    overlap(proxy, transform, |proxy, transform| unsafe {
        ffi::b3OverlapSphere(sphere.raw(), transform, proxy)
    })
}

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

pub fn overlap_hull(hull: &Hull, transform: Transform, proxy: &ShapeProxy) -> Result<bool> {
    overlap(proxy, transform, |proxy, transform| unsafe {
        ffi::b3OverlapHull(hull.as_ptr(), transform, proxy)
    })
}

pub fn overlap_mesh(
    mesh: &MeshData,
    scale: impl Into<Vec3>,
    transform: Transform,
    proxy: &ShapeProxy,
) -> Result<bool> {
    let raw_mesh = ffi::b3Mesh {
        data: mesh.as_ptr(),
        scale: scale.into().validate()?.into_raw(),
    };
    overlap(proxy, transform, |proxy, transform| unsafe {
        ffi::b3OverlapMesh(&raw_mesh, transform, proxy)
    })
}

pub fn overlap_height_field(
    height_field: &HeightField,
    transform: Transform,
    proxy: &ShapeProxy,
) -> Result<bool> {
    overlap(proxy, transform, |proxy, transform| unsafe {
        ffi::b3OverlapHeightField(height_field.as_ptr(), transform, proxy)
    })
}

pub fn overlap_compound(
    compound: &Compound,
    transform: Transform,
    proxy: &ShapeProxy,
) -> Result<bool> {
    overlap(proxy, transform, |proxy, transform| unsafe {
        ffi::b3OverlapCompound(compound.as_ptr(), transform, proxy)
    })
}

pub fn ray_cast_sphere(sphere: &Sphere, input: RayCastInput) -> Result<CastOutput> {
    sphere.validate()?;
    ray_cast(input, |input| unsafe {
        ffi::b3RayCastSphere(sphere.raw(), input)
    })
}

pub fn ray_cast_hollow_sphere(sphere: &Sphere, input: RayCastInput) -> Result<CastOutput> {
    sphere.validate()?;
    ray_cast(input, |input| unsafe {
        ffi::b3RayCastHollowSphere(sphere.raw(), input)
    })
}

pub fn ray_cast_capsule(capsule: &Capsule, input: RayCastInput) -> Result<CastOutput> {
    capsule.validate()?;
    ray_cast(input, |input| unsafe {
        ffi::b3RayCastCapsule(capsule.raw(), input)
    })
}

pub fn ray_cast_hull(hull: &Hull, input: RayCastInput) -> Result<CastOutput> {
    ray_cast(input, |input| unsafe {
        ffi::b3RayCastHull(hull.as_ptr(), input)
    })
}

pub fn ray_cast_mesh(
    mesh: &MeshData,
    scale: impl Into<Vec3>,
    input: RayCastInput,
) -> Result<CastOutput> {
    let raw_mesh = ffi::b3Mesh {
        data: mesh.as_ptr(),
        scale: scale.into().validate()?.into_raw(),
    };
    ray_cast(input, |input| unsafe {
        ffi::b3RayCastMesh(&raw_mesh, input)
    })
}

pub fn ray_cast_height_field(
    height_field: &HeightField,
    input: RayCastInput,
) -> Result<CastOutput> {
    ray_cast(input, |input| unsafe {
        ffi::b3RayCastHeightField(height_field.as_ptr(), input)
    })
}

pub fn ray_cast_compound(compound: &Compound, input: RayCastInput) -> Result<CastOutput> {
    ray_cast(input, |input| unsafe {
        ffi::b3RayCastCompound(compound.as_ptr(), input)
    })
}

pub fn shape_cast_sphere(sphere: &Sphere, input: ShapeCastInput) -> Result<CastOutput> {
    sphere.validate()?;
    shape_cast(input, |input| unsafe {
        ffi::b3ShapeCastSphere(sphere.raw(), input)
    })
}

pub fn shape_cast_capsule(capsule: &Capsule, input: ShapeCastInput) -> Result<CastOutput> {
    capsule.validate()?;
    shape_cast(input, |input| unsafe {
        ffi::b3ShapeCastCapsule(capsule.raw(), input)
    })
}

pub fn shape_cast_hull(hull: &Hull, input: ShapeCastInput) -> Result<CastOutput> {
    shape_cast(input, |input| unsafe {
        ffi::b3ShapeCastHull(hull.as_ptr(), input)
    })
}

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
    let mut points: Vec<MaybeUninit<ffi::b3LocalManifoldPoint>> =
        Vec::with_capacity(ffi::B3_MAX_MANIFOLD_POINTS as usize);
    let mut raw: ffi::b3LocalManifold = unsafe { std::mem::zeroed() };
    raw.points = points.as_mut_ptr().cast();
    f(&mut raw, ffi::B3_MAX_MANIFOLD_POINTS as i32);
    let count = raw.pointCount.clamp(0, ffi::B3_MAX_MANIFOLD_POINTS as i32) as usize;
    unsafe { points.set_len(count) };
    let initialized: Vec<_> = points
        .into_iter()
        .map(|point| unsafe { point.assume_init() })
        .collect();
    Ok(unsafe { LocalManifold::from_raw(raw, &initialized) })
}

fn validate_density(density: f32) -> Result<()> {
    if density.is_finite() && density >= 0.0 {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}
