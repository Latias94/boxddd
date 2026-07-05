//! Debug draw collection and optional Bevy gizmo rendering.

use crate::errors::report_error;
#[cfg(feature = "debug-gizmos")]
use crate::math::{to_bevy_pos, to_bevy_quat, to_bevy_transform, to_bevy_vec3};
use crate::messages::{BoxdddErrorMessage, BoxdddOperation};
use crate::resources::{BoxdddPhysicsContext, BoxdddPhysicsSettings};
use bevy_ecs::message::MessageWriter;
use bevy_ecs::prelude::{NonSendMut, Res, ResMut, Resource};
use std::collections::HashMap;

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

/// Last collected Box3D debug draw frame and persistent shape asset cache.
#[derive(Resource, Clone, Debug, Default)]
pub struct BoxdddDebugDrawFrame {
    frame: boxddd::DebugDrawFrame,
    assets: HashMap<boxddd::DebugShapeHandle, boxddd::DebugShapeAsset>,
}

impl BoxdddDebugDrawFrame {
    /// Returns the complete frame collected during the most recent debug draw pass.
    pub fn frame(&self) -> &boxddd::DebugDrawFrame {
        &self.frame
    }

    /// Returns debug shape lifecycle events collected during the most recent pass.
    pub fn events(&self) -> &[boxddd::DebugShapeEvent] {
        &self.frame.events
    }

    /// Returns the commands collected during the most recent debug draw pass.
    pub fn commands(&self) -> &[boxddd::DebugDrawCommand] {
        &self.frame.commands
    }

    /// Returns non-fatal diagnostics collected while building the frame.
    pub fn diagnostics(&self) -> &[boxddd::DebugDrawDiagnostic] {
        &self.frame.diagnostics
    }

    /// Returns persistent debug shape assets keyed by handle.
    pub fn assets(&self) -> &HashMap<boxddd::DebugShapeHandle, boxddd::DebugShapeAsset> {
        &self.assets
    }

    /// Returns one cached debug shape asset.
    pub fn asset(&self, handle: boxddd::DebugShapeHandle) -> Option<&boxddd::DebugShapeAsset> {
        self.assets.get(&handle)
    }

    /// Clears the most recent frame while preserving cached debug shape assets.
    pub fn clear(&mut self) {
        self.frame.clear();
    }

    /// Clears cached debug shape assets without touching the most recent frame.
    ///
    /// This is useful when a renderer rebuilds all of its GPU resources.
    pub fn clear_cached_assets(&mut self) {
        self.assets.clear();
    }

    /// Clears both the current frame and all cached debug shape assets.
    pub fn clear_all(&mut self) {
        self.clear();
        self.clear_cached_assets();
    }

    fn apply_events(&mut self) {
        for event in &self.frame.events {
            match event {
                boxddd::DebugShapeEvent::Created(asset) => {
                    self.assets.insert(asset.handle, asset.clone());
                }
                boxddd::DebugShapeEvent::Destroyed { handle } => {
                    self.assets.remove(handle);
                }
                boxddd::DebugShapeEvent::ClearAll => {
                    self.assets.clear();
                }
            }
        }
    }

    fn record_missing_asset_diagnostics(&mut self) {
        let missing = self
            .frame
            .commands
            .iter()
            .filter_map(|command| match command {
                boxddd::DebugDrawCommand::Shape {
                    handle: Some(handle),
                    ..
                } if !self.assets.contains_key(handle) => Some(*handle),
                _ => None,
            })
            .fold(Vec::new(), |mut handles, handle| {
                if !handles.contains(&handle) {
                    handles.push(handle);
                }
                handles
            });

        for handle in missing {
            self.frame.diagnostics.push(boxddd::DebugDrawDiagnostic {
                message: format!(
                    "debug shape asset missing for handle {}:{}",
                    handle.index, handle.generation
                ),
            });
        }
    }
}

/// Backwards-compatible name for the Bevy debug draw frame resource.
pub type BoxdddDebugDrawCommands = BoxdddDebugDrawFrame;

/// Collects native Box3D debug draw data into [`BoxdddDebugDrawFrame`].
pub fn collect_debug_draw_commands(
    mut context: NonSendMut<BoxdddPhysicsContext>,
    settings: Res<BoxdddPhysicsSettings>,
    debug_settings: Res<BoxdddDebugDrawSettings>,
    mut debug_frame: ResMut<BoxdddDebugDrawFrame>,
    mut errors: MessageWriter<BoxdddErrorMessage>,
) {
    if !debug_settings.enabled {
        debug_frame.clear();
        return;
    }

    let Some(world) = context.world_mut() else {
        debug_frame.clear_all();
        return;
    };

    let result = world.try_debug_draw_frame_into(&mut debug_frame.frame, debug_settings.options);

    if let Err(error) = result {
        debug_frame.clear();
        report_error(
            &settings,
            &mut errors,
            BoxdddErrorMessage {
                operation: BoxdddOperation::DebugDraw,
                entity: None,
                error,
            },
        );
    } else {
        debug_frame.apply_events();
        debug_frame.record_missing_asset_diagnostics();
    }
}

#[cfg(feature = "debug-gizmos")]
/// Renders collected Box3D debug draw commands using Bevy `Gizmos`.
pub fn draw_debug_gizmos(
    debug_frame: Res<BoxdddDebugDrawFrame>,
    mut gizmos: bevy_gizmos::prelude::Gizmos,
) {
    for command in debug_frame.commands() {
        draw_debug_command(&mut gizmos, &debug_frame, command);
    }
}

