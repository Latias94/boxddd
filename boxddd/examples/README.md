# boxddd Examples

Run examples from the workspace root with `cargo run -p boxddd --example <name>`.

## Basics

- `hello_world.rs`: minimal world, static ground hull, dynamic box hull, stepping, and body position reads.

## Joints

- `joints.rs`: creates a distance joint and reads runtime joint state after simulation.

## Recording And Replay

- `recording_replay.rs`: records a short simulation, creates a replay player, steps it to the end, and reports divergence status.
- `determinism.rs`: records a simulation and validates replay determinism explicitly with worker count `1`.

## Interop

- `mint_interop.rs`: uses `mint::Vector3`, `mint::Point3`, and `mint::Quaternion` conversions in a normal world setup.

  ```bash
  cargo run -p boxddd --example mint_interop --features mint
  ```

## Suggested Verification Sweep

```bash
cargo check -p boxddd --examples
cargo run -p boxddd --example hello_world
cargo run -p boxddd --example joints
cargo run -p boxddd --example recording_replay
cargo run -p boxddd --example determinism
cargo run -p boxddd --example mint_interop --features mint
```
