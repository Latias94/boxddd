use crate::core::{box3d_lock, callback_state};
use crate::error::{Error, Result};
use crate::types::{BodyId, JointId, Quat, Transform, Vec3};
use crate::world::World;
use boxddd_sys::ffi;
use std::ffi::c_void;

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

    fn validate(self) -> Result<()> {
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

            #[inline]
            pub fn user_data(mut self, user_data: *mut c_void) -> Self {
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

    fn validate(&self) -> Result<()> {
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

    fn validate(&self) -> Result<()> {
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

    fn validate(&self) -> Result<()> {
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

    fn validate(&self) -> Result<()> {
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

    fn validate(&self) -> Result<()> {
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

    fn validate(&self) -> Result<()> {
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

    fn validate(&self) -> Result<()> {
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

    fn validate(&self) -> Result<()> {
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

    fn validate(&self) -> Result<()> {
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

impl World {
    pub fn try_create_parallel_joint(&mut self, def: ParallelJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw.base, |world| unsafe {
            ffi::b3CreateParallelJoint(world, def.raw())
        })
    }

    pub fn create_parallel_joint(&mut self, def: ParallelJointDef) -> JointId {
        self.try_create_parallel_joint(def)
            .expect("Box3D failed to create parallel joint")
    }

    pub fn try_create_distance_joint(&mut self, def: DistanceJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw.base, |world| unsafe {
            ffi::b3CreateDistanceJoint(world, def.raw())
        })
    }

    pub fn create_distance_joint(&mut self, def: DistanceJointDef) -> JointId {
        self.try_create_distance_joint(def)
            .expect("Box3D failed to create distance joint")
    }

    pub fn try_create_motor_joint(&mut self, def: MotorJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw.base, |world| unsafe {
            ffi::b3CreateMotorJoint(world, def.raw())
        })
    }

    pub fn create_motor_joint(&mut self, def: MotorJointDef) -> JointId {
        self.try_create_motor_joint(def)
            .expect("Box3D failed to create motor joint")
    }

    pub fn try_create_filter_joint(&mut self, def: FilterJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw.base, |world| unsafe {
            ffi::b3CreateFilterJoint(world, def.raw())
        })
    }

    pub fn create_filter_joint(&mut self, def: FilterJointDef) -> JointId {
        self.try_create_filter_joint(def)
            .expect("Box3D failed to create filter joint")
    }

    pub fn try_create_prismatic_joint(&mut self, def: PrismaticJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw.base, |world| unsafe {
            ffi::b3CreatePrismaticJoint(world, def.raw())
        })
    }

    pub fn create_prismatic_joint(&mut self, def: PrismaticJointDef) -> JointId {
        self.try_create_prismatic_joint(def)
            .expect("Box3D failed to create prismatic joint")
    }

    pub fn try_create_revolute_joint(&mut self, def: RevoluteJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw.base, |world| unsafe {
            ffi::b3CreateRevoluteJoint(world, def.raw())
        })
    }

    pub fn create_revolute_joint(&mut self, def: RevoluteJointDef) -> JointId {
        self.try_create_revolute_joint(def)
            .expect("Box3D failed to create revolute joint")
    }

    pub fn try_create_spherical_joint(&mut self, def: SphericalJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw.base, |world| unsafe {
            ffi::b3CreateSphericalJoint(world, def.raw())
        })
    }

    pub fn create_spherical_joint(&mut self, def: SphericalJointDef) -> JointId {
        self.try_create_spherical_joint(def)
            .expect("Box3D failed to create spherical joint")
    }

    pub fn try_create_weld_joint(&mut self, def: WeldJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw.base, |world| unsafe {
            ffi::b3CreateWeldJoint(world, def.raw())
        })
    }

    pub fn create_weld_joint(&mut self, def: WeldJointDef) -> JointId {
        self.try_create_weld_joint(def)
            .expect("Box3D failed to create weld joint")
    }

    pub fn try_create_wheel_joint(&mut self, def: WheelJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw.base, |world| unsafe {
            ffi::b3CreateWheelJoint(world, def.raw())
        })
    }

    pub fn create_wheel_joint(&mut self, def: WheelJointDef) -> JointId {
        self.try_create_wheel_joint(def)
            .expect("Box3D failed to create wheel joint")
    }

    fn create_joint(
        &mut self,
        base: ffi::b3JointDef,
        create: impl FnOnce(ffi::b3WorldId) -> ffi::b3JointId,
    ) -> Result<JointId> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let body_a = BodyId::from_raw(base.bodyIdA);
        let body_b = BodyId::from_raw(base.bodyIdB);
        check_body_valid_raw(body_a)?;
        check_body_valid_raw(body_b)?;
        check_joint_body_pair_valid(body_a, body_b)?;
        check_joint_targets_world(self.raw(), body_a, body_b)?;
        joint_id_from_raw(create(self.raw()))
    }

    pub fn try_destroy_joint(&mut self, joint_id: JointId, wake_attached: bool) -> Result<()> {
        let _guard = lock_joint_checked(joint_id)?;
        unsafe { ffi::b3DestroyJoint(joint_id.into_raw(), wake_attached) };
        Ok(())
    }

    pub fn destroy_joint(&mut self, joint_id: JointId, wake_attached: bool) {
        self.try_destroy_joint(joint_id, wake_attached)
            .expect("invalid JointId");
    }

    pub fn try_joint_type(&self, joint_id: JointId) -> Result<JointType> {
        let _guard = lock_joint_checked(joint_id)?;
        JointType::from_raw(unsafe { ffi::b3Joint_GetType(joint_id.into_raw()) })
            .ok_or(Error::WrongJointType)
    }

    pub fn joint_type(&self, joint_id: JointId) -> JointType {
        self.try_joint_type(joint_id).expect("invalid JointId")
    }

    pub fn try_joint_body_a(&self, joint_id: JointId) -> Result<BodyId> {
        let _guard = lock_joint_checked(joint_id)?;
        Ok(BodyId::from_raw(unsafe {
            ffi::b3Joint_GetBodyA(joint_id.into_raw())
        }))
    }

    pub fn try_joint_body_b(&self, joint_id: JointId) -> Result<BodyId> {
        let _guard = lock_joint_checked(joint_id)?;
        Ok(BodyId::from_raw(unsafe {
            ffi::b3Joint_GetBodyB(joint_id.into_raw())
        }))
    }

    pub fn try_set_joint_local_frame_a(
        &mut self,
        joint_id: JointId,
        frame: Transform,
    ) -> Result<()> {
        frame.validate()?;
        let _guard = lock_joint_checked(joint_id)?;
        unsafe { ffi::b3Joint_SetLocalFrameA(joint_id.into_raw(), frame.into_raw()) };
        Ok(())
    }

    pub fn try_joint_local_frame_a(&self, joint_id: JointId) -> Result<Transform> {
        let _guard = lock_joint_checked(joint_id)?;
        Ok(Transform::from_raw(unsafe {
            ffi::b3Joint_GetLocalFrameA(joint_id.into_raw())
        }))
    }

    pub fn try_set_joint_local_frame_b(
        &mut self,
        joint_id: JointId,
        frame: Transform,
    ) -> Result<()> {
        frame.validate()?;
        let _guard = lock_joint_checked(joint_id)?;
        unsafe { ffi::b3Joint_SetLocalFrameB(joint_id.into_raw(), frame.into_raw()) };
        Ok(())
    }

    pub fn try_joint_local_frame_b(&self, joint_id: JointId) -> Result<Transform> {
        let _guard = lock_joint_checked(joint_id)?;
        Ok(Transform::from_raw(unsafe {
            ffi::b3Joint_GetLocalFrameB(joint_id.into_raw())
        }))
    }

    pub fn try_set_joint_collide_connected(
        &mut self,
        joint_id: JointId,
        collide: bool,
    ) -> Result<()> {
        let _guard = lock_joint_checked(joint_id)?;
        unsafe { ffi::b3Joint_SetCollideConnected(joint_id.into_raw(), collide) };
        Ok(())
    }

    pub fn try_joint_collide_connected(&self, joint_id: JointId) -> Result<bool> {
        let _guard = lock_joint_checked(joint_id)?;
        Ok(unsafe { ffi::b3Joint_GetCollideConnected(joint_id.into_raw()) })
    }

    pub fn try_set_joint_user_data(
        &mut self,
        joint_id: JointId,
        user_data: *mut c_void,
    ) -> Result<()> {
        let _guard = lock_joint_checked(joint_id)?;
        unsafe { ffi::b3Joint_SetUserData(joint_id.into_raw(), user_data) };
        Ok(())
    }

    pub fn try_joint_user_data(&self, joint_id: JointId) -> Result<*mut c_void> {
        let _guard = lock_joint_checked(joint_id)?;
        Ok(unsafe { ffi::b3Joint_GetUserData(joint_id.into_raw()) })
    }

    pub fn try_wake_joint_bodies(&mut self, joint_id: JointId) -> Result<()> {
        let _guard = lock_joint_checked(joint_id)?;
        unsafe { ffi::b3Joint_WakeBodies(joint_id.into_raw()) };
        Ok(())
    }

    pub fn try_joint_constraint_force(&self, joint_id: JointId) -> Result<Vec3> {
        let _guard = lock_joint_checked(joint_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Joint_GetConstraintForce(joint_id.into_raw())
        }))
    }

    pub fn try_joint_constraint_torque(&self, joint_id: JointId) -> Result<Vec3> {
        let _guard = lock_joint_checked(joint_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Joint_GetConstraintTorque(joint_id.into_raw())
        }))
    }

    pub fn try_joint_linear_separation(&self, joint_id: JointId) -> Result<f32> {
        let _guard = lock_joint_checked(joint_id)?;
        Ok(unsafe { ffi::b3Joint_GetLinearSeparation(joint_id.into_raw()) })
    }

    pub fn try_joint_angular_separation(&self, joint_id: JointId) -> Result<f32> {
        let _guard = lock_joint_checked(joint_id)?;
        Ok(unsafe { ffi::b3Joint_GetAngularSeparation(joint_id.into_raw()) })
    }

    pub fn try_set_joint_constraint_tuning(
        &mut self,
        joint_id: JointId,
        tuning: JointTuning,
    ) -> Result<()> {
        tuning.validate()?;
        let _guard = lock_joint_checked(joint_id)?;
        unsafe {
            ffi::b3Joint_SetConstraintTuning(
                joint_id.into_raw(),
                tuning.hertz,
                tuning.damping_ratio,
            )
        };
        Ok(())
    }

    pub fn try_joint_constraint_tuning(&self, joint_id: JointId) -> Result<JointTuning> {
        let _guard = lock_joint_checked(joint_id)?;
        let mut hertz = 0.0;
        let mut damping_ratio = 0.0;
        unsafe {
            ffi::b3Joint_GetConstraintTuning(joint_id.into_raw(), &mut hertz, &mut damping_ratio)
        };
        Ok(JointTuning::new(hertz, damping_ratio))
    }

    pub fn try_set_joint_force_threshold(
        &mut self,
        joint_id: JointId,
        threshold: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(threshold)?;
        let _guard = lock_joint_checked(joint_id)?;
        unsafe { ffi::b3Joint_SetForceThreshold(joint_id.into_raw(), threshold) };
        Ok(())
    }

    pub fn try_joint_force_threshold(&self, joint_id: JointId) -> Result<f32> {
        let _guard = lock_joint_checked(joint_id)?;
        Ok(unsafe { ffi::b3Joint_GetForceThreshold(joint_id.into_raw()) })
    }

    pub fn try_set_joint_torque_threshold(
        &mut self,
        joint_id: JointId,
        threshold: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(threshold)?;
        let _guard = lock_joint_checked(joint_id)?;
        unsafe { ffi::b3Joint_SetTorqueThreshold(joint_id.into_raw(), threshold) };
        Ok(())
    }

    pub fn try_joint_torque_threshold(&self, joint_id: JointId) -> Result<f32> {
        let _guard = lock_joint_checked(joint_id)?;
        Ok(unsafe { ffi::b3Joint_GetTorqueThreshold(joint_id.into_raw()) })
    }
}

