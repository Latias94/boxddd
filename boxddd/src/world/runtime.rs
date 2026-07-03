use super::*;

impl World {
    pub fn step(&mut self, time_step: f32, sub_step_count: i32) {
        self.try_step(time_step, sub_step_count)
            .expect("Box3D failed to step world");
    }

    #[inline]
    pub fn try_step(&mut self, time_step: f32, sub_step_count: i32) -> Result<()> {
        callback_state::check_not_in_callback()?;
        if !time_step.is_finite() || time_step < 0.0 || sub_step_count < 0 {
            return Err(Error::InvalidArgument);
        }
        let _guard = box3d_lock::lock();
        self.callbacks.reset_panics();
        if let Some(task_system) = self.task_system.as_ref() {
            task_system.reset_panics();
        }
        unsafe { ffi::b3World_Step(self.raw, time_step, sub_step_count) };
        if self.callbacks.panicked()
            || self
                .task_system
                .as_ref()
                .is_some_and(|task_system| task_system.panicked())
        {
            return Err(Error::CallbackPanicked);
        }
        Ok(())
    }

    #[inline]
    pub fn gravity(&self) -> Vec3 {
        self.try_gravity().expect("invalid Box3D world")
    }

    #[inline]
    pub fn try_gravity(&self) -> Result<Vec3> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        if !unsafe { ffi::b3World_IsValid(self.raw) } {
            return Err(Error::InvalidWorldId);
        }
        Ok(Vec3::from_raw(unsafe { ffi::b3World_GetGravity(self.raw) }))
    }

    #[inline]
    pub fn set_gravity(&mut self, gravity: impl Into<Vec3>) {
        self.try_set_gravity(gravity)
            .expect("invalid gravity or Box3D world");
    }

    #[inline]
    pub fn try_set_gravity(&mut self, gravity: impl Into<Vec3>) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let gravity = gravity.into().validate()?;
        let _guard = box3d_lock::lock();
        if !unsafe { ffi::b3World_IsValid(self.raw) } {
            return Err(Error::InvalidWorldId);
        }
        unsafe { ffi::b3World_SetGravity(self.raw, gravity.into_raw()) };
        Ok(())
    }

    pub fn explode(&mut self, explosion: &ExplosionDef) {
        self.try_explode(explosion)
            .expect("invalid explosion or Box3D world");
    }

    pub fn try_explode(&mut self, explosion: &ExplosionDef) -> Result<()> {
        callback_state::check_not_in_callback()?;
        explosion.validate()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_Explode(self.raw, explosion.raw()) };
        Ok(())
    }

    pub fn bounds(&self) -> Aabb {
        self.try_bounds().expect("invalid Box3D world")
    }

    #[inline]
    pub fn try_bounds(&self) -> Result<Aabb> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        if !unsafe { ffi::b3World_IsValid(self.raw) } {
            return Err(Error::InvalidWorldId);
        }
        Ok(Aabb::from_raw(unsafe { ffi::b3World_GetBounds(self.raw) }))
    }

    #[inline]
    pub fn profile(&self) -> Profile {
        self.try_profile().expect("invalid Box3D world")
    }

    #[inline]
    pub fn try_profile(&self) -> Result<Profile> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        if !unsafe { ffi::b3World_IsValid(self.raw) } {
            return Err(Error::InvalidWorldId);
        }
        Ok(Profile::from_raw(unsafe {
            ffi::b3World_GetProfile(self.raw)
        }))
    }

    #[inline]
    pub fn counters(&self) -> Counters {
        self.try_counters().expect("invalid Box3D world")
    }

    #[inline]
    pub fn try_counters(&self) -> Result<Counters> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        if !unsafe { ffi::b3World_IsValid(self.raw) } {
            return Err(Error::InvalidWorldId);
        }
        Ok(Counters::from_raw(unsafe {
            ffi::b3World_GetCounters(self.raw)
        }))
    }

    pub fn try_enable_sleeping(&mut self, enabled: bool) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_EnableSleeping(self.raw, enabled) };
        Ok(())
    }

    pub fn enable_sleeping(&mut self, enabled: bool) {
        self.try_enable_sleeping(enabled)
            .expect("invalid Box3D world");
    }

    pub fn try_sleeping_enabled(&self) -> Result<bool> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_IsSleepingEnabled(self.raw) })
    }

    pub fn sleeping_enabled(&self) -> bool {
        self.try_sleeping_enabled().expect("invalid Box3D world")
    }

    pub fn try_enable_continuous(&mut self, enabled: bool) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_EnableContinuous(self.raw, enabled) };
        Ok(())
    }

    pub fn enable_continuous(&mut self, enabled: bool) {
        self.try_enable_continuous(enabled)
            .expect("invalid Box3D world");
    }

    pub fn try_continuous_enabled(&self) -> Result<bool> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_IsContinuousEnabled(self.raw) })
    }

    pub fn continuous_enabled(&self) -> bool {
        self.try_continuous_enabled().expect("invalid Box3D world")
    }

    pub fn try_set_restitution_threshold(&mut self, value: f32) -> Result<()> {
        validate_nonnegative_scalar(value)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_SetRestitutionThreshold(self.raw, value) };
        Ok(())
    }

    pub fn restitution_threshold(&self) -> f32 {
        self.try_restitution_threshold()
            .expect("invalid Box3D world")
    }

    pub fn try_restitution_threshold(&self) -> Result<f32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_GetRestitutionThreshold(self.raw) })
    }

    pub fn try_set_hit_event_threshold(&mut self, value: f32) -> Result<()> {
        validate_nonnegative_scalar(value)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_SetHitEventThreshold(self.raw, value) };
        Ok(())
    }

    pub fn hit_event_threshold(&self) -> f32 {
        self.try_hit_event_threshold().expect("invalid Box3D world")
    }

    pub fn try_hit_event_threshold(&self) -> Result<f32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_GetHitEventThreshold(self.raw) })
    }

    pub fn try_set_contact_tuning(
        &mut self,
        hertz: f32,
        damping_ratio: f32,
        contact_speed: f32,
    ) -> Result<()> {
        validate_nonnegative_scalar(hertz)?;
        validate_nonnegative_scalar(damping_ratio)?;
        validate_nonnegative_scalar(contact_speed)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_SetContactTuning(self.raw, hertz, damping_ratio, contact_speed) };
        Ok(())
    }

    pub fn try_set_contact_recycle_distance(&mut self, distance: f32) -> Result<()> {
        validate_nonnegative_scalar(distance)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_SetContactRecycleDistance(self.raw, distance) };
        Ok(())
    }

    pub fn contact_recycle_distance(&self) -> f32 {
        self.try_contact_recycle_distance()
            .expect("invalid Box3D world")
    }

    pub fn try_contact_recycle_distance(&self) -> Result<f32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_GetContactRecycleDistance(self.raw) })
    }

    pub fn try_set_maximum_linear_speed(&mut self, speed: f32) -> Result<()> {
        validate_nonnegative_scalar(speed)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_SetMaximumLinearSpeed(self.raw, speed) };
        Ok(())
    }

    pub fn maximum_linear_speed(&self) -> f32 {
        self.try_maximum_linear_speed()
            .expect("invalid Box3D world")
    }

    pub fn try_maximum_linear_speed(&self) -> Result<f32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_GetMaximumLinearSpeed(self.raw) })
    }

    pub fn try_enable_warm_starting(&mut self, enabled: bool) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_EnableWarmStarting(self.raw, enabled) };
        Ok(())
    }

    pub fn warm_starting_enabled(&self) -> bool {
        self.try_warm_starting_enabled()
            .expect("invalid Box3D world")
    }

    pub fn try_warm_starting_enabled(&self) -> Result<bool> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_IsWarmStartingEnabled(self.raw) })
    }

    pub fn try_enable_speculative(&mut self, enabled: bool) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_EnableSpeculative(self.raw, enabled) };
        Ok(())
    }

    pub fn awake_body_count(&self) -> i32 {
        self.try_awake_body_count().expect("invalid Box3D world")
    }

    pub fn try_awake_body_count(&self) -> Result<i32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_GetAwakeBodyCount(self.raw) })
    }

    pub fn max_capacity(&self) -> Capacity {
        self.try_max_capacity().expect("invalid Box3D world")
    }

    pub fn try_max_capacity(&self) -> Result<Capacity> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(Capacity::from_raw(unsafe {
            ffi::b3World_GetMaxCapacity(self.raw)
        }))
    }

    pub fn try_set_worker_count(&mut self, count: i32) -> Result<()> {
        if count < 0 {
            return Err(Error::InvalidArgument);
        }
        #[cfg(target_arch = "wasm32")]
        if count > 1 {
            return Err(Error::UnsupportedOnWasm);
        }
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_SetWorkerCount(self.raw, count) };
        Ok(())
    }

    pub fn worker_count(&self) -> i32 {
        self.try_worker_count().expect("invalid Box3D world")
    }

    pub fn try_worker_count(&self) -> Result<i32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        Ok(unsafe { ffi::b3World_GetWorkerCount(self.raw) })
    }

    pub fn try_rebuild_static_tree(&mut self) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        unsafe { ffi::b3World_RebuildStaticTree(self.raw) };
        Ok(())
    }
}
