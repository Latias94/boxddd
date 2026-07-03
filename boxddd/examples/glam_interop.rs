use boxddd::prelude::*;

fn main() -> boxddd::Result<()> {
    #[cfg(not(feature = "double-precision"))]
    let start_position = glam::Vec3::new(0.0, 2.0, 0.0);
    #[cfg(feature = "double-precision")]
    let start_position = glam::DVec3::new(0.0, 2.0, 0.0);

    let mut world = World::new(
        WorldDef::builder()
            .gravity(glam::Vec3::new(0.0, -9.8, 0.0))
            .build(),
    )?;

    let body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position(start_position)
            .build(),
    );
    world.create_sphere_shape(
        body,
        &ShapeDef::builder().density(1.0).build(),
        &Sphere::new(glam::Vec3::ZERO, 0.25),
    );

    world.try_step(1.0 / 60.0, 4)?;

    #[cfg(not(feature = "double-precision"))]
    let body_position: glam::Vec3 = world.body_position(body).into();
    #[cfg(feature = "double-precision")]
    let body_position: glam::DVec3 = world.body_position(body).into();
    let rotation: glam::Quat = world.body_rotation(body).into();
    let velocity: glam::Vec3 = world.body_linear_velocity(body).into();

    println!(
        "glam_interop: y={:.3}, qw={:.3}, vy={:.3}",
        body_position.y, rotation.w, velocity.y
    );
    Ok(())
}
