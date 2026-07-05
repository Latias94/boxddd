#![cfg_attr(all(target_arch = "wasm32", boxddd_wasm_provider), allow(dead_code))]

use crate::core::{box3d_lock, callback_state};
use crate::error::{Error, Result};
use crate::shapes::ShapeType;
use crate::types::{Aabb, Plane, Pos, ShapeId, Transform, Vec3, WorldTransform};
use crate::world::World;
use boxddd_sys::ffi;
use std::cell::RefCell;
#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
use std::collections::HashMap;
use std::ffi::{CStr, c_void};
use std::fmt;
use std::mem;
use std::slice;

/// Packed Box3D debug color.
///
/// Box3D stores RGB in the low 24 bits and may use the high byte for a debug
/// material preset. Use [`HexColor::rgb_u32`] when only the visible color is
/// needed, and [`HexColor::raw_u32`] when preserving renderer metadata.
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

    /// Creates a color from a raw Box3D debug color payload.
    #[inline]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the full raw payload, including the high material byte.
    #[inline]
    pub const fn raw_u32(self) -> u32 {
        self.0
    }

    /// Returns this color as a packed `0xRRGGBB` value.
    #[inline]
    pub const fn rgb_u32(self) -> u32 {
        self.0 & 0x00ff_ffff
    }

    /// Returns the full raw Box3D color payload.
    #[inline]
    pub const fn into_raw(self) -> u32 {
        self.0
    }

    #[inline]
    fn from_ffi(raw: ffi::b3HexColor) -> Self {
        Self(raw as u32)
    }
}

/// Stable typed handle for a persistent Box3D debug shape asset.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DebugShapeHandle {
    /// Slot index owned by the debug shape store.
    pub index: u32,
    /// Generation used to reject stale renderer cache entries.
    pub generation: u32,
}

impl DebugShapeHandle {
    /// Creates a handle. Generation zero is reserved as invalid.
    #[inline]
    pub const fn new(index: u32, generation: u32) -> Option<Self> {
        if generation == 0 {
            None
        } else {
            Some(Self { index, generation })
        }
    }

    /// Returns whether this handle is non-zero and usable as a renderer cache key.
    #[inline]
    pub const fn is_valid(self) -> bool {
        self.generation != 0
    }
}

/// Legacy shape metadata emitted by the `0.1` debug draw command model.
///
/// `0.2` debug drawing uses [`DebugShapeHandle`] for frame commands and
/// [`DebugShapeAsset`] for owned geometry snapshots. This type remains as a
/// small migration aid for code that stored the former metadata shape.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DebugShape {
    /// Shape associated with the result.
    pub shape_id: ShapeId,
    /// Shape type reported by Box3D, when available.
    pub shape_type: Option<ShapeType>,
}

impl DebugShape {
    /// Converts an owned debug shape asset into its `0.1` metadata view.
    #[inline]
    pub const fn from_asset(asset: &DebugShapeAsset) -> Self {
        Self {
            shape_id: asset.shape_id,
            shape_type: Some(asset.shape_type),
        }
    }
}

/// Owned asset emitted when Box3D creates a persistent debug shape.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct DebugShapeAsset {
    /// Stable handle referenced by subsequent shape draw commands.
    pub handle: DebugShapeHandle,
    /// Box3D shape that produced this debug asset.
    pub shape_id: ShapeId,
    /// Box3D shape kind.
    pub shape_type: ShapeType,
    /// Owned renderer-agnostic geometry snapshot.
    pub geometry: DebugShapeGeometry,
}

/// Lifecycle event for persistent debug shape assets.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum DebugShapeEvent {
    /// A new renderer asset should be created or refreshed.
    Created(DebugShapeAsset),
    /// A previously emitted asset should be removed from renderer caches.
    Destroyed {
        /// Retired handle.
        handle: DebugShapeHandle,
    },
    /// The owning world has invalidated every debug handle.
    ClearAll,
}

/// Diagnostic captured while collecting a debug draw frame.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugDrawDiagnostic {
    /// Human-readable message.
    pub message: String,
}

/// Face polygon copied from a convex hull.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct DebugHullFace {
    /// Indices into [`DebugShapeGeometry::Hull::points`].
    pub indices: Vec<u32>,
    /// Local plane for the face.
    pub plane: Plane,
}

/// Owned triangle mesh snapshot used by mesh-like debug geometry.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct DebugMesh {
    /// Local-space bounds.
    pub bounds: Aabb,
    /// Mesh vertices.
    pub vertices: Vec<Vec3>,
    /// Mesh triangles.
    pub triangles: Vec<DebugMeshTriangle>,
    /// Number of material slots stored by the source shape.
    pub material_count: i32,
}

/// Triangle copied from cooked mesh-like data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DebugMeshTriangle {
    /// Indices into [`DebugMesh::vertices`].
    pub indices: [u32; 3],
    /// Optional per-triangle material index.
    pub material_index: Option<u8>,
}

/// Owned child geometry inside a compound debug shape.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct DebugCompoundChild {
    /// Transform from compound-local space to child-local space.
    pub transform: Transform,
    /// Material indices reported by Box3D for this child.
    pub material_indices: [i32; ffi::B3_MAX_COMPOUND_MESH_MATERIALS as usize],
    /// Owned child geometry.
    pub geometry: DebugShapeGeometry,
}

