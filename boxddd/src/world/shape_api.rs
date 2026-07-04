use super::*;
use crate::query::ShapeRayHit;

impl World {
    /// Tries to destroy a shape attached to this world.
    pub fn try_destroy_shape(&mut self, shape_id: ShapeId, update_body_mass: bool) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3DestroyShape(shape_id.into_raw(), update_body_mass) };
        drop(_guard);
        self.resources.remove(&shape_id);
        Ok(())
    }

    /// Destroys a shape or panics if the shape handle is invalid.
    pub fn destroy_shape(&mut self, shape_id: ShapeId, update_body_mass: bool) {
        self.try_destroy_shape(shape_id, update_body_mass)
            .expect("invalid ShapeId");
    }

    /// Tries to return the shape type.
    pub fn try_shape_type(&self, shape_id: ShapeId) -> Result<ShapeType> {
        let _guard = self.lock_shape_checked(shape_id)?;
        ShapeType::from_raw(unsafe { ffi::b3Shape_GetType(shape_id.into_raw()) })
            .ok_or(Error::InvalidArgument)
    }

    /// Tries to return the body that owns the shape.
    pub fn try_shape_body(&self, shape_id: ShapeId) -> Result<BodyId> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(BodyId::from_raw(unsafe {
            ffi::b3Shape_GetBody(shape_id.into_raw())
        }))
    }

    /// Tries to return whether the shape is configured as a sensor.
    pub fn try_shape_sensor(&self, shape_id: ShapeId) -> Result<bool> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_IsSensor(shape_id.into_raw()) })
    }

    /// Tries to set the shape density, optionally updating body mass.
    pub fn try_set_shape_density(
        &mut self,
        shape_id: ShapeId,
        density: f32,
        update_body_mass: bool,
    ) -> Result<()> {
        validate_nonnegative_scalar(density)?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetDensity(shape_id.into_raw(), density, update_body_mass) };
        Ok(())
    }

    /// Tries to return the shape density.
    pub fn try_shape_density(&self, shape_id: ShapeId) -> Result<f32> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_GetDensity(shape_id.into_raw()) })
    }

    /// Tries to set the shape friction coefficient.
    pub fn try_set_shape_friction(&mut self, shape_id: ShapeId, friction: f32) -> Result<()> {
        validate_nonnegative_scalar(friction)?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetFriction(shape_id.into_raw(), friction) };
        Ok(())
    }

    /// Tries to return the shape friction coefficient.
    pub fn try_shape_friction(&self, shape_id: ShapeId) -> Result<f32> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_GetFriction(shape_id.into_raw()) })
    }

    /// Tries to set the shape restitution coefficient.
    pub fn try_set_shape_restitution(&mut self, shape_id: ShapeId, restitution: f32) -> Result<()> {
        validate_nonnegative_scalar(restitution)?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetRestitution(shape_id.into_raw(), restitution) };
        Ok(())
    }

    /// Tries to return the shape restitution coefficient.
    pub fn try_shape_restitution(&self, shape_id: ShapeId) -> Result<f32> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_GetRestitution(shape_id.into_raw()) })
    }

    /// Tries to replace the shape's base surface material.
    pub fn try_set_shape_surface_material(
        &mut self,
        shape_id: ShapeId,
        material: SurfaceMaterial,
    ) -> Result<()> {
        material.validate()?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetSurfaceMaterial(shape_id.into_raw(), material.into_raw()) };
        Ok(())
    }

    /// Tries to return the shape's base surface material.
    pub fn try_shape_surface_material(&self, shape_id: ShapeId) -> Result<SurfaceMaterial> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(SurfaceMaterial::from_raw(unsafe {
            ffi::b3Shape_GetSurfaceMaterial(shape_id.into_raw())
        }))
    }

    /// Tries to return the number of mesh material slots on the shape.
    pub fn try_shape_mesh_material_count(&self, shape_id: ShapeId) -> Result<i32> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_GetMeshMaterialCount(shape_id.into_raw()) })
    }

    /// Tries to set a mesh material slot on the shape.
    pub fn try_set_shape_mesh_material(
        &mut self,
        shape_id: ShapeId,
        index: i32,
        material: SurfaceMaterial,
    ) -> Result<()> {
        material.validate()?;
        let _guard = self.lock_shape_checked(shape_id)?;
        let count = unsafe { ffi::b3Shape_GetMeshMaterialCount(shape_id.into_raw()) };
        if index < 0 || index >= count {
            return Err(Error::IndexOutOfRange);
        }
        unsafe { ffi::b3Shape_SetMeshMaterial(shape_id.into_raw(), material.into_raw(), index) };
        Ok(())
    }

    /// Tries to return a mesh material slot from the shape.
    pub fn try_shape_mesh_surface_material(
        &self,
        shape_id: ShapeId,
        index: i32,
    ) -> Result<SurfaceMaterial> {
        let _guard = self.lock_shape_checked(shape_id)?;
        let count = unsafe { ffi::b3Shape_GetMeshMaterialCount(shape_id.into_raw()) };
        if index < 0 || index >= count {
            return Err(Error::IndexOutOfRange);
        }
        Ok(SurfaceMaterial::from_raw(unsafe {
            ffi::b3Shape_GetMeshSurfaceMaterial(shape_id.into_raw(), index)
        }))
    }

    /// Tries to return the shape collision filter.
    pub fn try_shape_filter(&self, shape_id: ShapeId) -> Result<Filter> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(Filter::from_raw(unsafe {
            ffi::b3Shape_GetFilter(shape_id.into_raw())
        }))
    }

    /// Tries to set the shape collision filter.
    pub fn try_set_shape_filter(
        &mut self,
        shape_id: ShapeId,
        filter: Filter,
        invoke_contacts: bool,
    ) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetFilter(shape_id.into_raw(), filter.into_raw(), invoke_contacts) };
        Ok(())
    }

    /// Tries to enable or disable begin/end sensor events for the shape.
    pub fn try_enable_shape_sensor_events(
        &mut self,
        shape_id: ShapeId,
        enabled: bool,
    ) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_EnableSensorEvents(shape_id.into_raw(), enabled) };
        Ok(())
    }

    /// Tries to return whether sensor events are enabled for the shape.
    pub fn try_shape_sensor_events_enabled(&self, shape_id: ShapeId) -> Result<bool> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_AreSensorEventsEnabled(shape_id.into_raw()) })
    }

    /// Tries to enable or disable contact begin/end events for the shape.
    pub fn try_enable_shape_contact_events(
        &mut self,
        shape_id: ShapeId,
        enabled: bool,
    ) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_EnableContactEvents(shape_id.into_raw(), enabled) };
        Ok(())
    }

    /// Tries to return whether contact events are enabled for the shape.
    pub fn try_shape_contact_events_enabled(&self, shape_id: ShapeId) -> Result<bool> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_AreContactEventsEnabled(shape_id.into_raw()) })
    }

    /// Tries to enable or disable pre-solve callbacks for the shape.
    pub fn try_enable_shape_pre_solve_events(
        &mut self,
        shape_id: ShapeId,
        enabled: bool,
    ) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_EnablePreSolveEvents(shape_id.into_raw(), enabled) };
        Ok(())
    }

    /// Tries to return whether pre-solve callbacks are enabled for the shape.
    pub fn try_shape_pre_solve_events_enabled(&self, shape_id: ShapeId) -> Result<bool> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_ArePreSolveEventsEnabled(shape_id.into_raw()) })
    }

    /// Tries to enable or disable hit events for the shape.
    pub fn try_enable_shape_hit_events(&mut self, shape_id: ShapeId, enabled: bool) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_EnableHitEvents(shape_id.into_raw(), enabled) };
        Ok(())
    }

    /// Tries to return whether hit events are enabled for the shape.
    pub fn try_shape_hit_events_enabled(&self, shape_id: ShapeId) -> Result<bool> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_AreHitEventsEnabled(shape_id.into_raw()) })
    }

    /// Tries to return the shape's world-space AABB.
    pub fn try_shape_aabb(&self, shape_id: ShapeId) -> Result<Aabb> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(Aabb::from_raw(unsafe {
            ffi::b3Shape_GetAABB(shape_id.into_raw())
        }))
    }

    /// Tries to ray cast against a single shape.
    pub fn try_shape_cast_ray(
        &self,
        shape_id: ShapeId,
        origin: impl Into<Pos>,
        translation: impl Into<Vec3>,
    ) -> Result<Option<ShapeRayHit>> {
        let origin = origin.into().validate()?;
        let translation = translation.into().validate()?;
        let _guard = self.lock_shape_checked(shape_id)?;
        let raw = unsafe {
            ffi::b3Shape_RayCast(
                shape_id.into_raw(),
                origin.into_raw(),
                translation.into_raw(),
            )
        };
        Ok(ShapeRayHit::from_raw(raw))
    }

    /// Tries to compute mass data for the shape.
    pub fn try_shape_mass_data(&self, shape_id: ShapeId) -> Result<MassData> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(MassData::from_raw(unsafe {
            ffi::b3Shape_ComputeMassData(shape_id.into_raw())
        }))
    }

    /// Tries to return the closest point on the shape to `target`.
    pub fn try_shape_closest_point(
        &self,
        shape_id: ShapeId,
        target: impl Into<Vec3>,
    ) -> Result<Vec3> {
        let target = target.into().validate()?;
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(Vec3::from_raw(unsafe {
            ffi::b3Shape_GetClosestPoint(shape_id.into_raw(), target.into_raw())
        }))
    }

    /// Tries to collect current contacts touching the shape.
    pub fn try_shape_contacts(&self, shape_id: ShapeId) -> Result<Vec<ContactData>> {
        let mut out = Vec::new();
        self.try_shape_contacts_into(shape_id, &mut out)?;
        Ok(out)
    }

    /// Tries to write current contacts touching the shape into `out`.
    pub fn try_shape_contacts_into(
        &self,
        shape_id: ShapeId,
        out: &mut Vec<ContactData>,
    ) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        let capacity =
            unsafe { ffi::b3Shape_GetContactCapacity(shape_id.into_raw()) }.max(0) as usize;
        let raw = unsafe {
            ffi_vec::read_from_ffi(capacity, |ptr, cap| {
                ffi::b3Shape_GetContactData(shape_id.into_raw(), ptr, cap)
            })
        };
        out.clear();
        out.extend(
            raw.into_iter()
                .map(|contact| unsafe { ContactData::from_raw(contact) }),
        );
        Ok(())
    }

    /// Tries to collect shapes currently touching this sensor shape.
    pub fn try_shape_sensor_data(&self, shape_id: ShapeId) -> Result<Vec<ShapeId>> {
        let mut out = Vec::new();
        self.try_shape_sensor_data_into(shape_id, &mut out)?;
        Ok(out)
    }

    /// Tries to write shapes currently touching this sensor shape into `out`.
    pub fn try_shape_sensor_data_into(
        &self,
        shape_id: ShapeId,
        out: &mut Vec<ShapeId>,
    ) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        let world0 = self.world0_locked()?;
        let capacity =
            unsafe { ffi::b3Shape_GetSensorCapacity(shape_id.into_raw()) }.max(0) as usize;
        let raw = unsafe {
            ffi_vec::read_from_ffi(capacity, |ptr: *mut ffi::b3ShapeId, cap| {
                ffi::b3Shape_GetSensorData(shape_id.into_raw(), ptr, cap)
            })
        };
        out.clear();
        out.extend(raw.into_iter().filter_map(|raw_shape| {
            let shape = ShapeId::from_raw(raw_shape);
            (unsafe { ffi::b3Shape_IsValid(raw_shape) } && shape.world0 == world0).then_some(shape)
        }));
        Ok(())
    }

    /// Tries to apply aerodynamic wind forces to a shape.
    pub fn try_apply_shape_wind(
        &mut self,
        shape_id: ShapeId,
        wind: impl Into<Vec3>,
        drag: f32,
        lift: f32,
        max_speed: f32,
        wake: bool,
    ) -> Result<()> {
        let wind = wind.into().validate()?;
        validate_nonnegative_scalar(drag)?;
        validate_nonnegative_scalar(lift)?;
        validate_positive_scalar(max_speed)?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe {
            ffi::b3Shape_ApplyWind(
                shape_id.into_raw(),
                wind.into_raw(),
                drag,
                lift,
                max_speed,
                wake,
            )
        };
        Ok(())
    }

    /// Tries to return the sphere geometry for a sphere shape.
    pub fn try_shape_sphere(&self, shape_id: ShapeId) -> Result<Sphere> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(Sphere::from_raw(unsafe {
            ffi::b3Shape_GetSphere(shape_id.into_raw())
        }))
    }

    /// Tries to return the capsule geometry for a capsule shape.
    pub fn try_shape_capsule(&self, shape_id: ShapeId) -> Result<Capsule> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(Capsule::from_raw(unsafe {
            ffi::b3Shape_GetCapsule(shape_id.into_raw())
        }))
    }

    /// Tries to borrow hull geometry from a hull shape.
    ///
    /// The returned view is tied to `&self` and must not outlive the owning
    /// world or shape.
    pub fn try_shape_hull(&self, shape_id: ShapeId) -> Result<ShapeHull<'_>> {
        let _guard = self.lock_shape_checked(shape_id)?;
        if ShapeType::from_raw(unsafe { ffi::b3Shape_GetType(shape_id.into_raw()) })
            != Some(ShapeType::Hull)
        {
            return Err(Error::InvalidArgument);
        }
        let raw = unsafe { ffi::b3Shape_GetHull(shape_id.into_raw()) };
        unsafe { raw.as_ref() }
            .map(ShapeHull::from_raw)
            .ok_or(Error::InvalidArgument)
    }

    /// Tries to borrow mesh geometry from a mesh shape.
    ///
    /// The returned view is tied to `&self` and must not outlive the owning
    /// world or shape.
    pub fn try_shape_mesh(&self, shape_id: ShapeId) -> Result<ShapeMesh<'_>> {
        let _guard = self.lock_shape_checked(shape_id)?;
        if ShapeType::from_raw(unsafe { ffi::b3Shape_GetType(shape_id.into_raw()) })
            != Some(ShapeType::Mesh)
        {
            return Err(Error::InvalidArgument);
        }
        ShapeMesh::from_raw(unsafe { ffi::b3Shape_GetMesh(shape_id.into_raw()) })
            .ok_or(Error::InvalidArgument)
    }

    /// Tries to borrow height-field geometry from a height-field shape.
    ///
    /// The returned view is tied to `&self` and must not outlive the owning
    /// world or shape.
    pub fn try_shape_height_field(&self, shape_id: ShapeId) -> Result<ShapeHeightField<'_>> {
        let _guard = self.lock_shape_checked(shape_id)?;
        if ShapeType::from_raw(unsafe { ffi::b3Shape_GetType(shape_id.into_raw()) })
            != Some(ShapeType::HeightField)
        {
            return Err(Error::InvalidArgument);
        }
        let raw = unsafe { ffi::b3Shape_GetHeightField(shape_id.into_raw()) };
        unsafe { raw.as_ref() }
            .map(ShapeHeightField::from_raw)
            .ok_or(Error::InvalidArgument)
    }

    /// Tries to borrow the compound resource backing a compound shape.
    pub fn try_shape_compound(&self, shape_id: ShapeId) -> Result<&Compound> {
        let _guard = self.lock_shape_checked(shape_id)?;
        if ShapeType::from_raw(unsafe { ffi::b3Shape_GetType(shape_id.into_raw()) })
            != Some(ShapeType::Compound)
        {
            return Err(Error::InvalidArgument);
        }
        match self.resources.get(&shape_id) {
            Some(ShapeResource::Compound { _data }) => Ok(_data),
            _ => Err(Error::InvalidArgument),
        }
    }

    /// Tries to replace a shape's geometry with a sphere.
    pub fn try_set_shape_sphere(&mut self, shape_id: ShapeId, sphere: &Sphere) -> Result<()> {
        sphere.validate()?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetSphere(shape_id.into_raw(), sphere.raw()) };
        drop(_guard);
        self.resources.remove(&shape_id);
        Ok(())
    }

    /// Tries to replace a shape's geometry with a capsule.
    pub fn try_set_shape_capsule(&mut self, shape_id: ShapeId, capsule: &Capsule) -> Result<()> {
        capsule.validate()?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetCapsule(shape_id.into_raw(), capsule.raw()) };
        drop(_guard);
        self.resources.remove(&shape_id);
        Ok(())
    }

    /// Tries to replace a shape's geometry with an owned hull.
    pub fn try_set_shape_hull(&mut self, shape_id: ShapeId, hull: &Hull) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetHull(shape_id.into_raw(), hull.as_ptr()) };
        drop(_guard);
        self.resources.remove(&shape_id);
        Ok(())
    }

    /// Tries to replace a static shape's geometry with an owned mesh.
    pub fn try_set_shape_mesh(
        &mut self,
        shape_id: ShapeId,
        mesh: MeshData,
        scale: impl Into<Vec3>,
    ) -> Result<()> {
        let scale = validate_mesh_scale(scale.into())?;
        let mesh_ptr = mesh.as_ptr();
        let _guard = self.lock_shape_checked(shape_id)?;
        let body = unsafe { ffi::b3Shape_GetBody(shape_id.into_raw()) };
        if BodyType::from_raw(unsafe { ffi::b3Body_GetType(body) }) != Some(BodyType::Static) {
            return Err(Error::InvalidArgument);
        }
        unsafe { ffi::b3Shape_SetMesh(shape_id.into_raw(), mesh_ptr, scale.into_raw()) };
        drop(_guard);
        self.resources
            .insert(shape_id, ShapeResource::Mesh { _data: mesh });
        Ok(())
    }
}
