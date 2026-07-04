use super::*;

impl World {
    /// Tries to set the linear velocity on a motor joint.
    pub fn try_set_motor_joint_linear_velocity(
        &mut self,
        joint_id: JointId,
        velocity: impl Into<Vec3>,
    ) -> Result<()> {
        let velocity = velocity.into().validate()?;
        family_method!(self, joint_id, JointType::Motor, {
            unsafe {
                ffi::b3MotorJoint_SetLinearVelocity(joint_id.into_raw(), velocity.into_raw())
            };
        })
    }

    /// Returns the linear velocity of a motor joint.
    pub fn try_motor_joint_linear_velocity(&self, joint_id: JointId) -> Result<Vec3> {
        family_method!(self, joint_id, JointType::Motor, {
            Vec3::from_raw(unsafe { ffi::b3MotorJoint_GetLinearVelocity(joint_id.into_raw()) })
        })
    }

    /// Tries to set the angular velocity on a motor joint.
    pub fn try_set_motor_joint_angular_velocity(
        &mut self,
        joint_id: JointId,
        velocity: impl Into<Vec3>,
    ) -> Result<()> {
        let velocity = velocity.into().validate()?;
        family_method!(self, joint_id, JointType::Motor, {
            unsafe {
                ffi::b3MotorJoint_SetAngularVelocity(joint_id.into_raw(), velocity.into_raw())
            };
        })
    }

    /// Returns the angular velocity of a motor joint.
    pub fn try_motor_joint_angular_velocity(&self, joint_id: JointId) -> Result<Vec3> {
        family_method!(self, joint_id, JointType::Motor, {
            Vec3::from_raw(unsafe { ffi::b3MotorJoint_GetAngularVelocity(joint_id.into_raw()) })
        })
    }

    /// Tries to set the max velocity force on a motor joint.
    pub fn try_set_motor_joint_max_velocity_force(
        &mut self,
        joint_id: JointId,
        force: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(force)?;
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetMaxVelocityForce(joint_id.into_raw(), force) };
        })
    }

    /// Returns the max velocity force of a motor joint.
    pub fn try_motor_joint_max_velocity_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetMaxVelocityForce(joint_id.into_raw()) }
        })
    }

    /// Tries to set the max velocity torque on a motor joint.
    pub fn try_set_motor_joint_max_velocity_torque(
        &mut self,
        joint_id: JointId,
        torque: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(torque)?;
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetMaxVelocityTorque(joint_id.into_raw(), torque) };
        })
    }

    /// Returns the max velocity torque of a motor joint.
    pub fn try_motor_joint_max_velocity_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetMaxVelocityTorque(joint_id.into_raw()) }
        })
    }

    /// Tries to set the linear hertz on a motor joint.
    pub fn try_set_motor_joint_linear_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetLinearHertz(joint_id.into_raw(), hertz) };
        })
    }

    /// Returns the linear hertz of a motor joint.
    pub fn try_motor_joint_linear_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetLinearHertz(joint_id.into_raw()) }
        })
    }

    /// Tries to set the linear damping ratio on a motor joint.
    pub fn try_set_motor_joint_linear_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping)?;
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetLinearDampingRatio(joint_id.into_raw(), damping) };
        })
    }

    /// Returns the linear damping ratio of a motor joint.
    pub fn try_motor_joint_linear_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetLinearDampingRatio(joint_id.into_raw()) }
        })
    }

    /// Tries to set the angular hertz on a motor joint.
    pub fn try_set_motor_joint_angular_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetAngularHertz(joint_id.into_raw(), hertz) };
        })
    }

    /// Returns the angular hertz of a motor joint.
    pub fn try_motor_joint_angular_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetAngularHertz(joint_id.into_raw()) }
        })
    }

    /// Tries to set the angular damping ratio on a motor joint.
    pub fn try_set_motor_joint_angular_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping)?;
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetAngularDampingRatio(joint_id.into_raw(), damping) };
        })
    }

    /// Returns the angular damping ratio of a motor joint.
    pub fn try_motor_joint_angular_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetAngularDampingRatio(joint_id.into_raw()) }
        })
    }

    /// Tries to set the max spring force on a motor joint.
    pub fn try_set_motor_joint_max_spring_force(
        &mut self,
        joint_id: JointId,
        force: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(force)?;
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetMaxSpringForce(joint_id.into_raw(), force) };
        })
    }

    /// Returns the max spring force of a motor joint.
    pub fn try_motor_joint_max_spring_force(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetMaxSpringForce(joint_id.into_raw()) }
        })
    }

    /// Tries to set the max spring torque on a motor joint.
    pub fn try_set_motor_joint_max_spring_torque(
        &mut self,
        joint_id: JointId,
        torque: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(torque)?;
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_SetMaxSpringTorque(joint_id.into_raw(), torque) };
        })
    }

    /// Returns the max spring torque of a motor joint.
    pub fn try_motor_joint_max_spring_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Motor, {
            unsafe { ffi::b3MotorJoint_GetMaxSpringTorque(joint_id.into_raw()) }
        })
    }
}
