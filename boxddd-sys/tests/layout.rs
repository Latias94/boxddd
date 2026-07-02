use std::mem::{align_of, size_of};

use boxddd_sys::ffi;

#[test]
fn representative_public_api_symbols_are_bound() {
    let _world_step: unsafe extern "C" fn(ffi::b3WorldId, f32, i32) = ffi::b3World_Step;
    let _body_transform: unsafe extern "C" fn(ffi::b3BodyId) -> ffi::b3WorldTransform =
        ffi::b3Body_GetTransform;
    let _sphere_shape: unsafe extern "C" fn(
        ffi::b3BodyId,
        *const ffi::b3ShapeDef,
        *const ffi::b3Sphere,
    ) -> ffi::b3ShapeId = ffi::b3CreateSphereShape;
    let _distance_joint: unsafe extern "C" fn() -> ffi::b3DistanceJointDef =
        ffi::b3DefaultDistanceJointDef;
    let _query_filter: unsafe extern "C" fn() -> ffi::b3QueryFilter = ffi::b3DefaultQueryFilter;
    let _recording: unsafe extern "C" fn(ffi::b3WorldId, *mut ffi::b3Recording) =
        ffi::b3World_StartRecording;
    let _mover: unsafe extern "C" fn(
        ffi::b3WorldId,
        ffi::b3Pos,
        *const ffi::b3Capsule,
        ffi::b3QueryFilter,
        ffi::b3PlaneResultFcn,
        *mut std::ffi::c_void,
    ) = ffi::b3World_CollideMover;
    let _collision: unsafe extern "C" fn(
        *mut ffi::b3LocalManifold,
        i32,
        *const ffi::b3Sphere,
        *const ffi::b3Sphere,
        ffi::b3Transform,
    ) = ffi::b3CollideSpheres;
}

#[test]
fn selected_abi_mode_matches_runtime_library() {
    let is_double = unsafe { ffi::b3IsDoublePrecision() };

    #[cfg(feature = "double-precision")]
    {
        assert!(is_double);
        assert_eq!(size_of::<ffi::b3Pos>(), 24);
        assert_eq!(align_of::<ffi::b3Pos>(), 8);
        assert_eq!(size_of::<ffi::b3WorldTransform>(), 40);
    }

    #[cfg(not(feature = "double-precision"))]
    {
        assert!(!is_double);
        assert_eq!(size_of::<ffi::b3Pos>(), size_of::<ffi::b3Vec3>());
        assert_eq!(align_of::<ffi::b3Pos>(), align_of::<ffi::b3Vec3>());
        assert_eq!(
            size_of::<ffi::b3WorldTransform>(),
            size_of::<ffi::b3Transform>()
        );
    }
}

#[test]
fn representative_layouts_are_stable_for_default_bindings() {
    assert_eq!(size_of::<ffi::b3WorldId>(), 4);
    assert_eq!(size_of::<ffi::b3BodyId>(), 8);
    assert_eq!(size_of::<ffi::b3ShapeId>(), 8);
    assert_eq!(size_of::<ffi::b3JointId>(), 8);
    assert_eq!(size_of::<ffi::b3Vec3>(), 12);
    assert_eq!(size_of::<ffi::b3Quat>(), 16);
    assert!(size_of::<ffi::b3WorldDef>() >= size_of::<ffi::b3Vec3>());
    assert!(size_of::<ffi::b3ShapeDef>() >= size_of::<ffi::b3SurfaceMaterial>());
}
