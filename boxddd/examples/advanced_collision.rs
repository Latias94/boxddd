use boxddd::prelude::*;

fn main() -> boxddd::Result<()> {
    let sphere_a = ShapeProxy::sphere(0.5)?;
    let sphere_b = ShapeProxy::sphere(0.5)?;

    let separated = Transform::new(Vec3::new(1.4, 0.0, 0.0), Quat::IDENTITY);
    let distance = shape_distance(DistanceInput::new(
        sphere_a.clone(),
        sphere_b.clone(),
        separated,
    )?)?;
    println!(
        "sphere distance = {:.3}, normal = {:?}",
        distance.distance, distance.normal
    );

    let cast = shape_cast_pair(ShapeCastPairInput::new(
        sphere_a.clone(),
        sphere_b.clone(),
        Transform::new(Vec3::new(3.0, 0.0, 0.0), Quat::IDENTITY),
        Vec3::new(-4.0, 0.0, 0.0),
    )?)?;
    println!(
        "shape cast hit = {}, fraction = {:.3}, point = {:?}",
        cast.hit, cast.fraction, cast.point
    );

    let manifold = collide_spheres(
        &Sphere::new(Vec3::ZERO, 0.5),
        &Sphere::new(Vec3::ZERO, 0.5),
        Transform::new(Vec3::new(0.75, 0.0, 0.0), Quat::IDENTITY),
    )?;
    println!(
        "overlap manifold: {} point(s), normal = {:?}",
        manifold.points.len(),
        manifold.normal
    );

    let mut planes = [CollisionPlane::new(
        Plane {
            normal: Vec3::X,
            offset: 0.0,
        },
        1.0,
        true,
    )?];
    let solved = solve_planes(Vec3::new(-2.0, 0.0, 0.0), &mut planes)?;
    let clipped = clip_vector(Vec3::new(-5.0, 1.0, 0.0), &planes)?;
    println!(
        "plane solve delta = {:?}, iterations = {}, clipped velocity = {:?}",
        solved.delta, solved.iteration_count, clipped
    );

    Ok(())
}
