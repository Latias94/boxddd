use bevy_app::App;
use bevy_boxddd::prelude::*;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::message::Messages;
use bevy_math::Vec3;
use bevy_time::{TimePlugin, TimeUpdateStrategy};
use bevy_transform::components::Transform;
use static_assertions::assert_impl_all;

assert_impl_all!(Collider: Send, Sync);

const fn assert_static<T: 'static>() {}
const _: fn() = assert_static::<Collider>;

fn physics_app(settings: BoxdddPhysicsSettings) -> App {
    let mut app = App::new();
    app.add_plugins(TimePlugin)
        .insert_resource(TimeUpdateStrategy::FixedTimesteps(1))
        .add_plugins(BoxdddPhysicsPlugin::new(settings));
    app
}

fn run_fixed_frames(app: &mut App, count: usize) {
    for _ in 0..count {
        app.update();
    }
}

#[test]
fn plugin_inserts_non_send_context_and_uses_settings() {
    let settings = BoxdddPhysicsSettings {
        gravity: Vec3::new(0.0, -3.0, 0.0),
        ..Default::default()
    };
    let app = physics_app(settings);

    let context = app
        .world()
        .get_non_send::<BoxdddPhysicsContext>()
        .expect("plugin should insert the non-send physics context");
    let gravity = context
        .world()
        .expect("world should be initialized")
        .gravity();

    assert_eq!(gravity, boxddd::Vec3::new(0.0, -3.0, 0.0));
}

#[test]
fn invalid_world_settings_emit_error_message() {
    let settings = BoxdddPhysicsSettings {
        gravity: Vec3::new(f32::NAN, 0.0, 0.0),
        ..Default::default()
    };
    let mut app = physics_app(settings);

    let messages = app
        .world_mut()
        .resource_mut::<Messages<BoxdddErrorMessage>>()
        .drain()
        .collect::<Vec<_>>();
    assert!(messages.iter().any(|message| {
        message.operation == BoxdddOperation::CreateWorld
            && message.error == boxddd::Error::InvalidArgument
    }));

    let context = app
        .world()
        .get_non_send::<BoxdddPhysicsContext>()
        .expect("disabled context should still be inserted");
    assert!(context.world().is_none());
}

#[test]
fn invalid_fixed_timestep_settings_emit_error_message() {
    let settings = BoxdddPhysicsSettings {
        fixed_timestep_seconds: Some(0.0),
        ..Default::default()
    };
    let mut app = physics_app(settings);

    let messages = app
        .world_mut()
        .resource_mut::<Messages<BoxdddErrorMessage>>()
        .drain()
        .collect::<Vec<_>>();
    assert!(messages.iter().any(|message| {
        message.operation == BoxdddOperation::ConfigureFixedTimestep
            && message.error == boxddd::Error::InvalidArgument
    }));

    let context = app
        .world()
        .get_non_send::<BoxdddPhysicsContext>()
        .expect("valid world context should still be inserted");
    assert!(context.world().is_some());
}

#[test]
fn bodies_and_simple_colliders_are_created_and_cleaned_up() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let entity = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::capsule_y(0.5, 0.2),
            Transform::from_xyz(0.0, 2.0, 0.0),
        ))
        .id();

    run_fixed_frames(&mut app, 2);

    let body = *app
        .world()
        .entity(entity)
        .get::<BoxdddBody>()
        .expect("plugin should insert BoxdddBody");
    let shape = *app
        .world()
        .entity(entity)
        .get::<BoxdddShape>()
        .expect("plugin should insert BoxdddShape");
    assert!(body.id().is_valid());
    assert!(shape.id().is_valid());

    app.world_mut().entity_mut(entity).despawn();
    run_fixed_frames(&mut app, 2);

    assert!(!body.id().is_valid());
}

#[test]
fn removing_body_component_recreates_body_and_shape_without_stale_ids() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let entity = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            Transform::from_xyz(0.0, 2.0, 0.0),
        ))
        .id();

    run_fixed_frames(&mut app, 2);

    let old_body = *app.world().entity(entity).get::<BoxdddBody>().unwrap();
    let old_shape = *app.world().entity(entity).get::<BoxdddShape>().unwrap();

    app.world_mut().entity_mut(entity).remove::<BoxdddBody>();
    run_fixed_frames(&mut app, 2);

    let new_body = *app
        .world()
        .entity(entity)
        .get::<BoxdddBody>()
        .expect("body should be recreated when the authoring component remains");
    let new_shape = *app
        .world()
        .entity(entity)
        .get::<BoxdddShape>()
        .expect("shape should be recreated against the replacement body");

    assert!(!old_body.id().is_valid());
    assert!(!old_shape.id().is_valid());
    assert!(new_body.id().is_valid());
    assert!(new_shape.id().is_valid());
}

