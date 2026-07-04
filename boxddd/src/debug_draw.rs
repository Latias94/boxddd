#![cfg_attr(all(target_arch = "wasm32", boxddd_wasm_provider), allow(dead_code))]

use crate::core::{box3d_lock, callback_state};
use crate::error::{Error, Result};
use crate::shapes::ShapeType;
use crate::types::{Aabb, Pos, ShapeId, Vec3, WorldTransform};
use crate::world::World;
use boxddd_sys::ffi;
use std::ffi::{CStr, c_void};
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Packed RGB color used by Box3D debug drawing.
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct HexColor(u32);

impl HexColor {
    /// Black.
    pub const BLACK: Self = Self::from_rgb_u32(0x000000);
    /// White.
    pub const WHITE: Self = Self::from_rgb_u32(0xffffff);
    /// Red.
    pub const RED: Self = Self::from_rgb_u32(0xff0000);
    /// Green.
    pub const GREEN: Self = Self::from_rgb_u32(0x00ff00);
    /// Blue.
    pub const BLUE: Self = Self::from_rgb_u32(0x0000ff);

    /// Creates a color from red, green, and blue components.
    #[inline]
    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self(((red as u32) << 16) | ((green as u32) << 8) | blue as u32)
    }

    /// Creates a color from a packed `0xRRGGBB` value.
    #[inline]
    pub const fn from_rgb_u32(rgb: u32) -> Self {
        Self(rgb & 0x00ff_ffff)
    }

    /// Converts a raw Box3D color into `HexColor`.
    #[inline]
    pub const fn from_raw(raw: ffi::b3HexColor) -> Self {
        Self::from_rgb_u32(raw as u32)
    }

    /// Returns this color as a packed `0xRRGGBB` value.
    #[inline]
    pub const fn rgb_u32(self) -> u32 {
        self.0
    }

    /// Converts this color into the raw Box3D color type.
    #[inline]
    pub const fn into_raw(self) -> ffi::b3HexColor {
        self.0 as ffi::b3HexColor
    }
}

/// Opaque debug shape handle managed by Box3D callbacks.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DebugShape {
    /// Shape associated with the result.
    pub shape_id: ShapeId,
    /// Shape type reported by Box3D, when available.
    pub shape_type: Option<ShapeType>,
}

/// Debug draw command emitted by Box3D.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum DebugDrawCommand {
    /// Draw a shape outline.
    Shape {
        /// Optional shape handle associated with the draw command.
        shape: Option<DebugShape>,
        /// World transform of the shape.
        transform: WorldTransform,
        /// Shape color.
        color: HexColor,
    },
    /// Draw a line segment.
    Segment {
        /// First endpoint.
        p1: Pos,
        /// Second endpoint.
        p2: Pos,
        /// Segment color.
        color: HexColor,
    },
    /// Draw a transform basis.
    Transform(WorldTransform),
    /// Draw a point marker.
    Point {
        /// Point position.
        position: Pos,
        /// Point size in debug-draw units.
        size: f32,
        /// Point color.
        color: HexColor,
    },
    /// Draw a sphere.
    Sphere {
        /// Sphere center.
        center: Pos,
        /// Sphere radius.
        radius: f32,
        /// Sphere color.
        color: HexColor,
        /// Sphere alpha.
        alpha: f32,
    },
    /// Draw a capsule.
    Capsule {
        /// First capsule endpoint.
        p1: Pos,
        /// Second capsule endpoint.
        p2: Pos,
        /// Capsule radius.
        radius: f32,
        /// Capsule color.
        color: HexColor,
        /// Capsule alpha.
        alpha: f32,
    },
    /// Draw an AABB.
    Bounds {
        /// Bounds to draw.
        aabb: Aabb,
        /// Bounds color.
        color: HexColor,
    },
    /// Draw an oriented box.
    Box {
        /// Box half extents.
        extents: Vec3,
        /// Box world transform.
        transform: WorldTransform,
        /// Box color.
        color: HexColor,
    },
    /// Draw text.
    String {
        /// Text position.
        position: Pos,
        /// Text content.
        text: String,
        /// Text color.
        color: HexColor,
    },
}

/// Trait implemented by debug draw sinks.
///
/// Box3D calls these methods while walking the world for debug output. All
/// positions and transforms are in world coordinates. The callbacks run inside
/// Box3D callback context, so safe `World` APIs called reentrantly return
/// [`Error::InCallback`], and a panic is caught and reported by
/// [`World::try_debug_draw`] as [`Error::CallbackPanicked`].
pub trait DebugDraw {
    /// Draws a shape outline.
    ///
    /// Returning `false` asks Box3D to stop drawing further shapes.
    fn draw_shape(
        &mut self,
        _shape: Option<DebugShape>,
        _transform: WorldTransform,
        _color: HexColor,
    ) -> bool {
        true
    }