macro_rules! family_method {
    ($joint:expr, $ty:expr, $body:block) => {{
        let _guard = lock_typed_joint_checked($joint, $ty)?;
        let result = $body;
        Ok(result)
    }};
}

impl World {
    pub fn try_set_parallel_joint_spring_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(joint_id, JointType::Parallel, {
            unsafe { ffi::b3ParallelJoint_SetSpringHertz(joint_id.into_raw(), hertz) };
        })
    }

    pub fn try_parallel_joint_spring_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Parallel, {
            unsafe { ffi::b3ParallelJoint_GetSpringHertz(joint_id.into_raw()) }
        })
    }

    pub fn try_set_parallel_joint_spring_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(joint_id, JointType::Parallel, {
            unsafe {
                ffi::b3ParallelJoint_SetSpringDampingRatio(joint_id.into_raw(), damping_ratio)
            };
        })
    }

    pub fn try_parallel_joint_spring_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Parallel, {
            unsafe { ffi::b3ParallelJoint_GetSpringDampingRatio(joint_id.into_raw()) }
        })
    }

    pub fn try_set_parallel_joint_max_torque(
        &mut self,
        joint_id: JointId,
        torque: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(torque)?;
        family_method!(joint_id, JointType::Parallel, {
            unsafe { ffi::b3ParallelJoint_SetMaxTorque(joint_id.into_raw(), torque) };
        })
    }

    pub fn try_parallel_joint_max_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Parallel, {
            unsafe { ffi::b3ParallelJoint_GetMaxTorque(joint_id.into_raw()) }
        })
    }

    pub fn try_set_distance_joint_length(&mut self, joint_id: JointId, length: f32) -> Result<()> {
        validate_nonnegative_scalar(length)?;
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_SetLength(joint_id.into_raw(), length) };
        })
    }

    pub fn try_distance_joint_length(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetLength(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_distance_joint_spring(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_EnableSpring(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_distance_joint_spring_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_IsSpringEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_distance_joint_spring_force_range(
        &mut self,
        joint_id: JointId,
        lower: f32,
        upper: f32,
    ) -> Result<()> {
        validate_scalar(lower)?;
        validate_scalar(upper)?;
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_SetSpringForceRange(joint_id.into_raw(), lower, upper) };
        })
    }

    pub fn try_distance_joint_spring_force_range(&self, joint_id: JointId) -> Result<(f32, f32)> {
        family_method!(joint_id, JointType::Distance, {
            let mut lower = 0.0;
            let mut upper = 0.0;
            unsafe {
                ffi::b3DistanceJoint_GetSpringForceRange(
                    joint_id.into_raw(),
                    &mut lower,
                    &mut upper,
                )
            };
            (lower, upper)
        })
    }

    pub fn try_set_distance_joint_spring_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_SetSpringHertz(joint_id.into_raw(), hertz) };
        })
    }

    pub fn try_distance_joint_spring_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetSpringHertz(joint_id.into_raw()) }
        })
    }

    pub fn try_set_distance_joint_spring_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(joint_id, JointType::Distance, {
            unsafe {
                ffi::b3DistanceJoint_SetSpringDampingRatio(joint_id.into_raw(), damping_ratio)
            };
        })
    }

    pub fn try_distance_joint_spring_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetSpringDampingRatio(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_distance_joint_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_EnableLimit(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_distance_joint_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_IsLimitEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_distance_joint_length_range(
        &mut self,
        joint_id: JointId,
        min: f32,
        max: f32,
    ) -> Result<()> {
        validate_length_range(min, max)?;
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_SetLengthRange(joint_id.into_raw(), min, max) };
        })
    }

    pub fn try_distance_joint_min_length(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMinLength(joint_id.into_raw()) }
        })
    }

    pub fn try_distance_joint_max_length(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMaxLength(joint_id.into_raw()) }
        })
    }

    pub fn try_distance_joint_current_length(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetCurrentLength(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_distance_joint_motor(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_EnableMotor(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_distance_joint_motor_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_IsMotorEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_distance_joint_motor_speed(
        &mut self,
        joint_id: JointId,
        speed: f32,
    ) -> Result<()> {
        validate_scalar(speed)?;
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_SetMotorSpeed(joint_id.into_raw(), speed) };
        })
    }

    pub fn try_distance_joint_motor_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMotorSpeed(joint_id.into_raw()) }
        })
    }

    pub fn try_set_distance_joint_max_motor_force(
        &mut self,
        joint_id: JointId,
        force: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(force)?;
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_SetMaxMotorForce(joint_id.into_raw(), force) };
        })
    }

    pub fn try_distance_joint_max_motor_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMaxMotorForce(joint_id.into_raw()) }
        })
    }

    pub fn try_distance_joint_motor_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMotorForce(joint_id.into_raw()) }
        })
    }

    pub fn try_set_motor_joint_linear_velocity(
        &mut self,
        joint_id: JointId,
        velocity: impl Into<Vec3>,
    ) -> Result<()> {
        let velocity = velocity.into().validate()?;
        family_method!(joint_id, JointType::Motor, {
            unsafe {
                ffi::b3MotorJoint_SetLinearVelocity(joint_id.into_raw(), velocity.into_raw())
            };
        })
    }

    pub fn try_motor_joint_linear_velocity(&self, joint_id: JointId) -> Result<Vec3> {
        family_method!(joint_id, JointType::Motor, {
            Vec3::from_raw(unsafe { ffi::b3MotorJoint_GetLinearVelocity(joint_id.into_raw()) })
        })
    }

    pub fn try_set_motor_joint_angular_velocity(
        &mut self,
        joint_id: JointId,
        velocity: impl Into<Vec3>,
    ) -> Result<()> {
        let velocity = velocity.into().validate()?;
        family_method!(joint_id, JointType::Motor, {
            unsafe {
                ffi::b3MotorJoint_SetAngularVelocity(joint_id.into_raw(), velocity.into_raw())
            };
        })
    }

    pub fn try_motor_joint_angular_velocity(&self, joint_id: JointId) -> Result<Vec3> {
        family_method!(joint_id, JointType::Motor, {
            Vec3::from_raw(unsafe { ffi::b3MotorJoint_GetAngularVelocity(joint_id.into_raw()) })
        })
    }

    pub fn try_set_motor_joint_max_velocity_force(
        &mut self,
        joint_id: JointId,
        force: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(force)?;
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetMaxVelocityForce(joint_id.into_raw(), force) };
        })
    }

    pub fn try_motor_joint_max_velocity_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetMaxVelocityForce(joint_id.into_raw()) }
        })
    }

    pub fn try_set_motor_joint_max_velocity_torque(
        &mut self,
        joint_id: JointId,
        torque: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(torque)?;
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetMaxVelocityTorque(joint_id.into_raw(), torque) };
        })
    }

    pub fn try_motor_joint_max_velocity_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetMaxVelocityTorque(joint_id.into_raw()) }
        })
    }

    pub fn try_set_motor_joint_linear_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetLinearHertz(joint_id.into_raw(), hertz) };
        })
    }

    pub fn try_motor_joint_linear_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetLinearHertz(joint_id.into_raw()) }
        })
    }

    pub fn try_set_motor_joint_linear_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping)?;
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetLinearDampingRatio(joint_id.into_raw(), damping) };
        })
    }

    pub fn try_motor_joint_linear_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetLinearDampingRatio(joint_id.into_raw()) }
        })
    }

    pub fn try_set_motor_joint_angular_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetAngularHertz(joint_id.into_raw(), hertz) };
        })
    }

    pub fn try_motor_joint_angular_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetAngularHertz(joint_id.into_raw()) }
        })
    }

    pub fn try_set_motor_joint_angular_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping)?;
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetAngularDampingRatio(joint_id.into_raw(), damping) };
        })
    }

    pub fn try_motor_joint_angular_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetAngularDampingRatio(joint_id.into_raw()) }
        })
    }

    pub fn try_set_motor_joint_max_spring_force(
        &mut self,
        joint_id: JointId,
        force: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(force)?;
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetMaxSpringForce(joint_id.into_raw(), force) };
        })
    }

    pub fn try_motor_joint_max_spring_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetMaxSpringForce(joint_id.into_raw()) }
        })
    }

    pub fn try_set_motor_joint_max_spring_torque(
        &mut self,
        joint_id: JointId,
        torque: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(torque)?;
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetMaxSpringTorque(joint_id.into_raw(), torque) };
        })
    }

    pub fn try_motor_joint_max_spring_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetMaxSpringTorque(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_prismatic_joint_spring(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_EnableSpring(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_prismatic_joint_spring_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_IsSpringEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_prismatic_joint_spring_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_SetSpringHertz(joint_id.into_raw(), hertz) };
        })
    }

    pub fn try_prismatic_joint_spring_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetSpringHertz(joint_id.into_raw()) }
        })
    }

    pub fn try_set_prismatic_joint_spring_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(joint_id, JointType::Prismatic, {
            unsafe {
                ffi::b3PrismaticJoint_SetSpringDampingRatio(joint_id.into_raw(), damping_ratio)
            };
        })
    }

    pub fn try_prismatic_joint_spring_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetSpringDampingRatio(joint_id.into_raw()) }
        })
    }

    pub fn try_set_prismatic_joint_target_translation(
        &mut self,
        joint_id: JointId,
        target: f32,
    ) -> Result<()> {
        validate_scalar(target)?;
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_SetTargetTranslation(joint_id.into_raw(), target) };
        })
    }

    pub fn try_prismatic_joint_target_translation(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetTargetTranslation(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_prismatic_joint_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_EnableLimit(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_prismatic_joint_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_IsLimitEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_prismatic_joint_lower_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetLowerLimit(joint_id.into_raw()) }
        })
    }

    pub fn try_prismatic_joint_upper_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetUpperLimit(joint_id.into_raw()) }
        })
    }

    pub fn try_set_prismatic_joint_limits(
        &mut self,
        joint_id: JointId,
        lower: f32,
        upper: f32,
    ) -> Result<()> {
        validate_range(lower, upper)?;
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_SetLimits(joint_id.into_raw(), lower, upper) };
        })
    }

    pub fn try_enable_prismatic_joint_motor(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_EnableMotor(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_prismatic_joint_motor_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_IsMotorEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_prismatic_joint_motor_speed(
        &mut self,
        joint_id: JointId,
        speed: f32,
    ) -> Result<()> {
        validate_scalar(speed)?;
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_SetMotorSpeed(joint_id.into_raw(), speed) };
        })
    }

    pub fn try_prismatic_joint_motor_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetMotorSpeed(joint_id.into_raw()) }
        })
    }

    pub fn try_set_prismatic_joint_max_motor_force(
        &mut self,
        joint_id: JointId,
        force: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(force)?;
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_SetMaxMotorForce(joint_id.into_raw(), force) };
        })
    }

    pub fn try_prismatic_joint_max_motor_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetMaxMotorForce(joint_id.into_raw()) }
        })
    }

    pub fn try_prismatic_joint_motor_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetMotorForce(joint_id.into_raw()) }
        })
    }

    pub fn try_prismatic_joint_translation(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetTranslation(joint_id.into_raw()) }
        })
    }

    pub fn try_prismatic_joint_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetSpeed(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_revolute_joint_spring(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_EnableSpring(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_revolute_joint_spring_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_IsSpringEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_revolute_joint_spring_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_SetSpringHertz(joint_id.into_raw(), hertz) };
        })
    }

    pub fn try_revolute_joint_spring_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetSpringHertz(joint_id.into_raw()) }
        })
    }

    pub fn try_set_revolute_joint_spring_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(joint_id, JointType::Revolute, {
            unsafe {
                ffi::b3RevoluteJoint_SetSpringDampingRatio(joint_id.into_raw(), damping_ratio)
            };
        })
    }

    pub fn try_revolute_joint_spring_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetSpringDampingRatio(joint_id.into_raw()) }
        })
    }

    pub fn try_set_revolute_joint_target_angle(
        &mut self,
        joint_id: JointId,
        target: f32,
    ) -> Result<()> {
        validate_scalar(target)?;
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_SetTargetAngle(joint_id.into_raw(), target) };
        })
    }

    pub fn try_revolute_joint_target_angle(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetTargetAngle(joint_id.into_raw()) }
        })
    }

    pub fn try_revolute_joint_angle(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetAngle(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_revolute_joint_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_EnableLimit(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_revolute_joint_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_IsLimitEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_revolute_joint_lower_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetLowerLimit(joint_id.into_raw()) }
        })
    }

    pub fn try_revolute_joint_upper_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetUpperLimit(joint_id.into_raw()) }
        })
    }

    pub fn try_set_revolute_joint_limits(
        &mut self,
        joint_id: JointId,
        lower: f32,
        upper: f32,
    ) -> Result<()> {
        validate_range(lower, upper)?;
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_SetLimits(joint_id.into_raw(), lower, upper) };
        })
    }

    pub fn try_enable_revolute_joint_motor(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_EnableMotor(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_revolute_joint_motor_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_IsMotorEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_revolute_joint_motor_speed(
        &mut self,
        joint_id: JointId,
        speed: f32,
    ) -> Result<()> {
        validate_scalar(speed)?;
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_SetMotorSpeed(joint_id.into_raw(), speed) };
        })
    }

    pub fn try_revolute_joint_motor_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetMotorSpeed(joint_id.into_raw()) }
        })
    }

    pub fn try_revolute_joint_motor_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetMotorTorque(joint_id.into_raw()) }
        })
    }

    pub fn try_set_revolute_joint_max_motor_torque(
        &mut self,
        joint_id: JointId,
        torque: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(torque)?;
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_SetMaxMotorTorque(joint_id.into_raw(), torque) };
        })
    }

    pub fn try_revolute_joint_max_motor_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetMaxMotorTorque(joint_id.into_raw()) }
        })
    }
}

