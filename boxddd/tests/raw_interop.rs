use boxddd::{
    BodyDef, BodyId, BodyType, DistanceJointDef, Error, JointId, ShapeDef, ShapeId, Sphere, Vec3,
    World, WorldDef, raw,
};
use std::ffi::c_void;

fn scene() -> (World, BodyId, BodyId, ShapeId, JointId) {
    let mut world = World::new(WorldDef::default()).unwrap();
    let a = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([-0.5, 0.0, 0.0])
            .build(),
    );
    let b = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([0.5, 0.0, 0.0])
            .build(),
    );
    let shape = world.create_sphere_shape(a, &ShapeDef::default(), &Sphere::new(Vec3::ZERO, 0.25));
    let joint = world.create_distance_joint(DistanceJointDef::new(a, b).length(1.0));
    (world, a, b, shape, joint)
}

#[test]
fn raw_user_data_round_trips_for_world_body_shape_and_joint() {
    let (mut world, body, _, shape, joint) = scene();

    let mut world_marker = 1_i32;
    let mut body_marker = 2_i32;
    let mut shape_marker = 3_i32;
    let mut joint_marker = 4_i32;

    let world_ptr = (&mut world_marker as *mut i32).cast::<c_void>();
    let body_ptr = (&mut body_marker as *mut i32).cast::<c_void>();
    let shape_ptr = (&mut shape_marker as *mut i32).cast::<c_void>();
    let joint_ptr = (&mut joint_marker as *mut i32).cast::<c_void>();

    unsafe {
        raw::try_set_world_raw_user_data(&mut world, world_ptr).unwrap();
        raw::try_set_body_raw_user_data(&mut world, body, body_ptr).unwrap();
        raw::try_set_shape_raw_user_data(&mut world, shape, shape_ptr).unwrap();
        raw::try_set_joint_raw_user_data(&mut world, joint, joint_ptr).unwrap();

        assert_eq!(raw::try_world_raw_user_data(&world).unwrap(), world_ptr);
        assert_eq!(raw::try_body_raw_user_data(&world, body).unwrap(), body_ptr);
        assert_eq!(
            raw::try_shape_raw_user_data(&world, shape).unwrap(),
            shape_ptr
        );
        assert_eq!(
            raw::try_joint_raw_user_data(&world, joint).unwrap(),
            joint_ptr
        );
    }
}

#[test]
fn raw_user_data_rejects_foreign_handles() {
    let (mut world, _, _, _, _) = scene();
    let (mut other_world, other_body, _, other_shape, other_joint) = scene();
    let mut marker = 7_i32;
    let ptr = (&mut marker as *mut i32).cast::<c_void>();

    assert_eq!(
        unsafe { raw::try_set_body_raw_user_data(&mut world, other_body, ptr).unwrap_err() },
        Error::InvalidBodyId
    );
    assert_eq!(
        unsafe { raw::try_shape_raw_user_data(&world, other_shape).unwrap_err() },
        Error::InvalidShapeId
    );
    assert_eq!(
        unsafe { raw::try_set_joint_raw_user_data(&mut world, other_joint, ptr).unwrap_err() },
        Error::InvalidJointId
    );

    unsafe { raw::try_set_world_raw_user_data(&mut other_world, ptr).unwrap() };
    assert_eq!(
        unsafe { raw::try_world_raw_user_data(&other_world).unwrap() },
        ptr
    );
}

#[test]
fn process_global_scalar_utilities_validate_inputs() {
    struct Restore {
        length_units: f32,
        stall_threshold: f32,
    }

    impl Drop for Restore {
        fn drop(&mut self) {
            let _ = raw::try_set_length_units_per_meter(self.length_units);
            let _ = raw::try_set_stall_threshold(self.stall_threshold);
        }
    }

    let _restore = Restore {
        length_units: raw::length_units_per_meter(),
        stall_threshold: raw::stall_threshold(),
    };

    assert_eq!(
        raw::try_set_length_units_per_meter(0.0).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        raw::try_set_length_units_per_meter(f32::NAN).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        raw::try_set_stall_threshold(-1.0).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        raw::try_set_stall_threshold(f32::INFINITY).unwrap_err(),
        Error::InvalidArgument
    );

    raw::try_set_length_units_per_meter(2.0).unwrap();
    raw::try_set_stall_threshold(0.25).unwrap();

    assert_eq!(raw::length_units_per_meter(), 2.0);
    assert_eq!(raw::stall_threshold(), 0.25);
}

#[test]
fn raw_interop_obeys_callback_guard() {
    let (mut world, body, _, _, _) = scene();
    let mut marker = 9_i32;
    let ptr = (&mut marker as *mut i32).cast::<c_void>();
    let _guard = boxddd::__private::enter_callback_guard_for_test();

    assert_eq!(
        unsafe { raw::try_set_world_raw_user_data(&mut world, ptr).unwrap_err() },
        Error::InCallback
    );
    assert_eq!(
        unsafe { raw::try_set_body_raw_user_data(&mut world, body, ptr).unwrap_err() },
        Error::InCallback
    );
    assert_eq!(
        raw::try_length_units_per_meter().unwrap_err(),
        Error::InCallback
    );
    assert_eq!(raw::try_stall_threshold().unwrap_err(), Error::InCallback);
}
