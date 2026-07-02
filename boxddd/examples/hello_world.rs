use boxddd::{BodyDef, BodyType, BoxHull, ShapeDef, Vec3, World, WorldDef};

fn main() -> boxddd::Result<()> {
    let mut world = World::new(
        WorldDef::builder()
            .gravity(Vec3::new(0.0, -10.0, 0.0))
            .build(),
    )?;

    let ground = world.create_body(
        BodyDef::builder()
            .position(Vec3::new(0.0, -10.0, 0.0))
            .build(),
    );
    let ground_box = BoxHull::new(50.0, 10.0, 50.0);
    world.create_hull_shape(ground, &ShapeDef::default(), &ground_box);

    let body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position(Vec3::new(0.0, 4.0, 0.0))
            .build(),
    );
    let cube = BoxHull::cube(1.0);
    let shape_def = ShapeDef::builder().density(1.0).friction(0.3).build();
    world.create_hull_shape(body, &shape_def, &cube);

    for step in 0..90 {
        world.step(1.0 / 60.0, 4);
        if step % 15 == 0 {
            let p = world.body_position(body);
            println!("{step:02}: {:.2} {:.2} {:.2}", p.x, p.y, p.z);
        }
    }

    Ok(())
}
