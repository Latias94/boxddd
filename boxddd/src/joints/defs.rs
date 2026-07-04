use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Native Box3D joint family reported for a joint handle.
pub enum JointType {
    /// A parallel joint that keeps two body frames parallel with optional spring torque.
    Parallel,
    /// A distance joint that constrains the distance between two body frames.
    Distance,
    /// A filter joint that disables collision between the connected bodies.
    Filter,
    /// A motor joint that drives relative linear and angular velocity.
    Motor,
    /// A prismatic joint that allows translation along one axis.
    Prismatic,
    /// A revolute joint that allows rotation around one axis.
    Revolute,
    /// A spherical joint that allows ball-and-socket rotation.
    Spherical,
    /// A weld joint that locks relative translation and rotation.
    Weld,
    /// A wheel joint with suspension, spin motor, and steering controls.
    Wheel,
}

impl JointType {
    /// Converts a raw Box3D joint type value into the safe Rust enum.
    pub const fn from_raw(raw: ffi::b3JointType) -> Option<Self> {
        match raw {
            ffi::b3JointType_b3_parallelJoint => Some(Self::Parallel),
            ffi::b3JointType_b3_distanceJoint => Some(Self::Distance),
            ffi::b3JointType_b3_filterJoint => Some(Self::Filter),
            ffi::b3JointType_b3_motorJoint => Some(Self::Motor),
            ffi::b3JointType_b3_prismaticJoint => Some(Self::Prismatic),
            ffi::b3JointType_b3_revoluteJoint => Some(Self::Revolute),
            ffi::b3JointType_b3_sphericalJoint => Some(Self::Spherical),
            ffi::b3JointType_b3_weldJoint => Some(Self::Weld),
            ffi::b3JointType_b3_wheelJoint => Some(Self::Wheel),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
/// Frequency and damping parameters shared by Box3D joint constraints.
///
/// `hertz` is cycles per second. `damping_ratio` is dimensionless and must be
/// non-negative when the value is passed to a fallible API.
pub struct JointTuning {
    /// Constraint frequency in hertz.
    pub hertz: f32,
    /// Non-negative damping ratio for the constraint response.
    pub damping_ratio: f32,
}

impl JointTuning {
    #[inline]
    /// Creates a tuning value from a frequency and damping ratio.
    pub const fn new(hertz: f32, damping_ratio: f32) -> Self {
        Self {
            hertz,
            damping_ratio,
        }
    }

    pub(super) fn validate(self) -> Result<()> {
        validate_nonnegative_scalar(self.hertz)?;
        validate_nonnegative_scalar(self.damping_ratio)
    }
}

macro_rules! impl_joint_def_common {
    ($ty:ident, $raw:ty) => {
        impl $ty {
            #[inline]
            /// Returns the raw Box3D definition backing this builder.
            pub fn raw(&self) -> &$raw {
                &self.raw
            }

            #[inline]
            /// Sets the two bodies connected by the joint.
            pub fn body_ids(mut self, body_a: BodyId, body_b: BodyId) -> Self {
                self.raw.base.bodyIdA = body_a.into_raw();
                self.raw.base.bodyIdB = body_b.into_raw();
                self
            }

            #[inline]
            /// Sets the local constraint frame on body A.
            ///
            /// Box3D measures joint local frames from each body's origin, not
            /// from its center of mass. This keeps the joint stable when body
            /// shapes are later added, removed, or have mass recomputed.
            pub fn local_frame_a(mut self, frame: Transform) -> Self {
                self.raw.base.localFrameA = frame.into_raw();
                self
            }

            #[inline]
            /// Sets the local constraint frame on body B.
            ///
            /// Box3D measures joint local frames from each body's origin, not
            /// from its center of mass.
            pub fn local_frame_b(mut self, frame: Transform) -> Self {
                self.raw.base.localFrameB = frame.into_raw();
                self
            }

            #[inline]
            /// Controls whether the connected bodies may collide with each other.
            pub fn collide_connected(mut self, collide_connected: bool) -> Self {
                self.raw.base.collideConnected = collide_connected;
                self
            }

            #[inline]
            /// Sets the force threshold, in newtons, used to emit joint events.
            pub fn force_threshold(mut self, threshold: f32) -> Self {
                self.raw.base.forceThreshold = threshold;
                self
            }

            #[inline]
            /// Sets the torque threshold, in newton-meters, used to emit joint events.
            pub fn torque_threshold(mut self, threshold: f32) -> Self {
                self.raw.base.torqueThreshold = threshold;
                self
            }

            #[inline]
            /// Sets the common constraint frequency and damping ratio.
            ///
            /// The frequency is in hertz and the damping ratio is dimensionless.
            pub fn constraint_tuning(mut self, tuning: JointTuning) -> Self {
                self.raw.base.constraintHertz = tuning.hertz;
                self.raw.base.constraintDampingRatio = tuning.damping_ratio;
                self
            }

            #[inline]
            /// Sets the scale used by native debug drawing for this joint.
            pub fn draw_scale(mut self, draw_scale: f32) -> Self {
                self.raw.base.drawScale = draw_scale;
                self
            }

            /// Sets the raw Box3D `userData` pointer on this joint definition.
            ///
            /// # Safety
            ///
            /// The caller must ensure the pointer remains valid for every native Box3D use and
            /// must not rely on `boxddd` to manage, alias-check, or drop the pointed-to value.
            #[inline]
            pub unsafe fn raw_user_data(mut self, user_data: *mut c_void) -> Self {
                self.raw.base.userData = user_data;
                self
            }
        }
    };
}

#[derive(Copy, Clone, Debug)]
/// Builder for a Box3D parallel joint.
///
/// A parallel joint constrains the angle between the local z axes on the two
/// joint frames with a spring. It is useful for keeping an object upright.
pub struct ParallelJointDef {
    raw: ffi::b3ParallelJointDef,
}

impl Default for ParallelJointDef {
    fn default() -> Self {
        Self {
            raw: unsafe { ffi::b3DefaultParallelJointDef() },
        }
    }
}

impl_joint_def_common!(ParallelJointDef, ffi::b3ParallelJointDef);

impl ParallelJointDef {
    /// Creates a parallel joint definition for two bodies.
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    /// Sets the parallel spring frequency, damping ratio, and maximum torque.
    ///
    /// `hertz` is cycles per second, `damping_ratio` is dimensionless, and
    /// `max_torque` is typically newton-meters.
    pub fn spring(mut self, hertz: f32, damping_ratio: f32, max_torque: f32) -> Self {
        self.raw.hertz = hertz;
        self.raw.dampingRatio = damping_ratio;
        self.raw.maxTorque = max_torque;
        self
    }

    pub(super) fn validate(&self) -> Result<()> {
        validate_base(&self.raw.base)?;
        validate_nonnegative_scalar(self.raw.hertz)?;
        validate_nonnegative_scalar(self.raw.dampingRatio)?;
        validate_nonnegative_scalar(self.raw.maxTorque)
    }
}

#[derive(Copy, Clone, Debug)]
/// Builder for a Box3D distance joint.
///
/// A distance joint connects a point on body A to a point on body B by a
/// segment. It can act as a rigid rod, spring, length limit, or motorized rope.
pub struct DistanceJointDef {
    raw: ffi::b3DistanceJointDef,
}

impl Default for DistanceJointDef {
    fn default() -> Self {
        Self {
            raw: unsafe { ffi::b3DefaultDistanceJointDef() },
        }
    }
}

impl_joint_def_common!(DistanceJointDef, ffi::b3DistanceJointDef);

impl DistanceJointDef {
    /// Creates a distance joint definition for two bodies.
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    /// Sets the target distance, in world length units, between the joint anchors.
    pub fn length(mut self, length: f32) -> Self {
        self.raw.length = length;
        self
    }

    /// Enables or disables the distance spring and sets its tuning.
    ///
    /// `hertz` is cycles per second and `damping_ratio` is dimensionless.
    pub fn spring(mut self, enabled: bool, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.enableSpring = enabled;
        self.raw.hertz = hertz;
        self.raw.dampingRatio = damping_ratio;
        self
    }

    /// Sets the lower tension force and upper compression force range.
    pub fn spring_force_range(mut self, lower: f32, upper: f32) -> Self {
        self.raw.lowerSpringForce = lower;
        self.raw.upperSpringForce = upper;
        self
    }

    /// Enables or disables the distance limits and sets their length range.
    ///
    /// `min_length` and `max_length` are world length units and must satisfy
    /// `min_length <= max_length`.
    pub fn limit(mut self, enabled: bool, min_length: f32, max_length: f32) -> Self {
        self.raw.enableLimit = enabled;
        self.raw.minLength = min_length;
        self.raw.maxLength = max_length;
        self
    }

    /// Enables or disables the distance motor and sets speed and maximum force.
    ///
    /// `speed` is length units per second and `max_force` is typically newtons.
    pub fn motor(mut self, enabled: bool, speed: f32, max_force: f32) -> Self {
        self.raw.enableMotor = enabled;
        self.raw.motorSpeed = speed;
        self.raw.maxMotorForce = max_force;
        self
    }

    pub(super) fn validate(&self) -> Result<()> {
        validate_base(&self.raw.base)?;
        validate_nonnegative_scalar(self.raw.length)?;
        validate_scalar(self.raw.lowerSpringForce)?;
        validate_scalar(self.raw.upperSpringForce)?;
        validate_nonnegative_scalar(self.raw.hertz)?;
        validate_nonnegative_scalar(self.raw.dampingRatio)?;
        validate_length_range(self.raw.minLength, self.raw.maxLength)?;
        validate_nonnegative_scalar(self.raw.maxMotorForce)?;
        validate_scalar(self.raw.motorSpeed)
    }
}

#[derive(Copy, Clone, Debug)]
/// Builder for a Box3D motor joint.
///
/// A motor joint drives relative linear and angular velocity between two
/// bodies, with optional spring controls for position-style correction.
pub struct MotorJointDef {
    raw: ffi::b3MotorJointDef,
}

impl Default for MotorJointDef {
    fn default() -> Self {
        Self {
            raw: unsafe { ffi::b3DefaultMotorJointDef() },
        }
    }
}

impl_joint_def_common!(MotorJointDef, ffi::b3MotorJointDef);

impl MotorJointDef {
    /// Creates a motor joint definition for two bodies.
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    /// Sets the desired relative linear velocity.
    pub fn linear_velocity(mut self, velocity: impl Into<Vec3>) -> Self {
        self.raw.linearVelocity = velocity.into().into_raw();
        self
    }

    /// Sets the desired relative angular velocity.
    pub fn angular_velocity(mut self, velocity: impl Into<Vec3>) -> Self {
        self.raw.angularVelocity = velocity.into().into_raw();
        self
    }

    /// Sets the maximum force, in newtons, applied to reach the target linear velocity.
    pub fn max_velocity_force(mut self, force: f32) -> Self {
        self.raw.maxVelocityForce = force;
        self
    }

    /// Sets the maximum torque, in newton-meters, applied to reach the target angular velocity.
    pub fn max_velocity_torque(mut self, torque: f32) -> Self {
        self.raw.maxVelocityTorque = torque;
        self
    }

    pub(super) fn validate(&self) -> Result<()> {
        validate_base(&self.raw.base)?;
        Vec3::from_raw(self.raw.linearVelocity).validate()?;
        Vec3::from_raw(self.raw.angularVelocity).validate()?;
        validate_nonnegative_scalar(self.raw.maxVelocityForce)?;
        validate_nonnegative_scalar(self.raw.maxVelocityTorque)?;
        validate_nonnegative_scalar(self.raw.linearHertz)?;
        validate_nonnegative_scalar(self.raw.linearDampingRatio)?;
        validate_nonnegative_scalar(self.raw.maxSpringForce)?;
        validate_nonnegative_scalar(self.raw.angularHertz)?;
        validate_nonnegative_scalar(self.raw.angularDampingRatio)?;
        validate_nonnegative_scalar(self.raw.maxSpringTorque)
    }
}

#[derive(Copy, Clone, Debug)]
/// Builder for a Box3D filter joint.
pub struct FilterJointDef {
    raw: ffi::b3FilterJointDef,
}

impl Default for FilterJointDef {
    fn default() -> Self {
        Self {
            raw: unsafe { ffi::b3DefaultFilterJointDef() },
        }
    }
}

impl_joint_def_common!(FilterJointDef, ffi::b3FilterJointDef);

impl FilterJointDef {
    /// Creates a filter joint definition for two bodies.
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    pub(super) fn validate(&self) -> Result<()> {
        validate_base(&self.raw.base)
    }
}

#[derive(Copy, Clone, Debug)]
/// Builder for a Box3D prismatic joint.
///
/// Body B slides along the local x axis of frame A and cannot rotate relative
/// to body A. Translation is zero when the local frame origins coincide.
pub struct PrismaticJointDef {
    raw: ffi::b3PrismaticJointDef,
}

impl Default for PrismaticJointDef {
    fn default() -> Self {
        Self {
            raw: unsafe { ffi::b3DefaultPrismaticJointDef() },
        }
    }
}

impl_joint_def_common!(PrismaticJointDef, ffi::b3PrismaticJointDef);

impl PrismaticJointDef {
    /// Creates a prismatic joint definition for two bodies.
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    /// Enables or disables the translation spring and sets its tuning.
    ///
    /// `hertz` is cycles per second and `damping_ratio` is dimensionless.
    pub fn spring(mut self, enabled: bool, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.enableSpring = enabled;
        self.raw.hertz = hertz;
        self.raw.dampingRatio = damping_ratio;
        self
    }

    /// Sets the target translation, in world length units, along the prismatic axis.
    pub fn target_translation(mut self, target: f32) -> Self {
        self.raw.targetTranslation = target;
        self
    }

    /// Enables or disables translation limits and sets their range.
    ///
    /// `lower` and `upper` are world length units and must satisfy `lower <= upper`.
    pub fn limit(mut self, enabled: bool, lower: f32, upper: f32) -> Self {
        self.raw.enableLimit = enabled;
        self.raw.lowerTranslation = lower;
        self.raw.upperTranslation = upper;
        self
    }

    /// Enables or disables the translation motor and sets speed and maximum force.
    ///
    /// `speed` is length units per second and `max_force` is typically newtons.
    pub fn motor(mut self, enabled: bool, speed: f32, max_force: f32) -> Self {
        self.raw.enableMotor = enabled;
        self.raw.motorSpeed = speed;
        self.raw.maxMotorForce = max_force;
        self
    }

    pub(super) fn validate(&self) -> Result<()> {
        validate_base(&self.raw.base)?;
        validate_nonnegative_scalar(self.raw.hertz)?;
        validate_nonnegative_scalar(self.raw.dampingRatio)?;
        validate_scalar(self.raw.targetTranslation)?;
        validate_range(self.raw.lowerTranslation, self.raw.upperTranslation)?;
        validate_nonnegative_scalar(self.raw.maxMotorForce)?;
        validate_scalar(self.raw.motorSpeed)
    }
}

#[derive(Copy, Clone, Debug)]
/// Builder for a Box3D revolute joint.
///
/// A revolute joint fixes a point on body B to a point on body A and allows
/// relative rotation about the joint z axis.
pub struct RevoluteJointDef {
    raw: ffi::b3RevoluteJointDef,
}

impl Default for RevoluteJointDef {
    fn default() -> Self {
        Self {
            raw: unsafe { ffi::b3DefaultRevoluteJointDef() },
        }
    }
}

impl_joint_def_common!(RevoluteJointDef, ffi::b3RevoluteJointDef);

impl RevoluteJointDef {
    /// Creates a revolute joint definition for two bodies.
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    /// Enables or disables the angular spring and sets its tuning.
    ///
    /// `hertz` is cycles per second and `damping_ratio` is dimensionless.
    pub fn spring(mut self, enabled: bool, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.enableSpring = enabled;
        self.raw.hertz = hertz;
        self.raw.dampingRatio = damping_ratio;
        self
    }

    /// Sets the target angle, in radians, for the angular spring.
    pub fn target_angle(mut self, target_angle: f32) -> Self {
        self.raw.targetAngle = target_angle;
        self
    }

    /// Enables or disables angular limits and sets their angle range in radians.
    pub fn limit(mut self, enabled: bool, lower: f32, upper: f32) -> Self {
        self.raw.enableLimit = enabled;
        self.raw.lowerAngle = lower;
        self.raw.upperAngle = upper;
        self
    }

    /// Enables or disables the angular motor and sets speed and maximum torque.
    ///
    /// `speed` is radians per second and `max_torque` is typically newton-meters.
    pub fn motor(mut self, enabled: bool, speed: f32, max_torque: f32) -> Self {
        self.raw.enableMotor = enabled;
        self.raw.motorSpeed = speed;
        self.raw.maxMotorTorque = max_torque;
        self
    }

    pub(super) fn validate(&self) -> Result<()> {
        validate_base(&self.raw.base)?;
        validate_scalar(self.raw.targetAngle)?;
        validate_nonnegative_scalar(self.raw.hertz)?;
        validate_nonnegative_scalar(self.raw.dampingRatio)?;
        validate_range(self.raw.lowerAngle, self.raw.upperAngle)?;
        validate_nonnegative_scalar(self.raw.maxMotorTorque)?;
        validate_scalar(self.raw.motorSpeed)
    }
}

#[derive(Copy, Clone, Debug)]
/// Builder for a Box3D spherical joint.
///
/// A spherical joint fixes a point on body B to a point on body A and allows
/// ball-and-socket rotation about that shared point.
pub struct SphericalJointDef {
    raw: ffi::b3SphericalJointDef,
}

impl Default for SphericalJointDef {
    fn default() -> Self {
        Self {
            raw: unsafe { ffi::b3DefaultSphericalJointDef() },
        }
    }
}

impl_joint_def_common!(SphericalJointDef, ffi::b3SphericalJointDef);

impl SphericalJointDef {
    /// Creates a spherical joint definition for two bodies.
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    /// Enables or disables the rotational spring and sets its tuning.
    ///
    /// `hertz` is cycles per second and `damping_ratio` is dimensionless.
    pub fn spring(mut self, enabled: bool, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.enableSpring = enabled;
        self.raw.hertz = hertz;
        self.raw.dampingRatio = damping_ratio;
        self
    }

    /// Sets the target relative rotation for the spring.
    ///
    /// The target is joint frame B relative to joint frame A.
    pub fn target_rotation(mut self, rotation: Quat) -> Self {
        self.raw.targetRotation = rotation.into_raw();
        self
    }

    /// Enables or disables the cone limit and sets its maximum angle in radians.
    pub fn cone_limit(mut self, enabled: bool, angle: f32) -> Self {
        self.raw.enableConeLimit = enabled;
        self.raw.coneAngle = angle;
        self
    }

    /// Enables or disables the twist limit and sets its angle range in radians.
    pub fn twist_limit(mut self, enabled: bool, lower: f32, upper: f32) -> Self {
        self.raw.enableTwistLimit = enabled;
        self.raw.lowerTwistAngle = lower;
        self.raw.upperTwistAngle = upper;
        self
    }

    /// Enables or disables the angular motor and sets velocity and maximum torque.
    ///
    /// `velocity` is radians per second around each axis and `max_torque` is
    /// typically newton-meters.
    pub fn motor(mut self, enabled: bool, velocity: impl Into<Vec3>, max_torque: f32) -> Self {
        self.raw.enableMotor = enabled;
        self.raw.motorVelocity = velocity.into().into_raw();
        self.raw.maxMotorTorque = max_torque;
        self
    }

    pub(super) fn validate(&self) -> Result<()> {
        validate_base(&self.raw.base)?;
        validate_nonnegative_scalar(self.raw.hertz)?;
        validate_nonnegative_scalar(self.raw.dampingRatio)?;
        Quat::from_raw(self.raw.targetRotation).validate()?;
        validate_nonnegative_scalar(self.raw.coneAngle)?;
        validate_range(self.raw.lowerTwistAngle, self.raw.upperTwistAngle)?;
        validate_nonnegative_scalar(self.raw.maxMotorTorque)?;
        Vec3::from_raw(self.raw.motorVelocity)
            .validate()
            .map(|_| ())
    }
}

#[derive(Copy, Clone, Debug)]
/// Builder for a Box3D weld joint.
///
/// A weld joint locks relative translation and rotation, with spring tuning to
/// approximate soft-body behavior. Box3D's iterative solver cannot hold long
/// chains of welds perfectly rigid.
pub struct WeldJointDef {
    raw: ffi::b3WeldJointDef,
}

impl Default for WeldJointDef {
    fn default() -> Self {
        Self {
            raw: unsafe { ffi::b3DefaultWeldJointDef() },
        }
    }
}

impl_joint_def_common!(WeldJointDef, ffi::b3WeldJointDef);

impl WeldJointDef {
    /// Creates a weld joint definition for two bodies.
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    /// Sets the linear weld spring frequency and damping ratio.
    ///
    /// `hertz` is cycles per second; zero asks Box3D for maximum stiffness.
    pub fn linear_tuning(mut self, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.linearHertz = hertz;
        self.raw.linearDampingRatio = damping_ratio;
        self
    }

    /// Sets the angular weld spring frequency and damping ratio.
    ///
    /// `hertz` is cycles per second; zero asks Box3D for maximum stiffness.
    pub fn angular_tuning(mut self, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.angularHertz = hertz;
        self.raw.angularDampingRatio = damping_ratio;
        self
    }

    pub(super) fn validate(&self) -> Result<()> {
        validate_base(&self.raw.base)?;
        validate_nonnegative_scalar(self.raw.linearHertz)?;
        validate_nonnegative_scalar(self.raw.linearDampingRatio)?;
        validate_nonnegative_scalar(self.raw.angularHertz)?;
        validate_nonnegative_scalar(self.raw.angularDampingRatio)
    }
}

#[derive(Copy, Clone, Debug)]
/// Builder for a Box3D wheel joint.
///
/// Body A is the chassis and body B is the wheel. The wheel rotates around the
/// local z axis in frame B, translates along the local x axis in frame A, and
/// can optionally steer along the local x axis in frame A.
pub struct WheelJointDef {
    raw: ffi::b3WheelJointDef,
}

impl Default for WheelJointDef {
    fn default() -> Self {
        Self {
            raw: unsafe { ffi::b3DefaultWheelJointDef() },
        }
    }
}

impl_joint_def_common!(WheelJointDef, ffi::b3WheelJointDef);

impl WheelJointDef {
    /// Creates a wheel joint definition for two bodies.
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    /// Enables or disables suspension springing and sets its tuning.
    ///
    /// `hertz` is cycles per second and `damping_ratio` is dimensionless.
    pub fn suspension(mut self, enabled: bool, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.enableSuspensionSpring = enabled;
        self.raw.suspensionHertz = hertz;
        self.raw.suspensionDampingRatio = damping_ratio;
        self
    }

    /// Enables or disables suspension limits and sets their translation range.
    ///
    /// `lower` and `upper` are world length units and must satisfy `lower <= upper`.
    pub fn suspension_limit(mut self, enabled: bool, lower: f32, upper: f32) -> Self {
        self.raw.enableSuspensionLimit = enabled;
        self.raw.lowerSuspensionLimit = lower;
        self.raw.upperSuspensionLimit = upper;
        self
    }

    /// Enables or disables the wheel spin motor and sets speed and maximum torque.
    ///
    /// `speed` is radians per second and `max_torque` is typically newton-meters.
    pub fn spin_motor(mut self, enabled: bool, speed: f32, max_torque: f32) -> Self {
        self.raw.enableSpinMotor = enabled;
        self.raw.spinSpeed = speed;
        self.raw.maxSpinTorque = max_torque;
        self
    }

    /// Enables or disables steering and sets steering spring tuning.
    ///
    /// `hertz` is cycles per second and `damping_ratio` is dimensionless.
    pub fn steering(mut self, enabled: bool, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.enableSteering = enabled;
        self.raw.steeringHertz = hertz;
        self.raw.steeringDampingRatio = damping_ratio;
        self
    }

    /// Enables or disables steering limits and sets their angle range in radians.
    pub fn steering_limit(mut self, enabled: bool, lower: f32, upper: f32) -> Self {
        self.raw.enableSteeringLimit = enabled;
        self.raw.lowerSteeringLimit = lower;
        self.raw.upperSteeringLimit = upper;
        self
    }

    /// Sets the target steering angle and maximum steering torque.
    ///
    /// `angle` is radians and `max_torque` is in newton-meters.
    pub fn target_steering(mut self, angle: f32, max_torque: f32) -> Self {
        self.raw.targetSteeringAngle = angle;
        self.raw.maxSteeringTorque = max_torque;
        self
    }

    pub(super) fn validate(&self) -> Result<()> {
        validate_base(&self.raw.base)?;
        validate_nonnegative_scalar(self.raw.suspensionHertz)?;
        validate_nonnegative_scalar(self.raw.suspensionDampingRatio)?;
        validate_range(self.raw.lowerSuspensionLimit, self.raw.upperSuspensionLimit)?;
        validate_nonnegative_scalar(self.raw.maxSpinTorque)?;
        validate_scalar(self.raw.spinSpeed)?;
        validate_nonnegative_scalar(self.raw.steeringHertz)?;
        validate_nonnegative_scalar(self.raw.steeringDampingRatio)?;
        validate_scalar(self.raw.targetSteeringAngle)?;
        validate_nonnegative_scalar(self.raw.maxSteeringTorque)?;
        validate_range(self.raw.lowerSteeringLimit, self.raw.upperSteeringLimit)
    }
}