impl World {
    pub fn try_enable_spherical_joint_cone_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_EnableConeLimit(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_spherical_joint_cone_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_IsConeLimitEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_spherical_joint_cone_limit(
        &mut self,
        joint_id: JointId,
        angle: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(angle)?;
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_SetConeLimit(joint_id.into_raw(), angle) };
        })
    }

    pub fn try_spherical_joint_cone_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetConeLimit(joint_id.into_raw()) }
        })
    }

    pub fn try_spherical_joint_cone_angle(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetConeAngle(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_spherical_joint_twist_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_EnableTwistLimit(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_spherical_joint_twist_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_IsTwistLimitEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_spherical_joint_lower_twist_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetLowerTwistLimit(joint_id.into_raw()) }
        })
    }

    pub fn try_spherical_joint_upper_twist_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetUpperTwistLimit(joint_id.into_raw()) }
        })
    }

    pub fn try_set_spherical_joint_twist_limits(
        &mut self,
        joint_id: JointId,
        lower: f32,
        upper: f32,
    ) -> Result<()> {
        validate_range(lower, upper)?;
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_SetTwistLimits(joint_id.into_raw(), lower, upper) };
        })
    }

    pub fn try_spherical_joint_twist_angle(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetTwistAngle(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_spherical_joint_spring(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_EnableSpring(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_spherical_joint_spring_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_IsSpringEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_spherical_joint_spring_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_SetSpringHertz(joint_id.into_raw(), hertz) };
        })
    }

    pub fn try_spherical_joint_spring_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetSpringHertz(joint_id.into_raw()) }
        })
    }

    pub fn try_set_spherical_joint_spring_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(joint_id, JointType::Spherical, {
            unsafe {
                ffi::b3SphericalJoint_SetSpringDampingRatio(joint_id.into_raw(), damping_ratio)
            };
        })
    }

    pub fn try_spherical_joint_spring_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetSpringDampingRatio(joint_id.into_raw()) }
        })
    }

    pub fn try_set_spherical_joint_target_rotation(
        &mut self,
        joint_id: JointId,
        rotation: Quat,
    ) -> Result<()> {
        rotation.validate()?;
        family_method!(joint_id, JointType::Spherical, {
            unsafe {
                ffi::b3SphericalJoint_SetTargetRotation(joint_id.into_raw(), rotation.into_raw())
            };
        })
    }

    pub fn try_spherical_joint_target_rotation(&self, joint_id: JointId) -> Result<Quat> {
        family_method!(joint_id, JointType::Spherical, {
            Quat::from_raw(unsafe { ffi::b3SphericalJoint_GetTargetRotation(joint_id.into_raw()) })
        })
    }

    pub fn try_enable_spherical_joint_motor(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_EnableMotor(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_spherical_joint_motor_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_IsMotorEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_spherical_joint_motor_velocity(
        &mut self,
        joint_id: JointId,
        velocity: impl Into<Vec3>,
    ) -> Result<()> {
        let velocity = velocity.into().validate()?;
        family_method!(joint_id, JointType::Spherical, {
            unsafe {
                ffi::b3SphericalJoint_SetMotorVelocity(joint_id.into_raw(), velocity.into_raw())
            };
        })
    }

    pub fn try_spherical_joint_motor_velocity(&self, joint_id: JointId) -> Result<Vec3> {
        family_method!(joint_id, JointType::Spherical, {
            Vec3::from_raw(unsafe { ffi::b3SphericalJoint_GetMotorVelocity(joint_id.into_raw()) })
        })
    }

    pub fn try_spherical_joint_motor_torque(&self, joint_id: JointId) -> Result<Vec3> {
        family_method!(joint_id, JointType::Spherical, {
            Vec3::from_raw(unsafe { ffi::b3SphericalJoint_GetMotorTorque(joint_id.into_raw()) })
        })
    }

    pub fn try_set_spherical_joint_max_motor_torque(
        &mut self,
        joint_id: JointId,
        torque: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(torque)?;
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_SetMaxMotorTorque(joint_id.into_raw(), torque) };
        })
    }

    pub fn try_spherical_joint_max_motor_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetMaxMotorTorque(joint_id.into_raw()) }
        })
    }

    pub fn try_set_weld_joint_linear_hertz(&mut self, joint_id: JointId, hertz: f32) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_SetLinearHertz(joint_id.into_raw(), hertz) };
        })
    }

    pub fn try_weld_joint_linear_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_GetLinearHertz(joint_id.into_raw()) }
        })
    }

    pub fn try_set_weld_joint_linear_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_SetLinearDampingRatio(joint_id.into_raw(), damping_ratio) };
        })
    }

    pub fn try_weld_joint_linear_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_GetLinearDampingRatio(joint_id.into_raw()) }
        })
    }

    pub fn try_set_weld_joint_angular_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_SetAngularHertz(joint_id.into_raw(), hertz) };
        })
    }

    pub fn try_weld_joint_angular_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_GetAngularHertz(joint_id.into_raw()) }
        })
    }

    pub fn try_set_weld_joint_angular_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_SetAngularDampingRatio(joint_id.into_raw(), damping_ratio) };
        })
    }

    pub fn try_weld_joint_angular_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_GetAngularDampingRatio(joint_id.into_raw()) }
        })
    }
}

