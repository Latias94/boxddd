//! Bevy integration for the `boxddd` Box3D bindings.
//!
//! The core `boxddd` crate stays engine-agnostic. This crate owns the Bevy-specific
//! plugin, components, resources, systems, and teaching examples.

pub mod components;
pub mod debug_draw;
pub mod errors;
pub mod messages;
pub mod plugin;
pub mod prelude;
pub mod query;
pub mod resources;
pub mod systems;

pub use boxddd;
pub use components::*;
pub use debug_draw::*;
pub use messages::*;
pub use plugin::BoxdddPhysicsPlugin;
pub use query::*;
pub use resources::*;
