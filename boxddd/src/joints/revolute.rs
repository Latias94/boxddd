use super::*;

impl World {
    pub fn try_enable_revolute_joint_spring(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_EnableSpring(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_revolute_joint_spring_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_IsSpringEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_revolute_joint_spring_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_SetSpringHertz(joint_id.into_raw(), hertz) };
        })
    }

    pub fn try_revolute_joint_spring_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetSpringHertz(joint_id.into_raw()) }
        })
    }

    pub fn try_set_revolute_joint_spring_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe {
                ffi::b3RevoluteJoint_SetSpringDampingRatio(joint_id.into_raw(), damping_ratio)
            };
        })
    }

    pub fn try_revolute_joint_spring_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetSpringDampingRatio(joint_id.into_raw()) }
        })
    }

    pub fn try_set_revolute_joint_target_angle(
        &mut self,
        joint_id: JointId,
        target: f32,
    ) -> Result<()> {
        validate_scalar(target)?;
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_SetTargetAngle(joint_id.into_raw(), target) };
        })
    }

    pub fn try_revolute_joint_target_angle(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetTargetAngle(joint_id.into_raw()) }
        })
    }

    pub fn try_revolute_joint_angle(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetAngle(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_revolute_joint_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_EnableLimit(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_revolute_joint_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_IsLimitEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_revolute_joint_lower_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetLowerLimit(joint_id.into_raw()) }
        })
    }

    pub fn try_revolute_joint_upper_limit(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Revolute, {
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
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_SetLimits(joint_id.into_raw(), lower, upper) };
        })
    }

    pub fn try_enable_revolute_joint_motor(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_EnableMotor(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_revolute_joint_motor_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_IsMotorEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_revolute_joint_motor_speed(
        &mut self,
        joint_id: JointId,
        speed: f32,
    ) -> Result<()> {
        validate_scalar(speed)?;
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_SetMotorSpeed(joint_id.into_raw(), speed) };
        })
    }

    pub fn try_revolute_joint_motor_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetMotorSpeed(joint_id.into_raw()) }
        })
    }

    pub fn try_revolute_joint_motor_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetMotorTorque(joint_id.into_raw()) }
        })
    }

    pub fn try_set_revolute_joint_max_motor_torque(
        &mut self,
        joint_id: JointId,
        torque: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(torque)?;
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_SetMaxMotorTorque(joint_id.into_raw(), torque) };
        })
    }

    pub fn try_revolute_joint_max_motor_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Revolute, {
            unsafe { ffi::b3RevoluteJoint_GetMaxMotorTorque(joint_id.into_raw()) }
        })
    }
}