    /// Draws a line segment.
    fn draw_segment(&mut self, _p1: Pos, _p2: Pos, _color: HexColor) {}
    /// Draws a transform basis.
    fn draw_transform(&mut self, _transform: WorldTransform) {}
    /// Draws a point marker.
    fn draw_point(&mut self, _position: Pos, _size: f32, _color: HexColor) {}
    /// Draws a sphere.
    fn draw_sphere(&mut self, _center: Pos, _radius: f32, _color: HexColor, _alpha: f32) {}
    /// Draws a capsule.
    fn draw_capsule(&mut self, _p1: Pos, _p2: Pos, _radius: f32, _color: HexColor, _alpha: f32) {}
    /// Draws an AABB.
    fn draw_bounds(&mut self, _aabb: Aabb, _color: HexColor) {}
    /// Draws an oriented box.
    fn draw_box(&mut self, _extents: Vec3, _transform: WorldTransform, _color: HexColor) {}
    /// Draws text.
    fn draw_string(&mut self, _position: Pos, _text: &str, _color: HexColor) {}
}

/// Options passed to Box3D debug drawing.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug)]
pub struct DebugDrawOptions {
    /// Bounds limiting debug drawing.
    pub drawing_bounds: Aabb,
    /// Collision mask used by the query.
    pub mask_bits: u64,
    /// Scale applied to force visualizations.
    pub force_scale: f32,
    /// Scale applied to joint visualizations.
    pub joint_scale: f32,
    /// Whether shape outlines are drawn.
    pub draw_shapes: bool,
    /// Whether joints are drawn.
    pub draw_joints: bool,
    /// Whether joint extra details are drawn.
    pub draw_joint_extras: bool,
    /// Whether AABBs are drawn.
    pub draw_bounds: bool,
    /// Whether mass data is drawn.
    pub draw_mass: bool,
    /// Whether body names are drawn.
    pub draw_body_names: bool,
    /// Whether contacts are drawn.
    pub draw_contacts: bool,
    /// Anchor display mode used by Box3D.
    pub draw_anchor_a: i32,
    /// Whether graph-color debug coloring is drawn.
    pub draw_graph_colors: bool,
    /// Whether contact feature ids are drawn.
    pub draw_contact_features: bool,
    /// Whether contact normals are drawn.
    pub draw_contact_normals: bool,
    /// Whether contact forces are drawn.
    pub draw_contact_forces: bool,
    /// Whether friction forces are drawn.
    pub draw_friction_forces: bool,
    /// Whether solver islands are drawn.
    pub draw_islands: bool,
}

impl Default for DebugDrawOptions {
    fn default() -> Self {
        Self {
            drawing_bounds: Aabb {
                lower_bound: Vec3::new(-1.0e9, -1.0e9, -1.0e9),
                upper_bound: Vec3::new(1.0e9, 1.0e9, 1.0e9),
            },
            mask_bits: u64::MAX,
            force_scale: 1.0,
            joint_scale: 1.0,
            draw_shapes: true,
            draw_joints: true,
            draw_joint_extras: false,
            draw_bounds: false,
            draw_mass: false,
            draw_body_names: false,
            draw_contacts: false,
            draw_anchor_a: 0,
            draw_graph_colors: false,
            draw_contact_features: false,
            draw_contact_normals: false,
            draw_contact_forces: false,
            draw_friction_forces: false,
            draw_islands: false,
        }
    }
}

#[derive(Default)]
pub(crate) struct DebugShapeRegistry {
    created: AtomicUsize,
    destroyed: AtomicUsize,
}

impl fmt::Debug for DebugShapeRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DebugShapeRegistry")
            .field("created", &self.created.load(Ordering::Relaxed))
            .field("destroyed", &self.destroyed.load(Ordering::Relaxed))
            .finish()
    }
}

#[derive(Copy, Clone, Debug)]
struct DebugShapeResource {
    shape: DebugShape,
}