impl World {
    pub fn try_enable_wheel_joint_suspension(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_EnableSuspension(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_wheel_joint_suspension_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_IsSuspensionEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_wheel_joint_suspension_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetSuspensionHertz(joint_id.into_raw(), hertz) };
        })
    }

    pub fn try_wheel_joint_suspension_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSuspensionHertz(joint_id.into_raw()) }
        })
    }

    pub fn try_set_wheel_joint_suspension_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(joint_id, JointType::Wheel, {
            unsafe {
                ffi::b3WheelJoint_SetSuspensionDampingRatio(joint_id.into_raw(), damping_ratio)
            };
        })
    }

    pub fn try_wheel_joint_suspension_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSuspensionDampingRatio(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_wheel_joint_suspension_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_EnableSuspensionLimit(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_wheel_joint_suspension_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_IsSuspensionLimitEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_wheel_joint_lower_suspension_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetLowerSuspensionLimit(joint_id.into_raw()) }
        })
    }

    pub fn try_wheel_joint_upper_suspension_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetUpperSuspensionLimit(joint_id.into_raw()) }
        })
    }

    pub fn try_set_wheel_joint_suspension_limits(
        &mut self,
        joint_id: JointId,
        lower: f32,
        upper: f32,
    ) -> Result<()> {
        validate_range(lower, upper)?;
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetSuspensionLimits(joint_id.into_raw(), lower, upper) };
        })
    }

    pub fn try_enable_wheel_joint_spin_motor(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_EnableSpinMotor(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_wheel_joint_spin_motor_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_IsSpinMotorEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_wheel_joint_spin_motor_speed(
        &mut self,
        joint_id: JointId,
        speed: f32,
    ) -> Result<()> {
        validate_scalar(speed)?;
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetSpinMotorSpeed(joint_id.into_raw(), speed) };
        })
    }

    pub fn try_wheel_joint_spin_motor_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSpinMotorSpeed(joint_id.into_raw()) }
        })
    }

    pub fn try_set_wheel_joint_max_spin_torque(
        &mut self,
        joint_id: JointId,
        torque: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(torque)?;
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetMaxSpinTorque(joint_id.into_raw(), torque) };
        })
    }

    pub fn try_wheel_joint_max_spin_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetMaxSpinTorque(joint_id.into_raw()) }
        })
    }

    pub fn try_wheel_joint_spin_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSpinSpeed(joint_id.into_raw()) }
        })
    }

    pub fn try_wheel_joint_spin_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSpinTorque(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_wheel_joint_steering(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_EnableSteering(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_wheel_joint_steering_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_IsSteeringEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_wheel_joint_steering_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetSteeringHertz(joint_id.into_raw(), hertz) };
        })
    }

    pub fn try_wheel_joint_steering_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSteeringHertz(joint_id.into_raw()) }
        })
    }

    pub fn try_set_wheel_joint_steering_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(joint_id, JointType::Wheel, {
            unsafe {
                ffi::b3WheelJoint_SetSteeringDampingRatio(joint_id.into_raw(), damping_ratio)
            };
        })
    }

    pub fn try_wheel_joint_steering_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSteeringDampingRatio(joint_id.into_raw()) }
        })
    }

    pub fn try_set_wheel_joint_max_steering_torque(
        &mut self,
        joint_id: JointId,
        torque: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(torque)?;
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetMaxSteeringTorque(joint_id.into_raw(), torque) };
        })
    }

    pub fn try_wheel_joint_max_steering_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetMaxSteeringTorque(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_wheel_joint_steering_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_EnableSteeringLimit(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_wheel_joint_steering_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_IsSteeringLimitEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_wheel_joint_lower_steering_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetLowerSteeringLimit(joint_id.into_raw()) }
        })
    }

    pub fn try_wheel_joint_upper_steering_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetUpperSteeringLimit(joint_id.into_raw()) }
        })
    }

    pub fn try_set_wheel_joint_steering_limits(
        &mut self,
        joint_id: JointId,
        lower: f32,
        upper: f32,
    ) -> Result<()> {
        validate_range(lower, upper)?;
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetSteeringLimits(joint_id.into_raw(), lower, upper) };
        })
    }

    pub fn try_set_wheel_joint_target_steering_angle(
        &mut self,
        joint_id: JointId,
        radians: f32,
    ) -> Result<()> {
        validate_scalar(radians)?;
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetTargetSteeringAngle(joint_id.into_raw(), radians) };
        })
    }

    pub fn try_wheel_joint_target_steering_angle(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetTargetSteeringAngle(joint_id.into_raw()) }
        })
    }

    pub fn try_wheel_joint_steering_angle(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSteeringAngle(joint_id.into_raw()) }
        })
    }

    pub fn try_wheel_joint_steering_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSteeringTorque(joint_id.into_raw()) }
        })
    }
}

