//! Debug draw collection and optional Bevy gizmo rendering.

use crate::errors::report_error;
use crate::messages::{BoxdddErrorMessage, BoxdddOperation};
use crate::resources::{BoxdddPhysicsContext, BoxdddPhysicsSettings};
use bevy_ecs::message::MessageWriter;
use bevy_ecs::prelude::{NonSendMut, Res, ResMut, Resource};

/// Controls whether Box3D debug draw commands are collected after each step.
#[derive(Resource, Copy, Clone, Debug)]
pub struct BoxdddDebugDrawSettings {
    /// Enables debug draw command collection when true.
    pub enabled: bool,
    /// Box3D debug draw options forwarded to the native world.
    pub options: boxddd::DebugDrawOptions,
}

impl Default for BoxdddDebugDrawSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            options: boxddd::DebugDrawOptions::default(),
        }
    }
}

/// Last collected Box3D debug draw command buffer.
#[derive(Resource, Clone, Debug, Default)]
pub struct BoxdddDebugDrawCommands {
    commands: Vec<boxddd::DebugDrawCommand>,
}

impl BoxdddDebugDrawCommands {
    /// Returns the commands collected during the most recent debug draw pass.
    pub fn commands(&self) -> &[boxddd::DebugDrawCommand] {
        &self.commands
    }

    /// Clears the stored debug draw commands.
    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

/// Collects native Box3D debug draw commands into [`BoxdddDebugDrawCommands`].
pub fn collect_debug_draw_commands(
    mut context: NonSendMut<BoxdddPhysicsContext>,
    settings: Res<BoxdddPhysicsSettings>,
    debug_settings: Res<BoxdddDebugDrawSettings>,
    mut debug_commands: ResMut<BoxdddDebugDrawCommands>,
    mut errors: MessageWriter<BoxdddErrorMessage>,
) {
    if !debug_settings.enabled || context.world().is_none() {
        debug_commands.clear();
        return;
    }

    let result = context
        .world_mut()
        .expect("checked above")
        .try_debug_draw_collect_into(&mut debug_commands.commands, debug_settings.options);

    if let Err(error) = result {
        debug_commands.clear();
        report_error(
            &settings,
            &mut errors,
            BoxdddErrorMessage {
                operation: BoxdddOperation::DebugDraw,
                entity: None,
                error,
            },
        );
    }
}

#[cfg(feature = "debug-gizmos")]
/// Renders collected Box3D debug draw commands using Bevy `Gizmos`.
pub fn draw_debug_gizmos(
    debug_commands: Res<BoxdddDebugDrawCommands>,
    mut gizmos: bevy_gizmos::prelude::Gizmos,
) {
    for command in debug_commands.commands() {
        draw_debug_command(&mut gizmos, command);
    }
}

#[cfg(feature = "debug-gizmos")]
fn draw_debug_command(
    gizmos: &mut bevy_gizmos::prelude::Gizmos,
    command: &boxddd::DebugDrawCommand,
) {
    match command {
        boxddd::DebugDrawCommand::Shape {
            transform, color, ..
        } => {
            gizmos.axes(to_bevy_transform(*transform), 0.25);
            gizmos.sphere(to_bevy_pos(transform.p), 0.04, to_bevy_color(*color));
        }
        boxddd::DebugDrawCommand::Segment { p1, p2, color } => {
            gizmos.line(to_bevy_pos(*p1), to_bevy_pos(*p2), to_bevy_color(*color));
        }
        boxddd::DebugDrawCommand::Transform(transform) => {
            gizmos.axes(to_bevy_transform(*transform), 0.45);
        }
        boxddd::DebugDrawCommand::Point {
            position,
            size,
            color,
        } => {
            gizmos.sphere(
                to_bevy_pos(*position),
                (*size).max(1.0) * 0.01,
                to_bevy_color(*color),
            );
        }
        boxddd::DebugDrawCommand::Sphere {
            center,
            radius,
            color,
            ..
        } => {
            gizmos.sphere(to_bevy_pos(*center), *radius, to_bevy_color(*color));
        }
        boxddd::DebugDrawCommand::Capsule {
            p1,
            p2,
            radius,
            color,
            ..
        } => {
            let p1 = to_bevy_pos(*p1);
            let p2 = to_bevy_pos(*p2);
            let color = to_bevy_color(*color);
            gizmos.line(p1, p2, color);
            gizmos.sphere(p1, *radius, color);
            gizmos.sphere(p2, *radius, color);
        }
        boxddd::DebugDrawCommand::Bounds { aabb, color } => {
            let lower = to_bevy_vec3(aabb.lower_bound);
            let upper = to_bevy_vec3(aabb.upper_bound);
            let center = (lower + upper) * 0.5;
            let scale = upper - lower;
            gizmos.cube(
                bevy_transform::components::Transform::from_translation(center).with_scale(scale),
                to_bevy_color(*color),
            );
        }
        boxddd::DebugDrawCommand::Box {
            extents,
            transform,
            color,
        } => {
            gizmos.cube(
                to_bevy_transform(*transform).with_scale(to_bevy_vec3(*extents) * 2.0),
                to_bevy_color(*color),
            );
        }
        boxddd::DebugDrawCommand::String {
            position,
            text,
            color,
        } => {
            gizmos.text(
                bevy_math::Isometry3d::from_translation(to_bevy_pos(*position)),
                text,
                14.0,
                bevy_math::Vec2::ZERO,
                to_bevy_color(*color),
            );
        }
    }
}

#[cfg(feature = "debug-gizmos")]
fn to_bevy_color(color: boxddd::HexColor) -> bevy_color::Color {
    let rgb = color.rgb_u32();
    let red = ((rgb >> 16) & 0xff) as f32 / 255.0;
    let green = ((rgb >> 8) & 0xff) as f32 / 255.0;
    let blue = (rgb & 0xff) as f32 / 255.0;
    bevy_color::Color::srgb(red, green, blue)
}

#[cfg(feature = "debug-gizmos")]
fn to_bevy_transform(transform: boxddd::WorldTransform) -> bevy_transform::components::Transform {
    bevy_transform::components::Transform::from_translation(to_bevy_pos(transform.p))
        .with_rotation(to_bevy_quat(transform.q))
}

#[cfg(feature = "debug-gizmos")]
fn to_bevy_pos(value: boxddd::Pos) -> bevy_math::Vec3 {
    bevy_math::Vec3::new(value.x as f32, value.y as f32, value.z as f32)
}

#[cfg(feature = "debug-gizmos")]
fn to_bevy_vec3(value: boxddd::Vec3) -> bevy_math::Vec3 {
    bevy_math::Vec3::new(value.x, value.y, value.z)
}

#[cfg(feature = "debug-gizmos")]
fn to_bevy_quat(value: boxddd::Quat) -> bevy_math::Quat {
    bevy_math::Quat::from_xyzw(value.v.x, value.v.y, value.v.z, value.s)
}
