# WASM Callback Bridge Design

`boxddd` browser provider mode currently supports Box3D calls where the Rust
wasm module imports C symbols from an Emscripten-built Box3D provider module and
both modules share one `WebAssembly.Memory`. Debug draw collection was the first
callback-heavy API bridged across that boundary; world AABB overlap and ray-cast
visitors now use the same provider-dispatch model.

Callback-heavy APIs need a stricter bridge. A Rust function pointer or closure
token from the app module cannot be treated as a callable function pointer inside
the provider module unless both sides explicitly agree on the table, trampoline,
ownership, and panic policy.

## Callback Surfaces

The affected Box3D surfaces are:

- debug draw callbacks, implemented through the data-frame bridge;
- world query visitors with early-stop return values, with AABB overlap and
  ray-cast visitors implemented first;
- dynamic-tree query, ray-cast, box-cast, and closest visitors;
- standalone geometry visitors for mesh triangle queries, height-field triangle
  queries, and compound child AABB queries;
- world callbacks such as custom filtering, pre-solve, friction, and restitution
  mix callbacks;
- recording replay debug-shape callbacks;
- task-system callbacks.

Provider mode returns `Error::UnsupportedOnWasm` for the unimplemented surfaces
until each bridge is implemented and tested.

## Recommended Bridge

Supported bridges use JavaScript dispatch and provider-local C trampolines, not
raw cross-module function pointers. Debug draw follows this model; future
bridges should reuse the same constraints unless their callback semantics require
a stricter design.

1. Rust safe APIs allocate a callback token in a Rust-side registry. The token is
   scoped to the current call or to a RAII registration object.
2. The Box3D provider receives stable C trampoline function pointers compiled
   into the provider module.
3. The C trampoline forwards `kind`, `token`, and plain pointer/value arguments
   to an imported JavaScript dispatcher.
4. JavaScript calls exported Rust dispatcher functions on the app module.
5. Rust looks up the token, copies transient values into safe Rust values, invokes
   the closure or trait object, catches panics where possible, writes return data
   into shared memory when needed, and returns a primitive result to JavaScript.
6. JavaScript returns the primitive result to the C trampoline.

This mirrors the existing provider shape while making the callback boundary
explicit and auditable.

## Safety Contract

The bridge must preserve the native safe-wrapper guarantees:

- Rust panics do not unwind into C or JavaScript callback frames.
- Callback tokens are released on all normal and error paths.
- Callback borrowed data is copied before user code sees it.
- Reentrant safe `World` calls continue to return `Error::InCallback`.
- Visitor callbacks can short-circuit traversal with the same semantics as native
  APIs.
- A callback token never outlives the `World`, query call, or registration object
  that owns it.
- `World` remains non-send in the browser provider path.
- Unsupported worker, pthread, Atomics, and blocking task-system modes fail with
  typed errors instead of silently changing scheduler semantics.

## Implementation Order

1. Debug draw collection: implemented through provider-local trampolines and
   exported Rust frame dispatchers.
2. World query visitors: AABB overlap and ray-cast visitors are implemented;
   shape overlap, shape cast, and mover collision visitors remain future bridge
   work.
3. Dynamic tree visitors: standalone owner with the same token lifetime rules.
4. Standalone geometry visitors: mesh, height-field, and compound query
   callbacks that are not tied to a `World`.
5. Contact, filter, and material callbacks: registration lifetime plus
   simulation-step reentrancy.
6. Recording replay debug-shape callbacks.
7. Task-system callbacks: last, because Box3D requires `finishTask` to block
   until work completes. Browser workers need a separate policy for pthreads,
   `SharedArrayBuffer`, COOP/COEP headers, and Atomics.

## Testing Requirements

Each bridged surface needs:

- a Node provider smoke that exercises the callback through shared memory;
- a browser smoke page or Playwright check;
- native parity tests for return values and early-stop behavior;
- panic containment tests;
- token drop tests for success, error, and callback panic paths;
- provider-mode tests asserting remaining unsupported surfaces keep returning
  `Error::UnsupportedOnWasm` until their bridge is implemented.

The Pages examples should only expose callback-heavy tools after the matching
bridge is implemented. Debug draw is now exposed through the Bevy Web examples;
other callback-heavy tools should prefer app-authored visualization and
non-callback Box3D calls until their bridge exists.
