use super::*;
pub const MAX_MANIFOLD_POINTS: usize = 4;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ManifoldPoint {
    pub anchor_a: Vec3,
    pub anchor_b: Vec3,
    pub separation: f32,
    pub base_separation: f32,
    pub normal_impulse: f32,
    pub total_normal_impulse: f32,
    pub normal_velocity: f32,
    pub feature_id: u32,
    pub triangle_index: i32,
    pub persisted: bool,
}

impl ManifoldPoint {
    #[inline]
    pub const fn from_raw(raw: ffi::b3ManifoldPoint) -> Self {
        Self {
            anchor_a: Vec3::from_raw(raw.anchorA),
            anchor_b: Vec3::from_raw(raw.anchorB),
            separation: raw.separation,
            base_separation: raw.baseSeparation,
            normal_impulse: raw.normalImpulse,
            total_normal_impulse: raw.totalNormalImpulse,
            normal_velocity: raw.normalVelocity,
            feature_id: raw.featureId,
            triangle_index: raw.triangleIndex,
            persisted: raw.persisted,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Manifold {
    pub points: [ManifoldPoint; MAX_MANIFOLD_POINTS],
    pub normal: Vec3,
    pub twist_impulse: f32,
    pub friction_impulse: Vec3,
    pub rolling_impulse: Vec3,
    pub point_count: i32,
}

impl Manifold {
    #[inline]
    pub fn points(&self) -> &[ManifoldPoint] {
        let count = self.point_count.clamp(0, MAX_MANIFOLD_POINTS as i32) as usize;
        &self.points[..count]
    }

    #[inline]
    pub fn from_raw(raw: ffi::b3Manifold) -> Self {
        Self {
            points: raw.points.map(ManifoldPoint::from_raw),
            normal: Vec3::from_raw(raw.normal),
            twist_impulse: raw.twistImpulse,
            friction_impulse: Vec3::from_raw(raw.frictionImpulse),
            rolling_impulse: Vec3::from_raw(raw.rollingImpulse),
            point_count: raw.pointCount.clamp(0, MAX_MANIFOLD_POINTS as i32),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContactData {
    pub contact_id: ContactId,
    pub shape_id_a: ShapeId,
    pub shape_id_b: ShapeId,
    pub manifolds: Vec<Manifold>,
}

impl ContactData {
    #[inline]
    pub unsafe fn from_raw(raw: ffi::b3ContactData) -> Self {
        let manifolds = if raw.manifolds.is_null() || raw.manifoldCount <= 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(raw.manifolds, raw.manifoldCount as usize) }
                .iter()
                .copied()
                .map(Manifold::from_raw)
                .collect()
        };

        Self {
            contact_id: ContactId::from_raw(raw.contactId),
            shape_id_a: ShapeId::from_raw(raw.shapeIdA),
            shape_id_b: ShapeId::from_raw(raw.shapeIdB),
            manifolds,
        }
    }
}
