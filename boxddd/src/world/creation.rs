use super::*;

impl World {
    pub fn try_create_body(&mut self, def: BodyDef) -> Result<BodyId> {
        callback_state::check_not_in_callback()?;
        def.validate()?;
        let _guard = box3d_lock::lock();
        let raw = unsafe { ffi::b3CreateBody(self.raw, def.raw()) };
        if unsafe { ffi::b3Body_IsValid(raw) } {
            Ok(BodyId::from_raw(raw))
        } else {
            Err(Error::CreateBodyFailed)
        }
    }

    pub fn create_body(&mut self, def: BodyDef) -> BodyId {
        self.try_create_body(def)
            .expect("Box3D failed to create body")
    }

    pub fn try_create_sphere_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        sphere: &Sphere,
    ) -> Result<ShapeId> {
        callback_state::check_not_in_callback()?;
        def.validate()?;
        sphere.validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        let raw = unsafe { ffi::b3CreateSphereShape(body_id.into_raw(), def.raw(), sphere.raw()) };
        if unsafe { ffi::b3Shape_IsValid(raw) } {
            Ok(ShapeId::from_raw(raw))
        } else {
            Err(Error::CreateShapeFailed)
        }
    }

    pub fn create_sphere_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        sphere: &Sphere,
    ) -> ShapeId {
        self.try_create_sphere_shape(body_id, def, sphere)
            .expect("Box3D failed to create sphere shape")
    }

    pub fn try_create_hull_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        hull: &BoxHull,
    ) -> Result<ShapeId> {
        callback_state::check_not_in_callback()?;
        def.validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        let raw =
            unsafe { ffi::b3CreateHullShape(body_id.into_raw(), def.raw(), hull.hull_data()) };
        if unsafe { ffi::b3Shape_IsValid(raw) } {
            Ok(ShapeId::from_raw(raw))
        } else {
            Err(Error::CreateShapeFailed)
        }
    }

    pub fn create_hull_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        hull: &BoxHull,
    ) -> ShapeId {
        self.try_create_hull_shape(body_id, def, hull)
            .expect("Box3D failed to create hull shape")
    }

    pub fn try_create_capsule_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        capsule: &Capsule,
    ) -> Result<ShapeId> {
        callback_state::check_not_in_callback()?;
        def.validate()?;
        capsule.validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        let raw =
            unsafe { ffi::b3CreateCapsuleShape(body_id.into_raw(), def.raw(), capsule.raw()) };
        shape_id_from_raw(raw)
    }

    pub fn create_capsule_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        capsule: &Capsule,
    ) -> ShapeId {
        self.try_create_capsule_shape(body_id, def, capsule)
            .expect("Box3D failed to create capsule shape")
    }

    pub fn try_create_created_hull_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        hull: &Hull,
    ) -> Result<ShapeId> {
        callback_state::check_not_in_callback()?;
        def.validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        let raw = unsafe { ffi::b3CreateHullShape(body_id.into_raw(), def.raw(), hull.as_ptr()) };
        shape_id_from_raw(raw)
    }

    pub fn try_create_transformed_hull_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        hull: &Hull,
        transform: impl Into<crate::types::Transform>,
        scale: impl Into<Vec3>,
    ) -> Result<ShapeId> {
        callback_state::check_not_in_callback()?;
        def.validate()?;
        let transform = transform.into();
        transform.validate()?;
        let scale = scale.into().validate()?;
        let _guard = self.lock_body_checked(body_id)?;
        let raw = unsafe {
            ffi::b3CreateTransformedHullShape(
                body_id.into_raw(),
                def.raw(),
                hull.as_ptr(),
                transform.into_raw(),
                scale.into_raw(),
            )
        };
        shape_id_from_raw(raw)
    }

    pub fn try_create_mesh_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        mesh: MeshData,
        scale: impl Into<Vec3>,
    ) -> Result<ShapeId> {
        if self.try_body_type(body_id)? != BodyType::Static {
            return Err(Error::InvalidArgument);
        }
        callback_state::check_not_in_callback()?;
        def.validate()?;
        let scale = scale.into().validate()?;
        let mesh_ptr = mesh.as_ptr();
        let _guard = self.lock_body_checked(body_id)?;
        let raw = unsafe {
            ffi::b3CreateMeshShape(body_id.into_raw(), def.raw(), mesh_ptr, scale.into_raw())
        };
        let shape_id = shape_id_from_raw(raw)?;
        drop(_guard);
        self.resources
            .insert(shape_id, ShapeResource::Mesh { _data: mesh });
        Ok(shape_id)
    }

    pub fn try_create_height_field_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        height_field: HeightField,
    ) -> Result<ShapeId> {
        if self.try_body_type(body_id)? != BodyType::Static {
            return Err(Error::InvalidArgument);
        }
        callback_state::check_not_in_callback()?;
        def.validate()?;
        let height_field_ptr = height_field.as_ptr();
        let _guard = self.lock_body_checked(body_id)?;
        let raw = unsafe {
            ffi::b3CreateHeightFieldShape(body_id.into_raw(), def.raw(), height_field_ptr)
        };
        let shape_id = shape_id_from_raw(raw)?;
        drop(_guard);
        self.resources.insert(
            shape_id,
            ShapeResource::HeightField {
                _data: height_field,
            },
        );
        Ok(shape_id)
    }

    pub fn try_create_compound_shape(
        &mut self,
        body_id: BodyId,
        def: &ShapeDef,
        compound: Compound,
    ) -> Result<ShapeId> {
        if self.try_body_type(body_id)? != BodyType::Static {
            return Err(Error::InvalidArgument);
        }
        callback_state::check_not_in_callback()?;
        def.validate()?;
        let compound_ptr = compound.as_ptr();
        let mut raw_def = *def.raw();
        let _guard = self.lock_body_checked(body_id)?;
        let raw =
            unsafe { ffi::b3CreateCompoundShape(body_id.into_raw(), &mut raw_def, compound_ptr) };
        let shape_id = shape_id_from_raw(raw)?;
        drop(_guard);
        self.resources
            .insert(shape_id, ShapeResource::Compound { _data: compound });
        Ok(shape_id)
    }
}
