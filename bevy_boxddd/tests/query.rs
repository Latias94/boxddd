use bevy_app::App;
use bevy_boxddd::prelude::*;
use bevy_math::Vec3;
use bevy_time::{TimePlugin, TimeUpdateStrategy};
use bevy_transform::components::Transform;

fn physics_app() -> App {
    let mut app = App::new();
    app.add_plugins(TimePlugin)
        .insert_resource(TimeUpdateStrategy::FixedTimesteps(1))
        .add_plugins(BoxdddPhysicsPlugin::new(BoxdddPhysicsSettings::default()));
    app
}

fn run_fixed_frames(app: &mut App, count: usize) {
    for _ in 0..count {
        app.update();
    }
}

fn static_sphere(app: &mut App, position: Vec3, radius: f32) -> bevy_ecs::entity::Entity {
    app.world_mut()
        .spawn((
            RigidBody::Static,
            Collider::sphere(radius),
            Transform::from_translation(position),
        ))
        .id()
}

#[test]
fn closest_ray_hit_maps_to_nearest_bevy_entity() {
    let mut app = physics_app();
    let near = static_sphere(&mut app, Vec3::new(0.0, 0.0, 0.0), 0.35);
    let far = static_sphere(&mut app, Vec3::new(2.0, 0.0, 0.0), 0.35);

    run_fixed_frames(&mut app, 2);

    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();
    let hit = cast_ray_closest(
        context,
        Vec3::new(-2.0, 0.0, 0.0),
        Vec3::new(5.0, 0.0, 0.0),
        boxddd::QueryFilter::default(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(hit.entity, Some(near));
    assert_ne!(hit.entity, Some(far));
}

#[test]
fn ray_miss_returns_none() {
    let mut app = physics_app();
    static_sphere(&mut app, Vec3::new(0.0, 0.0, 0.0), 0.35);

    run_fixed_frames(&mut app, 2);

    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();
    let hit = cast_ray_closest(
        context,
        Vec3::new(-2.0, 4.0, 0.0),
        Vec3::new(5.0, 0.0, 0.0),
        boxddd::QueryFilter::default(),
    )
    .unwrap();

    assert!(hit.is_none());
}

#[test]
fn query_filter_mask_excludes_shapes() {
    let mut app = physics_app();
    let included = app
        .world_mut()
        .spawn((
            RigidBody::Static,
            Collider::sphere(0.35),
            Transform::from_xyz(0.0, 0.0, 0.0),
            PhysicsMaterial {
                filter: boxddd::Filter {
                    category_bits: 0b10,
                    mask_bits: u64::MAX,
                    group_index: 0,
                },
                ..Default::default()
            },
        ))
        .id();

    run_fixed_frames(&mut app, 2);

    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();
    let hits = overlap_aabb(
        context,
        Vec3::new(-1.0, -1.0, -1.0),
        Vec3::new(1.0, 1.0, 1.0),
        boxddd::QueryFilter::default().mask_bits(0b10),
    )
    .unwrap();
    assert!(hits.iter().any(|hit| hit.entity == Some(included)));

    let excluded = overlap_aabb(
        context,
        Vec3::new(-1.0, -1.0, -1.0),
        Vec3::new(1.0, 1.0, 1.0),
        boxddd::QueryFilter::default().mask_bits(0b100),
    )
    .unwrap();
    assert!(excluded.is_empty());
}
