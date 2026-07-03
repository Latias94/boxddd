use boxddd::prelude::*;

fn main() -> boxddd::Result<()> {
    let box_hull = BoxHull::cube(0.5);
    let left_material = SurfaceMaterial {
        user_material_id: 10,
        ..Default::default()
    };
    let right_material = SurfaceMaterial {
        user_material_id: 20,
        ..Default::default()
    };

    let mut builder = Compound::builder();
    builder.add_sphere(Sphere::new([-1.0, 0.0, 0.0], 0.45), left_material)?;
    builder.add_box_hull(
        &box_hull,
        Transform::new(Vec3::new(1.0, 0.0, 0.0), Quat::IDENTITY),
        right_material,
    )?;
    let compound = builder.build()?;

    let bounds = Aabb {
        lower_bound: [-1.5, -0.75, -0.75].into(),
        upper_bound: [1.5, 0.75, 0.75].into(),
    };

    let hits = compound.query_aabb(bounds)?;
    println!("compound_query: {} child hit(s)", hits.len());
    for hit in &hits {
        println!(
            "  child {}: {:?}, primary material {}",
            hit.child_index,
            hit.child.shape_type(),
            hit.child.primary_material_index()
        );
    }

    let mut reusable_hits = Vec::with_capacity(4);
    compound.query_aabb_into(bounds, &mut reusable_hits)?;
    println!("reusable buffer collected {} hit(s)", reusable_hits.len());
    drop(hits);
    drop(reusable_hits);

    let mut first_hit = None;
    compound.visit_query_aabb(bounds, |hit| {
        first_hit = Some((hit.child_index, hit.child.shape_type()));
        false
    })?;
    println!("first visitor hit: {first_hit:?}");

    let bytes = compound.into_bytes();
    println!("compound byte owner holds {} byte(s)", bytes.byte_count());
    let restored = bytes.into_compound()?;
    println!(
        "restored compound has {} child shape(s)",
        restored.child_count()
    );

    Ok(())
}
