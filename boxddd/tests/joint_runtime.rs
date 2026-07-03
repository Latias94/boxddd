use boxddd::{
    BodyDef, BodyId, BodyType, DistanceJointDef, Error, JointTuning, PrismaticJointDef, Quat,
    Transform, Vec3, World, WorldDef, raw,
};
use std::ffi::c_void;

fn body_pair() -> (World, BodyId, BodyId) {
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
    (world, a, b)
}

fn assert_close(a: f32, b: f32) {
    assert!((a - b).abs() < 1.0e-5, "{a} != {b}");
}

#[test]
fn common_joint_runtime_setters_getters_and_raw_user_data_work() {
    let (mut world, a, b) = body_pair();
    let joint = world.create_distance_joint(
        DistanceJointDef::new(a, b)
            .length(1.0)
            .collide_connected(true)
            .force_threshold(5.0)
            .torque_threshold(6.0)
            .constraint_tuning(JointTuning::new(30.0, 1.25)),
    );

    assert_eq!(world.try_joint_body_a(joint).unwrap(), a);
    assert_eq!(world.try_joint_body_b(joint).unwrap(), b);
    assert!(world.try_joint_collide_connected(joint).unwrap());
    world.try_set_joint_collide_connected(joint, false).unwrap();
    assert!(!world.try_joint_collide_connected(joint).unwrap());

    let frame_a = Transform::new(Vec3::new(0.1, 0.2, 0.3), Quat::IDENTITY);
    let frame_b = Transform::new(Vec3::new(-0.1, 0.0, 0.25), Quat::IDENTITY);
    world.try_set_joint_local_frame_a(joint, frame_a).unwrap();
    world.try_set_joint_local_frame_b(joint, frame_b).unwrap();
    assert_eq!(world.try_joint_local_frame_a(joint).unwrap(), frame_a);
    assert_eq!(world.try_joint_local_frame_b(joint).unwrap(), frame_b);

    world
        .try_set_joint_constraint_tuning(joint, JointTuning::new(12.0, 0.75))
        .unwrap();
    let tuning = world.try_joint_constraint_tuning(joint).unwrap();
    assert_close(tuning.hertz, 12.0);
    assert_close(tuning.damping_ratio, 0.75);
    world.try_set_joint_force_threshold(joint, 9.0).unwrap();
    world.try_set_joint_torque_threshold(joint, 10.0).unwrap();
    assert_close(world.try_joint_force_threshold(joint).unwrap(), 9.0);
    assert_close(world.try_joint_torque_threshold(joint).unwrap(), 10.0);

    let mut marker = 7_i32;
    let ptr = (&mut marker as *mut i32).cast::<c_void>();
    unsafe { raw::try_set_joint_raw_user_data(&mut world, joint, ptr).unwrap() };
    assert_eq!(
        unsafe { raw::try_joint_raw_user_data(&world, joint).unwrap() },
        ptr
    );

    world.try_wake_joint_bodies(joint).unwrap();
    assert!(world.try_joint_constraint_force(joint).unwrap().is_valid());
    assert!(world.try_joint_constraint_torque(joint).unwrap().is_valid());
    assert!(
        world
            .try_joint_linear_separation(joint)
            .unwrap()
            .is_finite()
    );
    assert!(
        world
            .try_joint_angular_separation(joint)
            .unwrap()
            .is_finite()
    );
}

#[test]
fn typed_joint_runtime_guards_wrong_family_and_destroyed_ids() {
    let (mut world, a, b) = body_pair();
    let distance = world.create_distance_joint(DistanceJointDef::new(a, b).length(1.0));
    assert_eq!(
        world.try_prismatic_joint_translation(distance).unwrap_err(),
        Error::WrongJointType
    );

    let prismatic = world.create_prismatic_joint(PrismaticJointDef::new(a, b));
    world.try_destroy_joint(prismatic, true).unwrap();
    assert_eq!(
        world
            .try_prismatic_joint_translation(prismatic)
            .unwrap_err(),
        Error::InvalidJointId
    );
}
