use super::*;

impl World {
    /// Tries to set the spring frequency, in hertz, on a parallel joint.
    ///
    /// All parallel-joint runtime methods return [`Error::WrongJointType`] when
    /// `joint_id` belongs to another joint family, and [`Error::InvalidJointId`]
    /// when the handle is stale or belongs to another world.
    pub fn try_set_parallel_joint_spring_hertz(
        &mut self,
        joint_id: JointId,
        hertz: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        family_method!(self, joint_id, JointType::Parallel, {
            unsafe { ffi::b3ParallelJoint_SetSpringHertz(joint_id.into_raw(), hertz) };
        })
    }

    /// Returns the spring frequency, in hertz, of a parallel joint.
    pub fn try_parallel_joint_spring_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Parallel, {
            unsafe { ffi::b3ParallelJoint_GetSpringHertz(joint_id.into_raw()) }
        })
    }

    /// Tries to set the dimensionless spring damping ratio on a parallel joint.
    pub fn try_set_parallel_joint_spring_damping_ratio(
        &mut self,
        joint_id: JointId,
        damping_ratio: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(damping_ratio)?;
        family_method!(self, joint_id, JointType::Parallel, {
            unsafe {
                ffi::b3ParallelJoint_SetSpringDampingRatio(joint_id.into_raw(), damping_ratio)
            };
        })
    }

    /// Returns the dimensionless spring damping ratio of a parallel joint.
    pub fn try_parallel_joint_spring_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Parallel, {
            unsafe { ffi::b3ParallelJoint_GetSpringDampingRatio(joint_id.into_raw()) }
        })
    }

    /// Tries to set the maximum spring torque, in newton-meters, on a parallel joint.
    pub fn try_set_parallel_joint_max_torque(
        &mut self,
        joint_id: JointId,
        torque: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(torque)?;
        family_method!(self, joint_id, JointType::Parallel, {
            unsafe { ffi::b3ParallelJoint_SetMaxTorque(joint_id.into_raw(), torque) };
        })
    }

    /// Returns the maximum spring torque, in newton-meters, of a parallel joint.
    pub fn try_parallel_joint_max_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Parallel, {
            unsafe { ffi::b3ParallelJoint_GetMaxTorque(joint_id.into_raw()) }
        })
    }
}
