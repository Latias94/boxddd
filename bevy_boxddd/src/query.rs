use crate::resources::BoxdddPhysicsContext;
use bevy_ecs::entity::Entity;
use bevy_math::Vec3;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PhysicsQueryHit {
    pub shape_id: boxddd::ShapeId,
    pub entity: Option<Entity>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PhysicsRayHit {
    pub shape_id: boxddd::ShapeId,
    pub entity: Option<Entity>,
    pub point: Vec3,
    pub normal: Vec3,
    pub fraction: f32,
    pub user_material_id: u64,
    pub triangle_index: i32,
    pub child_index: i32,
}

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
