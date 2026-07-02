# bevy_boxddd

`bevy_boxddd` integrates [`boxddd`](https://github.com/Latias94/boxddd) with Bevy.

This crate is intentionally separate from the engine-agnostic `boxddd` crate, so Bevy users get an idiomatic plugin without forcing Bevy dependencies onto non-Bevy users.

## Quickstart

```rust
use bevy::prelude::*;
use bevy_boxddd::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BoxdddPhysicsPlugin::default())
        .run();
}
```

See the examples in `examples/` for visible 3D scenes and message handling.
