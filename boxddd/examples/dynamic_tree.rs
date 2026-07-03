use boxddd::prelude::*;

fn aabb(lower: impl Into<Vec3>, upper: impl Into<Vec3>) -> Aabb {
    Aabb {
        lower_bound: lower.into(),
        upper_bound: upper.into(),
    }
}

fn main() -> boxddd::Result<()> {
    let mut tree = DynamicTree::new()?;

    let crate_proxy = tree.create_proxy_with_category_bits(
        aabb([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]),
        0b0001,
        100,
    )?;
    let sensor_proxy = tree.create_proxy_with_category_bits(
        aabb([2.0, -0.25, -0.25], [2.5, 0.25, 0.25]),
        0b0010,
        200,
    )?;

    println!("created proxies: {crate_proxy:?}, {sensor_proxy:?}");
    println!(
        "tree height = {}, bytes = {}",
        tree.height()?,
        tree.byte_count()?
    );

    let broad_phase_hits = tree.query(
        aabb([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]),
        DynamicTreeFilter::new(0b0001),
    )?;
    println!("overlap query hit user data: {broad_phase_hits:?}");

    let closest = tree.query_closest(
        Vec3::ZERO,
        DynamicTreeFilter::default(),
        1_000_000.0,
        |hit| {
            println!(
                "closest visitor saw user_data={} distance2={:.3}",
                hit.user_data, hit.min_distance_squared
            );
            hit.min_distance_squared
        },
    )?;
    println!(
        "closest query visits: nodes={}, leaves={}, min_distance2={:.3}",
        closest.stats.node_visits, closest.stats.leaf_visits, closest.min_distance_squared
    );

    let mut ray_hits = Vec::new();
    let ray_stats = tree.ray_cast(
        RayCastInput::new(Vec3::new(-3.0, 0.0, 0.0), Vec3::new(8.0, 0.0, 0.0))?,
        DynamicTreeFilter::default(),
        |hit| {
            ray_hits.push(hit.user_data);
            DynamicTreeCastControl::Continue
        },
    )?;
    println!(
        "ray hit user data: {ray_hits:?}; visits: nodes={}, leaves={}",
        ray_stats.node_visits, ray_stats.leaf_visits
    );

    tree.move_proxy(sensor_proxy, aabb([0.75, -0.25, -0.25], [1.25, 0.25, 0.25]))?;
    tree.rebuild(false)?;
    println!("after move/rebuild root bounds = {:?}", tree.root_bounds()?);

    Ok(())
}
