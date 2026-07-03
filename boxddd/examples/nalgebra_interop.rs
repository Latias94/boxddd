use boxddd::prelude::*;

fn main() -> boxddd::Result<()> {
    let mut world = World::new(
        WorldDef::builder()
            .gravity(nalgebra::Vector3::new(0.0, -9.8, 0.0))
            .build(),
    )?;

    let start_position = nalgebra::Point3::<boxddd::types::PosScalar>::new(0.0, 2.0, 0.0);
    let body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position(start_position)
            .build(),
    );
    world.create_sphere_shape(
        body,
        &ShapeDef::builder().density(1.0).build(),
        &Sphere::new(nalgebra::Vector3::new(0.0, 0.0, 0.0), 0.25),
    );

    world.try_step(1.0 / 60.0, 4)?;

    let body_position: nalgebra::Point3<boxddd::types::PosScalar> =
        world.body_position(body).into();
    let rotation: nalgebra::UnitQuaternion<f32> = world.body_rotation(body).into();
    #[cfg(not(feature = "double-precision"))]
    let transform: nalgebra::Isometry3<f32> = world.body_transform(body).into();
    #[cfg(not(feature = "double-precision"))]
    let transform_x = transform.translation.vector.x;
    #[cfg(feature = "double-precision")]
    let transform_x = body_position.x;

    println!(
        "nalgebra_interop: y={:.3}, angle={:.3}, tx={:.3}",
        body_position.y,
        rotation.angle(),
        transform_x
    );
    Ok(())
}
