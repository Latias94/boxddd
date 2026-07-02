//! Bevy integration for the `boxddd` Box3D bindings.
//!
//! The core `boxddd` crate stays engine-agnostic. This crate owns the Bevy-specific
//! plugin, components, resources, systems, and teaching examples.

pub mod components;
pub mod errors;
pub mod messages;
pub mod plugin;
pub mod prelude;
pub mod resources;
pub mod systems;

pub use boxddd;
pub use components::*;
pub use messages::*;
pub use plugin::BoxdddPhysicsPlugin;
pub use resources::*;
