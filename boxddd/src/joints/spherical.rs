use super::*;

impl World {
    /// Tries to enable or disable the cone limit on a spherical joint.
    ///
    /// All spherical-joint and weld-joint runtime methods in this module return
    /// [`Error::WrongJointType`] when `joint_id` belongs to another joint family,
    /// and [`Error::InvalidJointId`] when the handle is stale or belongs to
    /// another world.
    pub fn try_enable_spherical_joint_cone_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_EnableConeLimit(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the cone limit is enabled on a spherical joint.
    pub fn try_spherical_joint_cone_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_IsConeLimitEnabled(joint_id.into_raw()) }
        })
    }

    /// Tries to set the cone limit angle, in radians, on a spherical joint.
    pub fn try_set_spherical_joint_cone_limit(
        &mut self,
        joint_id: JointId,
        angle: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(angle)?;
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_SetConeLimit(joint_id.into_raw(), angle) };
        })
    }

    /// Returns the cone limit angle, in radians, of a spherical joint.
    pub fn try_spherical_joint_cone_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetConeLimit(joint_id.into_raw()) }
        })
    }

    /// Returns the current cone angle, in radians, of a spherical joint.
    pub fn try_spherical_joint_cone_angle(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetConeAngle(joint_id.into_raw()) }
        })
    }

    /// Tries to enable or disable the twist limit on a spherical joint.
    pub fn try_enable_spherical_joint_twist_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_EnableTwistLimit(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the twist limit is enabled on a spherical joint.
    pub fn try_spherical_joint_twist_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_IsTwistLimitEnabled(joint_id.into_raw()) }
        })
    }

    /// Returns the lower twist limit, in radians, of a spherical joint.
    pub fn try_spherical_joint_lower_twist_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetLowerTwistLimit(joint_id.into_raw()) }
        })
    }

    /// Returns the upper twist limit, in radians, of a spherical joint.
    pub fn try_spherical_joint_upper_twist_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetUpperTwistLimit(joint_id.into_raw()) }
        })
    }

    /// Tries to set twist limits, in radians, on a spherical joint.
    pub fn try_set_spherical_joint_twist_limits(
        &mut self,
        joint_id: JointId,
        lower: f32,
        upper: f32,
    ) -> Result<()> {
        validate_range(lower, upper)?;
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_SetTwistLimits(joint_id.into_raw(), lower, upper) };
        })
    }

    /// Returns the current twist angle, in radians, of a spherical joint.
    pub fn try_spherical_joint_twist_angle(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetTwistAngle(joint_id.into_raw()) }
        })
    }

    /// Tries to enable or disable the spring on a spherical joint.
    pub fn try_enable_spherical_joint_spring(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_EnableSpring(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the spring is enabled on a spherical joint.
    pub fn try_spherical_joint_spring_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_IsSpringEnabled(joint_id.into_raw()) }
        })
    }

    /// Tries to set the spring frequency, in hertz, on a spherical joint.
    pub fn try_set_spherical_joint_spring_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_SetSpringHertz(joint_id.into_raw(), hertz) };
        })
    }

    /// Returns the spring frequency, in hertz, of a spherical joint.
    pub fn try_spherical_joint_spring_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetSpringHertz(joint_id.into_raw()) }
        })
    }

    /// Tries to set the dimensionless spring damping ratio on a spherical joint.
    pub fn try_set_spherical_joint_spring_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe {
                ffi::b3SphericalJoint_SetSpringDampingRatio(joint_id.into_raw(), damping_ratio)
            };
        })
    }

    /// Returns the dimensionless spring damping ratio of a spherical joint.
    pub fn try_spherical_joint_spring_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetSpringDampingRatio(joint_id.into_raw()) }
        })
    }

    /// Tries to set the target rotation on a spherical joint.
    pub fn try_set_spherical_joint_target_rotation(
        &mut self,
        joint_id: JointId,
        rotation: Quat,
    ) -> Result<()> {
        rotation.validate()?;
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe {
                ffi::b3SphericalJoint_SetTargetRotation(joint_id.into_raw(), rotation.into_raw())
            };
        })
    }

    /// Returns the target rotation of a spherical joint.
    pub fn try_spherical_joint_target_rotation(&self, joint_id: JointId) -> Result<Quat> {
        family_method!(self, joint_id, JointType::Spherical, {
            Quat::from_raw(unsafe { ffi::b3SphericalJoint_GetTargetRotation(joint_id.into_raw()) })
        })
    }

    /// Tries to enable or disable the motor on a spherical joint.
    pub fn try_enable_spherical_joint_motor(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_EnableMotor(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the motor is enabled on a spherical joint.
    pub fn try_spherical_joint_motor_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_IsMotorEnabled(joint_id.into_raw()) }
        })
    }

    /// Tries to set the motor angular velocity, in radians per second, on a spherical joint.
    pub fn try_set_spherical_joint_motor_velocity(
        &mut self,
        joint_id: JointId,
        velocity: impl Into<Vec3>,
    ) -> Result<()> {
        let velocity = velocity.into().validate()?;
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe {
                ffi::b3SphericalJoint_SetMotorVelocity(joint_id.into_raw(), velocity.into_raw())
            };
        })
    }

    /// Returns the motor angular velocity, in radians per second, of a spherical joint.
    pub fn try_spherical_joint_motor_velocity(&self, joint_id: JointId) -> Result<Vec3> {
        family_method!(self, joint_id, JointType::Spherical, {
            Vec3::from_raw(unsafe { ffi::b3SphericalJoint_GetMotorVelocity(joint_id.into_raw()) })
        })
    }

    /// Returns the current motor torque, in newton-meters, of a spherical joint.
    pub fn try_spherical_joint_motor_torque(&self, joint_id: JointId) -> Result<Vec3> {
        family_method!(self, joint_id, JointType::Spherical, {
            Vec3::from_raw(unsafe { ffi::b3SphericalJoint_GetMotorTorque(joint_id.into_raw()) })
        })
    }

    /// Tries to set the maximum motor torque, in newton-meters, on a spherical joint.
    pub fn try_set_spherical_joint_max_motor_torque(
        &mut self,
        joint_id: JointId,
        torque: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(torque)?;
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_SetMaxMotorTorque(joint_id.into_raw(), torque) };
        })
    }

    /// Returns the maximum motor torque, in newton-meters, of a spherical joint.
    pub fn try_spherical_joint_max_motor_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Spherical, {
            unsafe { ffi::b3SphericalJoint_GetMaxMotorTorque(joint_id.into_raw()) }
        })
    }

    /// Tries to set the linear spring frequency, in hertz, on a weld joint.
    pub fn try_set_weld_joint_linear_hertz(&mut self, joint_id: JointId, hertz: f32) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(self, joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_SetLinearHertz(joint_id.into_raw(), hertz) };
        })
    }

    /// Returns the linear spring frequency, in hertz, of a weld joint.
    pub fn try_weld_joint_linear_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_GetLinearHertz(joint_id.into_raw()) }
        })
    }

    /// Tries to set the dimensionless linear damping ratio on a weld joint.
    pub fn try_set_weld_joint_linear_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(self, joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_SetLinearDampingRatio(joint_id.into_raw(), damping_ratio) };
        })
    }

    /// Returns the dimensionless linear damping ratio of a weld joint.
    pub fn try_weld_joint_linear_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_GetLinearDampingRatio(joint_id.into_raw()) }
        })
    }

    /// Tries to set the angular spring frequency, in hertz, on a weld joint.
    pub fn try_set_weld_joint_angular_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(self, joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_SetAngularHertz(joint_id.into_raw(), hertz) };
        })
    }

    /// Returns the angular spring frequency, in hertz, of a weld joint.
    pub fn try_weld_joint_angular_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_GetAngularHertz(joint_id.into_raw()) }
        })
    }

    /// Tries to set the dimensionless angular damping ratio on a weld joint.
    pub fn try_set_weld_joint_angular_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(self, joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_SetAngularDampingRatio(joint_id.into_raw(), damping_ratio) };
        })
    }

    /// Returns the dimensionless angular damping ratio of a weld joint.
    pub fn try_weld_joint_angular_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Weld, {
            unsafe { ffi::b3WeldJoint_GetAngularDampingRatio(joint_id.into_raw()) }
        })
    }
}
