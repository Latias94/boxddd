use super::*;

impl World {
    pub fn try_set_distance_joint_length(&mut self, joint_id: JointId, length: f32) -> Result<()> {
        validate_nonnegative_scalar(length)?;
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_SetLength(joint_id.into_raw(), length) };
        })
    }

    pub fn try_distance_joint_length(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetLength(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_distance_joint_spring(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_EnableSpring(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_distance_joint_spring_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Distance, {
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
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_SetSpringForceRange(joint_id.into_raw(), lower, upper) };
        })
    }

    pub fn try_distance_joint_spring_force_range(&self, joint_id: JointId) -> Result<(f32, f32)> {
        family_method!(self, joint_id, JointType::Distance, {
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
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_SetSpringHertz(joint_id.into_raw(), hertz) };
        })
    }

    pub fn try_distance_joint_spring_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetSpringHertz(joint_id.into_raw()) }
        })
    }

    pub fn try_set_distance_joint_spring_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(self, joint_id, JointType::Distance, {
            unsafe {
                ffi::b3DistanceJoint_SetSpringDampingRatio(joint_id.into_raw(), damping_ratio)
            };
        })
    }

    pub fn try_distance_joint_spring_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetSpringDampingRatio(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_distance_joint_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_EnableLimit(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_distance_joint_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Distance, {
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
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_SetLengthRange(joint_id.into_raw(), min, max) };
        })
    }

    pub fn try_distance_joint_min_length(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMinLength(joint_id.into_raw()) }
        })
    }

    pub fn try_distance_joint_max_length(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMaxLength(joint_id.into_raw()) }
        })
    }

    pub fn try_distance_joint_current_length(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetCurrentLength(joint_id.into_raw()) }
        })
    }

    pub fn try_enable_distance_joint_motor(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_EnableMotor(joint_id.into_raw(), enabled) };
        })
    }

    pub fn try_distance_joint_motor_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_IsMotorEnabled(joint_id.into_raw()) }
        })
    }

    pub fn try_set_distance_joint_motor_speed(
        &mut self,
        joint_id: JointId,
        speed: f32,
    ) -> Result<()> {
        validate_scalar(speed)?;
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_SetMotorSpeed(joint_id.into_raw(), speed) };
        })
    }

    pub fn try_distance_joint_motor_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMotorSpeed(joint_id.into_raw()) }
        })
    }

    pub fn try_set_distance_joint_max_motor_force(
        &mut self,
        joint_id: JointId,
        force: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(force)?;
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_SetMaxMotorForce(joint_id.into_raw(), force) };
        })
    }

    pub fn try_distance_joint_max_motor_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMaxMotorForce(joint_id.into_raw()) }
        })
    }

    pub fn try_distance_joint_motor_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMotorForce(joint_id.into_raw()) }
        })
    }
}
