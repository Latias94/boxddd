use super::*;
use crate::collision::{ShapeCastInput, ShapeProxy};
use crate::query::{BodyCastHit, BodyClosestPoint, QueryFilter};

impl World {
    pub fn body_position(&self, body_id: BodyId) -> Pos {
        self.try_body_position(body_id).expect("invalid BodyId")
    }

    #[inline]
    pub fn try_body_position(&self, body_id: BodyId) -> Result<Pos> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Pos::from_raw(unsafe {
            ffi::b3Body_GetPosition(body_id.into_raw())
        }))
    }

    #[inline]
    pub fn body_rotation(&self, body_id: BodyId) -> Quat {
        self.try_body_rotation(body_id).expect("invalid BodyId")
    }

    #[inline]
    pub fn try_body_rotation(&self, body_id: BodyId) -> Result<Quat> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Quat::from_raw(unsafe {
            ffi::b3Body_GetRotation(body_id.into_raw())
        }))
    }

    #[inline]
    pub fn body_transform(&self, body_id: BodyId) -> WorldTransform {
        self.try_body_transform(body_id).expect("invalid BodyId")
    }

    #[inline]
    pub fn try_body_transform(&self, body_id: BodyId) -> Result<WorldTransform> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(WorldTransform::from_raw(unsafe {
            ffi::b3Body_GetTransform(body_id.into_raw())
        }))
    }

    pub fn try_destroy_body(&mut self, body_id: BodyId) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        let shape_ids = body_shape_ids_locked(body_id);
        unsafe { ffi::b3DestroyBody(body_id.into_raw()) };
        drop(_guard);
        for shape_id in shape_ids {
            self.resources.remove(&shape_id);
        }
        Ok(())
    }

    #[track_caller]
    pub fn destroy_body(&mut self, body_id: BodyId) {
        self.try_destroy_body(body_id).expect("invalid BodyId");
    }
    pub fn try_body_type(&self, body_id: BodyId) -> Result<BodyType> {
        let _guard = self.lock_body_checked(body_id)?;
        BodyType::from_raw(unsafe { ffi::b3Body_GetType(body_id.into_raw()) })
            .ok_or(Error::InvalidArgument)
    }

    pub fn body_type(&self, body_id: BodyId) -> BodyType {
        self.try_body_type(body_id).expect("invalid BodyId")
    }

    pub fn try_set_body_type(&mut self, body_id: BodyId, body_type: BodyType) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetType(body_id.into_raw(), body_type.into_raw()) };
        Ok(())
    }

    pub fn try_set_body_name(&mut self, body_id: BodyId, name: impl Into<Vec<u8>>) -> Result<()> {
        let name = CString::new(name).map_err(|_| Error::NulByteInString)?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetName(body_id.into_raw(), name.as_ptr()) };
        Ok(())
    }

    pub fn try_body_name(&self, body_id: BodyId) -> Result<Option<String>> {
        let _guard = self.lock_body_checked(body_id)?;
        let ptr = unsafe { ffi::b3Body_GetName(body_id.into_raw()) };
        if ptr.is_null() {
            Ok(None)
        } else {
            Ok(Some(
                unsafe { CStr::from_ptr(ptr) }
                    .to_string_lossy()
                    .into_owned(),
            ))
        }
    }

    pub fn try_set_body_transform(
        &mut self,
        body_id: BodyId,
        position: impl Into<Pos>,
        rotation: Quat,
    ) -> Result<()> {
        let position = position.into().validate()?;
        rotation.validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe {
            ffi::b3Body_SetTransform(body_id.into_raw(), position.into_raw(), rotation.into_raw())
        };
        Ok(())
    }

    pub fn try_set_body_target_transform(
        &mut self,
        body_id: BodyId,
        target: WorldTransform,
        time_step: f32,
        wake: bool,
    ) -> Result<()> {
        if !target.is_valid() || !time_step.is_finite() || time_step <= 0.0 {
            return Err(Error::InvalidArgument);
        }
        let _guard = self.lock_body_checked(body_id)?;
        unsafe {
            ffi::b3Body_SetTargetTransform(body_id.into_raw(), target.into_raw(), time_step, wake)
        };
        Ok(())
    }

    pub fn try_body_local_point(
        &self,
        body_id: BodyId,
        world_point: impl Into<Pos>,
    ) -> Result<Vec3> {
        let world_point = world_point.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Body_GetLocalPoint(body_id.into_raw(), world_point.into_raw())
        }))
    }

    pub fn try_body_world_point(
        &self,
        body_id: BodyId,
        local_point: impl Into<Vec3>,
    ) -> Result<Pos> {
        let local_point = local_point.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Pos::from_raw(unsafe {
            ffi::b3Body_GetWorldPoint(body_id.into_raw(), local_point.into_raw())
        }))
    }

    pub fn try_body_local_vector(
        &self,
        body_id: BodyId,
        world_vector: impl Into<Vec3>,
    ) -> Result<Vec3> {
        let world_vector = world_vector.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Body_GetLocalVector(body_id.into_raw(), world_vector.into_raw())
        }))
    }

    pub fn try_body_world_vector(
        &self,
        body_id: BodyId,
        local_vector: impl Into<Vec3>,
    ) -> Result<Vec3> {
        let local_vector = local_vector.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Body_GetWorldVector(body_id.into_raw(), local_vector.into_raw())
        }))
    }

    pub fn try_body_local_point_velocity(
        &self,
        body_id: BodyId,
        local_point: impl Into<Vec3>,
    ) -> Result<Vec3> {
        let local_point = local_point.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Body_GetLocalPointVelocity(body_id.into_raw(), local_point.into_raw())
        }))
    }

    pub fn try_body_world_point_velocity(
        &self,
        body_id: BodyId,
        world_point: impl Into<Pos>,
    ) -> Result<Vec3> {
        let world_point = world_point.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Body_GetWorldPointVelocity(body_id.into_raw(), world_point.into_raw())
        }))
    }

    pub fn try_body_linear_velocity(&self, body_id: BodyId) -> Result<Vec3> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Body_GetLinearVelocity(body_id.into_raw())
        }))
    }

    pub fn body_linear_velocity(&self, body_id: BodyId) -> Vec3 {
        self.try_body_linear_velocity(body_id)
            .expect("invalid BodyId")
    }

    pub fn try_set_body_linear_velocity(
        &mut self,
        body_id: BodyId,
        velocity: impl Into<Vec3>,
    ) -> Result<()> {
        let velocity = velocity.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetLinearVelocity(body_id.into_raw(), velocity.into_raw()) };
        Ok(())
    }

    pub fn try_body_angular_velocity(&self, body_id: BodyId) -> Result<Vec3> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Body_GetAngularVelocity(body_id.into_raw())
        }))
    }

    pub fn body_angular_velocity(&self, body_id: BodyId) -> Vec3 {
        self.try_body_angular_velocity(body_id)
            .expect("invalid BodyId")
    }

    pub fn try_set_body_angular_velocity(
        &mut self,
        body_id: BodyId,
        velocity: impl Into<Vec3>,
    ) -> Result<()> {
        let velocity = velocity.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetAngularVelocity(body_id.into_raw(), velocity.into_raw()) };
        Ok(())
    }

    pub fn try_apply_force(
        &mut self,
        body_id: BodyId,
        force: impl Into<Vec3>,
        point: impl Into<Pos>,
        wake: bool,
    ) -> Result<()> {
        let force = force.into().validate()?;
        let point = point.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe {
            ffi::b3Body_ApplyForce(body_id.into_raw(), force.into_raw(), point.into_raw(), wake)
        };
        Ok(())
    }

    pub fn try_apply_force_to_center(
        &mut self,
        body_id: BodyId,
        force: impl Into<Vec3>,
        wake: bool,
    ) -> Result<()> {
        let force = force.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_ApplyForceToCenter(body_id.into_raw(), force.into_raw(), wake) };
        Ok(())
    }

    pub fn try_apply_torque(
        &mut self,
        body_id: BodyId,
        torque: impl Into<Vec3>,
        wake: bool,
    ) -> Result<()> {
        let torque = torque.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_ApplyTorque(body_id.into_raw(), torque.into_raw(), wake) };
        Ok(())
    }

    pub fn try_apply_linear_impulse(
        &mut self,
        body_id: BodyId,
        impulse: impl Into<Vec3>,
        point: impl Into<Pos>,
        wake: bool,
    ) -> Result<()> {
        let impulse = impulse.into().validate()?;
        let point = point.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe {
            ffi::b3Body_ApplyLinearImpulse(
                body_id.into_raw(),
                impulse.into_raw(),
                point.into_raw(),
                wake,
            )
        };
        Ok(())
    }

    pub fn try_apply_linear_impulse_to_center(
        &mut self,
        body_id: BodyId,
        impulse: impl Into<Vec3>,
        wake: bool,
    ) -> Result<()> {
        let impulse = impulse.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe {
            ffi::b3Body_ApplyLinearImpulseToCenter(body_id.into_raw(), impulse.into_raw(), wake)
        };
        Ok(())
    }

    pub fn try_apply_angular_impulse(
        &mut self,
        body_id: BodyId,
        impulse: impl Into<Vec3>,
        wake: bool,
    ) -> Result<()> {
        let impulse = impulse.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_ApplyAngularImpulse(body_id.into_raw(), impulse.into_raw(), wake) };
        Ok(())
    }

    pub fn try_body_mass(&self, body_id: BodyId) -> Result<f32> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_GetMass(body_id.into_raw()) })
    }

    pub fn try_body_local_rotational_inertia(&self, body_id: BodyId) -> Result<Matrix3> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Matrix3::from_raw(unsafe {
            ffi::b3Body_GetLocalRotationalInertia(body_id.into_raw())
        }))
    }

    pub fn try_body_inverse_mass(&self, body_id: BodyId) -> Result<f32> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_GetInverseMass(body_id.into_raw()) })
    }

    pub fn try_body_world_inverse_rotational_inertia(&self, body_id: BodyId) -> Result<Matrix3> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Matrix3::from_raw(unsafe {
            ffi::b3Body_GetWorldInverseRotationalInertia(body_id.into_raw())
        }))
    }

    pub fn try_body_local_center_of_mass(&self, body_id: BodyId) -> Result<Vec3> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Body_GetLocalCenterOfMass(body_id.into_raw())
        }))
    }

    pub fn try_body_world_center_of_mass(&self, body_id: BodyId) -> Result<Pos> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Pos::from_raw(unsafe {
            ffi::b3Body_GetWorldCenterOfMass(body_id.into_raw())
        }))
    }

    pub fn try_body_mass_data(&self, body_id: BodyId) -> Result<MassData> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(MassData::from_raw(unsafe {
            ffi::b3Body_GetMassData(body_id.into_raw())
        }))
    }

    pub fn try_set_body_mass_data(&mut self, body_id: BodyId, mass_data: MassData) -> Result<()> {
        validate_nonnegative_scalar(mass_data.mass)?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetMassData(body_id.into_raw(), mass_data.into_raw()) };
        Ok(())
    }

    pub fn try_apply_mass_from_shapes(&mut self, body_id: BodyId) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_ApplyMassFromShapes(body_id.into_raw()) };
        Ok(())
    }

    pub fn try_set_body_linear_damping(&mut self, body_id: BodyId, damping: f32) -> Result<()> {
        validate_nonnegative_scalar(damping)?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetLinearDamping(body_id.into_raw(), damping) };
        Ok(())
    }

    pub fn try_body_linear_damping(&self, body_id: BodyId) -> Result<f32> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_GetLinearDamping(body_id.into_raw()) })
    }

    pub fn try_set_body_angular_damping(&mut self, body_id: BodyId, damping: f32) -> Result<()> {
        validate_nonnegative_scalar(damping)?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetAngularDamping(body_id.into_raw(), damping) };
        Ok(())
    }

    pub fn try_body_angular_damping(&self, body_id: BodyId) -> Result<f32> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_GetAngularDamping(body_id.into_raw()) })
    }

    pub fn try_set_body_gravity_scale(&mut self, body_id: BodyId, scale: f32) -> Result<()> {
        validate_scalar(scale)?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetGravityScale(body_id.into_raw(), scale) };
        Ok(())
    }

    pub fn try_body_gravity_scale(&self, body_id: BodyId) -> Result<f32> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_GetGravityScale(body_id.into_raw()) })
    }

    pub fn try_body_awake(&self, body_id: BodyId) -> Result<bool> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_IsAwake(body_id.into_raw()) })
    }

    pub fn try_set_body_awake(&mut self, body_id: BodyId, awake: bool) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetAwake(body_id.into_raw(), awake) };
        Ok(())
    }

    pub fn try_enable_body_sleep(&mut self, body_id: BodyId, enabled: bool) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_EnableSleep(body_id.into_raw(), enabled) };
        Ok(())
    }

    pub fn try_body_sleep_enabled(&self, body_id: BodyId) -> Result<bool> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_IsSleepEnabled(body_id.into_raw()) })
    }

    pub fn try_set_body_sleep_threshold(&mut self, body_id: BodyId, threshold: f32) -> Result<()> {
        validate_nonnegative_scalar(threshold)?;
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetSleepThreshold(body_id.into_raw(), threshold) };
        Ok(())
    }

    pub fn try_body_sleep_threshold(&self, body_id: BodyId) -> Result<f32> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_GetSleepThreshold(body_id.into_raw()) })
    }

    pub fn try_body_enabled(&self, body_id: BodyId) -> Result<bool> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_IsEnabled(body_id.into_raw()) })
    }

    pub fn try_enable_body(&mut self, body_id: BodyId) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_Enable(body_id.into_raw()) };
        Ok(())
    }

    pub fn try_disable_body(&mut self, body_id: BodyId) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_Disable(body_id.into_raw()) };
        Ok(())
    }

    pub fn try_set_body_motion_locks(&mut self, body_id: BodyId, locks: MotionLocks) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetMotionLocks(body_id.into_raw(), locks.into_raw()) };
        Ok(())
    }

    pub fn try_body_motion_locks(&self, body_id: BodyId) -> Result<MotionLocks> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(MotionLocks::from_raw(unsafe {
            ffi::b3Body_GetMotionLocks(body_id.into_raw())
        }))
    }

    pub fn try_set_body_bullet(&mut self, body_id: BodyId, bullet: bool) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_SetBullet(body_id.into_raw(), bullet) };
        Ok(())
    }

    pub fn try_body_bullet(&self, body_id: BodyId) -> Result<bool> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_IsBullet(body_id.into_raw()) })
    }

    pub fn try_enable_body_contact_recycling(
        &mut self,
        body_id: BodyId,
        enabled: bool,
    ) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_EnableContactRecycling(body_id.into_raw(), enabled) };
        Ok(())
    }

    pub fn try_body_contact_recycling_enabled(&self, body_id: BodyId) -> Result<bool> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe { ffi::b3Body_IsContactRecyclingEnabled(body_id.into_raw()) })
    }

    pub fn try_enable_body_hit_events(&mut self, body_id: BodyId, enabled: bool) -> Result<()> {
        let _guard = self.lock_body_checked(body_id)?;
        unsafe { ffi::b3Body_EnableHitEvents(body_id.into_raw(), enabled) };
        Ok(())
    }

    pub fn try_body_aabb(&self, body_id: BodyId) -> Result<Aabb> {
        let _guard = self.lock_body_checked(body_id)?;
        Ok(Aabb::from_raw(unsafe {
            ffi::b3Body_ComputeAABB(body_id.into_raw())
        }))
    }

    pub fn try_body_closest_point(
        &self,
        body_id: BodyId,
        target: impl Into<Vec3>,
    ) -> Result<BodyClosestPoint> {
        let target = target.into().validate()?;
        let mut point = Vec3::ZERO.into_raw();
        let _guard = self.lock_body_checked(body_id)?;
        let distance = unsafe {
            ffi::b3Body_GetClosestPoint(body_id.into_raw(), &mut point, target.into_raw())
        };
        Ok(BodyClosestPoint {
            point: Vec3::from_raw(point),
            distance,
        })
    }

    pub fn try_body_cast_ray(
        &self,
        body_id: BodyId,
        origin: impl Into<Pos>,
        translation: impl Into<Vec3>,
        filter: QueryFilter,
    ) -> Result<Option<BodyCastHit>> {
        let body_transform = self.try_body_transform(body_id)?;
        self.try_body_cast_ray_with_transform(body_id, origin, translation, filter, body_transform)
    }

    pub fn try_body_cast_ray_with_transform(
        &self,
        body_id: BodyId,
        origin: impl Into<Pos>,
        translation: impl Into<Vec3>,
        filter: QueryFilter,
        body_transform: WorldTransform,
    ) -> Result<Option<BodyCastHit>> {
        self.try_body_cast_ray_with_options(
            body_id,
            origin,
            translation,
            filter,
            1.0,
            body_transform,
        )
    }

    pub fn try_body_cast_ray_with_options(
        &self,
        body_id: BodyId,
        origin: impl Into<Pos>,
        translation: impl Into<Vec3>,
        filter: QueryFilter,
        max_fraction: f32,
        body_transform: WorldTransform,
    ) -> Result<Option<BodyCastHit>> {
        let origin = origin.into().validate()?;
        let translation = translation.into().validate()?;
        validate_nonnegative_scalar(max_fraction)?;
        validate_world_transform(body_transform)?;
        let _guard = self.lock_body_checked(body_id)?;
        let raw = unsafe {
            ffi::b3Body_CastRay(
                body_id.into_raw(),
                origin.into_raw(),
                translation.into_raw(),
                filter.raw(),
                max_fraction,
                body_transform.into_raw(),
            )
        };
        Ok(BodyCastHit::from_raw(raw))
    }

    pub fn try_body_cast_shape(
        &self,
        body_id: BodyId,
        origin: impl Into<Pos>,
        input: ShapeCastInput,
        filter: QueryFilter,
    ) -> Result<Option<BodyCastHit>> {
        let body_transform = self.try_body_transform(body_id)?;
        self.try_body_cast_shape_with_transform(body_id, origin, input, filter, body_transform)
    }

    pub fn try_body_cast_shape_with_transform(
        &self,
        body_id: BodyId,
        origin: impl Into<Pos>,
        input: ShapeCastInput,
        filter: QueryFilter,
        body_transform: WorldTransform,
    ) -> Result<Option<BodyCastHit>> {
        let origin = origin.into().validate()?;
        validate_world_transform(body_transform)?;
        let raw_input = input.raw();
        let _guard = self.lock_body_checked(body_id)?;
        let raw = unsafe {
            ffi::b3Body_CastShape(
                body_id.into_raw(),
                origin.into_raw(),
                &raw_input.proxy,
                raw_input.translation,
                filter.raw(),
                raw_input.maxFraction,
                raw_input.canEncroach,
                body_transform.into_raw(),
            )
        };
        Ok(BodyCastHit::from_raw(raw))
    }

    pub fn try_body_overlap_shape(
        &self,
        body_id: BodyId,
        origin: impl Into<Pos>,
        proxy: &ShapeProxy,
        filter: QueryFilter,
    ) -> Result<bool> {
        let body_transform = self.try_body_transform(body_id)?;
        self.try_body_overlap_shape_with_transform(body_id, origin, proxy, filter, body_transform)
    }

    pub fn try_body_overlap_shape_with_transform(
        &self,
        body_id: BodyId,
        origin: impl Into<Pos>,
        proxy: &ShapeProxy,
        filter: QueryFilter,
        body_transform: WorldTransform,
    ) -> Result<bool> {
        let origin = origin.into().validate()?;
        validate_world_transform(body_transform)?;
        let raw_proxy = proxy.raw();
        let _guard = self.lock_body_checked(body_id)?;
        Ok(unsafe {
            ffi::b3Body_OverlapShape(
                body_id.into_raw(),
                origin.into_raw(),
                &raw_proxy,
                filter.raw(),
                body_transform.into_raw(),
            )
        })
    }

    pub fn body_shapes(&self, body_id: BodyId) -> Vec<ShapeId> {
        self.try_body_shapes(body_id).expect("invalid BodyId")
    }

    pub fn try_body_shapes(&self, body_id: BodyId) -> Result<Vec<ShapeId>> {
        let mut out = Vec::new();
        self.try_body_shapes_into(body_id, &mut out)?;
        Ok(out)
    }

    pub fn try_body_shapes_into(&self, body_id: BodyId, out: &mut Vec<ShapeId>) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_body_belongs_locked(body_id)?;
        let capacity = unsafe { ffi::b3Body_GetShapeCount(body_id.into_raw()) }.max(0) as usize;
        unsafe {
            ffi_vec::fill_from_ffi(out, capacity, |ptr, cap| {
                ffi::b3Body_GetShapes(body_id.into_raw(), ptr.cast(), cap)
            })
        };
        Ok(())
    }

    pub fn try_body_joints(&self, body_id: BodyId) -> Result<Vec<JointId>> {
        let mut out = Vec::new();
        self.try_body_joints_into(body_id, &mut out)?;
        Ok(out)
    }

    pub fn try_body_joints_into(&self, body_id: BodyId, out: &mut Vec<JointId>) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_body_belongs_locked(body_id)?;
        let capacity = unsafe { ffi::b3Body_GetJointCount(body_id.into_raw()) }.max(0) as usize;
        unsafe {
            ffi_vec::fill_from_ffi(out, capacity, |ptr, cap| {
                ffi::b3Body_GetJoints(body_id.into_raw(), ptr.cast(), cap)
            })
        };
        Ok(())
    }

    pub fn try_body_contacts(&self, body_id: BodyId) -> Result<Vec<ContactData>> {
        let mut out = Vec::new();
        self.try_body_contacts_into(body_id, &mut out)?;
        Ok(out)
    }

    pub fn try_body_contacts_into(
        &self,
        body_id: BodyId,
        out: &mut Vec<ContactData>,
    ) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_body_belongs_locked(body_id)?;
        let capacity =
            unsafe { ffi::b3Body_GetContactCapacity(body_id.into_raw()) }.max(0) as usize;
        let raw = unsafe {
            crate::core::ffi_vec::read_from_ffi(capacity, |ptr, cap| {
                ffi::b3Body_GetContactData(body_id.into_raw(), ptr, cap)
            })
        };
        out.clear();
        out.extend(
            raw.into_iter()
                .map(|contact| unsafe { ContactData::from_raw(contact) }),
        );
        Ok(())
    }
}

#[inline]
fn validate_world_transform(transform: WorldTransform) -> Result<()> {
    if transform.is_valid() {
        Ok(())
    } else {
        Err(Error::InvalidArgument)
    }
}
