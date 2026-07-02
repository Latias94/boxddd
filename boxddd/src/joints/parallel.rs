use super::*;

impl World {
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

    pub fn try_parallel_joint_spring_hertz(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Parallel, {
            unsafe { ffi::b3ParallelJoint_GetSpringHertz(joint_id.into_raw()) }
        })
    }

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

    pub fn try_parallel_joint_spring_damping_ratio(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Parallel, {
            unsafe { ffi::b3ParallelJoint_GetSpringDampingRatio(joint_id.into_raw()) }
        })
    }

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

    pub fn try_parallel_joint_max_torque(&self, joint_id: JointId) -> Result<f32> {
        family_method!(self, joint_id, JointType::Parallel, {
            unsafe { ffi::b3ParallelJoint_GetMaxTorque(joint_id.into_raw()) }
        })
    }
}
