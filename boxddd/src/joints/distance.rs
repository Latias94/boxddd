use super::*;

impl World {
    /// Tries to set the length on a distance joint.
    pub fn try_set_distance_joint_length(&mut self, joint_id: JointId, length: f32) -> Result<()> {
        validate_nonnegative_scalar(length)?;
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_SetLength(joint_id.into_raw(), length) };
        })
    }

    /// Returns the length of a distance joint.
    pub fn try_distance_joint_length(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetLength(joint_id.into_raw()) }
        })
    }

    /// Tries to enable or disable the spring on a distance joint.
    pub fn try_enable_distance_joint_spring(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_EnableSpring(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the spring is enabled on a distance joint.
    pub fn try_distance_joint_spring_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_IsSpringEnabled(joint_id.into_raw()) }
        })
    }

    /// Tries to set the spring force range on a distance joint.
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

    /// Returns the spring force range of a distance joint.
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

    /// Tries to set the spring hertz on a distance joint.
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

    /// Returns the spring hertz of a distance joint.
    pub fn try_distance_joint_spring_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetSpringHertz(joint_id.into_raw()) }
        })
    }

    /// Tries to set the spring damping ratio on a distance joint.
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

    /// Returns the spring damping ratio of a distance joint.
    pub fn try_distance_joint_spring_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetSpringDampingRatio(joint_id.into_raw()) }
        })
    }

    /// Tries to enable or disable the limit on a distance joint.
    pub fn try_enable_distance_joint_limit(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_EnableLimit(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the limit is enabled on a distance joint.
    pub fn try_distance_joint_limit_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_IsLimitEnabled(joint_id.into_raw()) }
        })
    }

    /// Tries to set the length range on a distance joint.
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

    /// Returns the min length of a distance joint.
    pub fn try_distance_joint_min_length(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMinLength(joint_id.into_raw()) }
        })
    }

    /// Returns the max length of a distance joint.
    pub fn try_distance_joint_max_length(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMaxLength(joint_id.into_raw()) }
        })
    }

    /// Returns the current length of a distance joint.
    pub fn try_distance_joint_current_length(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetCurrentLength(joint_id.into_raw()) }
        })
    }

    /// Tries to enable or disable the motor on a distance joint.
    pub fn try_enable_distance_joint_motor(
        &mut self,
        joint_id: JointId,
        enabled: bool,
    ) -> Result<()> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_EnableMotor(joint_id.into_raw(), enabled) };
        })
    }

    /// Returns whether the motor is enabled on a distance joint.
    pub fn try_distance_joint_motor_enabled(&self, joint_id: JointId) -> Result<bool> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_IsMotorEnabled(joint_id.into_raw()) }
        })
    }

    /// Tries to set the motor speed on a distance joint.
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

    /// Returns the motor speed of a distance joint.
    pub fn try_distance_joint_motor_speed(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMotorSpeed(joint_id.into_raw()) }
        })
    }

    /// Tries to set the max motor force on a distance joint.
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

    /// Returns the max motor force of a distance joint.
    pub fn try_distance_joint_max_motor_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMaxMotorForce(joint_id.into_raw()) }
        })
    }

    /// Returns the motor force of a distance joint.
    pub fn try_distance_joint_motor_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Distance, {
            unsafe { ffi::b3DistanceJoint_GetMotorForce(joint_id.into_raw()) }
        })
    }
}
