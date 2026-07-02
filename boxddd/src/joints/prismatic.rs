use super::*;

impl World {
    pub fn try_enable_prismatic_joint_spring(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_EnableSpring(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_prismatic_joint_spring_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_IsSpringEnabled(joint_id.into_raw()) }
        })
    }

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

    pub fn try_prismatic_joint_spring_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetSpringHertz(joint_id.into_raw()) }
        })
    }

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

    pub fn try_prismatic_joint_spring_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetSpringDampingRatio(joint_id.into_raw()) }
        })
    }

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

    pub fn try_prismatic_joint_target_translation(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetTargetTranslation(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_prismatic_joint_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_EnableLimit(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_prismatic_joint_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_IsLimitEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_prismatic_joint_lower_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetLowerLimit(joint_id.into_raw()) }
        })
    }

    pub fn try_prismatic_joint_upper_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
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
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_SetLimits(joint_id.into_raw(), lower, upper) };
        })
    }

    pub fn try_enable_prismatic_joint_motor(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_EnableMotor(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_prismatic_joint_motor_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_IsMotorEnabled(joint_id.into_raw()) }
        })
    }

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

    pub fn try_prismatic_joint_motor_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetMotorSpeed(joint_id.into_raw()) }
        })
    }

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

    pub fn try_prismatic_joint_max_motor_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetMaxMotorForce(joint_id.into_raw()) }
        })
    }

    pub fn try_prismatic_joint_motor_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetMotorForce(joint_id.into_raw()) }
        })
    }

    pub fn try_prismatic_joint_translation(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetTranslation(joint_id.into_raw()) }
        })
    }

    pub fn try_prismatic_joint_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Prismatic, {
            unsafe { ffi::b3PrismaticJoint_GetSpeed(joint_id.into_raw()) }
        })
    }
}