fn validate_base(base: &ffi::b3JointDef) -> Result<()> {
    Transform::from_raw(base.localFrameA).validate()?;
    Transform::from_raw(base.localFrameB).validate()?;
    validate_nonnegative_scalar(base.forceThreshold)?;
    validate_nonnegative_scalar(base.torqueThreshold)?;
    validate_nonnegative_scalar(base.constraintHertz)?;
    validate_nonnegative_scalar(base.constraintDampingRatio)?;
    validate_nonnegative_scalar(base.drawScale)
}

#[inline]
fn check_joint_body_pair_valid(body_a: BodyId, body_b: BodyId) -> Result<()> {
    if body_a.world0 == body_b.world0 && body_a != body_b {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}

#[inline]
fn check_joint_targets_world(
    world_id: ffi::b3WorldId,
    body_a: BodyId,
    body_b: BodyId,
) -> Result<()> {
    let world0 = world_id
        .index1
        .checked_sub(1)
        .ok_or(Error::InvalidWorldId)?;
    if body_a.world0 == world0 && body_b.world0 == world0 {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}

#[inline]
fn validate_scalar(value: f32) -> Result<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}

#[inline]
fn validate_nonnegative_scalar(value: f32) -> Result<()> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}

#[inline]
fn validate_range(lower: f32, upper: f32) -> Result<()> {
    validate_scalar(lower)?;
    validate_scalar(upper)?;
    if lower <= upper {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}

#[inline]
fn validate_length_range(lower: f32, upper: f32) -> Result<()> {
    validate_nonnegative_scalar(lower)?;
    validate_nonnegative_scalar(upper)?;
    if lower <= upper {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}

#[inline]
fn check_body_valid_raw(body_id: BodyId) -> Result<()> {
    if unsafe { ffi::b3Body_IsValid(body_id.into_raw()) } {
        Ok(())
    } else {
        Err(Error::InvalidBodyId)
    }
}

#[inline]
fn check_joint_valid_raw(joint_id: JointId) -> Result<()> {
    if unsafe { ffi::b3Joint_IsValid(joint_id.into_raw()) } {
        Ok(())
    } else {
        Err(Error::InvalidJointId)
    }
}

#[inline]
fn lock_joint_checked(joint_id: JointId) -> Result<std::sync::MutexGuard<'static, ()>> {
    callback_state::check_not_in_callback()?;
    let guard = box3d_lock::lock();
    check_joint_valid_raw(joint_id)?;
    Ok(guard)
}

#[inline]
fn lock_typed_joint_checked(
    joint_id: JointId,
    expected: JointType,
) -> Result<std::sync::MutexGuard<'static, ()>> {
    let guard = lock_joint_checked(joint_id)?;
    let actual = JointType::from_raw(unsafe { ffi::b3Joint_GetType(joint_id.into_raw()) })
        .ok_or(Error::WrongJointType)?;
    if actual == expected {
        Ok(guard)
    } else {
        Err(Error::WrongJointType)
    }
}

#[inline]
fn joint_id_from_raw(raw: ffi::b3JointId) -> Result<JointId> {
    if unsafe { ffi::b3Joint_IsValid(raw) } {
        Ok(JointId::from_raw(raw))
    } else {
        Err(Error::InvalidJointId)
    }
}
