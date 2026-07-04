use super::*;

impl World {
    /// Tries to enable or disable the suspension on a wheel joint.
    ///
    /// All wheel-joint runtime methods return [`Error::WrongJointType`] when
    /// `joint_id` belongs to another joint family, and [`Error::InvalidJointId`]
    /// when the handle is stale or belongs to another world.
    pub fn try_enable_wheel_joint_suspension(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_EnableSuspension(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the suspension is enabled on a wheel joint.
    pub fn try_wheel_joint_suspension_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_IsSuspensionEnabled(joint_id.into_raw()) }
        })
    }

    /// Tries to set the suspension frequency, in hertz, on a wheel joint.
    pub fn try_set_wheel_joint_suspension_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetSuspensionHertz(joint_id.into_raw(), hertz) };
        })
    }

    /// Returns the suspension frequency, in hertz, of a wheel joint.
    pub fn try_wheel_joint_suspension_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSuspensionHertz(joint_id.into_raw()) }
        })
    }

    /// Tries to set the dimensionless suspension damping ratio on a wheel joint.
    pub fn try_set_wheel_joint_suspension_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe {
                ffi::b3WheelJoint_SetSuspensionDampingRatio(joint_id.into_raw(), damping_ratio)
            };
        })
    }

    /// Returns the dimensionless suspension damping ratio of a wheel joint.
    pub fn try_wheel_joint_suspension_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSuspensionDampingRatio(joint_id.into_raw()) }
        })
    }

    /// Tries to enable or disable the suspension limit on a wheel joint.
    pub fn try_enable_wheel_joint_suspension_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_EnableSuspensionLimit(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the suspension limit is enabled on a wheel joint.
    pub fn try_wheel_joint_suspension_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_IsSuspensionLimitEnabled(joint_id.into_raw()) }
        })
    }

    /// Returns the lower suspension limit, in world length units, of a wheel joint.
    pub fn try_wheel_joint_lower_suspension_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetLowerSuspensionLimit(joint_id.into_raw()) }
        })
    }

    /// Returns the upper suspension limit, in world length units, of a wheel joint.
    pub fn try_wheel_joint_upper_suspension_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetUpperSuspensionLimit(joint_id.into_raw()) }
        })
    }

    /// Tries to set suspension limits, in world length units, on a wheel joint.
    pub fn try_set_wheel_joint_suspension_limits(
        &mut self,
        joint_id: JointId,
        lower: f32,
        upper: f32,
    ) -> Result<()> {
        validate_range(lower, upper)?;
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetSuspensionLimits(joint_id.into_raw(), lower, upper) };
        })
    }

    /// Tries to enable or disable the spin motor on a wheel joint.
    pub fn try_enable_wheel_joint_spin_motor(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_EnableSpinMotor(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the spin motor is enabled on a wheel joint.
    pub fn try_wheel_joint_spin_motor_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_IsSpinMotorEnabled(joint_id.into_raw()) }
        })
    }

    /// Tries to set the spin motor speed, in radians per second, on a wheel joint.
    pub fn try_set_wheel_joint_spin_motor_speed(
        &mut self,
        joint_id: JointId,
        speed: f32,
    ) -> Result<()> {
        validate_scalar(speed)?;
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetSpinMotorSpeed(joint_id.into_raw(), speed) };
        })
    }

    /// Returns the spin motor speed, in radians per second, of a wheel joint.
    pub fn try_wheel_joint_spin_motor_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSpinMotorSpeed(joint_id.into_raw()) }
        })
    }

    /// Tries to set the maximum spin torque, in newton-meters, on a wheel joint.
    pub fn try_set_wheel_joint_max_spin_torque(
        &mut self,
        joint_id: JointId,
        torque: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(torque)?;
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetMaxSpinTorque(joint_id.into_raw(), torque) };
        })
    }

    /// Returns the maximum spin torque, in newton-meters, of a wheel joint.
    pub fn try_wheel_joint_max_spin_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetMaxSpinTorque(joint_id.into_raw()) }
        })
    }

    /// Returns the current spin speed, in radians per second, of a wheel joint.
    pub fn try_wheel_joint_spin_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSpinSpeed(joint_id.into_raw()) }
        })
    }

    /// Returns the current spin torque, in newton-meters, of a wheel joint.
    pub fn try_wheel_joint_spin_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSpinTorque(joint_id.into_raw()) }
        })
    }

    /// Tries to enable or disable the steering on a wheel joint.
    pub fn try_enable_wheel_joint_steering(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_EnableSteering(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the steering is enabled on a wheel joint.
    pub fn try_wheel_joint_steering_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_IsSteeringEnabled(joint_id.into_raw()) }
        })
    }

    /// Tries to set the steering frequency, in hertz, on a wheel joint.
    pub fn try_set_wheel_joint_steering_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetSteeringHertz(joint_id.into_raw(), hertz) };
        })
    }

    /// Returns the steering frequency, in hertz, of a wheel joint.
    pub fn try_wheel_joint_steering_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSteeringHertz(joint_id.into_raw()) }
        })
    }

    /// Tries to set the dimensionless steering damping ratio on a wheel joint.
    pub fn try_set_wheel_joint_steering_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe {
                ffi::b3WheelJoint_SetSteeringDampingRatio(joint_id.into_raw(), damping_ratio)
            };
        })
    }

    /// Returns the dimensionless steering damping ratio of a wheel joint.
    pub fn try_wheel_joint_steering_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSteeringDampingRatio(joint_id.into_raw()) }
        })
    }

    /// Tries to set the maximum steering torque, in newton-meters, on a wheel joint.
    pub fn try_set_wheel_joint_max_steering_torque(
        &mut self,
        joint_id: JointId,
        torque: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(torque)?;
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetMaxSteeringTorque(joint_id.into_raw(), torque) };
        })
    }

    /// Returns the maximum steering torque, in newton-meters, of a wheel joint.
    pub fn try_wheel_joint_max_steering_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetMaxSteeringTorque(joint_id.into_raw()) }
        })
    }

    /// Tries to enable or disable the steering limit on a wheel joint.
    pub fn try_enable_wheel_joint_steering_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_EnableSteeringLimit(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the steering limit is enabled on a wheel joint.
    pub fn try_wheel_joint_steering_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_IsSteeringLimitEnabled(joint_id.into_raw()) }
        })
    }

    /// Returns the lower steering limit, in radians, of a wheel joint.
    pub fn try_wheel_joint_lower_steering_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetLowerSteeringLimit(joint_id.into_raw()) }
        })
    }

    /// Returns the upper steering limit, in radians, of a wheel joint.
    pub fn try_wheel_joint_upper_steering_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetUpperSteeringLimit(joint_id.into_raw()) }
        })
    }

    /// Tries to set steering limits, in radians, on a wheel joint.
    pub fn try_set_wheel_joint_steering_limits(
        &mut self,
        joint_id: JointId,
        lower: f32,
        upper: f32,
    ) -> Result<()> {
        validate_range(lower, upper)?;
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetSteeringLimits(joint_id.into_raw(), lower, upper) };
        })
    }

    /// Tries to set the target steering angle, in radians, on a wheel joint.
    pub fn try_set_wheel_joint_target_steering_angle(
        &mut self,
        joint_id: JointId,
        radians: f32,
    ) -> Result<()> {
        validate_scalar(radians)?;
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_SetTargetSteeringAngle(joint_id.into_raw(), radians) };
        })
    }

    /// Returns the target steering angle, in radians, of a wheel joint.
    pub fn try_wheel_joint_target_steering_angle(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetTargetSteeringAngle(joint_id.into_raw()) }
        })
    }

    /// Returns the current steering angle, in radians, of a wheel joint.
    pub fn try_wheel_joint_steering_angle(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSteeringAngle(joint_id.into_raw()) }
        })
    }

    /// Returns the current steering torque, in newton-meters, of a wheel joint.
    pub fn try_wheel_joint_steering_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Wheel, {
            unsafe { ffi::b3WheelJoint_GetSteeringTorque(joint_id.into_raw()) }
        })
    }
}
