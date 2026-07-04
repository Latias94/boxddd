//! Runtime capacity, profiling, counter, and version snapshots.

use super::*;

/// Native world capacity snapshot.
#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Capacity {
    /// Static shape capacity.
    pub static_shape_count: i32,
    /// Dynamic shape capacity.
    pub dynamic_shape_count: i32,
    /// Static body capacity.
    pub static_body_count: i32,
    /// Dynamic body capacity.
    pub dynamic_body_count: i32,
    /// Contact capacity.
    pub contact_count: i32,
}

impl Capacity {
    /// Converts a raw Box3D capacity snapshot into the Rust value type.
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

    /// Converts this value into the raw Box3D representation.
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

/// Per-step timing profile reported by Box3D.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Profile {
    /// Total step time.
    pub step: f32,
    /// Broad-phase pair update time.
    pub pairs: f32,
    /// Collision detection time.
    pub collide: f32,
    /// Solver time.
    pub solve: f32,
    /// Solver setup time.
    pub solver_setup: f32,
    /// Constraint solve time.
    pub constraints: f32,
    /// Constraint preparation time.
    pub prepare_constraints: f32,
    /// Velocity integration time.
    pub integrate_velocities: f32,
    /// Warm-start time.
    pub warm_start: f32,
    /// Impulse solve time.
    pub solve_impulses: f32,
    /// Position integration time.
    pub integrate_positions: f32,
    /// Impulse relaxation time.
    pub relax_impulses: f32,
    /// Restitution application time.
    pub apply_restitution: f32,
    /// Impulse storage time.
    pub store_impulses: f32,
    /// Island splitting time.
    pub split_islands: f32,
    /// Transform update time.
    pub transforms: f32,
    /// Sensor hit processing time.
    pub sensor_hits: f32,
    /// Joint event processing time.
    pub joint_events: f32,
    /// Hit event processing time.
    pub hit_events: f32,
    /// Broad-phase refit time.
    pub refit: f32,
    /// Bullet/continuous collision time.
    pub bullets: f32,
    /// Sleep island processing time.
    pub sleep_islands: f32,
    /// Sensor processing time.
    pub sensors: f32,
}

impl Profile {
    /// Converts a raw Box3D profile into the Rust value type.
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

/// World counters reported by Box3D for diagnostics and tests.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Counters {
    /// Number of live bodies.
    pub body_count: i32,
    /// Number of live shapes.
    pub shape_count: i32,
    /// Number of live contacts.
    pub contact_count: i32,
    /// Number of live joints.
    pub joint_count: i32,
    /// Number of solver islands.
    pub island_count: i32,
    /// Stack memory used by Box3D.
    pub stack_used: i32,
    /// Arena memory capacity.
    pub arena_capacity: i32,
    /// Static broad-phase tree height.
    pub static_tree_height: i32,
    /// Dynamic broad-phase tree height.
    pub tree_height: i32,
    /// Number of separating-axis test calls.
    pub sat_call_count: i32,
    /// Number of separating-axis cache hits.
    pub sat_cache_hit_count: i32,
    /// Native byte count reported by Box3D.
    pub byte_count: i32,
    /// Number of native tasks scheduled by the last step.
    pub task_count: i32,
    /// Solver graph color distribution.
    pub color_counts: [i32; 24],
    /// Contact manifold point-count distribution.
    pub manifold_counts: [i32; 8],
    /// Number of awake contacts.
    pub awake_contact_count: i32,
    /// Number of recycled contacts.
    pub recycled_contact_count: i32,
    /// Distance solver iteration count.
    pub distance_iterations: i32,
    /// Push-back solver iteration count.
    pub push_back_iterations: i32,
    /// Root solver iteration count.
    pub root_iterations: i32,
}

impl Counters {
    /// Converts raw Box3D counters into the Rust value type.
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

/// Box3D version number.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Version {
    /// Major version.
    pub major: i32,
    /// Minor version.
    pub minor: i32,
    /// Revision or patch version.
    pub revision: i32,
}

impl Version {
    /// Converts a raw Box3D version into the Rust value type.
    #[inline]
    pub const fn from_raw(raw: ffi::b3Version) -> Self {
        Self {
            major: raw.major,
            minor: raw.minor,
            revision: raw.revision,
        }
    }
}