#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
pub(crate) unsafe extern "C" fn create_debug_shape(
    debug_shape: *const ffi::b3DebugShape,
    user_context: *mut c_void,
) -> *mut c_void {
    if debug_shape.is_null() {
        return std::ptr::null_mut();
    }
    if !user_context.is_null() {
        let registry = unsafe { &*(user_context as *const DebugShapeRegistry) };
        registry.created.fetch_add(1, Ordering::Relaxed);
    }

    let raw = unsafe { &*debug_shape };
    let shape = DebugShape {
        shape_id: ShapeId::from_raw(raw.shapeId),
        shape_type: ShapeType::from_raw(raw.type_),
    };
    Box::into_raw(Box::new(DebugShapeResource { shape })) as *mut c_void
}

#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
pub(crate) unsafe extern "C" fn destroy_debug_shape(
    user_shape: *mut c_void,
    user_context: *mut c_void,
) {
    if !user_context.is_null() {
        let registry = unsafe { &*(user_context as *const DebugShapeRegistry) };
        registry.destroyed.fetch_add(1, Ordering::Relaxed);
    }
    if !user_shape.is_null() {
        drop(unsafe { Box::from_raw(user_shape as *mut DebugShapeResource) });
    }
}

struct DebugDrawContext<'a> {
    drawer: &'a mut dyn DebugDraw,
    panicked: bool,
}

fn run_debug_draw_callback<R>(
    context: &mut DebugDrawContext<'_>,
    default: R,
    callback: impl FnOnce(&mut dyn DebugDraw) -> R,
) -> R {
    if context.panicked {
        return default;
    }

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = callback_state::CallbackGuard::enter();
        callback(context.drawer)
    })) {
        Ok(value) => value,
        Err(_) => {
            context.panicked = true;
            default
        }
    }
}

fn debug_shape_from_user_shape(user_shape: *mut c_void) -> Option<DebugShape> {
    if user_shape.is_null() {
        None
    } else {
        Some(unsafe { (*(user_shape as *const DebugShapeResource)).shape })
    }
}

fn apply_options(draw: &mut ffi::b3DebugDraw, options: DebugDrawOptions, context: *mut c_void) {
    draw.drawingBounds = options.drawing_bounds.into_raw();
    draw.forceScale = options.force_scale;
    draw.jointScale = options.joint_scale;
    draw.drawShapes = options.draw_shapes;
    draw.drawJoints = options.draw_joints;
    draw.drawJointExtras = options.draw_joint_extras;
    draw.drawBounds = options.draw_bounds;
    draw.drawMass = options.draw_mass;
    draw.drawBodyNames = options.draw_body_names;
    draw.drawContacts = options.draw_contacts;
    draw.drawAnchorA = options.draw_anchor_a;
    draw.drawGraphColors = options.draw_graph_colors;
    draw.drawContactFeatures = options.draw_contact_features;
    draw.drawContactNormals = options.draw_contact_normals;
    draw.drawContactForces = options.draw_contact_forces;
    draw.drawFrictionForces = options.draw_friction_forces;
    draw.drawIslands = options.draw_islands;
    draw.context = context;
}

pub(crate) struct CollectDebugDraw<'a> {
    commands: &'a mut Vec<DebugDrawCommand>,
    len: usize,
}

impl<'a> CollectDebugDraw<'a> {
    pub(crate) fn new(commands: &'a mut Vec<DebugDrawCommand>) -> Self {
        Self { commands, len: 0 }
    }

    pub(crate) fn finish(self) {
        self.commands.truncate(self.len);
    }

    fn replace_or_push(&mut self, command: DebugDrawCommand) {
        if let Some(slot) = self.commands.get_mut(self.len) {
            *slot = command;
        } else {
            self.commands.push(command);
        }
        self.len += 1;
    }
}

impl DebugDraw for CollectDebugDraw<'_> {
    fn draw_shape(
        &mut self,
        shape: Option<DebugShape>,
        transform: WorldTransform,
        color: HexColor,
    ) -> bool {
        self.replace_or_push(DebugDrawCommand::Shape {
            shape,
            transform,
            color,
        });
        true
    }

    fn draw_segment(&mut self, p1: Pos, p2: Pos, color: HexColor) {
        self.replace_or_push(DebugDrawCommand::Segment { p1, p2, color });
    }

    fn draw_transform(&mut self, transform: WorldTransform) {
        self.replace_or_push(DebugDrawCommand::Transform(transform));
    }

    fn draw_point(&mut self, position: Pos, size: f32, color: HexColor) {
        self.replace_or_push(DebugDrawCommand::Point {
            position,
            size,
            color,
        });
    }

    fn draw_sphere(&mut self, center: Pos, radius: f32, color: HexColor, alpha: f32) {
        self.replace_or_push(DebugDrawCommand::Sphere {
            center,
            radius,
            color,
            alpha,
        });
    }

    fn draw_capsule(&mut self, p1: Pos, p2: Pos, radius: f32, color: HexColor, alpha: f32) {
        self.replace_or_push(DebugDrawCommand::Capsule {
            p1,
            p2,
            radius,
            color,
            alpha,
        });
    }

    fn draw_bounds(&mut self, aabb: Aabb, color: HexColor) {
        self.replace_or_push(DebugDrawCommand::Bounds { aabb, color });
    }

    fn draw_box(&mut self, extents: Vec3, transform: WorldTransform, color: HexColor) {
        self.replace_or_push(DebugDrawCommand::Box {
            extents,
            transform,
            color,
        });
    }

    fn draw_string(&mut self, position: Pos, text: &str, color: HexColor) {
        match self.commands.get_mut(self.len) {
            Some(DebugDrawCommand::String {
                position: stored_position,
                text: stored_text,
                color: stored_color,
            }) => {
                *stored_position = position;
                stored_text.clear();
                stored_text.push_str(text);
                *stored_color = color;
                self.len += 1;
            }
            _ => self.replace_or_push(DebugDrawCommand::String {
                position,
                text: text.to_owned(),
                color,
            }),
        }
    }
}

