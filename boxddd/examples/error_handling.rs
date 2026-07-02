mod common;

use anyhow::{Context, Result};
use boxddd::prelude::*;

fn main() -> Result<()> {
    let mut scene = common::falling_stack_scene().context("failed to create the demo scene")?;

    let bad_shape = ShapeDef::builder().density(f32::NAN).build();
    let first_body = scene
        .tracked_bodies
        .first()
        .context("the demo scene should create at least one dynamic body")?
        .id;
    match scene
        .world
        .try_create_sphere_shape(first_body, &bad_shape, &Sphere::new(Vec3::ZERO, 0.5))
    {
        Err(Error::InvalidArgument) => {
            println!("invalid shape input was rejected before entering Box3D");
        }
        Err(error) => return Err(error).context("unexpected safe API error"),
        Ok(_) => anyhow::bail!("invalid shape input unexpectedly created a native shape"),
    }

    for _ in 0..90 {
        scene
            .step(1.0 / 60.0, 4)
            .context("world step should remain recoverable")?;
    }

    for snapshot in scene.snapshots()? {
        println!(
            "{:<6} position = ({:>6.2}, {:>6.2}, {:>6.2})",
            snapshot.label, snapshot.position.x, snapshot.position.y, snapshot.position.z
        );
    }

    Ok(())
}
