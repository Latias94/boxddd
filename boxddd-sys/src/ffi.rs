#![allow(clippy::approx_constant)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unreadable_literal)]
#![allow(rustdoc::bare_urls)]
#![allow(rustdoc::broken_intra_doc_links)]

#[cfg(all(feature = "double-precision", has_pregenerated))]
include!("bindings_pregenerated_double.rs");

#[cfg(all(not(feature = "double-precision"), has_pregenerated))]
include!("bindings_pregenerated.rs");

#[cfg(not(has_pregenerated))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
