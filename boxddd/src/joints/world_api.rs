use super::*;

impl World {
    pub fn try_create_parallel_joint(&mut self, def: ParallelJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw().base, |world| unsafe {
            ffi::b3CreateParallelJoint(world, def.raw())
        })
    }

    pub fn create_parallel_joint(&mut self, def: ParallelJointDef) -> JointId {
        self.try_create_parallel_joint(def)
            .expect("Box3D failed to create parallel joint")
    }

    pub fn try_create_distance_joint(&mut self, def: DistanceJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw().base, |world| unsafe {
            ffi::b3CreateDistanceJoint(world, def.raw())
        })
    }

    pub fn create_distance_joint(&mut self, def: DistanceJointDef) -> JointId {
        self.try_create_distance_joint(def)
            .expect("Box3D failed to create distance joint")
    }

    pub fn try_create_motor_joint(&mut self, def: MotorJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw().base, |world| unsafe {
            ffi::b3CreateMotorJoint(world, def.raw())
        })
    }

    pub fn create_motor_joint(&mut self, def: MotorJointDef) -> JointId {
        self.try_create_motor_joint(def)
            .expect("Box3D failed to create motor joint")
    }

    pub fn try_create_filter_joint(&mut self, def: FilterJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw().base, |world| unsafe {
            ffi::b3CreateFilterJoint(world, def.raw())
        })
    }

    pub fn create_filter_joint(&mut self, def: FilterJointDef) -> JointId {
        self.try_create_filter_joint(def)
            .expect("Box3D failed to create filter joint")
    }

    pub fn try_create_prismatic_joint(&mut self, def: PrismaticJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw().base, |world| unsafe {
            ffi::b3CreatePrismaticJoint(world, def.raw())
        })
    }

    pub fn create_prismatic_joint(&mut self, def: PrismaticJointDef) -> JointId {
        self.try_create_prismatic_joint(def)
            .expect("Box3D failed to create prismatic joint")
    }

    pub fn try_create_revolute_joint(&mut self, def: RevoluteJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw().base, |world| unsafe {
            ffi::b3CreateRevoluteJoint(world, def.raw())
        })
    }

    pub fn create_revolute_joint(&mut self, def: RevoluteJointDef) -> JointId {
        self.try_create_revolute_joint(def)
            .expect("Box3D failed to create revolute joint")
    }

    pub fn try_create_spherical_joint(&mut self, def: SphericalJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw().base, |world| unsafe {
            ffi::b3CreateSphericalJoint(world, def.raw())
        })
    }

    pub fn create_spherical_joint(&mut self, def: SphericalJointDef) -> JointId {
        self.try_create_spherical_joint(def)
            .expect("Box3D failed to create spherical joint")
    }

    pub fn try_create_weld_joint(&mut self, def: WeldJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw().base, |world| unsafe {
            ffi::b3CreateWeldJoint(world, def.raw())
        })
    }

    pub fn create_weld_joint(&mut self, def: WeldJointDef) -> JointId {
        self.try_create_weld_joint(def)
            .expect("Box3D failed to create weld joint")
    }

    pub fn try_create_wheel_joint(&mut self, def: WheelJointDef) -> Result<JointId> {
        def.validate()?;
        self.create_joint(def.raw().base, |world| unsafe {
            ffi::b3CreateWheelJoint(world, def.raw())
        })
    }

    pub fn create_wheel_joint(&mut self, def: WheelJointDef) -> JointId {
        self.try_create_wheel_joint(def)
            .expect("Box3D failed to create wheel joint")
    }

    fn create_joint(
        &mut self,
        base: ffi::b3JointDef,
        create: impl FnOnce(ffi::b3WorldId) -> ffi::b3JointId,
    ) -> Result<JointId> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let body_a = BodyId::from_raw(base.bodyIdA);
        let body_b = BodyId::from_raw(base.bodyIdB);
        debug_checks::check_body_valid_raw(body_a)?;
        debug_checks::check_body_valid_raw(body_b)?;
        check_joint_body_pair_valid(body_a, body_b)?;
        check_joint_targets_world(self.raw(), body_a, body_b)?;
        joint_id_from_raw(create(self.raw()))
    }

    pub fn try_destroy_joint(&mut self, joint_id: JointId, wake_attached: bool) -> Result<()> {
        let _guard = lock_joint_checked(self, joint_id)?;
        unsafe { ffi::b3DestroyJoint(joint_id.into_raw(), wake_attached) };
        Ok(())
    }

    pub fn destroy_joint(&mut self, joint_id: JointId, wake_attached: bool) {
        self.try_destroy_joint(joint_id, wake_attached)
            .expect("invalid JointId");
    }

    pub fn try_joint_type(&self, joint_id: JointId) -> Result<JointType> {
        let _guard = lock_joint_checked(self, joint_id)?;
        JointType::from_raw(unsafe { ffi::b3Joint_GetType(joint_id.into_raw()) })
            .ok_or(Error::WrongJointType)
    }

    pub fn joint_type(&self, joint_id: JointId) -> JointType {
        self.try_joint_type(joint_id).expect("invalid JointId")
    }

    pub fn try_joint_body_a(&self, joint_id: JointId) -> Result<BodyId> {
        let _guard = lock_joint_checked(self, joint_id)?;
        Ok(BodyId::from_raw(unsafe {
            ffi::b3Joint_GetBodyA(joint_id.into_raw())
        }))
    }

    pub fn try_joint_body_b(&self, joint_id: JointId) -> Result<BodyId> {
        let _guard = lock_joint_checked(self, joint_id)?;
        Ok(BodyId::from_raw(unsafe {
            ffi::b3Joint_GetBodyB(joint_id.into_raw())
        }))
    }

    pub fn try_set_joint_local_frame_a(
        &mut self,
        joint_id: JointId,
        frame: Transform,
    ) -> Result<()> {
        frame.validate()?;
        let _guard = lock_joint_checked(self, joint_id)?;
        unsafe { ffi::b3Joint_SetLocalFrameA(joint_id.into_raw(), frame.into_raw()) };
        Ok(())
    }

    pub fn try_joint_local_frame_a(&self, joint_id: JointId) -> Result<Transform> {
        let _guard = lock_joint_checked(self, joint_id)?;
        Ok(Transform::from_raw(unsafe {
            ffi::b3Joint_GetLocalFrameA(joint_id.into_raw())
        }))
    }

    pub fn try_set_joint_local_frame_b(
        &mut self,
        joint_id: JointId,
        frame: Transform,
    ) -> Result<()> {
        frame.validate()?;
        let _guard = lock_joint_checked(self, joint_id)?;
        unsafe { ffi::b3Joint_SetLocalFrameB(joint_id.into_raw(), frame.into_raw()) };
        Ok(())
    }

    pub fn try_joint_local_frame_b(&self, joint_id: JointId) -> Result<Transform> {
        let _guard = lock_joint_checked(self, joint_id)?;
        Ok(Transform::from_raw(unsafe {
            ffi::b3Joint_GetLocalFrameB(joint_id.into_raw())
        }))
    }

    pub fn try_set_joint_collide_connected(
        &mut self,
        joint_id: JointId,
        collide: bool,
    ) -> Result<()> {
        let _guard = lock_joint_checked(self, joint_id)?;
        unsafe { ffi::b3Joint_SetCollideConnected(joint_id.into_raw(), collide) };
        Ok(())
    }

    pub fn try_joint_collide_connected(&self, joint_id: JointId) -> Result<bool> {
        let _guard = lock_joint_checked(self, joint_id)?;
        Ok(unsafe { ffi::b3Joint_GetCollideConnected(joint_id.into_raw()) })
    }

    pub fn try_wake_joint_bodies(&mut self, joint_id: JointId) -> Result<()> {
        let _guard = lock_joint_checked(self, joint_id)?;
        unsafe { ffi::b3Joint_WakeBodies(joint_id.into_raw()) };
        Ok(())
    }

    pub fn try_joint_constraint_force(&self, joint_id: JointId) -> Result<Vec3> {
        let _guard = lock_joint_checked(self, joint_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Joint_GetConstraintForce(joint_id.into_raw())
        }))
    }

    pub fn try_joint_constraint_torque(&self, joint_id: JointId) -> Result<Vec3> {
        let _guard = lock_joint_checked(self, joint_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Joint_GetConstraintTorque(joint_id.into_raw())
        }))
    }

    pub fn try_joint_linear_separation(&self, joint_id: JointId) -> Result<f32> {
        let _guard = lock_joint_checked(self, joint_id)?;
        Ok(unsafe { ffi::b3Joint_GetLinearSeparation(joint_id.into_raw()) })
    }

    pub fn try_joint_angular_separation(&self, joint_id: JointId) -> Result<f32> {
        let _guard = lock_joint_checked(self, joint_id)?;
        Ok(unsafe { ffi::b3Joint_GetAngularSeparation(joint_id.into_raw()) })
    }

    pub fn try_set_joint_constraint_tuning(
        &mut self,
        joint_id: JointId,
        tuning: JointTuning,
    ) -> Result<()> {
        tuning.validate()?;
        let _guard = lock_joint_checked(self, joint_id)?;
        unsafe {
            ffi::b3Joint_SetConstraintTuning(
                joint_id.into_raw(),
                tuning.hertz,
                tuning.damping_ratio,
            )
        };
        Ok(())
    }

    pub fn try_joint_constraint_tuning(&self, joint_id: JointId) -> Result<JointTuning> {
        let _guard = lock_joint_checked(self, joint_id)?;
        let mut hertz = 0.0;
        let mut damping_ratio = 0.0;
        unsafe {
            ffi::b3Joint_GetConstraintTuning(joint_id.into_raw(), &mut hertz, &mut damping_ratio)
        };
        Ok(JointTuning::new(hertz, damping_ratio))
    }

    pub fn try_set_joint_force_threshold(
        &mut self,
        joint_id: JointId,
        threshold: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(threshold)?;
        let _guard = lock_joint_checked(self, joint_id)?;
        unsafe { ffi::b3Joint_SetForceThreshold(joint_id.into_raw(), threshold) };
        Ok(())
    }

    pub fn try_joint_force_threshold(&self, joint_id: JointId) -> Result<f32> {
        let _guard = lock_joint_checked(self, joint_id)?;
        Ok(unsafe { ffi::b3Joint_GetForceThreshold(joint_id.into_raw()) })
    }

    pub fn try_set_joint_torque_threshold(
        &mut self,
        joint_id: JointId,
        threshold: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(threshold)?;
        let _guard = lock_joint_checked(self, joint_id)?;
        unsafe { ffi::b3Joint_SetTorqueThreshold(joint_id.into_raw(), threshold) };
        Ok(())
    }

    pub fn try_joint_torque_threshold(&self, joint_id: JointId) -> Result<f32> {
        let _guard = lock_joint_checked(self, joint_id)?;
        Ok(unsafe { ffi::b3Joint_GetTorqueThreshold(joint_id.into_raw()) })
    }
}