#[test]
fn removing_collider_component_destroys_shape_but_keeps_body() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let entity = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            Transform::from_xyz(0.0, 2.0, 0.0),
        ))
        .id();

    run_fixed_frames(&mut app, 2);

    let body = *app.world().entity(entity).get::<BoxdddBody>().unwrap();
    let shape = *app.world().entity(entity).get::<BoxdddShape>().unwrap();

    app.world_mut().entity_mut(entity).remove::<Collider>();
    run_fixed_frames(&mut app, 2);

    assert!(body.id().is_valid());
    assert!(!shape.id().is_valid());
    assert!(app.world().entity(entity).get::<BoxdddBody>().is_some());
    assert!(app.world().entity(entity).get::<BoxdddShape>().is_none());
}

#[test]
fn invalid_collider_dimensions_emit_error_message() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let entity = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(-1.0),
            Transform::from_xyz(0.0, 2.0, 0.0),
        ))
        .id();

    run_fixed_frames(&mut app, 2);

    assert!(app.world().entity(entity).get::<BoxdddBody>().is_some());
    assert!(app.world().entity(entity).get::<BoxdddShape>().is_none());

    let messages = app
        .world_mut()
        .resource_mut::<Messages<BoxdddErrorMessage>>()
        .drain()
        .collect::<Vec<_>>();
    assert!(messages.iter().any(|message| {
        message.operation == BoxdddOperation::CreateShape
            && message.entity == Some(entity)
            && message.error == boxddd::Error::InvalidArgument
    }));
}

#[test]
fn child_colliders_create_multiple_shapes_for_one_body() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let body = app
        .world_mut()
        .spawn((RigidBody::Dynamic, Transform::from_xyz(0.0, 2.0, 0.0)))
        .id();
    let left = app
        .world_mut()
        .spawn((Collider::sphere(0.25), ChildOf(body)))
        .id();
    let right = app
        .world_mut()
        .spawn((Collider::sphere(0.35), ChildOf(body)))
        .id();

    run_fixed_frames(&mut app, 2);

    let body_id = app.world().entity(body).get::<BoxdddBody>().unwrap().id();
    let left_shape = app.world().entity(left).get::<BoxdddShape>().unwrap().id();
    let right_shape = app.world().entity(right).get::<BoxdddShape>().unwrap().id();
    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();

    assert_ne!(left_shape, right_shape);
    assert_eq!(context.shape_entity(left_shape), Some(left));
    assert_eq!(context.shape_entity(right_shape), Some(right));
    assert_eq!(
        context.world().unwrap().try_shape_body(left_shape).unwrap(),
        body_id
    );
    assert_eq!(
        context
            .world()
            .unwrap()
            .try_shape_body(right_shape)
            .unwrap(),
        body_id
    );
}

#[test]
fn child_collider_transform_offsets_shape_geometry() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let body = app
        .world_mut()
        .spawn((RigidBody::Static, Transform::from_xyz(0.0, 0.0, 0.0)))
        .id();
    let collider = app
        .world_mut()
        .spawn((
            Collider::sphere(0.25),
            Transform::from_xyz(1.0, 0.0, 0.0),
            ChildOf(body),
        ))
        .id();

    run_fixed_frames(&mut app, 2);

    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();
    let hit = bevy_boxddd::cast_ray_closest(
        context,
        Vec3::new(1.0, -1.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        boxddd::QueryFilter::default(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(hit.entity, Some(collider));
    assert!((hit.point.x - 1.0).abs() < 0.01, "hit: {hit:?}");
}

#[test]
fn removing_one_child_collider_only_destroys_that_shape() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let body = app
        .world_mut()
        .spawn((RigidBody::Dynamic, Transform::from_xyz(0.0, 2.0, 0.0)))
        .id();
    let left = app
        .world_mut()
        .spawn((Collider::sphere(0.25), ChildOf(body)))
        .id();
    let right = app
        .world_mut()
        .spawn((Collider::sphere(0.35), ChildOf(body)))
        .id();

    run_fixed_frames(&mut app, 2);

    let left_shape = app.world().entity(left).get::<BoxdddShape>().unwrap().id();
    let right_shape = app.world().entity(right).get::<BoxdddShape>().unwrap().id();

    app.world_mut().entity_mut(left).despawn();
    run_fixed_frames(&mut app, 2);

    assert!(!left_shape.is_valid());
    assert!(right_shape.is_valid());
    assert!(app.world().entity(right).get::<BoxdddShape>().is_some());
}

#[test]
fn despawning_body_destroys_child_collider_shapes() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let body = app
        .world_mut()
        .spawn((RigidBody::Dynamic, Transform::from_xyz(0.0, 2.0, 0.0)))
        .id();
    let left = app
        .world_mut()
        .spawn((Collider::sphere(0.25), ChildOf(body)))
        .id();
    let right = app
        .world_mut()
        .spawn((Collider::sphere(0.35), ChildOf(body)))
        .id();

    run_fixed_frames(&mut app, 2);

    let body_id = app.world().entity(body).get::<BoxdddBody>().unwrap().id();
    let left_shape = app.world().entity(left).get::<BoxdddShape>().unwrap().id();
    let right_shape = app.world().entity(right).get::<BoxdddShape>().unwrap().id();

    app.world_mut().entity_mut(body).despawn();
    run_fixed_frames(&mut app, 2);

    assert!(!body_id.is_valid());
    assert!(!left_shape.is_valid());
    assert!(!right_shape.is_valid());
}