/// Renderer-agnostic owned geometry for persistent debug shapes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum DebugShapeGeometry {
    /// Sphere geometry in the shape's local space.
    Sphere {
        /// Local center.
        center: Vec3,
        /// Radius.
        radius: f32,
    },
    /// Capsule geometry in the shape's local space.
    Capsule {
        /// First local endpoint.
        center1: Vec3,
        /// Second local endpoint.
        center2: Vec3,
        /// Radius.
        radius: f32,
    },
    /// Convex hull geometry.
    Hull {
        /// Local-space bounds.
        aabb: Aabb,
        /// Hull points.
        points: Vec<Vec3>,
        /// Convex faces as point index polygons.
        faces: Vec<DebugHullFace>,
    },
    /// Cooked triangle mesh geometry.
    Mesh {
        /// Owned mesh data.
        mesh: DebugMesh,
        /// Per-shape scale.
        scale: Vec3,
    },
    /// Height-field data expanded into a mesh snapshot.
    HeightField {
        /// Owned mesh data.
        mesh: DebugMesh,
    },
    /// Compound geometry flattened into owned children.
    Compound {
        /// Flattened child shapes.
        children: Vec<DebugCompoundChild>,
    },
}

impl DebugShapeGeometry {
    /// Returns the Box3D shape type represented by this geometry.
    #[inline]
    pub const fn shape_type(&self) -> Option<ShapeType> {
        match self {
            Self::Sphere { .. } => Some(ShapeType::Sphere),
            Self::Capsule { .. } => Some(ShapeType::Capsule),
            Self::Hull { .. } => Some(ShapeType::Hull),
            Self::Mesh { .. } => Some(ShapeType::Mesh),
            Self::HeightField { .. } => Some(ShapeType::HeightField),
            Self::Compound { .. } => Some(ShapeType::Compound),
        }
    }
}

/// Debug draw command emitted for one frame.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum DebugDrawCommand {
    /// Draw a persistent shape asset.
    Shape {
        /// Optional persistent debug shape handle.
        handle: Option<DebugShapeHandle>,
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

/// Complete data collected from one debug draw pass.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DebugDrawFrame {
    /// Persistent shape lifecycle events emitted since the previous drain.
    pub events: Vec<DebugShapeEvent>,
    /// Immediate draw commands for the current frame.
    pub commands: Vec<DebugDrawCommand>,
    /// Non-fatal diagnostics captured while copying debug shape data.
    pub diagnostics: Vec<DebugDrawDiagnostic>,
}

impl DebugDrawFrame {
    /// Clears all events, commands, and diagnostics while preserving capacity.
    pub fn clear(&mut self) {
        self.events.clear();
        self.commands.clear();
        self.diagnostics.clear();
    }
}

/// Trait implemented by low-level debug draw sinks.
///
/// Most users should prefer [`World::try_debug_draw_frame`], which exposes a
/// lifecycle-correct data model. This trait remains useful for internal helpers
/// such as recording query visualization and panic containment tests.
pub trait DebugDraw {
    /// Draws a persistent shape outline.
    fn draw_shape(
        &mut self,
        _handle: Option<DebugShapeHandle>,
        _transform: WorldTransform,
        _color: HexColor,
    ) {
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
    inner: RefCell<DebugShapeStore>,
}

#[derive(Default)]
struct DebugShapeStore {
    slots: Vec<DebugShapeSlot>,
    free: Vec<usize>,
    events: Vec<DebugShapeEvent>,
    diagnostics: Vec<DebugDrawDiagnostic>,
    cleared: bool,
}

#[derive(Debug)]
struct DebugShapeSlot {
    generation: u32,
    asset: Option<DebugShapeAsset>,
}

impl fmt::Debug for DebugShapeRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.inner.borrow();
        f.debug_struct("DebugShapeRegistry")
            .field("slots", &inner.slots.len())
            .field("free", &inner.free.len())
            .field("pending_events", &inner.events.len())
            .field("pending_diagnostics", &inner.diagnostics.len())
            .finish()
    }
}

#[derive(Copy, Clone, Debug)]
struct DebugShapeResource {
    handle: DebugShapeHandle,
}

impl DebugShapeRegistry {
    fn create_asset_handle(&self, raw: &ffi::b3DebugShape) -> Option<DebugShapeHandle> {
        let mut store = self.inner.borrow_mut();
        let snapshot = match unsafe { snapshot_debug_shape(raw) } {
            Ok(snapshot) => snapshot,
            Err(message) => {
                store.diagnostics.push(DebugDrawDiagnostic {
                    message: message.to_owned(),
                });
                return None;
            }
        };
        let handle = store.alloc_handle();
        let asset = DebugShapeAsset {
            handle,
            shape_id: snapshot.shape_id,
            shape_type: snapshot.shape_type,
            geometry: snapshot.geometry,
        };
        store.slots[handle.index as usize].asset = Some(asset.clone());
        store.events.push(DebugShapeEvent::Created(asset));
        Some(handle)
    }

    fn create_native_resource(&self, raw: &ffi::b3DebugShape) -> *mut c_void {
        let Some(handle) = self.create_asset_handle(raw) else {
            return std::ptr::null_mut();
        };
        Box::into_raw(Box::new(DebugShapeResource { handle })) as *mut c_void
    }

    fn destroy_native_resource(&self, resource: DebugShapeResource) {
        self.destroy_handle(resource.handle);
    }

    fn destroy_handle(&self, handle: DebugShapeHandle) {
        self.inner.borrow_mut().destroy_handle(handle);
    }

    pub(crate) fn drain_into(&self, frame: &mut DebugDrawFrame) {
        let mut store = self.inner.borrow_mut();
        frame.events.extend(store.events.drain(..));
        frame.diagnostics.extend(store.diagnostics.drain(..));
    }

    #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
    fn push_diagnostic(&self, message: impl Into<String>) {
        self.inner
            .borrow_mut()
            .diagnostics
            .push(DebugDrawDiagnostic {
                message: message.into(),
            });
    }

