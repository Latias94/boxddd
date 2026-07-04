//! Contact manifold snapshots returned by collision and world contact APIs.

use super::*;

/// Maximum number of points in a Box3D contact manifold.
pub const MAX_MANIFOLD_POINTS: usize = 4;

/// One point in a contact manifold.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ManifoldPoint {
    /// Contact anchor on shape A in local contact coordinates.
    pub anchor_a: Vec3,
    /// Contact anchor on shape B in local contact coordinates.
    pub anchor_b: Vec3,
    /// Current separation at the contact point.
    pub separation: f32,
    /// Separation before solver updates.
    pub base_separation: f32,
    /// Normal impulse applied at this point during the current solve.
    pub normal_impulse: f32,
    /// Accumulated normal impulse for this contact point.
    pub total_normal_impulse: f32,
    /// Relative normal velocity at the contact point.
    pub normal_velocity: f32,
    /// Feature id used by Box3D to track contact persistence.
    pub feature_id: u32,
    /// Triangle index for mesh or height-field contacts, or Box3D's sentinel value.
    pub triangle_index: i32,
    /// Whether this contact point persisted from the previous step.
    pub persisted: bool,
}

impl ManifoldPoint {
    /// Converts a raw Box3D manifold point into the Rust value type.
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

/// Contact manifold containing up to [`MAX_MANIFOLD_POINTS`] points.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Manifold {
    /// Fixed-size storage for native contact points.
    pub points: [ManifoldPoint; MAX_MANIFOLD_POINTS],
    /// Contact normal.
    pub normal: Vec3,
    /// Twist friction impulse.
    pub twist_impulse: f32,
    /// Friction impulse vector.
    pub friction_impulse: Vec3,
    /// Rolling friction impulse vector.
    pub rolling_impulse: Vec3,
    /// Number of valid entries in `points`.
    pub point_count: i32,
}

impl Manifold {
    /// Returns only the valid manifold points.
    #[inline]
    pub fn points(&self) -> &[ManifoldPoint] {
        let count = self.point_count.clamp(0, MAX_MANIFOLD_POINTS as i32) as usize;
        &self.points[..count]
    }

    /// Converts a raw Box3D manifold into the Rust value type.
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

/// Contact data snapshot for a native contact pair.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContactData {
    /// Native contact id.
    pub contact_id: ContactId,
    /// First native shape in the contact pair.
    pub shape_id_a: ShapeId,
    /// Second native shape in the contact pair.
    pub shape_id_b: ShapeId,
    /// Contact manifolds copied from Box3D.
    pub manifolds: Vec<Manifold>,
}

impl ContactData {
    /// Copies contact data from a raw Box3D contact snapshot.
    ///
    /// # Safety
    ///
    /// `raw.manifolds` must either be null or point to `raw.manifoldCount`
    /// initialized `b3Manifold` values for the duration of this call.
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
