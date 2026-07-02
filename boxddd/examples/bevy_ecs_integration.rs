mod common;

use anyhow::{Context, Result};
use bevy_app::{App, Startup, Update};
use bevy_ecs::prelude::*;
use bevy_transform::components::{GlobalTransform, Transform};
use boxddd::prelude::*;

#[derive(Component)]
struct PhysicsBody(BodyId);

struct PhysicsState {
    scene: common::DemoScene,
}

fn main() -> Result<()> {
    let mut app = App::new();
    app.insert_non_send(PhysicsState {
        scene: common::falling_stack_scene()?,
    });
    app.add_systems(Startup, spawn_entities);
    app.add_systems(Update, step_physics);

    for _ in 0..120 {
        app.update();
    }

    let mut query = app.world_mut().query::<(&PhysicsBody, &Transform)>();
    for (_, transform) in query.iter(app.world()) {
        println!("bevy transform translation: {:?}", transform.translation);
    }

    Ok(())
}

fn spawn_entities(mut commands: Commands, physics: NonSend<PhysicsState>) {
    for body in &physics.scene.tracked_bodies {
        commands.spawn((
            PhysicsBody(body.id),
            Transform::default(),
            GlobalTransform::default(),
        ));
    }
}

fn step_physics(
    mut physics: NonSendMut<PhysicsState>,
    mut bodies: Query<(&PhysicsBody, &mut Transform)>,
) {
    physics
        .scene
        .step(1.0 / 60.0, 4)
        .context("Box3D step failed inside Bevy system")
        .unwrap();

    for (body, mut transform) in &mut bodies {
        let position = physics.scene.world.body_position(body.0);
        transform.translation.x = position.x as f32;
        transform.translation.y = position.y as f32;
        transform.translation.z = position.z as f32;
    }
}