#[test]
fn readding_child_collider_component_replaces_shape_id() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let body = app
        .world_mut()
        .spawn((RigidBody::Dynamic, Transform::from_xyz(0.0, 2.0, 0.0)))
        .id();
    let collider = app
        .world_mut()
        .spawn((Collider::sphere(0.25), ChildOf(body)))
        .id();

    run_fixed_frames(&mut app, 2);

    let old_shape = app
        .world()
        .entity(collider)
        .get::<BoxdddShape>()
        .unwrap()
        .id();

    app.world_mut().entity_mut(collider).remove::<Collider>();
    run_fixed_frames(&mut app, 2);
    assert!(!old_shape.is_valid());
    assert!(app.world().entity(collider).get::<BoxdddShape>().is_none());

    app.world_mut()
        .entity_mut(collider)
        .insert(Collider::sphere(0.35));
    run_fixed_frames(&mut app, 2);

    let new_shape = app
        .world()
        .entity(collider)
        .get::<BoxdddShape>()
        .unwrap()
        .id();
    assert!(new_shape.is_valid());
    assert_ne!(old_shape, new_shape);
}

#[test]
fn changing_collider_descriptor_recreates_native_shape() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let entity = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.25),
            Transform::from_xyz(0.0, 1.0, 0.0),
        ))
        .id();

    run_fixed_frames(&mut app, 2);
    let old_shape = app
        .world()
        .entity(entity)
        .get::<BoxdddShape>()
        .unwrap()
        .id();

    app.world_mut()
        .entity_mut(entity)
        .insert(Collider::capsule_y(0.35, 0.12));
    run_fixed_frames(&mut app, 3);

    let new_shape = app
        .world()
        .entity(entity)
        .get::<BoxdddShape>()
        .unwrap()
        .id();
    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();
    let world = context.world().unwrap();

    assert!(!old_shape.is_valid());
    assert!(new_shape.is_valid());
    assert_ne!(old_shape, new_shape);
    assert_eq!(
        world.try_shape_type(new_shape).unwrap(),
        boxddd::ShapeType::Capsule
    );
}

#[test]
fn changing_physics_material_recreates_native_shape() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let entity = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.25),
            PhysicsMaterial {
                filter: boxddd::Filter {
                    category_bits: 0b10,
                    mask_bits: u64::MAX,
                    group_index: 0,
                },
                ..Default::default()
            },
            Transform::from_xyz(0.0, 1.0, 0.0),
        ))
        .id();

    run_fixed_frames(&mut app, 2);
    let old_shape = app
        .world()
        .entity(entity)
        .get::<BoxdddShape>()
        .unwrap()
        .id();

    app.world_mut().entity_mut(entity).insert(PhysicsMaterial {
        is_sensor: true,
        enable_sensor_events: true,
        filter: boxddd::Filter {
            category_bits: 0b100,
            mask_bits: 0b1000,
            group_index: 0,
        },
        ..Default::default()
    });
    run_fixed_frames(&mut app, 3);

    let new_shape = app
        .world()
        .entity(entity)
        .get::<BoxdddShape>()
        .unwrap()
        .id();
    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();
    let world = context.world().unwrap();
    let filter = world.try_shape_filter(new_shape).unwrap();

    assert!(!old_shape.is_valid());
    assert!(new_shape.is_valid());
    assert_ne!(old_shape, new_shape);
    assert!(world.try_shape_sensor(new_shape).unwrap());
    assert_eq!(filter.category_bits, 0b100);
    assert_eq!(filter.mask_bits, 0b1000);
}