unsafe extern "C" fn draw_shape(
    user_shape: *mut c_void,
    transform: ffi::b3WorldTransform,
    color: ffi::b3HexColor,
    context: *mut c_void,
) -> bool {
    let context = unsafe { &mut *(context as *mut DebugDrawContext<'_>) };
    run_debug_draw_callback(context, false, |drawer| {
        drawer.draw_shape(
            debug_shape_from_user_shape(user_shape),
            WorldTransform::from_raw(transform),
            HexColor::from_raw(color),
        )
    })
}

unsafe extern "C" fn draw_segment(
    p1: ffi::b3Pos,
    p2: ffi::b3Pos,
    color: ffi::b3HexColor,
    context: *mut c_void,
) {
    let context = unsafe { &mut *(context as *mut DebugDrawContext<'_>) };
    run_debug_draw_callback(context, (), |drawer| {
        drawer.draw_segment(
            Pos::from_raw(p1),
            Pos::from_raw(p2),
            HexColor::from_raw(color),
        );
    });
}

unsafe extern "C" fn draw_transform(transform: ffi::b3WorldTransform, context: *mut c_void) {
    let context = unsafe { &mut *(context as *mut DebugDrawContext<'_>) };
    run_debug_draw_callback(context, (), |drawer| {
        drawer.draw_transform(WorldTransform::from_raw(transform));
    });
}

unsafe extern "C" fn draw_point(
    position: ffi::b3Pos,
    size: f32,
    color: ffi::b3HexColor,
    context: *mut c_void,
) {
    let context = unsafe { &mut *(context as *mut DebugDrawContext<'_>) };
    run_debug_draw_callback(context, (), |drawer| {
        drawer.draw_point(Pos::from_raw(position), size, HexColor::from_raw(color));
    });
}

unsafe extern "C" fn draw_sphere(
    center: ffi::b3Pos,
    radius: f32,
    color: ffi::b3HexColor,
    alpha: f32,
    context: *mut c_void,
) {
    let context = unsafe { &mut *(context as *mut DebugDrawContext<'_>) };
    run_debug_draw_callback(context, (), |drawer| {
        drawer.draw_sphere(
            Pos::from_raw(center),
            radius,
            HexColor::from_raw(color),
            alpha,
        );
    });
}

unsafe extern "C" fn draw_capsule(
    p1: ffi::b3Pos,
    p2: ffi::b3Pos,
    radius: f32,
    color: ffi::b3HexColor,
    alpha: f32,
    context: *mut c_void,
) {
    let context = unsafe { &mut *(context as *mut DebugDrawContext<'_>) };
    run_debug_draw_callback(context, (), |drawer| {
        drawer.draw_capsule(
            Pos::from_raw(p1),
            Pos::from_raw(p2),
            radius,
            HexColor::from_raw(color),
            alpha,
        );
    });
}

unsafe extern "C" fn draw_bounds(aabb: ffi::b3AABB, color: ffi::b3HexColor, context: *mut c_void) {
    let context = unsafe { &mut *(context as *mut DebugDrawContext<'_>) };
    run_debug_draw_callback(context, (), |drawer| {
        drawer.draw_bounds(Aabb::from_raw(aabb), HexColor::from_raw(color));
    });
}

unsafe extern "C" fn draw_box(
    extents: ffi::b3Vec3,
    transform: ffi::b3WorldTransform,
    color: ffi::b3HexColor,
    context: *mut c_void,
) {
    let context = unsafe { &mut *(context as *mut DebugDrawContext<'_>) };
    run_debug_draw_callback(context, (), |drawer| {
        drawer.draw_box(
            Vec3::from_raw(extents),
            WorldTransform::from_raw(transform),
            HexColor::from_raw(color),
        );
    });
}

unsafe extern "C" fn draw_string(
    position: ffi::b3Pos,
    text: *const std::ffi::c_char,
    color: ffi::b3HexColor,
    context: *mut c_void,
) {
    if text.is_null() {
        return;
    }
    let context = unsafe { &mut *(context as *mut DebugDrawContext<'_>) };
    let text = unsafe { CStr::from_ptr(text) }.to_string_lossy();
    run_debug_draw_callback(context, (), |drawer| {
        drawer.draw_string(Pos::from_raw(position), &text, HexColor::from_raw(color));
    });
}

pub(crate) fn with_debug_draw(
    drawer: &mut dyn DebugDraw,
    options: DebugDrawOptions,
    invoke: impl FnOnce(&mut ffi::b3DebugDraw) -> Result<()>,
) -> Result<()> {
    callback_state::check_not_in_callback()?;
    #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
    {
        let _ = (drawer, options, invoke);
        Err(Error::UnsupportedOnWasm)
    }
    #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
    {
        if !options.force_scale.is_finite()
            || !options.joint_scale.is_finite()
            || !options.drawing_bounds.is_valid()
        {
            return Err(Error::InvalidArgument);
        }

        let mut context = DebugDrawContext {
            drawer,
            panicked: false,
        };
        let mut draw = unsafe { ffi::b3DefaultDebugDraw() };
        draw.DrawShapeFcn = Some(draw_shape);
        draw.DrawSegmentFcn = Some(draw_segment);
        draw.DrawTransformFcn = Some(draw_transform);
        draw.DrawPointFcn = Some(draw_point);
        draw.DrawSphereFcn = Some(draw_sphere);
        draw.DrawCapsuleFcn = Some(draw_capsule);
        draw.DrawBoundsFcn = Some(draw_bounds);
        draw.DrawBoxFcn = Some(draw_box);
        draw.DrawStringFcn = Some(draw_string);
        apply_options(&mut draw, options, &mut context as *mut _ as *mut c_void);

        invoke(&mut draw)?;
        if context.panicked {
            Err(Error::CallbackPanicked)
        } else {
            Ok(())
        }
    }
}

impl World {
    /// Collects debug draw commands or panics if Box3D rejects the draw.
    pub fn debug_draw_collect(&mut self, options: DebugDrawOptions) -> Vec<DebugDrawCommand> {
        self.try_debug_draw_collect(options)
            .expect("Box3D debug draw failed")
    }

    /// Tries to collect debug draw commands.
    pub fn try_debug_draw_collect(
        &mut self,
        options: DebugDrawOptions,
    ) -> Result<Vec<DebugDrawCommand>> {
        let mut commands = Vec::new();
        self.try_debug_draw_collect_into(&mut commands, options)?;
        Ok(commands)
    }

    /// Collects debug draw commands into `out` or panics if Box3D rejects the draw.
    pub fn debug_draw_collect_into(
        &mut self,
        out: &mut Vec<DebugDrawCommand>,
        options: DebugDrawOptions,
    ) {
        self.try_debug_draw_collect_into(out, options)
            .expect("Box3D debug draw failed");
    }

    /// Tries to collect debug draw commands into `out`.
    pub fn try_debug_draw_collect_into(
        &mut self,
        out: &mut Vec<DebugDrawCommand>,
        options: DebugDrawOptions,
    ) -> Result<()> {
        let mut collector = CollectDebugDraw::new(out);
        self.try_debug_draw(&mut collector, options)?;
        collector.finish();
        Ok(())
    }

    /// Runs debug drawing or panics if Box3D rejects the draw.
    pub fn debug_draw(&mut self, drawer: &mut impl DebugDraw, options: DebugDrawOptions) {
        self.try_debug_draw(drawer, options)
            .expect("Box3D debug draw failed");
    }

    /// Tries to run debug drawing with a custom sink.
    pub fn try_debug_draw(
        &mut self,
        drawer: &mut impl DebugDraw,
        options: DebugDrawOptions,
    ) -> Result<()> {
        with_debug_draw(drawer, options, |draw| {
            let _guard = box3d_lock::lock();
            self.check_world_valid_locked()?;
            unsafe { ffi::b3World_Draw(self.raw(), draw, options.mask_bits) };
            Ok(())
        })
    }
}
