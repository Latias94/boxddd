use super::*;

impl World {
    pub fn try_destroy_shape(&mut self, shape_id: ShapeId, update_body_mass: bool) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3DestroyShape(shape_id.into_raw(), update_body_mass) };
        drop(_guard);
        self.resources.remove(&shape_id);
        Ok(())
    }

    pub fn destroy_shape(&mut self, shape_id: ShapeId, update_body_mass: bool) {
        self.try_destroy_shape(shape_id, update_body_mass)
            .expect("invalid ShapeId");
    }

    pub fn try_shape_type(&self, shape_id: ShapeId) -> Result<ShapeType> {
        let _guard = self.lock_shape_checked(shape_id)?;
        ShapeType::from_raw(unsafe { ffi::b3Shape_GetType(shape_id.into_raw()) })
            .ok_or(Error::InvalidArgument)
    }

    pub fn try_shape_body(&self, shape_id: ShapeId) -> Result<BodyId> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(BodyId::from_raw(unsafe {
            ffi::b3Shape_GetBody(shape_id.into_raw())
        }))
    }

    pub fn try_shape_sensor(&self, shape_id: ShapeId) -> Result<bool> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_IsSensor(shape_id.into_raw()) })
    }

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

    pub fn try_shape_density(&self, shape_id: ShapeId) -> Result<f32> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_GetDensity(shape_id.into_raw()) })
    }

    pub fn try_set_shape_friction(&mut self, shape_id: ShapeId, friction: f32) -> Result<()> {
        validate_nonnegative_scalar(friction)?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetFriction(shape_id.into_raw(), friction) };
        Ok(())
    }

    pub fn try_shape_friction(&self, shape_id: ShapeId) -> Result<f32> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_GetFriction(shape_id.into_raw()) })
    }

    pub fn try_set_shape_restitution(&mut self, shape_id: ShapeId, restitution: f32) -> Result<()> {
        validate_nonnegative_scalar(restitution)?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetRestitution(shape_id.into_raw(), restitution) };
        Ok(())
    }

    pub fn try_shape_restitution(&self, shape_id: ShapeId) -> Result<f32> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_GetRestitution(shape_id.into_raw()) })
    }

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

    pub fn try_shape_surface_material(&self, shape_id: ShapeId) -> Result<SurfaceMaterial> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(SurfaceMaterial::from_raw(unsafe {
            ffi::b3Shape_GetSurfaceMaterial(shape_id.into_raw())
        }))
    }

    pub fn try_shape_mesh_material_count(&self, shape_id: ShapeId) -> Result<i32> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(unsafe { ffi::b3Shape_GetMeshMaterialCount(shape_id.into_raw()) })
    }

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

    pub fn try_shape_filter(&self, shape_id: ShapeId) -> Result<Filter> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(Filter::from_raw(unsafe {
            ffi::b3Shape_GetFilter(shape_id.into_raw())
        }))
    }

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

    pub fn try_enable_shape_sensor_events(
        &mut self,
        shape_id: ShapeId,
        enabled: bool,
    ) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_EnableSensorEvents(shape_id.into_raw(), enabled) };
        Ok(())
    }

    pub fn try_enable_shape_contact_events(
        &mut self,
        shape_id: ShapeId,
        enabled: bool,
    ) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_EnableContactEvents(shape_id.into_raw(), enabled) };
        Ok(())
    }

    pub fn try_enable_shape_pre_solve_events(
        &mut self,
        shape_id: ShapeId,
        enabled: bool,
    ) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_EnablePreSolveEvents(shape_id.into_raw(), enabled) };
        Ok(())
    }

    pub fn try_enable_shape_hit_events(&mut self, shape_id: ShapeId, enabled: bool) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_EnableHitEvents(shape_id.into_raw(), enabled) };
        Ok(())
    }

    pub fn try_shape_aabb(&self, shape_id: ShapeId) -> Result<Aabb> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(Aabb::from_raw(unsafe {
            ffi::b3Shape_GetAABB(shape_id.into_raw())
        }))
    }

    pub fn try_shape_sphere(&self, shape_id: ShapeId) -> Result<Sphere> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(Sphere::from_raw(unsafe {
            ffi::b3Shape_GetSphere(shape_id.into_raw())
        }))
    }

    pub fn try_shape_capsule(&self, shape_id: ShapeId) -> Result<Capsule> {
        let _guard = self.lock_shape_checked(shape_id)?;
        Ok(Capsule::from_raw(unsafe {
            ffi::b3Shape_GetCapsule(shape_id.into_raw())
        }))
    }

    pub fn try_set_shape_sphere(&mut self, shape_id: ShapeId, sphere: &Sphere) -> Result<()> {
        sphere.validate()?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetSphere(shape_id.into_raw(), sphere.raw()) };
        Ok(())
    }

    pub fn try_set_shape_capsule(&mut self, shape_id: ShapeId, capsule: &Capsule) -> Result<()> {
        capsule.validate()?;
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetCapsule(shape_id.into_raw(), capsule.raw()) };
        Ok(())
    }

    pub fn try_set_shape_hull(&mut self, shape_id: ShapeId, hull: &Hull) -> Result<()> {
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetHull(shape_id.into_raw(), hull.as_ptr()) };
        Ok(())
    }

    pub fn try_set_shape_mesh(
        &mut self,
        shape_id: ShapeId,
        mesh: MeshData,
        scale: impl Into<Vec3>,
    ) -> Result<()> {
        let scale = scale.into().validate()?;
        let mesh_ptr = mesh.as_ptr();
        let _guard = self.lock_shape_checked(shape_id)?;
        unsafe { ffi::b3Shape_SetMesh(shape_id.into_raw(), mesh_ptr, scale.into_raw()) };
        drop(_guard);
        self.resources
            .insert(shape_id, ShapeResource::Mesh { _data: mesh });
        Ok(())
    }
}
