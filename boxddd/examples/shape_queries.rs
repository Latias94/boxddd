use boxddd::prelude::*;

fn main() -> boxddd::Result<()> {
    let mut world = World::new(WorldDef::default())?;

    let body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Static)
            .position(Vec3::ZERO)
            .build(),
    );
    let sphere = world.create_sphere_shape(
        body,
        &ShapeDef::builder().density(1.0).build(),
        &Sphere::new([-1.0, 0.0, 0.0], 0.5),
    );
    let cube = world.create_hull_shape(
        body,
        &ShapeDef::builder().density(1.0).build(),
        &BoxHull::cube(0.4),
    );

    let shapes = world.try_body_shapes(body)?;
    println!("body owns {} shape(s): {shapes:?}", shapes.len());

    let overlap = world.overlap_aabb(
        Aabb {
            lower_bound: [-2.0, -1.0, -1.0].into(),
            upper_bound: [0.5, 1.0, 1.0].into(),
        },
        QueryFilter::default(),
    )?;
    println!("world overlap hits: {overlap:?}");

    let closest_ray_hit =
        world.cast_ray_closest([-3.0, 0.0, 0.0], [5.0, 0.0, 0.0], QueryFilter::default())?;
    println!("closest ray hit: {closest_ray_hit:?}");

    let body_hit = world.try_body_cast_ray(
        body,
        [-3.0, 0.0, 0.0],
        [5.0, 0.0, 0.0],
        QueryFilter::default(),
    )?;
    println!("body-scoped ray hit: {body_hit:?}");

    let sphere_closest = world.try_shape_closest_point(sphere, [2.0, 0.0, 0.0])?;
    let cube_hit = world.try_shape_cast_ray(cube, [-2.0, 0.0, 0.0], [5.0, 0.0, 0.0])?;
    println!("sphere closest point to x=2: {sphere_closest:?}");
    println!("cube shape-only ray hit: {cube_hit:?}");

    Ok(())
}
