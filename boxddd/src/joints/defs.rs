use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JointType {
    Parallel,
    Distance,
    Filter,
    Motor,
    Prismatic,
    Revolute,
    Spherical,
    Weld,
    Wheel,
}

impl JointType {
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
pub struct JointTuning {
    pub hertz: f32,
    pub damping_ratio: f32,
}

impl JointTuning {
    #[inline]
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
            pub fn raw(&self) -> &$raw {
                &self.raw
            }

            #[inline]
            pub fn body_ids(mut self, body_a: BodyId, body_b: BodyId) -> Self {
                self.raw.base.bodyIdA = body_a.into_raw();
                self.raw.base.bodyIdB = body_b.into_raw();
                self
            }

            #[inline]
            pub fn local_frame_a(mut self, frame: Transform) -> Self {
                self.raw.base.localFrameA = frame.into_raw();
                self
            }

            #[inline]
            pub fn local_frame_b(mut self, frame: Transform) -> Self {
                self.raw.base.localFrameB = frame.into_raw();
                self
            }

            #[inline]
            pub fn collide_connected(mut self, collide_connected: bool) -> Self {
                self.raw.base.collideConnected = collide_connected;
                self
            }

            #[inline]
            pub fn force_threshold(mut self, threshold: f32) -> Self {
                self.raw.base.forceThreshold = threshold;
                self
            }

            #[inline]
            pub fn torque_threshold(mut self, threshold: f32) -> Self {
                self.raw.base.torqueThreshold = threshold;
                self
            }

            #[inline]
            pub fn constraint_tuning(mut self, tuning: JointTuning) -> Self {
                self.raw.base.constraintHertz = tuning.hertz;
                self.raw.base.constraintDampingRatio = tuning.damping_ratio;
                self
            }

            #[inline]
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
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

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
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    pub fn length(mut self, length: f32) -> Self {
        self.raw.length = length;
        self
    }

    pub fn spring(mut self, enabled: bool, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.enableSpring = enabled;
        self.raw.hertz = hertz;
        self.raw.dampingRatio = damping_ratio;
        self
    }

    pub fn spring_force_range(mut self, lower: f32, upper: f32) -> Self {
        self.raw.lowerSpringForce = lower;
        self.raw.upperSpringForce = upper;
        self
    }

    pub fn limit(mut self, enabled: bool, min_length: f32, max_length: f32) -> Self {
        self.raw.enableLimit = enabled;
        self.raw.minLength = min_length;
        self.raw.maxLength = max_length;
        self
    }

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
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    pub fn linear_velocity(mut self, velocity: impl Into<Vec3>) -> Self {
        self.raw.linearVelocity = velocity.into().into_raw();
        self
    }

    pub fn angular_velocity(mut self, velocity: impl Into<Vec3>) -> Self {
        self.raw.angularVelocity = velocity.into().into_raw();
        self
    }

    pub fn max_velocity_force(mut self, force: f32) -> Self {
        self.raw.maxVelocityForce = force;
        self
    }

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
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    pub(super) fn validate(&self) -> Result<()> {
        validate_base(&self.raw.base)
    }
}

#[derive(Copy, Clone, Debug)]
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
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    pub fn spring(mut self, enabled: bool, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.enableSpring = enabled;
        self.raw.hertz = hertz;
        self.raw.dampingRatio = damping_ratio;
        self
    }

    pub fn target_translation(mut self, target: f32) -> Self {
        self.raw.targetTranslation = target;
        self
    }

    pub fn limit(mut self, enabled: bool, lower: f32, upper: f32) -> Self {
        self.raw.enableLimit = enabled;
        self.raw.lowerTranslation = lower;
        self.raw.upperTranslation = upper;
        self
    }

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
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    pub fn spring(mut self, enabled: bool, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.enableSpring = enabled;
        self.raw.hertz = hertz;
        self.raw.dampingRatio = damping_ratio;
        self
    }

    pub fn target_angle(mut self, target_angle: f32) -> Self {
        self.raw.targetAngle = target_angle;
        self
    }

    pub fn limit(mut self, enabled: bool, lower: f32, upper: f32) -> Self {
        self.raw.enableLimit = enabled;
        self.raw.lowerAngle = lower;
        self.raw.upperAngle = upper;
        self
    }

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
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    pub fn spring(mut self, enabled: bool, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.enableSpring = enabled;
        self.raw.hertz = hertz;
        self.raw.dampingRatio = damping_ratio;
        self
    }

    pub fn target_rotation(mut self, rotation: Quat) -> Self {
        self.raw.targetRotation = rotation.into_raw();
        self
    }

    pub fn cone_limit(mut self, enabled: bool, angle: f32) -> Self {
        self.raw.enableConeLimit = enabled;
        self.raw.coneAngle = angle;
        self
    }

    pub fn twist_limit(mut self, enabled: bool, lower: f32, upper: f32) -> Self {
        self.raw.enableTwistLimit = enabled;
        self.raw.lowerTwistAngle = lower;
        self.raw.upperTwistAngle = upper;
        self
    }

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
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    pub fn linear_tuning(mut self, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.linearHertz = hertz;
        self.raw.linearDampingRatio = damping_ratio;
        self
    }

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
    pub fn new(body_a: BodyId, body_b: BodyId) -> Self {
        Self::default().body_ids(body_a, body_b)
    }

    pub fn suspension(mut self, enabled: bool, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.enableSuspensionSpring = enabled;
        self.raw.suspensionHertz = hertz;
        self.raw.suspensionDampingRatio = damping_ratio;
        self
    }

    pub fn suspension_limit(mut self, enabled: bool, lower: f32, upper: f32) -> Self {
        self.raw.enableSuspensionLimit = enabled;
        self.raw.lowerSuspensionLimit = lower;
        self.raw.upperSuspensionLimit = upper;
        self
    }

    pub fn spin_motor(mut self, enabled: bool, speed: f32, max_torque: f32) -> Self {
        self.raw.enableSpinMotor = enabled;
        self.raw.spinSpeed = speed;
        self.raw.maxSpinTorque = max_torque;
        self
    }

    pub fn steering(mut self, enabled: bool, hertz: f32, damping_ratio: f32) -> Self {
        self.raw.enableSteering = enabled;
        self.raw.steeringHertz = hertz;
        self.raw.steeringDampingRatio = damping_ratio;
        self
    }

    pub fn steering_limit(mut self, enabled: bool, lower: f32, upper: f32) -> Self {
        self.raw.enableSteeringLimit = enabled;
        self.raw.lowerSteeringLimit = lower;
        self.raw.upperSteeringLimit = upper;
        self
    }

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
