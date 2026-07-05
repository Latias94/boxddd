#![allow(clippy::approx_constant)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unreadable_literal)]
#![allow(rustdoc::bare_urls)]
#![allow(rustdoc::broken_intra_doc_links)]

#[cfg(boxddd_sys_wasm_provider)]
include!(concat!(env!("OUT_DIR"), "/wasm_provider_bindings.rs"));

#[cfg(boxddd_sys_wasm_provider)]
#[link(wasm_import_module = "box3d-sys-v0")]
unsafe extern "C" {
    pub fn boxddd_provider_debug_install_world_def(def: *mut b3WorldDef, token: u32);
    pub fn boxddd_provider_debug_init_draw(draw: *mut b3DebugDraw, token: u32);
    pub fn boxddd_provider_debug_take_error(token: u32) -> i32;
}

#[cfg(all(
    not(boxddd_sys_wasm_provider),
    feature = "double-precision",
    has_pregenerated,
    not(force_bindgen)
))]
include!("bindings_pregenerated_double.rs");

#[cfg(all(
    not(boxddd_sys_wasm_provider),
    not(feature = "double-precision"),
    has_pregenerated,
    not(force_bindgen)
))]
include!("bindings_pregenerated.rs");

#[cfg(all(
    not(boxddd_sys_wasm_provider),
    any(force_bindgen, not(has_pregenerated))
))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
