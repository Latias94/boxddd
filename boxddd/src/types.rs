use boxddd_sys::ffi;

#[inline]
pub fn is_valid_float(value: f32) -> bool {
    unsafe { ffi::b3IsValidFloat(value) }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3Vec2) -> Self {
        Self { x: raw.x, y: raw.y }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Vec2 {
        ffi::b3Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl From<[f32; 2]> for Vec2 {
    #[inline]
    fn from(value: [f32; 2]) -> Self {
        Self::new(value[0], value[1])
    }
}

impl From<(f32, f32)> for Vec2 {
    #[inline]
    fn from(value: (f32, f32)) -> Self {
        Self::new(value.0, value.1)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    pub const X: Self = Self::new(1.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3Vec3) -> Self {
        Self {
            x: raw.x,
            y: raw.y,
            z: raw.z,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Vec3 {
        ffi::b3Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3IsValidVec3(self.into_raw()) }
    }

    #[inline]
    pub fn validate(self) -> crate::error::Result<Self> {
        if self.is_valid() {
            Ok(self)
        } else {
            Err(crate::error::Error::InvalidArgument)
        }
    }
}

impl From<[f32; 3]> for Vec3 {
    #[inline]
    fn from(value: [f32; 3]) -> Self {
        Self::new(value[0], value[1], value[2])
    }
}

impl From<(f32, f32, f32)> for Vec3 {
    #[inline]
    fn from(value: (f32, f32, f32)) -> Self {
        Self::new(value.0, value.1, value.2)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Quat {
    pub v: Vec3,
    pub s: f32,
}

impl Quat {
    pub const IDENTITY: Self = Self {
        v: Vec3::ZERO,
        s: 1.0,
    };

    #[inline]
    pub const fn new(v: Vec3, s: f32) -> Self {
        Self { v, s }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3Quat) -> Self {
        Self {
            v: Vec3::from_raw(raw.v),
            s: raw.s,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Quat {
        ffi::b3Quat {
            v: self.v.into_raw(),
            s: self.s,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3IsValidQuat(self.into_raw()) }
    }

    #[inline]
    pub fn validate(self) -> crate::error::Result<Self> {
        if self.is_valid() {
            Ok(self)
        } else {
            Err(crate::error::Error::InvalidArgument)
        }
    }
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Transform {
    pub p: Vec3,
    pub q: Quat,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        p: Vec3::ZERO,
        q: Quat::IDENTITY,
    };

    #[inline]
    pub const fn new(p: Vec3, q: Quat) -> Self {
        Self { p, q }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3Transform) -> Self {
        Self {
            p: Vec3::from_raw(raw.p),
            q: Quat::from_raw(raw.q),
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Transform {
        ffi::b3Transform {
            p: self.p.into_raw(),
            q: self.q.into_raw(),
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3IsValidTransform(self.into_raw()) }
    }

    #[inline]
    pub fn validate(self) -> crate::error::Result<Self> {
        if self.is_valid() {
            Ok(self)
        } else {
            Err(crate::error::Error::InvalidArgument)
        }
    }
}

#[cfg(not(feature = "double-precision"))]
pub type PosScalar = f32;
#[cfg(feature = "double-precision")]
pub type PosScalar = f64;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Pos {
    pub x: PosScalar,
    pub y: PosScalar,
    pub z: PosScalar,
}

impl Pos {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    #[inline]
    pub const fn new(x: PosScalar, y: PosScalar, z: PosScalar) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3Pos) -> Self {
        Self {
            x: raw.x,
            y: raw.y,
            z: raw.z,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Pos {
        ffi::b3Pos {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3IsValidPosition(self.into_raw()) }
    }

    #[inline]
    pub fn validate(self) -> crate::error::Result<Self> {
        if self.is_valid() {
            Ok(self)
        } else {
            Err(crate::error::Error::InvalidArgument)
        }
    }
}

impl From<Vec3> for Pos {
    #[inline]
    fn from(value: Vec3) -> Self {
        Self::new(value.x.into(), value.y.into(), value.z.into())
    }
}

impl From<[f32; 3]> for Pos {
    #[inline]
    fn from(value: [f32; 3]) -> Self {
        Self::from(Vec3::from(value))
    }
}

impl From<(f32, f32, f32)> for Pos {
    #[inline]
    fn from(value: (f32, f32, f32)) -> Self {
        Self::from(Vec3::from(value))
    }
}

#[cfg(feature = "double-precision")]
impl From<[f64; 3]> for Pos {
    #[inline]
    fn from(value: [f64; 3]) -> Self {
        Self::new(value[0], value[1], value[2])
    }
}

#[cfg(feature = "double-precision")]
impl From<(f64, f64, f64)> for Pos {
    #[inline]
    fn from(value: (f64, f64, f64)) -> Self {
        Self::new(value.0, value.1, value.2)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct WorldTransform {
    pub p: Pos,
    pub q: Quat,
}

impl WorldTransform {
    pub const IDENTITY: Self = Self {
        p: Pos::ZERO,
        q: Quat::IDENTITY,
    };

    #[inline]
    pub const fn new(p: Pos, q: Quat) -> Self {
        Self { p, q }
    }

    #[inline]
    pub const fn from_raw(raw: ffi::b3WorldTransform) -> Self {
        Self {
            p: Pos::from_raw(raw.p),
            q: Quat::from_raw(raw.q),
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3WorldTransform {
        ffi::b3WorldTransform {
            p: self.p.into_raw(),
            q: self.q.into_raw(),
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3IsValidWorldTransform(self.into_raw()) }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Matrix3 {
    pub cx: Vec3,
    pub cy: Vec3,
    pub cz: Vec3,
}

impl Matrix3 {
    #[inline]
    pub const fn from_raw(raw: ffi::b3Matrix3) -> Self {
        Self {
            cx: Vec3::from_raw(raw.cx),
            cy: Vec3::from_raw(raw.cy),
            cz: Vec3::from_raw(raw.cz),
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Matrix3 {
        ffi::b3Matrix3 {
            cx: self.cx.into_raw(),
            cy: self.cy.into_raw(),
            cz: self.cz.into_raw(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Aabb {
    pub lower_bound: Vec3,
    pub upper_bound: Vec3,
}

impl Aabb {
    #[inline]
    pub const fn from_raw(raw: ffi::b3AABB) -> Self {
        Self {
            lower_bound: Vec3::from_raw(raw.lowerBound),
            upper_bound: Vec3::from_raw(raw.upperBound),
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3AABB {
        ffi::b3AABB {
            lowerBound: self.lower_bound.into_raw(),
            upperBound: self.upper_bound.into_raw(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Plane {
    pub normal: Vec3,
    pub offset: f32,
}

impl Plane {
    #[inline]
    pub const fn from_raw(raw: ffi::b3Plane) -> Self {
        Self {
            normal: Vec3::from_raw(raw.normal),
            offset: raw.offset,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Plane {
        ffi::b3Plane {
            normal: self.normal.into_raw(),
            offset: self.offset,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    pub category_bits: u64,
    pub mask_bits: u64,
    pub group_index: i32,
}

impl Filter {
    #[inline]
    pub const fn from_raw(raw: ffi::b3Filter) -> Self {
        Self {
            category_bits: raw.categoryBits,
            mask_bits: raw.maskBits,
            group_index: raw.groupIndex,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Filter {
        ffi::b3Filter {
            categoryBits: self.category_bits,
            maskBits: self.mask_bits,
            groupIndex: self.group_index,
        }
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::from_raw(unsafe { ffi::b3DefaultFilter() })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct MassData {
    pub mass: f32,
    pub center: Vec3,
    pub inertia: Matrix3,
}

impl MassData {
    #[inline]
    pub const fn from_raw(raw: ffi::b3MassData) -> Self {
        Self {
            mass: raw.mass,
            center: Vec3::from_raw(raw.center),
            inertia: Matrix3::from_raw(raw.inertia),
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3MassData {
        ffi::b3MassData {
            mass: self.mass,
            center: self.center.into_raw(),
            inertia: self.inertia.into_raw(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Capacity {
    pub static_shape_count: i32,
    pub dynamic_shape_count: i32,
    pub static_body_count: i32,
    pub dynamic_body_count: i32,
    pub contact_count: i32,
}

impl Capacity {
    #[inline]
    pub const fn from_raw(raw: ffi::b3Capacity) -> Self {
        Self {
            static_shape_count: raw.staticShapeCount,
            dynamic_shape_count: raw.dynamicShapeCount,
            static_body_count: raw.staticBodyCount,
            dynamic_body_count: raw.dynamicBodyCount,
            contact_count: raw.contactCount,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3Capacity {
        ffi::b3Capacity {
            staticShapeCount: self.static_shape_count,
            dynamicShapeCount: self.dynamic_shape_count,
            staticBodyCount: self.static_body_count,
            dynamicBodyCount: self.dynamic_body_count,
            contactCount: self.contact_count,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Profile {
    pub step: f32,
    pub pairs: f32,
    pub collide: f32,
    pub solve: f32,
    pub solver_setup: f32,
    pub constraints: f32,
    pub prepare_constraints: f32,
    pub integrate_velocities: f32,
    pub warm_start: f32,
    pub solve_impulses: f32,
    pub integrate_positions: f32,
    pub relax_impulses: f32,
    pub apply_restitution: f32,
    pub store_impulses: f32,
    pub split_islands: f32,
    pub transforms: f32,
    pub sensor_hits: f32,
    pub joint_events: f32,
    pub hit_events: f32,
    pub refit: f32,
    pub bullets: f32,
    pub sleep_islands: f32,
    pub sensors: f32,
}

impl Profile {
    #[inline]
    pub const fn from_raw(raw: ffi::b3Profile) -> Self {
        Self {
            step: raw.step,
            pairs: raw.pairs,
            collide: raw.collide,
            solve: raw.solve,
            solver_setup: raw.solverSetup,
            constraints: raw.constraints,
            prepare_constraints: raw.prepareConstraints,
            integrate_velocities: raw.integrateVelocities,
            warm_start: raw.warmStart,
            solve_impulses: raw.solveImpulses,
            integrate_positions: raw.integratePositions,
            relax_impulses: raw.relaxImpulses,
            apply_restitution: raw.applyRestitution,
            store_impulses: raw.storeImpulses,
            split_islands: raw.splitIslands,
            transforms: raw.transforms,
            sensor_hits: raw.sensorHits,
            joint_events: raw.jointEvents,
            hit_events: raw.hitEvents,
            refit: raw.refit,
            bullets: raw.bullets,
            sleep_islands: raw.sleepIslands,
            sensors: raw.sensors,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Counters {
    pub body_count: i32,
    pub shape_count: i32,
    pub contact_count: i32,
    pub joint_count: i32,
    pub island_count: i32,
    pub stack_used: i32,
    pub arena_capacity: i32,
    pub static_tree_height: i32,
    pub tree_height: i32,
    pub sat_call_count: i32,
    pub sat_cache_hit_count: i32,
    pub byte_count: i32,
    pub task_count: i32,
    pub color_counts: [i32; 24],
    pub manifold_counts: [i32; 8],
    pub awake_contact_count: i32,
    pub recycled_contact_count: i32,
    pub distance_iterations: i32,
    pub push_back_iterations: i32,
    pub root_iterations: i32,
}

impl Counters {
    #[inline]
    pub const fn from_raw(raw: ffi::b3Counters) -> Self {
        Self {
            body_count: raw.bodyCount,
            shape_count: raw.shapeCount,
            contact_count: raw.contactCount,
            joint_count: raw.jointCount,
            island_count: raw.islandCount,
            stack_used: raw.stackUsed,
            arena_capacity: raw.arenaCapacity,
            static_tree_height: raw.staticTreeHeight,
            tree_height: raw.treeHeight,
            sat_call_count: raw.satCallCount,
            sat_cache_hit_count: raw.satCacheHitCount,
            byte_count: raw.byteCount,
            task_count: raw.taskCount,
            color_counts: raw.colorCounts,
            manifold_counts: raw.manifoldCounts,
            awake_contact_count: raw.awakeContactCount,
            recycled_contact_count: raw.recycledContactCount,
            distance_iterations: raw.distanceIterations,
            push_back_iterations: raw.pushBackIterations,
            root_iterations: raw.rootIterations,
        }
    }
}

impl Default for Counters {
    fn default() -> Self {
        Self {
            body_count: 0,
            shape_count: 0,
            contact_count: 0,
            joint_count: 0,
            island_count: 0,
            stack_used: 0,
            arena_capacity: 0,
            static_tree_height: 0,
            tree_height: 0,
            sat_call_count: 0,
            sat_cache_hit_count: 0,
            byte_count: 0,
            task_count: 0,
            color_counts: [0; 24],
            manifold_counts: [0; 8],
            awake_contact_count: 0,
            recycled_contact_count: 0,
            distance_iterations: 0,
            push_back_iterations: 0,
            root_iterations: 0,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub revision: i32,
}

impl Version {
    #[inline]
    pub const fn from_raw(raw: ffi::b3Version) -> Self {
        Self {
            major: raw.major,
            minor: raw.minor,
            revision: raw.revision,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BodyId {
    pub index1: i32,
    pub world0: u16,
    pub generation: u16,
}

impl BodyId {
    #[inline]
    pub const fn from_raw(raw: ffi::b3BodyId) -> Self {
        Self {
            index1: raw.index1,
            world0: raw.world0,
            generation: raw.generation,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3BodyId {
        ffi::b3BodyId {
            index1: self.index1,
            world0: self.world0,
            generation: self.generation,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3Body_IsValid(self.into_raw()) }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ShapeId {
    pub index1: i32,
    pub world0: u16,
    pub generation: u16,
}

impl ShapeId {
    #[inline]
    pub const fn from_raw(raw: ffi::b3ShapeId) -> Self {
        Self {
            index1: raw.index1,
            world0: raw.world0,
            generation: raw.generation,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3ShapeId {
        ffi::b3ShapeId {
            index1: self.index1,
            world0: self.world0,
            generation: self.generation,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3Shape_IsValid(self.into_raw()) }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct JointId {
    pub index1: i32,
    pub world0: u16,
    pub generation: u16,
}

impl JointId {
    #[inline]
    pub const fn from_raw(raw: ffi::b3JointId) -> Self {
        Self {
            index1: raw.index1,
            world0: raw.world0,
            generation: raw.generation,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3JointId {
        ffi::b3JointId {
            index1: self.index1,
            world0: self.world0,
            generation: self.generation,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3Joint_IsValid(self.into_raw()) }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ContactId {
    pub index1: i32,
    pub world0: u16,
    pub padding: i16,
    pub generation: u32,
}

impl ContactId {
    #[inline]
    pub const fn from_raw(raw: ffi::b3ContactId) -> Self {
        Self {
            index1: raw.index1,
            world0: raw.world0,
            padding: raw.padding,
            generation: raw.generation,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3ContactId {
        ffi::b3ContactId {
            index1: self.index1,
            world0: self.world0,
            padding: self.padding,
            generation: self.generation,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { ffi::b3Contact_IsValid(self.into_raw()) }
    }
}

const _: () = {
    assert!(::core::mem::size_of::<Vec2>() == ::core::mem::size_of::<ffi::b3Vec2>());
    assert!(::core::mem::align_of::<Vec2>() == ::core::mem::align_of::<ffi::b3Vec2>());
    assert!(::core::mem::size_of::<Vec3>() == ::core::mem::size_of::<ffi::b3Vec3>());
    assert!(::core::mem::align_of::<Vec3>() == ::core::mem::align_of::<ffi::b3Vec3>());
    assert!(::core::mem::size_of::<Quat>() == ::core::mem::size_of::<ffi::b3Quat>());
    assert!(::core::mem::align_of::<Quat>() == ::core::mem::align_of::<ffi::b3Quat>());
    assert!(::core::mem::size_of::<Transform>() == ::core::mem::size_of::<ffi::b3Transform>());
    assert!(::core::mem::align_of::<Transform>() == ::core::mem::align_of::<ffi::b3Transform>());
    assert!(::core::mem::size_of::<Pos>() == ::core::mem::size_of::<ffi::b3Pos>());
    assert!(::core::mem::align_of::<Pos>() == ::core::mem::align_of::<ffi::b3Pos>());
    assert!(
        ::core::mem::size_of::<WorldTransform>() == ::core::mem::size_of::<ffi::b3WorldTransform>()
    );
    assert!(
        ::core::mem::align_of::<WorldTransform>()
            == ::core::mem::align_of::<ffi::b3WorldTransform>()
    );
    assert!(::core::mem::size_of::<Matrix3>() == ::core::mem::size_of::<ffi::b3Matrix3>());
    assert!(::core::mem::align_of::<Matrix3>() == ::core::mem::align_of::<ffi::b3Matrix3>());
    assert!(::core::mem::size_of::<Aabb>() == ::core::mem::size_of::<ffi::b3AABB>());
    assert!(::core::mem::align_of::<Aabb>() == ::core::mem::align_of::<ffi::b3AABB>());
    assert!(::core::mem::size_of::<Plane>() == ::core::mem::size_of::<ffi::b3Plane>());
    assert!(::core::mem::align_of::<Plane>() == ::core::mem::align_of::<ffi::b3Plane>());
    assert!(::core::mem::size_of::<Filter>() == ::core::mem::size_of::<ffi::b3Filter>());
    assert!(::core::mem::align_of::<Filter>() == ::core::mem::align_of::<ffi::b3Filter>());
    assert!(::core::mem::size_of::<MassData>() == ::core::mem::size_of::<ffi::b3MassData>());
    assert!(::core::mem::align_of::<MassData>() == ::core::mem::align_of::<ffi::b3MassData>());
    assert!(::core::mem::size_of::<Capacity>() == ::core::mem::size_of::<ffi::b3Capacity>());
    assert!(::core::mem::align_of::<Capacity>() == ::core::mem::align_of::<ffi::b3Capacity>());
    assert!(::core::mem::size_of::<BodyId>() == ::core::mem::size_of::<ffi::b3BodyId>());
    assert!(::core::mem::align_of::<BodyId>() == ::core::mem::align_of::<ffi::b3BodyId>());
    assert!(::core::mem::size_of::<ShapeId>() == ::core::mem::size_of::<ffi::b3ShapeId>());
    assert!(::core::mem::align_of::<ShapeId>() == ::core::mem::align_of::<ffi::b3ShapeId>());
    assert!(::core::mem::size_of::<JointId>() == ::core::mem::size_of::<ffi::b3JointId>());
    assert!(::core::mem::align_of::<JointId>() == ::core::mem::align_of::<ffi::b3JointId>());
    assert!(::core::mem::size_of::<ContactId>() == ::core::mem::size_of::<ffi::b3ContactId>());
    assert!(::core::mem::align_of::<ContactId>() == ::core::mem::align_of::<ffi::b3ContactId>());
};
