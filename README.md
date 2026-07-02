# boxddd

Safe, ergonomic Rust bindings for Box3D.

This project intentionally starts as a sibling of `boxdd` instead of adding a 3D feature flag to the 2D crate. Box3D has a similar C API style, but the Rust-facing model is genuinely 3D: vectors, quaternions, hulls, meshes, height fields, compounds, and recording all deserve their own safe interface.

## Status

Experimental. The first slice wraps enough of Box3D to create a world, create bodies, attach sphere and box-hull shapes, step the simulation, and read body transforms.

## Quickstart

```rust
use boxddd::{BodyDef, BodyType, BoxHull, ShapeDef, Vec3, World, WorldDef};

let mut world = World::new(WorldDef::default()).unwrap();

let ground = world.create_body(BodyDef::builder().position([0.0, -10.0, 0.0]).build());
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

world.step(1.0 / 60.0, 4);
```