#[cfg(feature = "debug-gizmos")]
fn draw_debug_command(
    gizmos: &mut bevy_gizmos::prelude::Gizmos,
    debug_frame: &BoxdddDebugDrawFrame,
    command: &boxddd::DebugDrawCommand,
) {
    match command {
        boxddd::DebugDrawCommand::Shape {
            handle: Some(handle),
            transform,
            color,
        } => {
            if let Some(asset) = debug_frame.asset(*handle) {
                draw_debug_shape_geometry(
                    gizmos,
                    &asset.geometry,
                    to_bevy_pos(transform.p),
                    to_bevy_quat(transform.q),
                    to_bevy_color(*color),
                );
            }
        }
        boxddd::DebugDrawCommand::Shape { handle: None, .. } => {}
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
fn draw_debug_shape_geometry(
    gizmos: &mut bevy_gizmos::prelude::Gizmos,
    geometry: &boxddd::DebugShapeGeometry,
    origin: bevy_math::Vec3,
    rotation: bevy_math::Quat,
    color: bevy_color::Color,
) {
    match geometry {
        boxddd::DebugShapeGeometry::Sphere { center, radius } => {
            gizmos.sphere(
                transform_local_point(origin, rotation, *center),
                *radius,
                color,
            );
        }
        boxddd::DebugShapeGeometry::Capsule {
            center1,
            center2,
            radius,
        } => {
            let p1 = transform_local_point(origin, rotation, *center1);
            let p2 = transform_local_point(origin, rotation, *center2);
            gizmos.line(p1, p2, color);
            gizmos.sphere(p1, *radius, color);
            gizmos.sphere(p2, *radius, color);
        }
        boxddd::DebugShapeGeometry::Hull { points, faces, .. } => {
            for face in faces {
                draw_indexed_polyline(gizmos, points, &face.indices, origin, rotation, color);
            }
        }
        boxddd::DebugShapeGeometry::Mesh { mesh, scale } => {
            draw_debug_mesh(gizmos, mesh, origin, rotation, to_bevy_vec3(*scale), color);
        }
        boxddd::DebugShapeGeometry::HeightField { mesh } => {
            draw_debug_mesh(gizmos, mesh, origin, rotation, bevy_math::Vec3::ONE, color);
        }
        boxddd::DebugShapeGeometry::Compound { children } => {
            for child in children {
                let child_origin = transform_local_point(origin, rotation, child.transform.p);
                let child_rotation = rotation * to_bevy_quat(child.transform.q);
                draw_debug_shape_geometry(
                    gizmos,
                    &child.geometry,
                    child_origin,
                    child_rotation,
                    color,
                );
            }
        }
    }
}

#[cfg(feature = "debug-gizmos")]
fn draw_debug_mesh(
    gizmos: &mut bevy_gizmos::prelude::Gizmos,
    mesh: &boxddd::DebugMesh,
    origin: bevy_math::Vec3,
    rotation: bevy_math::Quat,
    scale: bevy_math::Vec3,
    color: bevy_color::Color,
) {
    for triangle in &mesh.triangles {
        let Some(a) = mesh.vertices.get(triangle.indices[0] as usize) else {
            continue;
        };
        let Some(b) = mesh.vertices.get(triangle.indices[1] as usize) else {
            continue;
        };
        let Some(c) = mesh.vertices.get(triangle.indices[2] as usize) else {
            continue;
        };
        let a = transform_local_scaled_point(origin, rotation, *a, scale);
        let b = transform_local_scaled_point(origin, rotation, *b, scale);
        let c = transform_local_scaled_point(origin, rotation, *c, scale);
        gizmos.line(a, b, color);
        gizmos.line(b, c, color);
        gizmos.line(c, a, color);
    }
}

#[cfg(feature = "debug-gizmos")]
fn draw_indexed_polyline(
    gizmos: &mut bevy_gizmos::prelude::Gizmos,
    points: &[boxddd::Vec3],
    indices: &[u32],
    origin: bevy_math::Vec3,
    rotation: bevy_math::Quat,
    color: bevy_color::Color,
) {
    let mut vertices = indices
        .iter()
        .filter_map(|index| points.get(*index as usize))
        .copied()
        .map(|point| transform_local_point(origin, rotation, point));
    let Some(first) = vertices.next() else {
        return;
    };
    let mut previous = first;
    for current in vertices {
        gizmos.line(previous, current, color);
        previous = current;
    }
    gizmos.line(previous, first, color);
}

#[cfg(feature = "debug-gizmos")]
fn transform_local_point(
    origin: bevy_math::Vec3,
    rotation: bevy_math::Quat,
    point: boxddd::Vec3,
) -> bevy_math::Vec3 {
    origin + rotation * to_bevy_vec3(point)
}

#[cfg(feature = "debug-gizmos")]
fn transform_local_scaled_point(
    origin: bevy_math::Vec3,
    rotation: bevy_math::Quat,
    point: boxddd::Vec3,
    scale: bevy_math::Vec3,
) -> bevy_math::Vec3 {
    origin + rotation * (to_bevy_vec3(point) * scale)
}

#[cfg(feature = "debug-gizmos")]
fn to_bevy_color(color: boxddd::HexColor) -> bevy_color::Color {
    let rgb = color.rgb_u32();
    let red = ((rgb >> 16) & 0xff) as f32 / 255.0;
    let green = ((rgb >> 8) & 0xff) as f32 / 255.0;
    let blue = (rgb & 0xff) as f32 / 255.0;
    bevy_color::Color::srgb(red, green, blue)
}