#[test]
fn reparenting_child_collider_recreates_shape_on_new_body() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let first_body = app
        .world_mut()
        .spawn((RigidBody::Dynamic, Transform::from_xyz(0.0, 2.0, 0.0)))
        .id();
    let second_body = app
        .world_mut()
        .spawn((RigidBody::Dynamic, Transform::from_xyz(2.0, 2.0, 0.0)))
        .id();
    let collider = app
        .world_mut()
        .spawn((Collider::sphere(0.25), ChildOf(first_body)))
        .id();

    run_fixed_frames(&mut app, 2);

    let old_shape = app
        .world()
        .entity(collider)
        .get::<BoxdddShape>()
        .unwrap()
        .id();

    app.world_mut()
        .entity_mut(collider)
        .insert(ChildOf(second_body));
    run_fixed_frames(&mut app, 2);

    let new_shape = app
        .world()
        .entity(collider)
        .get::<BoxdddShape>()
        .unwrap()
        .id();
    let second_body_id = app
        .world()
        .entity(second_body)
        .get::<BoxdddBody>()
        .unwrap()
        .id();
    let context = app.world().get_non_send::<BoxdddPhysicsContext>().unwrap();

    assert!(!old_shape.is_valid());
    assert!(new_shape.is_valid());
    assert_eq!(
        context.world().unwrap().try_shape_body(new_shape).unwrap(),
        second_body_id
    );
}

#[test]
fn advanced_static_colliders_create_shapes() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let descriptors = [
        Collider::mesh_box(Vec3::ZERO, Vec3::splat(1.0), Vec3::ONE, true),
        Collider::mesh_grid(3, 3, 1.0, 1, Vec3::ONE, true),
        Collider::height_field_grid(3, 3, Vec3::ONE, false),
        Collider::compound_sphere(Vec3::ZERO, 0.5, boxddd::SurfaceMaterial::default()),
        Collider::created_rock_hull(0.5),
        Collider::transformed_rock_hull(0.5, Vec3::ZERO, bevy_math::Quat::IDENTITY, Vec3::ONE),
    ];

    for (index, collider) in descriptors.into_iter().enumerate() {
        app.world_mut().spawn((
            RigidBody::Static,
            collider,
            Transform::from_xyz(index as f32 * 2.0, 0.0, 0.0),
        ));
    }

    run_fixed_frames(&mut app, 2);

    let mut query = app.world_mut().query::<(&Collider, Option<&BoxdddShape>)>();
    let created = query
        .iter(app.world())
        .filter(|(_, shape)| shape.is_some())
        .count();

    assert_eq!(created, descriptors.len());
}

#[test]
fn advanced_colliders_on_dynamic_bodies_emit_errors() {
    let mut app = physics_app(BoxdddPhysicsSettings::default());
    let static_only_descriptors = [
        Collider::mesh_box(Vec3::ZERO, Vec3::splat(1.0), Vec3::ONE, true),
        Collider::mesh_grid(3, 3, 1.0, 1, Vec3::ONE, true),
        Collider::height_field_grid(3, 3, Vec3::ONE, false),
        Collider::compound_sphere(Vec3::ZERO, 0.5, boxddd::SurfaceMaterial::default()),
    ];
    let dynamic_hull_descriptors = [
        Collider::created_rock_hull(0.5),
        Collider::transformed_rock_hull(0.5, Vec3::ZERO, bevy_math::Quat::IDENTITY, Vec3::ONE),
    ];

    for (index, collider) in static_only_descriptors
        .into_iter()
        .chain(dynamic_hull_descriptors)
        .enumerate()
    {
        app.world_mut().spawn((
            RigidBody::Dynamic,
            collider,
            Transform::from_xyz(index as f32 * 2.0, 2.0, 0.0),
        ));
    }

    run_fixed_frames(&mut app, 2);

    let mut query = app.world_mut().query::<(&Collider, Option<&BoxdddShape>)>();
    let created = query
        .iter(app.world())
        .filter(|(_, shape)| shape.is_some())
        .count();
    assert_eq!(created, dynamic_hull_descriptors.len());

    let messages = app
        .world_mut()
        .resource_mut::<Messages<BoxdddErrorMessage>>()
        .drain()
        .collect::<Vec<_>>();
    let create_shape_errors = messages
        .iter()
        .filter(|message| {
            message.operation == BoxdddOperation::CreateShape
                && message.error == boxddd::Error::InvalidArgument
        })
        .count();

    assert_eq!(create_shape_errors, static_only_descriptors.len());
}
