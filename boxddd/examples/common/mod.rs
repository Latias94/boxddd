use boxddd::prelude::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VisualShape {
    Cube,
    Sphere,
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub struct TrackedBody {
    pub id: BodyId,
    pub label: &'static str,
    pub shape: VisualShape,
    pub radius: f32,
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub struct BodySnapshot {
    pub label: &'static str,
    pub shape: VisualShape,
    pub radius: f32,
    pub position: Pos,
}

pub struct DemoScene {
    pub world: World,
    pub tracked_bodies: Vec<TrackedBody>,
}

impl DemoScene {
    pub fn step(&mut self, time_step: f32, sub_step_count: i32) -> boxddd::Result<()> {
        self.world.try_step(time_step, sub_step_count)
    }

    #[allow(dead_code)]
    pub fn snapshots(&self) -> boxddd::Result<Vec<BodySnapshot>> {
        self.tracked_bodies
            .iter()
            .map(|body| {
                Ok(BodySnapshot {
                    label: body.label,
                    shape: body.shape,
                    radius: body.radius,
                    position: self.world.try_body_position(body.id)?,
                })
            })
            .collect()
    }
}

pub fn falling_stack_scene() -> boxddd::Result<DemoScene> {
    let mut world = World::new(WorldDef::builder().gravity([0.0, -10.0, 0.0]).build())?;

    let ground = world.try_create_body(
        BodyDef::builder()
            .body_type(BodyType::Static)
            .position([0.0, -0.25, 0.0])
            .name("ground")
            .build(),
    )?;
    world.try_create_hull_shape(
        ground,
        &ShapeDef::builder().friction(0.8).build(),
        &BoxHull::new(7.5, 0.25, 7.5),
    )?;

    let shape_def = ShapeDef::builder()
        .density(1.0)
        .friction(0.55)
        .restitution(0.05)
        .build();
    let mut tracked_bodies = Vec::new();

    for index in 0..5 {
        let x = (index as f32 - 2.0) * 0.85;
        let body = world.try_create_body(
            BodyDef::builder()
                .body_type(BodyType::Dynamic)
                .position([x, 2.0 + index as f32 * 0.85, 0.0])
                .name(format!("box-{index}"))
                .build(),
        )?;
        world.try_create_hull_shape(body, &shape_def, &BoxHull::cube(0.35))?;
        tracked_bodies.push(TrackedBody {
            id: body,
            label: "box",
            shape: VisualShape::Cube,
            radius: 0.35,
        });
    }

    for index in 0..3 {
        let body = world.try_create_body(
            BodyDef::builder()
                .body_type(BodyType::Dynamic)
                .position([-1.0 + index as f32, 5.2 + index as f32 * 0.45, 0.6])
                .name(format!("sphere-{index}"))
                .build(),
        )?;
        world.try_create_sphere_shape(body, &shape_def, &Sphere::new(Vec3::ZERO, 0.32))?;
        tracked_bodies.push(TrackedBody {
            id: body,
            label: "sphere",
            shape: VisualShape::Sphere,
            radius: 0.32,
        });
    }

    Ok(DemoScene {
        world,
        tracked_bodies,
    })
}
