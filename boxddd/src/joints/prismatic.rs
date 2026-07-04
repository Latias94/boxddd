use super::*;

impl World {
    /// Tries to enable or disable the spring on a prismatic joint.
    pub fn try_enable_prismatic_joint_spring(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_EnableSpring(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the spring is enabled on a prismatic joint.
    pub fn try_prismatic_joint_spring_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_IsSpringEnabled(joint_id.into_raw()) }
        })
    }

    /// Tries to set the spring hertz on a prismatic joint.
    pub fn try_set_prismatic_joint_spring_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_SetSpringHertz(joint_id.into_raw(), hertz) };
        })
    }

    /// Returns the spring hertz of a prismatic joint.
    pub fn try_prismatic_joint_spring_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetSpringHertz(joint_id.into_raw()) }
        })
    }

    /// Tries to set the spring damping ratio on a prismatic joint.
    pub fn try_set_prismatic_joint_spring_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe {
                ffi::b3PrismaticJoint_SetSpringDampingRatio(joint_id.into_raw(), damping_ratio)
            };
        })
    }

    /// Returns the spring damping ratio of a prismatic joint.
    pub fn try_prismatic_joint_spring_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetSpringDampingRatio(joint_id.into_raw()) }
        })
    }

    /// Tries to set the target translation on a prismatic joint.
    pub fn try_set_prismatic_joint_target_translation(
        &mut self,
        joint_id: JointId,
        target: f32,
    ) -> Result<()> {
        validate_scalar(target)?;
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_SetTargetTranslation(joint_id.into_raw(), target) };
        })
    }

    /// Returns the target translation of a prismatic joint.
    pub fn try_prismatic_joint_target_translation(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetTargetTranslation(joint_id.into_raw()) }
        })
    }

    /// Tries to enable or disable the limit on a prismatic joint.
    pub fn try_enable_prismatic_joint_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_EnableLimit(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the limit is enabled on a prismatic joint.
    pub fn try_prismatic_joint_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_IsLimitEnabled(joint_id.into_raw()) }
        })
    }

    /// Returns the lower limit of a prismatic joint.
    pub fn try_prismatic_joint_lower_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetLowerLimit(joint_id.into_raw()) }
        })
    }

    /// Returns the upper limit of a prismatic joint.
    pub fn try_prismatic_joint_upper_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetUpperLimit(joint_id.into_raw()) }
        })
    }

    /// Tries to set the limits on a prismatic joint.
    pub fn try_set_prismatic_joint_limits(
        &mut self,
        joint_id: JointId,
        lower: f32,
        upper: f32,
    ) -> Result<()> {
        validate_range(lower, upper)?;
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_SetLimits(joint_id.into_raw(), lower, upper) };
        })
    }

    /// Tries to enable or disable the motor on a prismatic joint.
    pub fn try_enable_prismatic_joint_motor(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_EnableMotor(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the motor is enabled on a prismatic joint.
    pub fn try_prismatic_joint_motor_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_IsMotorEnabled(joint_id.into_raw()) }
        })
    }

    /// Tries to set the motor speed on a prismatic joint.
    pub fn try_set_prismatic_joint_motor_speed(
        &mut self,
        joint_id: JointId,
        speed: f32,
    ) -> Result<()> {
        validate_scalar(speed)?;
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_SetMotorSpeed(joint_id.into_raw(), speed) };
        })
    }

    /// Returns the motor speed of a prismatic joint.
    pub fn try_prismatic_joint_motor_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetMotorSpeed(joint_id.into_raw()) }
        })
    }

    /// Tries to set the max motor force on a prismatic joint.
    pub fn try_set_prismatic_joint_max_motor_force(
        &mut self,
        joint_id: JointId,
        force: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(force)?;
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_SetMaxMotorForce(joint_id.into_raw(), force) };
        })
    }

    /// Returns the max motor force of a prismatic joint.
    pub fn try_prismatic_joint_max_motor_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetMaxMotorForce(joint_id.into_raw()) }
        })
    }

    /// Returns the motor force of a prismatic joint.
    pub fn try_prismatic_joint_motor_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetMotorForce(joint_id.into_raw()) }
        })
    }

    /// Returns the translation of a prismatic joint.
    pub fn try_prismatic_joint_translation(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetTranslation(joint_id.into_raw()) }
        })
    }

    /// Returns the speed of a prismatic joint.
    pub fn try_prismatic_joint_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetSpeed(joint_id.into_raw()) }
        })
    }
}