    pub(crate) fn clear_all(&self) {
        let mut store = self.inner.borrow_mut();
        for slot in &mut store.slots {
            if slot.asset.take().is_some() {
                slot.generation = next_generation(slot.generation);
            }
        }
        let slot_len = store.slots.len();
        store.free.clear();
        store.free.extend(0..slot_len);
        store.events.clear();
        store.diagnostics.clear();
        store.events.push(DebugShapeEvent::ClearAll);
        store.cleared = true;
    }
}

impl DebugShapeStore {
    fn alloc_handle(&mut self) -> DebugShapeHandle {
        self.cleared = false;
        if let Some(index) = self.free.pop() {
            let generation = self.slots[index].generation;
            DebugShapeHandle {
                index: index as u32,
                generation,
            }
        } else {
            let index = self.slots.len();
            self.slots.push(DebugShapeSlot {
                generation: 1,
                asset: None,
            });
            DebugShapeHandle {
                index: index as u32,
                generation: 1,
            }
        }
    }

    fn destroy_handle(&mut self, handle: DebugShapeHandle) {
        let Some(slot) = self.slots.get_mut(handle.index as usize) else {
            self.diagnostics.push(DebugDrawDiagnostic {
                message: "debug shape destroy referenced an unknown handle".to_owned(),
            });
            return;
        };
        if slot.generation != handle.generation || slot.asset.is_none() {
            if self.cleared {
                return;
            }
            self.diagnostics.push(DebugDrawDiagnostic {
                message: "debug shape destroy referenced a stale handle".to_owned(),
            });
            return;
        }

        slot.asset = None;
        slot.generation = next_generation(slot.generation);
        self.events.push(DebugShapeEvent::Destroyed { handle });
        self.free.push(handle.index as usize);
    }
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
thread_local! {
    static PROVIDER_DEBUG: RefCell<ProviderDebugRegistry> = RefCell::new(ProviderDebugRegistry::default());
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
#[derive(Default)]
struct ProviderDebugRegistry {
    registries: HashMap<u32, ProviderDebugWorld>,
    shapes: HashMap<u32, ProviderDebugShape>,
    next_token: u32,
    next_shape: u32,
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
struct ProviderDebugWorld {
    registry: *const DebugShapeRegistry,
    active_commands: Option<*mut Vec<DebugDrawCommand>>,
    first_error: Option<Error>,
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
#[derive(Copy, Clone)]
struct ProviderDebugShape {
    token: u32,
    handle: DebugShapeHandle,
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
impl ProviderDebugRegistry {
    fn allocate_token_id(&mut self) -> Option<u32> {
        self.next_token = self.next_token.checked_add(1)?;
        Some(self.next_token)
    }

    fn allocate_shape_id(&mut self) -> Option<u32> {
        self.next_shape = self.next_shape.checked_add(1)?;
        Some(self.next_shape)
    }
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
pub(crate) struct ProviderDebugFrameGuard {
    token: u32,
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
impl ProviderDebugFrameGuard {
    pub(crate) fn new(token: u32, commands: &mut Vec<DebugDrawCommand>) -> Self {
        set_provider_debug_frame(token, commands);
        Self { token }
    }
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
impl Drop for ProviderDebugFrameGuard {
    fn drop(&mut self) {
        clear_provider_debug_frame(self.token);
    }
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
pub(crate) fn register_provider_debug_registry(registry: &DebugShapeRegistry) -> Option<u32> {
    PROVIDER_DEBUG.with(|state| {
        let mut state = state.borrow_mut();
        let token = state.allocate_token_id()?;
        state.registries.insert(
            token,
            ProviderDebugWorld {
                registry: registry as *const DebugShapeRegistry,
                active_commands: None,
                first_error: None,
            },
        );
        Some(token)
    })
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
pub(crate) fn unregister_provider_debug_registry(token: u32) {
    if token == 0 {
        return;
    }
    PROVIDER_DEBUG.with(|state| {
        let mut state = state.borrow_mut();
        state.registries.remove(&token);
        state.shapes.retain(|_, shape| shape.token != token);
    });
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
fn set_provider_debug_frame(token: u32, commands: &mut Vec<DebugDrawCommand>) {
    PROVIDER_DEBUG.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(world) = state.registries.get_mut(&token) {
            world.active_commands = Some(commands as *mut Vec<DebugDrawCommand>);
            world.first_error = None;
        }
    });
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
fn clear_provider_debug_frame(token: u32) {
    PROVIDER_DEBUG.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(world) = state.registries.get_mut(&token) {
            world.active_commands = None;
        }
    });
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
pub(crate) fn take_provider_debug_error(token: u32) -> Option<Error> {
    if token == 0 {
        return None;
    }
    PROVIDER_DEBUG.with(|state| {
        let mut state = state.borrow_mut();
        state
            .registries
            .get_mut(&token)
            .and_then(|world| world.first_error.take())
    })
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
fn provider_registry(token: u32) -> Option<*const DebugShapeRegistry> {
    if token == 0 {
        return None;
    }
    PROVIDER_DEBUG.with(|state| {
        let state = state.borrow();
        state.registries.get(&token).map(|world| world.registry)
    })
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
fn provider_record_diagnostic(token: u32, message: impl Into<String>) {
    if let Some(registry) = provider_registry(token) {
        unsafe { &*registry }.push_diagnostic(message);
    }
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
fn provider_fail(token: u32, message: impl Into<String>) {
    if token != 0 {
        PROVIDER_DEBUG.with(|state| {
            let mut state = state.borrow_mut();
            if let Some(world) = state.registries.get_mut(&token) {
                world
                    .first_error
                    .get_or_insert(Error::ProviderCallbackFailed);
            }
        });
    }
    provider_record_diagnostic(token, message);
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
fn provider_alloc_shape(token: u32, handle: DebugShapeHandle) -> Option<u32> {
    PROVIDER_DEBUG.with(|state| {
        let mut state = state.borrow_mut();
        let raw_shape = state.allocate_shape_id()?;
        state
            .shapes
            .insert(raw_shape, ProviderDebugShape { token, handle });
        Some(raw_shape)
    })
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
fn provider_can_alloc_shape() -> bool {
    PROVIDER_DEBUG.with(|state| state.borrow().next_shape < u32::MAX)
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
fn provider_take_shape(token: u32, raw_shape: u32) -> Option<DebugShapeHandle> {
    if raw_shape == 0 {
        return None;
    }
    PROVIDER_DEBUG.with(|state| {
        let mut state = state.borrow_mut();
        let shape = state.shapes.get(&raw_shape).copied()?;
        if shape.token != token {
            return None;
        }
        state.shapes.remove(&raw_shape);
        Some(shape.handle)
    })
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
fn provider_shape_handle(token: u32, raw_shape: u32) -> Option<DebugShapeHandle> {
    if raw_shape == 0 {
        return None;
    }
    PROVIDER_DEBUG.with(|state| {
        let state = state.borrow();
        state
            .shapes
            .get(&raw_shape)
            .copied()
            .filter(|shape| shape.token == token)
            .map(|shape| shape.handle)
    })
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
fn provider_push_command(token: u32, command: DebugDrawCommand) -> bool {
    if token == 0 {
        return false;
    }
    let mut missing_frame = false;
    let pushed = PROVIDER_DEBUG.with(|state| {
        let state = state.borrow();
        let Some(world) = state.registries.get(&token) else {
            missing_frame = true;
            return false;
        };
        let Some(commands) = world.active_commands else {
            missing_frame = true;
            return false;
        };
        unsafe { (*commands).push(command) };
        true
    });
    if !pushed && missing_frame {
        provider_fail(
            token,
            "debug draw provider callback arrived without an active frame",
        );
    }
    pushed
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
unsafe fn provider_read<T: Copy>(ptr: *const T) -> Option<T> {
    unsafe { ptr.as_ref().copied() }
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
#[unsafe(no_mangle)]
pub extern "C" fn boxddd_debug_report_error(token: u32, code: u32) {
    let message = match code {
        1 => "debug draw provider exports were not registered or were incomplete",
        2 => "debug draw provider dispatcher threw an exception",
        _ => "debug draw provider dispatcher failed",
    };
    provider_fail(token, message);
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
#[unsafe(no_mangle)]
pub extern "C" fn boxddd_debug_shape_create(
    token: u32,
    debug_shape: *const ffi::b3DebugShape,
) -> u32 {
    let Some(registry) = provider_registry(token) else {
        return 0;
    };
    let Some(raw) = (unsafe { debug_shape.as_ref() }) else {
        provider_fail(token, "debug shape create callback received a null shape");
        return 0;
    };
    if !provider_can_alloc_shape() {
        provider_fail(token, "debug draw provider shape handle table is full");
        return 0;
    }
    let Some(handle) = (unsafe { &*registry }).create_asset_handle(raw) else {
        return 0;
    };
    match provider_alloc_shape(token, handle) {
        Some(raw_shape) => raw_shape,
        None => {
            provider_fail(token, "debug draw provider shape handle table is full");
            0
        }
    }
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
#[unsafe(no_mangle)]
pub extern "C" fn boxddd_debug_shape_destroy(token: u32, raw_shape: u32) {
    let Some(handle) = provider_take_shape(token, raw_shape) else {
        provider_record_diagnostic(
            token,
            "debug shape destroy referenced an unknown provider handle",
        );
        return;
    };
    if let Some(registry) = provider_registry(token) {
        unsafe { &*registry }.destroy_handle(handle);
    }
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
#[unsafe(no_mangle)]
pub extern "C" fn boxddd_debug_draw_shape(
    token: u32,
    raw_shape: u32,
    transform: *const ffi::b3WorldTransform,
    color: u32,
) -> i32 {
    let Some(transform) = (unsafe { provider_read(transform) }) else {
        provider_fail(token, "debug draw shape callback received a null transform");
        return 0;
    };
    let handle = provider_shape_handle(token, raw_shape);
    provider_push_command(
        token,
        DebugDrawCommand::Shape {
            handle,
            transform: WorldTransform::from_raw(transform),
            color: HexColor::from_raw(color),
        },
    ) as i32
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
#[unsafe(no_mangle)]
pub extern "C" fn boxddd_debug_draw_segment(
    token: u32,
    p1: *const ffi::b3Pos,
    p2: *const ffi::b3Pos,
    color: u32,
) {
    let (Some(p1), Some(p2)) = (unsafe { provider_read(p1) }, unsafe { provider_read(p2) }) else {
        provider_fail(
            token,
            "debug draw segment callback received a null endpoint",
        );
        return;
    };
    provider_push_command(
        token,
        DebugDrawCommand::Segment {
            p1: Pos::from_raw(p1),
            p2: Pos::from_raw(p2),
            color: HexColor::from_raw(color),
        },
    );
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
#[unsafe(no_mangle)]
pub extern "C" fn boxddd_debug_draw_transform(token: u32, transform: *const ffi::b3WorldTransform) {
    let Some(transform) = (unsafe { provider_read(transform) }) else {
        provider_fail(
            token,
            "debug draw transform callback received a null transform",
        );
        return;
    };
    provider_push_command(
        token,
        DebugDrawCommand::Transform(WorldTransform::from_raw(transform)),
    );
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
#[unsafe(no_mangle)]
pub extern "C" fn boxddd_debug_draw_point(
    token: u32,
    position: *const ffi::b3Pos,
    size: f32,
    color: u32,
) {
    let Some(position) = (unsafe { provider_read(position) }) else {
        provider_fail(token, "debug draw point callback received a null position");
        return;
    };
    provider_push_command(
        token,
        DebugDrawCommand::Point {
            position: Pos::from_raw(position),
            size,
            color: HexColor::from_raw(color),
        },
    );
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
#[unsafe(no_mangle)]
pub extern "C" fn boxddd_debug_draw_sphere(
    token: u32,
    center: *const ffi::b3Pos,
    radius: f32,
    color: u32,
    alpha: f32,
) {
    let Some(center) = (unsafe { provider_read(center) }) else {
        provider_fail(token, "debug draw sphere callback received a null center");
        return;
    };
    provider_push_command(
        token,
        DebugDrawCommand::Sphere {
            center: Pos::from_raw(center),
            radius,
            color: HexColor::from_raw(color),
            alpha,
        },
    );
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
#[unsafe(no_mangle)]
pub extern "C" fn boxddd_debug_draw_capsule(
    token: u32,
    p1: *const ffi::b3Pos,
    p2: *const ffi::b3Pos,
    radius: f32,
    color: u32,
    alpha: f32,
) {
    let (Some(p1), Some(p2)) = (unsafe { provider_read(p1) }, unsafe { provider_read(p2) }) else {
        provider_fail(
            token,
            "debug draw capsule callback received a null endpoint",
        );
        return;
    };
    provider_push_command(
        token,
        DebugDrawCommand::Capsule {
            p1: Pos::from_raw(p1),
            p2: Pos::from_raw(p2),
            radius,
            color: HexColor::from_raw(color),
            alpha,
        },
    );
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
#[unsafe(no_mangle)]
pub extern "C" fn boxddd_debug_draw_bounds(token: u32, aabb: *const ffi::b3AABB, color: u32) {
    let Some(aabb) = (unsafe { provider_read(aabb) }) else {
        provider_fail(token, "debug draw bounds callback received a null AABB");
        return;
    };
    provider_push_command(
        token,
        DebugDrawCommand::Bounds {
            aabb: Aabb::from_raw(aabb),
            color: HexColor::from_raw(color),
        },
    );
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
#[unsafe(no_mangle)]
pub extern "C" fn boxddd_debug_draw_box(
    token: u32,
    extents: *const ffi::b3Vec3,
    transform: *const ffi::b3WorldTransform,
    color: u32,
) {
    let (Some(extents), Some(transform)) = (unsafe { provider_read(extents) }, unsafe {
        provider_read(transform)
    }) else {
        provider_fail(token, "debug draw box callback received a null argument");
        return;
    };
    provider_push_command(
        token,
        DebugDrawCommand::Box {
            extents: Vec3::from_raw(extents),
            transform: WorldTransform::from_raw(transform),
            color: HexColor::from_raw(color),
        },
    );
}

#[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
#[unsafe(no_mangle)]
pub extern "C" fn boxddd_debug_draw_string(
    token: u32,
    position: *const ffi::b3Pos,
    text: *const std::ffi::c_char,
    color: u32,
) {
    let Some(position) = (unsafe { provider_read(position) }) else {
        provider_fail(token, "debug draw string callback received a null position");
        return;
    };
    if text.is_null() {
        provider_fail(
            token,
            "debug draw string callback received a null text pointer",
        );
        return;
    }
    let text = unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned();
    provider_push_command(
        token,
        DebugDrawCommand::String {
            position: Pos::from_raw(position),
            text,
            color: HexColor::from_raw(color),
        },
    );
}

#[inline]
const fn next_generation(current: u32) -> u32 {
    let next = current.wrapping_add(1);
    if next == 0 { 1 } else { next }
}

#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
pub(crate) unsafe extern "C" fn create_debug_shape(
    debug_shape: *const ffi::b3DebugShape,
    user_context: *mut c_void,
) -> *mut c_void {
    if debug_shape.is_null() || user_context.is_null() {
        return std::ptr::null_mut();
    }
    let registry = unsafe { &*(user_context as *const DebugShapeRegistry) };
    registry.create_native_resource(unsafe { &*debug_shape })
}

#[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
pub(crate) unsafe extern "C" fn destroy_debug_shape(
    user_shape: *mut c_void,
    user_context: *mut c_void,
) {
    if user_shape.is_null() {
        return;
    }
    let resource = unsafe { *Box::from_raw(user_shape as *mut DebugShapeResource) };
    if !user_context.is_null() {
        let registry = unsafe { &*(user_context as *const DebugShapeRegistry) };
        registry.destroy_native_resource(resource);
    }
}

struct DebugShapeSnapshot {
    shape_id: ShapeId,
    shape_type: ShapeType,
    geometry: DebugShapeGeometry,
}

type SnapshotResult<T> = std::result::Result<T, &'static str>;

unsafe fn snapshot_debug_shape(raw: &ffi::b3DebugShape) -> SnapshotResult<DebugShapeSnapshot> {
    let shape_type = ShapeType::from_raw(raw.type_).ok_or("debug shape has unknown shape type")?;
    let geometry = unsafe {
        match shape_type {
            ShapeType::Sphere => snapshot_sphere_ptr(raw.__bindgen_anon_1.sphere)?,
            ShapeType::Capsule => snapshot_capsule_ptr(raw.__bindgen_anon_1.capsule)?,
            ShapeType::Hull => snapshot_hull_ptr(raw.__bindgen_anon_1.hull)?,
            ShapeType::Mesh => snapshot_mesh_ptr(raw.__bindgen_anon_1.mesh)?,
            ShapeType::HeightField => snapshot_height_field_ptr(raw.__bindgen_anon_1.heightField)?,
            ShapeType::Compound => snapshot_compound_ptr(raw.__bindgen_anon_1.compound)?,
        }
    };
    Ok(DebugShapeSnapshot {
        shape_id: ShapeId::from_raw(raw.shapeId),
        shape_type,
        geometry,
    })
}

unsafe fn snapshot_child_shape(raw: ffi::b3ChildShape) -> SnapshotResult<DebugShapeGeometry> {
    unsafe {
        match ShapeType::from_raw(raw.type_).ok_or("compound child has unknown shape type")? {
            ShapeType::Sphere => Ok(snapshot_sphere(raw.__bindgen_anon_1.sphere)),
            ShapeType::Capsule => Ok(snapshot_capsule(raw.__bindgen_anon_1.capsule)),
            ShapeType::Hull => snapshot_hull_ptr(raw.__bindgen_anon_1.hull),
            ShapeType::Mesh => {
                let mesh = raw.__bindgen_anon_1.mesh;
                snapshot_mesh(&mesh)
            }
            ShapeType::Compound => Err("nested compound debug child is not supported by Box3D"),
            ShapeType::HeightField => Err("height-field debug child is not supported by Box3D"),
        }
    }
}

unsafe fn snapshot_sphere_ptr(ptr: *const ffi::b3Sphere) -> SnapshotResult<DebugShapeGeometry> {
    let sphere = unsafe { ptr.as_ref() }.ok_or("debug sphere pointer was null")?;
    Ok(snapshot_sphere(*sphere))
}

fn snapshot_sphere(raw: ffi::b3Sphere) -> DebugShapeGeometry {
    DebugShapeGeometry::Sphere {
        center: Vec3::from_raw(raw.center),
        radius: raw.radius,
    }
}

unsafe fn snapshot_capsule_ptr(ptr: *const ffi::b3Capsule) -> SnapshotResult<DebugShapeGeometry> {
    let capsule = unsafe { ptr.as_ref() }.ok_or("debug capsule pointer was null")?;
    Ok(snapshot_capsule(*capsule))
}

fn snapshot_capsule(raw: ffi::b3Capsule) -> DebugShapeGeometry {
    DebugShapeGeometry::Capsule {
        center1: Vec3::from_raw(raw.center1),
        center2: Vec3::from_raw(raw.center2),
        radius: raw.radius,
    }
}

unsafe fn snapshot_hull_ptr(ptr: *const ffi::b3HullData) -> SnapshotResult<DebugShapeGeometry> {
    let hull = unsafe { ptr.as_ref() }.ok_or("debug hull pointer was null")?;
    unsafe { snapshot_hull(hull) }
}

unsafe fn snapshot_hull(hull: &ffi::b3HullData) -> SnapshotResult<DebugShapeGeometry> {
    let points = unsafe {
        trailing_slice::<ffi::b3Vec3>(
            hull as *const _ as *const u8,
            hull.byteCount,
            hull.pointOffset,
            hull.vertexCount,
        )?
    }
    .iter()
    .copied()
    .map(Vec3::from_raw)
    .collect();
    let edges = unsafe {
        trailing_slice::<ffi::b3HullHalfEdge>(
            hull as *const _ as *const u8,
            hull.byteCount,
            hull.edgeOffset,
            hull.edgeCount,
        )?
    };
    let raw_faces = unsafe {
        trailing_slice::<ffi::b3HullFace>(
            hull as *const _ as *const u8,
            hull.byteCount,
            hull.faceOffset,
            hull.faceCount,
        )?
    };
    let planes = unsafe {
        trailing_slice::<ffi::b3Plane>(
            hull as *const _ as *const u8,
            hull.byteCount,
            hull.planeOffset,
            hull.faceCount,
        )?
    };

    let mut faces = Vec::with_capacity(raw_faces.len());
    for (face, plane) in raw_faces.iter().zip(planes) {
        let start = face.edge as usize;
        if start >= edges.len() {
            return Err("debug hull face edge index was out of range");
        }
        let mut indices = Vec::new();
        let mut edge_index = start;
        for _ in 0..=edges.len() {
            let edge = edges
                .get(edge_index)
                .ok_or("debug hull edge index was out of range")?;
            indices.push(edge.origin as u32);
            edge_index = edge.next as usize;
            if edge_index == start {
                break;
            }
        }
        if edge_index != start {
            return Err("debug hull face did not form a closed loop");
        }
        faces.push(DebugHullFace {
            indices,
            plane: Plane::from_raw(*plane),
        });
    }

    Ok(DebugShapeGeometry::Hull {
        aabb: Aabb::from_raw(hull.aabb),
        points,
        faces,
    })
}

unsafe fn snapshot_mesh_ptr(ptr: *const ffi::b3Mesh) -> SnapshotResult<DebugShapeGeometry> {
    let mesh = unsafe { ptr.as_ref() }.ok_or("debug mesh pointer was null")?;
    unsafe { snapshot_mesh(mesh) }
}

unsafe fn snapshot_mesh(mesh: &ffi::b3Mesh) -> SnapshotResult<DebugShapeGeometry> {
    let data = unsafe { mesh.data.as_ref() }.ok_or("debug mesh data pointer was null")?;
    let raw_vertices = unsafe {
        trailing_slice::<ffi::b3Vec3>(
            data as *const _ as *const u8,
            data.byteCount,
            data.vertexOffset,
            data.vertexCount,
        )?
    };
    let raw_triangles = unsafe {
        trailing_slice::<ffi::b3MeshTriangle>(
            data as *const _ as *const u8,
            data.byteCount,
            data.triangleOffset,
            data.triangleCount,
        )?
    };
    let raw_material_indices = if data.materialOffset == 0 {
        &[]
    } else {
        unsafe {
            trailing_slice::<u8>(
                data as *const _ as *const u8,
                data.byteCount,
                data.materialOffset,
                data.triangleCount,
            )?
        }
    };

    let vertices: Vec<_> = raw_vertices.iter().copied().map(Vec3::from_raw).collect();
    let mut triangles = Vec::with_capacity(raw_triangles.len());
    for (index, triangle) in raw_triangles.iter().enumerate() {
        let indices = [triangle.index1, triangle.index2, triangle.index3];
        if indices
            .iter()
            .any(|index| *index < 0 || *index as usize >= vertices.len())
        {
            return Err("debug mesh triangle index was out of range");
        }
        triangles.push(DebugMeshTriangle {
            indices: [indices[0] as u32, indices[1] as u32, indices[2] as u32],
            material_index: raw_material_indices.get(index).copied(),
        });
    }

    Ok(DebugShapeGeometry::Mesh {
        mesh: DebugMesh {
            bounds: Aabb::from_raw(data.bounds),
            vertices,
            triangles,
            material_count: data.materialCount,
        },
        scale: Vec3::from_raw(mesh.scale),
    })
}

unsafe fn snapshot_height_field_ptr(
    ptr: *const ffi::b3HeightFieldData,
) -> SnapshotResult<DebugShapeGeometry> {
    let height_field = unsafe { ptr.as_ref() }.ok_or("debug height-field pointer was null")?;
    unsafe { snapshot_height_field(height_field) }
}

unsafe fn snapshot_height_field(
    height_field: &ffi::b3HeightFieldData,
) -> SnapshotResult<DebugShapeGeometry> {
    let sample_count = checked_grid_count(height_field.rowCount, height_field.columnCount)?;
    let cell_count = checked_grid_count(height_field.rowCount - 1, height_field.columnCount - 1)?;
    let triangle_count = cell_count
        .checked_mul(2)
        .ok_or("debug height-field triangle count overflowed")?;
    let compressed_heights = unsafe {
        trailing_slice::<u16>(
            height_field as *const _ as *const u8,
            height_field.byteCount,
            height_field.heightsOffset,
            sample_count as i32,
        )?
    };
    let material_indices = unsafe {
        trailing_slice::<u8>(
            height_field as *const _ as *const u8,
            height_field.byteCount,
            height_field.materialOffset,
            cell_count as i32,
        )?
    };

    let mut vertices = Vec::with_capacity(sample_count);
    for row in 0..height_field.rowCount {
        for column in 0..height_field.columnCount {
            let index = (row * height_field.columnCount + column) as usize;
            let height = height_field.minHeight
                + height_field.heightScale * compressed_heights[index] as f32;
            vertices.push(Vec3::new(
                column as f32 * height_field.scale.x,
                height * height_field.scale.y,
                row as f32 * height_field.scale.z,
            ));
        }
    }

    let mut triangles = Vec::with_capacity(triangle_count);
    for row in 0..height_field.rowCount - 1 {
        for column in 0..height_field.columnCount - 1 {
            let cell_index = (row * (height_field.columnCount - 1) + column) as usize;
            let material_index = material_indices
                .get(cell_index)
                .copied()
                .ok_or("debug height-field material index was out of range")?;
            if material_index == ffi::B3_HEIGHT_FIELD_HOLE as u8 {
                continue;
            }
            let index11 = (row * height_field.columnCount + column) as u32;
            let index12 = index11 + 1;
            let index21 = ((row + 1) * height_field.columnCount + column) as u32;
            let index22 = index21 + 1;
            let first = if height_field.clockwise {
                [index11, index12, index21]
            } else {
                [index11, index21, index12]
            };
            let second = if height_field.clockwise {
                [index22, index21, index12]
            } else {
                [index22, index12, index21]
            };
            triangles.push(DebugMeshTriangle {
                indices: first,
                material_index: Some(material_index),
            });
            triangles.push(DebugMeshTriangle {
                indices: second,
                material_index: Some(material_index),
            });
        }
    }

    Ok(DebugShapeGeometry::HeightField {
        mesh: DebugMesh {
            bounds: Aabb::from_raw(height_field.aabb),
            vertices,
            triangles,
            material_count: 256,
        },
    })
}

unsafe fn snapshot_compound_ptr(
    ptr: *const ffi::b3CompoundData,
) -> SnapshotResult<DebugShapeGeometry> {
    let compound = unsafe { ptr.as_ref() }.ok_or("debug compound pointer was null")?;
    let total = compound
        .capsuleCount
        .checked_add(compound.hullCount)
        .and_then(|count| count.checked_add(compound.meshCount))
        .and_then(|count| count.checked_add(compound.sphereCount))
        .ok_or("debug compound child count overflowed")?;
    if total < 0 {
        return Err("debug compound child count was negative");
    }

    let mut children = Vec::with_capacity(total as usize);
    for index in 0..total {
        let child = unsafe { ffi::b3GetCompoundChild(compound, index) };
        children.push(DebugCompoundChild {
            transform: Transform::from_raw(child.transform),
            material_indices: child.materialIndices,
            geometry: unsafe { snapshot_child_shape(child)? },
        });
    }
    Ok(DebugShapeGeometry::Compound { children })
}

fn checked_grid_count(rows: i32, columns: i32) -> SnapshotResult<usize> {
    if rows <= 0 || columns <= 0 {
        return Err("debug height-field dimensions were invalid");
    }
    (rows as usize)
        .checked_mul(columns as usize)
        .ok_or("debug height-field dimensions overflowed")
}

unsafe fn trailing_slice<'a, T>(
    base: *const u8,
    byte_count: i32,
    offset: i32,
    count: i32,
) -> SnapshotResult<&'a [T]> {
    if base.is_null() || byte_count <= 0 || offset <= 0 || count < 0 {
        return Err("debug shape trailing storage was invalid");
    }
    let start = offset as usize;
    let len = count as usize;
    let bytes = len
        .checked_mul(mem::size_of::<T>())
        .ok_or("debug shape trailing storage length overflowed")?;
    let end = start
        .checked_add(bytes)
        .ok_or("debug shape trailing storage end overflowed")?;
    if end > byte_count as usize {
        return Err("debug shape trailing storage exceeded its allocation");
    }
    let ptr = unsafe { base.add(start) as *const T };
    Ok(unsafe { slice::from_raw_parts(ptr, len) })
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

fn debug_shape_handle_from_user_shape(user_shape: *mut c_void) -> Option<DebugShapeHandle> {
    if user_shape.is_null() {
        None
    } else {
        Some(unsafe { (*(user_shape as *const DebugShapeResource)).handle })
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
        handle: Option<DebugShapeHandle>,
        transform: WorldTransform,
        color: HexColor,
    ) {
        self.replace_or_push(DebugDrawCommand::Shape {
            handle,
            transform,
            color,
        });
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
    run_debug_draw_callback(context, (), |drawer| {
        drawer.draw_shape(
            debug_shape_handle_from_user_shape(user_shape),
            WorldTransform::from_raw(transform),
            HexColor::from_ffi(color),
        );
    });
    !context.panicked
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
            HexColor::from_ffi(color),
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
        drawer.draw_point(Pos::from_raw(position), size, HexColor::from_ffi(color));
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
            HexColor::from_ffi(color),
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
            HexColor::from_ffi(color),
            alpha,
        );
    });
}

unsafe extern "C" fn draw_bounds(aabb: ffi::b3AABB, color: ffi::b3HexColor, context: *mut c_void) {
    let context = unsafe { &mut *(context as *mut DebugDrawContext<'_>) };
    run_debug_draw_callback(context, (), |drawer| {
        drawer.draw_bounds(Aabb::from_raw(aabb), HexColor::from_ffi(color));
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
            HexColor::from_ffi(color),
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
        drawer.draw_string(Pos::from_raw(position), &text, HexColor::from_ffi(color));
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
        validate_options(options)?;

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

fn validate_options(options: DebugDrawOptions) -> Result<()> {
    if options.force_scale.is_finite()
        && options.joint_scale.is_finite()
        && options.drawing_bounds.is_valid()
    {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}

impl World {
    /// Collects a lifecycle-aware debug draw frame or panics if Box3D rejects the draw.
    pub fn debug_draw_frame(&mut self, options: DebugDrawOptions) -> DebugDrawFrame {
        self.try_debug_draw_frame(options)
            .expect("Box3D debug draw failed")
    }

    /// Tries to collect a lifecycle-aware debug draw frame.
    pub fn try_debug_draw_frame(&mut self, options: DebugDrawOptions) -> Result<DebugDrawFrame> {
        let mut frame = DebugDrawFrame::default();
        self.try_debug_draw_frame_into(&mut frame, options)?;
        Ok(frame)
    }

    /// Collects a debug draw frame into `out` or panics if Box3D rejects the draw.
    pub fn debug_draw_frame_into(&mut self, out: &mut DebugDrawFrame, options: DebugDrawOptions) {
        self.try_debug_draw_frame_into(out, options)
            .expect("Box3D debug draw failed");
    }

    /// Tries to collect a debug draw frame into `out`.
    pub fn try_debug_draw_frame_into(
        &mut self,
        out: &mut DebugDrawFrame,
        options: DebugDrawOptions,
    ) -> Result<()> {
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            out.clear();
            validate_options(options)?;
            callback_state::check_not_in_callback()?;

            let token = self.provider_debug_shapes_token;
            if token == 0 {
                return Err(Error::UnsupportedOnWasm);
            }

            let frame_guard = ProviderDebugFrameGuard::new(token, &mut out.commands);
            let mut draw = unsafe { ffi::b3DefaultDebugDraw() };
            unsafe { ffi::boxddd_provider_debug_init_draw(&mut draw, token) };
            apply_options(&mut draw, options, token as usize as *mut c_void);

            let result = {
                let _guard = box3d_lock::lock();
                self.check_world_valid_locked()?;
                unsafe { ffi::b3World_Draw(self.raw(), &mut draw, options.mask_bits) };
                Ok(())
            };
            drop(frame_guard);

            let provider_error = unsafe { ffi::boxddd_provider_debug_take_error(token) };
            if provider_error != 0 {
                boxddd_debug_report_error(token, provider_error as u32);
            }
            let result = result.and_then(|()| match take_provider_debug_error(token) {
                Some(error) => Err(error),
                None => Ok(()),
            });
            self.debug_shapes.drain_into(out);
            result
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            out.clear();
            let mut collector = CollectDebugDraw::new(&mut out.commands);
            let result = self.try_debug_draw(&mut collector, options);
            collector.finish();
            self.debug_shapes.drain_into(out);
            result
        }
    }

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
        let mut frame = DebugDrawFrame {
            commands: mem::take(out),
            ..DebugDrawFrame::default()
        };
        let result = self.try_debug_draw_frame_into(&mut frame, options);
        *out = frame.commands;
        result
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
