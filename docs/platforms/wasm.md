# WASM Support

`wasm32-unknown-unknown` is a compile-only platform for the current workspace.
The Rust crates can be type-checked for a narrow subset, but `boxddd-sys` does
not yet compile or link the vendored Box3D C sources for WASM.

## Support Matrix

| Surface | `wasm32-unknown-unknown` tier | Contract |
|---|---|---|
| `boxddd-sys` | compile-only | `cargo check` succeeds with pregenerated bindings, but build.rs skips Box3D C compilation for `wasm32`. Do not treat the produced artifact as runnable. |
| `boxddd` | compile-only | Safe Rust APIs type-check on top of the compile-only sys crate. Runtime world creation is not a supported browser contract yet. |
| `bevy_boxddd` minimal library | compile-only | `cargo check -p bevy_boxddd --target wasm32-unknown-unknown --no-default-features` succeeds without Bevy rendering dependencies. |
| `bevy_boxddd` `debug-gizmos` feature | native-only | The bridge is meant for native Bevy gizmos in this release. Browser rendering setup is not part of the supported contract. |
| `bevy_boxddd` `physics-picking` feature | native-only | The helper APIs are platform-neutral, but the windowed picking example is native-only in this release. |
| Windowed examples and testbed | native-only | Examples use Bevy `DefaultPlugins`, native windowing, and native rendering assumptions. They are CI compile-checked only for native targets. |

## CI Gate

The supported compile-only subset is:

```bash
cargo check -p boxddd-sys --target wasm32-unknown-unknown
cargo check -p boxddd --target wasm32-unknown-unknown
cargo check -p bevy_boxddd --target wasm32-unknown-unknown --no-default-features
```

The expected `boxddd-sys` build output includes:

```text
boxddd-sys does not build Box3D C sources for wasm32 yet
```

That warning is part of the current platform contract. A future browser-ready
contract needs a real Box3D C/WASM build path, browser scheduler review for
Box3D task callbacks, and separate Bevy web example validation.
