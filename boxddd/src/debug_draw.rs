use crate::core::{box3d_lock, callback_state};
use crate::error::{Error, Result};
use crate::shapes::ShapeType;
use crate::types::{Aabb, Pos, ShapeId, Vec3, WorldTransform};
use crate::world::World;
use boxddd_sys::ffi;
use std::ffi::{CStr, c_void};
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct HexColor(u32);

impl HexColor {
    pub const BLACK: Self = Self::from_rgb_u32(0x000000);
    pub const WHITE: Self = Self::from_rgb_u32(0xffffff);
    pub const RED: Self = Self::from_rgb_u32(0xff0000);
    pub const GREEN: Self = Self::from_rgb_u32(0x00ff00);
    pub const BLUE: Self = Self::from_rgb_u32(0x0000ff);

    #[inline]
    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self(((red as u32) << 16) | ((green as u32) << 8) | blue as u32)
    }

    #[inline]
    pub const fn from_rgb_u32(rgb: u32) -> Self {
        Self(rgb & 0x00ff_ffff)
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3HexColor) -> Self {
        Self::from_rgb_u32(raw as u32)
    }

    #[inline]
    pub const fn rgb_u32(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3HexColor {
        self.0 as ffi::b3HexColor
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DebugShape {
    pub shape_id: ShapeId,
    pub shape_type: Option<ShapeType>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DebugDrawCommand {
    Shape {
        shape: Option<DebugShape>,
        transform: WorldTransform,
        color: HexColor,
    },
    Segment {
        p1: Pos,
        p2: Pos,
        color: HexColor,
    },
    Transform(WorldTransform),
    Point {
        position: Pos,
        size: f32,
        color: HexColor,
    },
    Sphere {
        center: Pos,
        radius: f32,
        color: HexColor,
        alpha: f32,
    },
    Capsule {
        p1: Pos,
        p2: Pos,
        radius: f32,
        color: HexColor,
        alpha: f32,
    },
    Bounds {
        aabb: Aabb,
        color: HexColor,
    },
    Box {
        extents: Vec3,
        transform: WorldTransform,
        color: HexColor,
    },
    String {
        position: Pos,
        text: String,
        color: HexColor,
    },
}

pub trait DebugDraw {
    fn draw_shape(
        &mut self,
        _shape: Option<DebugShape>,
        _transform: WorldTransform,
        _color: HexColor,
    ) -> bool {
        true
    }

    fn draw_segment(&mut self, _p1: Pos, _p2: Pos, _color: HexColor) {}
    fn draw_transform(&mut self, _transform: WorldTransform) {}
    fn draw_point(&mut self, _position: Pos, _size: f32, _color: HexColor) {}
    fn draw_sphere(&mut self, _center: Pos, _radius: f32, _color: HexColor, _alpha: f32) {}
    fn draw_capsule(&mut self, _p1: Pos, _p2: Pos, _radius: f32, _color: HexColor, _alpha: f32) {}
    fn draw_bounds(&mut self, _aabb: Aabb, _color: HexColor) {}
    fn draw_box(&mut self, _extents: Vec3, _transform: WorldTransform, _color: HexColor) {}
    fn draw_string(&mut self, _position: Pos, _text: &str, _color: HexColor) {}
}

#[derive(Copy, Clone, Debug)]
pub struct DebugDrawOptions {
    pub drawing_bounds: Aabb,
    pub mask_bits: u64,
    pub force_scale: f32,
    pub joint_scale: f32,
    pub draw_shapes: bool,
    pub draw_joints: bool,
    pub draw_joint_extras: bool,
    pub draw_bounds: bool,
    pub draw_mass: bool,
    pub draw_body_names: bool,
    pub draw_contacts: bool,
    pub draw_anchor_a: i32,
    pub draw_graph_colors: bool,
    pub draw_contact_features: bool,
    pub draw_contact_normals: bool,
    pub draw_contact_forces: bool,
    pub draw_friction_forces: bool,
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

struct CollectDebugDraw<'a> {
    commands: &'a mut Vec<DebugDrawCommand>,
    len: usize,
}

impl<'a> CollectDebugDraw<'a> {
    fn new(commands: &'a mut Vec<DebugDrawCommand>) -> Self {
        Self { commands, len: 0 }
    }

    fn finish(self) {
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

impl World {
    pub fn debug_draw_collect(&mut self, options: DebugDrawOptions) -> Vec<DebugDrawCommand> {
        self.try_debug_draw_collect(options)
            .expect("Box3D debug draw failed")
    }

    pub fn try_debug_draw_collect(
        &mut self,
        options: DebugDrawOptions,
    ) -> Result<Vec<DebugDrawCommand>> {
        let mut commands = Vec::new();
        self.try_debug_draw_collect_into(&mut commands, options)?;
        Ok(commands)
    }

    pub fn debug_draw_collect_into(
        &mut self,
        out: &mut Vec<DebugDrawCommand>,
        options: DebugDrawOptions,
    ) {
        self.try_debug_draw_collect_into(out, options)
            .expect("Box3D debug draw failed");
    }

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

    pub fn debug_draw(&mut self, drawer: &mut impl DebugDraw, options: DebugDrawOptions) {
        self.try_debug_draw(drawer, options)
            .expect("Box3D debug draw failed");
    }

    pub fn try_debug_draw(
        &mut self,
        drawer: &mut impl DebugDraw,
        options: DebugDrawOptions,
    ) -> Result<()> {
        callback_state::check_not_in_callback()?;
        if !options.force_scale.is_finite()
            || !options.joint_scale.is_finite()
            || !options.drawing_bounds.lower_bound.is_valid()
            || !options.drawing_bounds.upper_bound.is_valid()
        {
            return Err(Error::InvalidArgument);
        }

        let mut context = DebugDrawContext {
            drawer,
            panicked: false,
        };
        let mut draw = unsafe { ffi::b3DefaultDebugDraw() };

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

        unsafe extern "C" fn draw_transform(
            transform: ffi::b3WorldTransform,
            context: *mut c_void,
        ) {
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

        unsafe extern "C" fn draw_bounds(
            aabb: ffi::b3AABB,
            color: ffi::b3HexColor,
            context: *mut c_void,
        ) {
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

        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_Draw(self.raw(), &mut draw, options.mask_bits) };
        if context.panicked {
            Err(Error::CallbackPanicked)
        } else {
            Ok(())
        }
    }
}
