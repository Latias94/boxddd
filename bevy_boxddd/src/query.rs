//! Renderer-independent physics query helpers for Bevy applications.

use crate::resources::BoxdddPhysicsContext;
use bevy_ecs::entity::Entity;
use bevy_math::Vec3;

/// Result from an AABB overlap query mapped back to a Bevy entity when possible.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PhysicsQueryHit {
    /// Native Box3D shape id that overlapped the query bounds.
    pub shape_id: boxddd::ShapeId,
    /// Bevy entity mapped to `shape_id`, if the shape is plugin-owned.
    pub entity: Option<Entity>,
}

/// Result from a ray query mapped back to a Bevy entity when possible.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PhysicsRayHit {
    /// Native Box3D shape id that was hit.
    pub shape_id: boxddd::ShapeId,
    /// Bevy entity mapped to `shape_id`, if the shape is plugin-owned.
    pub entity: Option<Entity>,
    /// Hit point in Bevy world coordinates.
    pub point: Vec3,
    /// Hit normal in Bevy world coordinates.
    pub normal: Vec3,
    /// Fraction along the input ray translation where the hit occurred.
    pub fraction: f32,
    /// Surface material id reported by Box3D.
    pub user_material_id: u64,
    /// Triangle index for mesh and height-field hits, or Box3D's sentinel value.
    pub triangle_index: i32,
    /// Child index for compound or aggregate shapes, or Box3D's sentinel value.
    pub child_index: i32,
}

/// Runs a Box3D AABB overlap query and maps native shape ids to Bevy entities.
pub fn overlap_aabb(
    context: &BoxdddPhysicsContext,
    lower_bound: Vec3,
    upper_bound: Vec3,
    filter: boxddd::QueryFilter,
) -> boxddd::Result<Vec<PhysicsQueryHit>> {
    let world = context.world().ok_or(boxddd::Error::InvalidWorldId)?;
    let hits = world.overlap_aabb(
        boxddd::Aabb {
            lower_bound: to_boxddd_vec3(lower_bound),
            upper_bound: to_boxddd_vec3(upper_bound),
        },
        filter,
    )?;
    Ok(hits
        .into_iter()
        .map(|hit| PhysicsQueryHit {
            shape_id: hit.shape_id,
            entity: context.shape_entity(hit.shape_id),
        })
        .collect())
}

/// Casts a ray against the Box3D world and returns all hits in Box3D order.
pub fn cast_ray(
    context: &BoxdddPhysicsContext,
    origin: Vec3,
    translation: Vec3,
    filter: boxddd::QueryFilter,
) -> boxddd::Result<Vec<PhysicsRayHit>> {
    let world = context.world().ok_or(boxddd::Error::InvalidWorldId)?;
    let hits = world.cast_ray(to_boxddd_pos(origin), to_boxddd_vec3(translation), filter)?;
    Ok(hits
        .into_iter()
        .map(|hit| map_ray_hit(context, hit))
        .collect())
}

/// Casts a ray against the Box3D world and returns the closest hit.
pub fn cast_ray_closest(
    context: &BoxdddPhysicsContext,
    origin: Vec3,
    translation: Vec3,
    filter: boxddd::QueryFilter,
) -> boxddd::Result<Option<PhysicsRayHit>> {
    let world = context.world().ok_or(boxddd::Error::InvalidWorldId)?;
    Ok(world
        .cast_ray_closest(to_boxddd_pos(origin), to_boxddd_vec3(translation), filter)?
        .map(|hit| PhysicsRayHit {
            shape_id: hit.shape_id,
            entity: context.shape_entity(hit.shape_id),
            point: to_bevy_pos(hit.point),
            normal: to_bevy_vec3(hit.normal),
            fraction: hit.fraction,
            user_material_id: hit.user_material_id,
            triangle_index: hit.triangle_index,
            child_index: hit.child_index,
        }))
}

fn map_ray_hit(context: &BoxdddPhysicsContext, hit: boxddd::query::RayHit) -> PhysicsRayHit {
    PhysicsRayHit {
        shape_id: hit.shape_id,
        entity: context.shape_entity(hit.shape_id),
        point: to_bevy_pos(hit.point),
        normal: to_bevy_vec3(hit.normal),
        fraction: hit.fraction,
        user_material_id: hit.user_material_id,
        triangle_index: hit.triangle_index,
        child_index: hit.child_index,
    }
}

fn to_boxddd_vec3(value: Vec3) -> boxddd::Vec3 {
    boxddd::Vec3::new(value.x, value.y, value.z)
}

fn to_boxddd_pos(value: Vec3) -> boxddd::Pos {
    boxddd::Pos::from(to_boxddd_vec3(value))
}

fn to_bevy_pos(value: boxddd::Pos) -> Vec3 {
    Vec3::new(value.x as f32, value.y as f32, value.z as f32)
}

fn to_bevy_vec3(value: boxddd::Vec3) -> Vec3 {
    Vec3::new(value.x, value.y, value.z)
}
