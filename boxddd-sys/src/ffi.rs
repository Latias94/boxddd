use std::ffi::{c_char, c_void};

pub type b3BodyType = u32;
pub const b3BodyType_b3_staticBody: b3BodyType = 0;
pub const b3BodyType_b3_kinematicBody: b3BodyType = 1;
pub const b3BodyType_b3_dynamicBody: b3BodyType = 2;

pub type b3ShapeType = u32;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct b3WorldId {
    pub index1: u16,
    pub generation: u16,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct b3BodyId {
    pub index1: i32,
    pub world0: u16,
    pub generation: u16,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct b3ShapeId {
    pub index1: i32,
    pub world0: u16,
    pub generation: u16,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct b3JointId {
    pub index1: i32,
    pub world0: u16,
    pub generation: u16,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct b3ContactId {
    pub index1: i32,
    pub world0: u16,
    pub padding: i16,
    pub generation: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct b3Vec2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct b3Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub type b3Pos = b3Vec3;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct b3CosSin {
    pub cosine: f32,
    pub sine: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct b3Quat {
    pub v: b3Vec3,
    pub s: f32,
}

impl Default for b3Quat {
    fn default() -> Self {
        Self {
            v: b3Vec3::default(),
            s: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct b3Transform {
    pub p: b3Vec3,
    pub q: b3Quat,
}

pub type b3WorldTransform = b3Transform;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct b3Matrix3 {
    pub cx: b3Vec3,
    pub cy: b3Vec3,
    pub cz: b3Vec3,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct b3AABB {
    pub lowerBound: b3Vec3,
    pub upperBound: b3Vec3,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct b3Plane {
    pub normal: b3Vec3,
    pub offset: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct b3Version {
    pub major: i32,
    pub minor: i32,
    pub revision: i32,
}

pub type b3TaskCallback = unsafe extern "C" fn(task_context: *mut c_void);
pub type b3EnqueueTaskCallback = unsafe extern "C" fn(
    task: Option<b3TaskCallback>,
    task_context: *mut c_void,
    user_context: *mut c_void,
    task_name: *const c_char,
) -> *mut c_void;
pub type b3FinishTaskCallback =
    unsafe extern "C" fn(user_task: *mut c_void, user_context: *mut c_void);
pub type b3CreateDebugShapeCallback =
    unsafe extern "C" fn(debug_shape: *const c_void, user_context: *mut c_void) -> *mut c_void;
pub type b3DestroyDebugShapeCallback =
    unsafe extern "C" fn(user_shape: *mut c_void, user_context: *mut c_void);
pub type b3FrictionCallback = unsafe extern "C" fn(
    friction_a: f32,
    user_material_id_a: u64,
    friction_b: f32,
    user_material_id_b: u64,
) -> f32;
pub type b3RestitutionCallback = unsafe extern "C" fn(
    restitution_a: f32,
    user_material_id_a: u64,
    restitution_b: f32,
    user_material_id_b: u64,
) -> f32;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct b3Capacity {
    pub staticShapeCount: i32,
    pub dynamicShapeCount: i32,
    pub staticBodyCount: i32,
    pub dynamicBodyCount: i32,
    pub contactCount: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct b3WorldDef {
    pub gravity: b3Vec3,
    pub restitutionThreshold: f32,
    pub hitEventThreshold: f32,
    pub contactHertz: f32,
    pub contactDampingRatio: f32,
    pub contactSpeed: f32,
    pub maximumLinearSpeed: f32,
    pub frictionCallback: Option<b3FrictionCallback>,
    pub restitutionCallback: Option<b3RestitutionCallback>,
    pub enableSleep: bool,
    pub enableContinuous: bool,
    pub workerCount: u32,
    pub enqueueTask: Option<b3EnqueueTaskCallback>,
    pub finishTask: Option<b3FinishTaskCallback>,
    pub userTaskContext: *mut c_void,
    pub userData: *mut c_void,
    pub createDebugShape: Option<b3CreateDebugShapeCallback>,
    pub destroyDebugShape: Option<b3DestroyDebugShapeCallback>,
    pub userDebugShapeContext: *mut c_void,
    pub capacity: b3Capacity,
    pub internalValue: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct b3MotionLocks {
    pub linearX: bool,
    pub linearY: bool,
    pub linearZ: bool,
    pub angularX: bool,
    pub angularY: bool,
    pub angularZ: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct b3BodyDef {
    pub type_: b3BodyType,
    pub position: b3Pos,
    pub rotation: b3Quat,
    pub linearVelocity: b3Vec3,
    pub angularVelocity: b3Vec3,
    pub linearDamping: f32,
    pub angularDamping: f32,
    pub gravityScale: f32,
    pub sleepThreshold: f32,
    pub name: *const c_char,
    pub userData: *mut c_void,
    pub motionLocks: b3MotionLocks,
    pub enableSleep: bool,
    pub isAwake: bool,
    pub isBullet: bool,
    pub isEnabled: bool,
    pub allowFastRotation: bool,
    pub enableContactRecycling: bool,
    pub internalValue: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct b3Filter {
    pub categoryBits: u64,
    pub maskBits: u64,
    pub groupIndex: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct b3SurfaceMaterial {
    pub friction: f32,
    pub restitution: f32,
    pub rollingResistance: f32,
    pub tangentVelocity: b3Vec3,
    pub userMaterialId: u64,
    pub customColor: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct b3ShapeDef {
    pub userData: *mut c_void,
    pub materials: *mut b3SurfaceMaterial,
    pub materialCount: i32,
    pub baseMaterial: b3SurfaceMaterial,
    pub density: f32,
    pub explosionScale: f32,
    pub filter: b3Filter,
    pub enableCustomFiltering: bool,
    pub isSensor: bool,
    pub enableSensorEvents: bool,
    pub enableContactEvents: bool,
    pub enableHitEvents: bool,
    pub enablePreSolveEvents: bool,
    pub invokeContactCreation: bool,
    pub updateBodyMass: bool,
    pub internalValue: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct b3MassData {
    pub mass: f32,
    pub center: b3Vec3,
    pub inertia: b3Matrix3,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct b3Sphere {
    pub center: b3Vec3,
    pub radius: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct b3Capsule {
    pub center1: b3Vec3,
    pub center2: b3Vec3,
    pub radius: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct b3HullVertex {
    pub edge: u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct b3HullHalfEdge {
    pub next: u8,
    pub twin: u8,
    pub origin: u8,
    pub face: u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct b3HullFace {
    pub edge: u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct b3HullData {
    pub version: u64,
    pub byteCount: i32,
    pub hash: u32,
    pub aabb: b3AABB,
    pub surfaceArea: f32,
    pub volume: f32,
    pub innerRadius: f32,
    pub center: b3Vec3,
    pub centralInertia: b3Matrix3,
    pub vertexCount: i32,
    pub vertexOffset: i32,
    pub pointOffset: i32,
    pub edgeCount: i32,
    pub edgeOffset: i32,
    pub faceCount: i32,
    pub faceOffset: i32,
    pub planeOffset: i32,
    pub padding: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct b3BoxHull {
    pub base: b3HullData,
    pub boxVertices: [b3HullVertex; 8],
    pub boxPoints: [b3Vec3; 8],
    pub boxEdges: [b3HullHalfEdge; 24],
    pub boxFaces: [b3HullFace; 6],
    pub padding: [u8; 2],
    pub boxPlanes: [b3Plane; 6],
}

#[allow(improper_ctypes)]
unsafe extern "C" {
    pub fn b3GetByteCount() -> i32;
    pub fn b3GetVersion() -> b3Version;
    pub fn b3IsDoublePrecision() -> bool;
    pub fn b3IsValidFloat(a: f32) -> bool;
    pub fn b3IsValidVec3(a: b3Vec3) -> bool;
    pub fn b3IsValidQuat(q: b3Quat) -> bool;
    pub fn b3IsValidTransform(a: b3Transform) -> bool;

    pub fn b3DefaultWorldDef() -> b3WorldDef;
    pub fn b3CreateWorld(def: *const b3WorldDef) -> b3WorldId;
    pub fn b3DestroyWorld(world_id: b3WorldId);
    pub fn b3World_IsValid(id: b3WorldId) -> bool;
    pub fn b3World_Step(world_id: b3WorldId, time_step: f32, sub_step_count: i32);
    pub fn b3World_GetGravity(world_id: b3WorldId) -> b3Vec3;
    pub fn b3World_SetGravity(world_id: b3WorldId, gravity: b3Vec3);

    pub fn b3DefaultBodyDef() -> b3BodyDef;
    pub fn b3CreateBody(world_id: b3WorldId, def: *const b3BodyDef) -> b3BodyId;
    pub fn b3DestroyBody(body_id: b3BodyId);
    pub fn b3Body_IsValid(id: b3BodyId) -> bool;
    pub fn b3Body_GetPosition(body_id: b3BodyId) -> b3Pos;
    pub fn b3Body_GetRotation(body_id: b3BodyId) -> b3Quat;

    pub fn b3DefaultFilter() -> b3Filter;
    pub fn b3DefaultSurfaceMaterial() -> b3SurfaceMaterial;
    pub fn b3DefaultShapeDef() -> b3ShapeDef;
    pub fn b3CreateSphereShape(
        body_id: b3BodyId,
        def: *const b3ShapeDef,
        sphere: *const b3Sphere,
    ) -> b3ShapeId;
    pub fn b3CreateHullShape(
        body_id: b3BodyId,
        def: *const b3ShapeDef,
        hull: *const b3HullData,
    ) -> b3ShapeId;
    pub fn b3DestroyShape(shape_id: b3ShapeId, update_body_mass: bool);
    pub fn b3Shape_IsValid(id: b3ShapeId) -> bool;

    pub fn b3MakeCubeHull(half_width: f32) -> b3BoxHull;
    pub fn b3MakeBoxHull(hx: f32, hy: f32, hz: f32) -> b3BoxHull;
}
