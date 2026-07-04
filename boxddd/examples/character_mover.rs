use boxddd::prelude::*;

fn main() -> boxddd::Result<()> {
    let mut world = World::new(WorldDef::default())?;

    let floor = world.create_body(BodyDef::builder().position([0.0, -0.5, 0.0]).build());
    world.create_hull_shape(floor, &ShapeDef::default(), &BoxHull::new(6.0, 0.5, 6.0));

    let obstacle = world.create_body(BodyDef::builder().position([1.25, 0.4, 0.0]).build());
    world.create_hull_shape(obstacle, &ShapeDef::default(), &BoxHull::new(0.5, 0.9, 0.5));

    let mover = Capsule::new([0.0, 0.3, 0.0], [0.0, 1.3, 0.0], 0.25);
    let start = Vec3::new(-1.5, 0.05, 0.0);
    let desired_delta = Vec3::new(3.0, 0.0, 0.0);
    let fraction = world.cast_mover(start, &mover, desired_delta, QueryFilter::default())?;
    let safe_delta = Vec3::new(
        desired_delta.x * fraction,
        desired_delta.y * fraction,
        desired_delta.z * fraction,
    );
    let final_origin = Vec3::new(
        start.x + safe_delta.x,
        start.y + safe_delta.y,
        start.z + safe_delta.z,
    );

    println!(
        "mover cast: fraction={fraction:.3}, safe_delta={safe_delta:?}, final_origin={final_origin:?}"
    );
    assert!(
        (0.0..1.0).contains(&fraction),
        "expected the mover cast to stop before the obstacle"
    );

    let mover_planes = world.collide_mover(final_origin, &mover, QueryFilter::default())?;
    println!("mover contact planes: {}", mover_planes.len());
    assert!(
        !mover_planes.is_empty(),
        "expected at least one mover contact plane"
    );
    for (index, plane) in mover_planes.iter().enumerate() {
        println!(
            "  plane {index}: normal={:?}, offset={:.3}, point={:?}",
            plane.plane.normal, plane.plane.offset, plane.point
        );
    }

    if !mover_planes.is_empty() {
        let mut solver_planes = mover_planes
            .iter()
            .map(|plane| CollisionPlane::new(plane.plane, 0.75, true))
            .collect::<boxddd::Result<Vec<_>>>()?;
        let correction = solve_planes(Vec3::new(0.0, -0.5, 0.0), &mut solver_planes)?;
        let clipped_velocity = clip_vector(Vec3::new(2.0, -3.0, 0.0), &solver_planes)?;

        println!(
            "plane solver: delta={:?}, iterations={}, clipped_velocity={:?}",
            correction.delta, correction.iteration_count, clipped_velocity
        );
    }

    Ok(())
}
