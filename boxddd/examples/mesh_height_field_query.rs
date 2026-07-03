use boxddd::prelude::*;

fn main() -> boxddd::Result<()> {
    let mesh = MeshData::box_mesh(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0), true)?;
    let bounds = Aabb {
        lower_bound: [-1.25, -1.25, -1.25].into(),
        upper_bound: [1.25, 1.25, 1.25].into(),
    };

    let mesh_hits = mesh.query_triangles(bounds, Vec3::new(1.0, 1.0, 1.0))?;
    println!(
        "mesh_height_field_query: mesh returned {} triangle hit(s)",
        mesh_hits.len()
    );
    if let Some(hit) = mesh_hits.first() {
        println!(
            "  first mesh triangle {}: a={:?}, b={:?}, c={:?}",
            hit.triangle_index, hit.a, hit.b, hit.c
        );
    }

    let mut first_mesh_triangle = None;
    mesh.visit_triangles(bounds, Vec3::new(1.0, 1.0, 1.0), |hit| {
        first_mesh_triangle = Some(hit.triangle_index);
        false
    })?;
    println!("mesh visitor stopped at triangle {first_mesh_triangle:?}");

    let height_field = HeightField::grid(4, 4, Vec3::new(1.0, 0.5, 1.0), false)?;
    let height_bounds = Aabb {
        lower_bound: [-0.25, -1.0, -0.25].into(),
        upper_bound: [3.25, 1.0, 3.25].into(),
    };
    let height_hits = height_field.query_triangles(height_bounds)?;
    println!(
        "height field returned {} triangle hit(s)",
        height_hits.len()
    );

    let mut height_visit_count = 0;
    height_field.visit_triangles(height_bounds, |_| {
        height_visit_count += 1;
    })?;
    println!("height field visitor saw {height_visit_count} triangle hit(s)");

    Ok(())
}
