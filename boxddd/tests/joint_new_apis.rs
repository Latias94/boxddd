use boxddd::{
    BodyDef, BodyId, BodyType, DistanceJointDef, MotorJointDef, ParallelJointDef,
    PrismaticJointDef, Quat, RevoluteJointDef, SphericalJointDef, Vec3, WeldJointDef,
    WheelJointDef, World, WorldDef,
};

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

fn assert_vec3_close(actual: Vec3, expected: Vec3) {
    assert_close(actual.x, expected.x);
    assert_close(actual.y, expected.y);
    assert_close(actual.z, expected.z);
}

fn assert_quat_close(actual: Quat, expected: Quat) {
    assert_vec3_close(actual.v, expected.v);
    assert_close(actual.s, expected.s);
}

#[test]
fn parallel_and_distance_runtime_apis_round_trip() {
    let (mut world, a, b) = body_pair();
    let parallel = world.create_parallel_joint(ParallelJointDef::new(a, b));
    world
        .try_set_parallel_joint_spring_hertz(parallel, 6.0)
        .unwrap();
    world
        .try_set_parallel_joint_spring_damping_ratio(parallel, 0.7)
        .unwrap();
    world
        .try_set_parallel_joint_max_torque(parallel, 20.0)
        .unwrap();
    assert_close(
        world.try_parallel_joint_spring_hertz(parallel).unwrap(),
        6.0,
    );
    assert_close(
        world
            .try_parallel_joint_spring_damping_ratio(parallel)
            .unwrap(),
        0.7,
    );
    assert_close(world.try_parallel_joint_max_torque(parallel).unwrap(), 20.0);

    let distance = world.create_distance_joint(DistanceJointDef::new(a, b).length(1.0));
    world.try_set_distance_joint_length(distance, 1.25).unwrap();
    world
        .try_enable_distance_joint_spring(distance, true)
        .unwrap();
    world
        .try_set_distance_joint_spring_force_range(distance, -2.0, 3.0)
        .unwrap();
    world
        .try_set_distance_joint_spring_hertz(distance, 5.0)
        .unwrap();
    world
        .try_set_distance_joint_spring_damping_ratio(distance, 0.6)
        .unwrap();
    world
        .try_enable_distance_joint_limit(distance, true)
        .unwrap();
    world
        .try_set_distance_joint_length_range(distance, 0.75, 1.5)
        .unwrap();
    world
        .try_enable_distance_joint_motor(distance, true)
        .unwrap();
    world
        .try_set_distance_joint_motor_speed(distance, 0.4)
        .unwrap();
    world
        .try_set_distance_joint_max_motor_force(distance, 8.0)
        .unwrap();

    assert_close(world.try_distance_joint_length(distance).unwrap(), 1.25);
    assert!(world.try_distance_joint_spring_enabled(distance).unwrap());
    assert_eq!(
        world
            .try_distance_joint_spring_force_range(distance)
            .unwrap(),
        (-2.0, 3.0)
    );
    assert_close(
        world.try_distance_joint_spring_hertz(distance).unwrap(),
        5.0,
    );
    assert_close(
        world
            .try_distance_joint_spring_damping_ratio(distance)
            .unwrap(),
        0.6,
    );
    assert!(world.try_distance_joint_limit_enabled(distance).unwrap());
    assert_close(world.try_distance_joint_min_length(distance).unwrap(), 0.75);
    assert_close(world.try_distance_joint_max_length(distance).unwrap(), 1.5);
    assert!(world.try_distance_joint_current_length(distance).unwrap() >= 0.0);
    assert!(world.try_distance_joint_motor_enabled(distance).unwrap());
    assert_close(world.try_distance_joint_motor_speed(distance).unwrap(), 0.4);
    assert_close(
        world.try_distance_joint_max_motor_force(distance).unwrap(),
        8.0,
    );
    assert!(
        world
            .try_distance_joint_motor_force(distance)
            .unwrap()
            .is_finite()
    );
}

#[test]
fn motor_prismatic_and_revolute_runtime_apis_round_trip() {
    let (mut world, a, b) = body_pair();
    let motor = world.create_motor_joint(MotorJointDef::new(a, b));
    world
        .try_set_motor_joint_linear_velocity(motor, [1.0, 2.0, 3.0])
        .unwrap();
    world
        .try_set_motor_joint_angular_velocity(motor, [0.1, 0.2, 0.3])
        .unwrap();
    world
        .try_set_motor_joint_max_velocity_force(motor, 11.0)
        .unwrap();
    world
        .try_set_motor_joint_max_velocity_torque(motor, 12.0)
        .unwrap();
    world.try_set_motor_joint_linear_hertz(motor, 13.0).unwrap();
    world
        .try_set_motor_joint_linear_damping_ratio(motor, 0.8)
        .unwrap();
    world
        .try_set_motor_joint_angular_hertz(motor, 14.0)
        .unwrap();
    world
        .try_set_motor_joint_angular_damping_ratio(motor, 0.9)
        .unwrap();
    world
        .try_set_motor_joint_max_spring_force(motor, 15.0)
        .unwrap();
    world
        .try_set_motor_joint_max_spring_torque(motor, 16.0)
        .unwrap();
    assert_vec3_close(
        world.try_motor_joint_linear_velocity(motor).unwrap(),
        Vec3::new(1.0, 2.0, 3.0),
    );
    assert_vec3_close(
        world.try_motor_joint_angular_velocity(motor).unwrap(),
        Vec3::new(0.1, 0.2, 0.3),
    );
    assert_close(
        world.try_motor_joint_max_velocity_force(motor).unwrap(),
        11.0,
    );
    assert_close(
        world.try_motor_joint_max_velocity_torque(motor).unwrap(),
        12.0,
    );
    assert_close(world.try_motor_joint_linear_hertz(motor).unwrap(), 13.0);
    assert_close(
        world.try_motor_joint_linear_damping_ratio(motor).unwrap(),
        0.8,
    );
    assert_close(world.try_motor_joint_angular_hertz(motor).unwrap(), 14.0);
    assert_close(
        world.try_motor_joint_angular_damping_ratio(motor).unwrap(),
        0.9,
    );
    assert_close(world.try_motor_joint_max_spring_force(motor).unwrap(), 15.0);
    assert_close(
        world.try_motor_joint_max_spring_torque(motor).unwrap(),
        16.0,
    );

    let prismatic = world.create_prismatic_joint(PrismaticJointDef::new(a, b));
    world
        .try_enable_prismatic_joint_spring(prismatic, true)
        .unwrap();
    world
        .try_set_prismatic_joint_spring_hertz(prismatic, 4.0)
        .unwrap();
    world
        .try_set_prismatic_joint_spring_damping_ratio(prismatic, 0.5)
        .unwrap();
    world
        .try_set_prismatic_joint_target_translation(prismatic, 0.25)
        .unwrap();
    world
        .try_enable_prismatic_joint_limit(prismatic, true)
        .unwrap();
    world
        .try_set_prismatic_joint_limits(prismatic, -0.25, 0.75)
        .unwrap();
    world
        .try_enable_prismatic_joint_motor(prismatic, true)
        .unwrap();
    world
        .try_set_prismatic_joint_motor_speed(prismatic, 0.2)
        .unwrap();
    world
        .try_set_prismatic_joint_max_motor_force(prismatic, 6.0)
        .unwrap();
    assert!(world.try_prismatic_joint_spring_enabled(prismatic).unwrap());
    assert_close(
        world.try_prismatic_joint_spring_hertz(prismatic).unwrap(),
        4.0,
    );
    assert_close(
        world
            .try_prismatic_joint_spring_damping_ratio(prismatic)
            .unwrap(),
        0.5,
    );
    assert_close(
        world
            .try_prismatic_joint_target_translation(prismatic)
            .unwrap(),
        0.25,
    );
    assert!(world.try_prismatic_joint_limit_enabled(prismatic).unwrap());
    assert_close(
        world.try_prismatic_joint_lower_limit(prismatic).unwrap(),
        -0.25,
    );
    assert_close(
        world.try_prismatic_joint_upper_limit(prismatic).unwrap(),
        0.75,
    );
    assert!(world.try_prismatic_joint_motor_enabled(prismatic).unwrap());
    assert_close(
        world.try_prismatic_joint_motor_speed(prismatic).unwrap(),
        0.2,
    );
    assert_close(
        world
            .try_prismatic_joint_max_motor_force(prismatic)
            .unwrap(),
        6.0,
    );
    assert!(
        world
            .try_prismatic_joint_motor_force(prismatic)
            .unwrap()
            .is_finite()
    );
    assert!(
        world
            .try_prismatic_joint_translation(prismatic)
            .unwrap()
            .is_finite()
    );
    assert!(
        world
            .try_prismatic_joint_speed(prismatic)
            .unwrap()
            .is_finite()
    );

    let revolute = world.create_revolute_joint(RevoluteJointDef::new(a, b));
    world
        .try_enable_revolute_joint_spring(revolute, true)
        .unwrap();
    world
        .try_set_revolute_joint_spring_hertz(revolute, 7.0)
        .unwrap();
    world
        .try_set_revolute_joint_spring_damping_ratio(revolute, 0.45)
        .unwrap();
    world
        .try_set_revolute_joint_target_angle(revolute, 0.1)
        .unwrap();
    world
        .try_enable_revolute_joint_limit(revolute, true)
        .unwrap();
    world
        .try_set_revolute_joint_limits(revolute, -0.5, 0.5)
        .unwrap();
    world
        .try_enable_revolute_joint_motor(revolute, true)
        .unwrap();
    world
        .try_set_revolute_joint_motor_speed(revolute, 0.3)
        .unwrap();
    world
        .try_set_revolute_joint_max_motor_torque(revolute, 9.0)
        .unwrap();
    assert!(world.try_revolute_joint_spring_enabled(revolute).unwrap());
    assert_close(
        world.try_revolute_joint_spring_hertz(revolute).unwrap(),
        7.0,
    );
    assert_close(
        world
            .try_revolute_joint_spring_damping_ratio(revolute)
            .unwrap(),
        0.45,
    );
    assert_close(
        world.try_revolute_joint_target_angle(revolute).unwrap(),
        0.1,
    );
    assert!(
        world
            .try_revolute_joint_angle(revolute)
            .unwrap()
            .is_finite()
    );
    assert!(world.try_revolute_joint_limit_enabled(revolute).unwrap());
    assert_close(
        world.try_revolute_joint_lower_limit(revolute).unwrap(),
        -0.5,
    );
    assert_close(world.try_revolute_joint_upper_limit(revolute).unwrap(), 0.5);
    assert!(world.try_revolute_joint_motor_enabled(revolute).unwrap());
    assert_close(world.try_revolute_joint_motor_speed(revolute).unwrap(), 0.3);
    assert!(
        world
            .try_revolute_joint_motor_torque(revolute)
            .unwrap()
            .is_finite()
    );
    assert_close(
        world.try_revolute_joint_max_motor_torque(revolute).unwrap(),
        9.0,
    );
}

#[test]
fn spherical_weld_and_wheel_runtime_apis_round_trip() {
    let (mut world, a, b) = body_pair();
    let spherical = world.create_spherical_joint(SphericalJointDef::new(a, b));
    let target_rotation = Quat::new(Vec3::new(0.0, 0.38268343, 0.0), 0.9238795);
    world
        .try_enable_spherical_joint_cone_limit(spherical, true)
        .unwrap();
    world
        .try_set_spherical_joint_cone_limit(spherical, 0.6)
        .unwrap();
    world
        .try_enable_spherical_joint_twist_limit(spherical, true)
        .unwrap();
    world
        .try_set_spherical_joint_twist_limits(spherical, -0.4, 0.4)
        .unwrap();
    world
        .try_enable_spherical_joint_spring(spherical, true)
        .unwrap();
    world
        .try_set_spherical_joint_spring_hertz(spherical, 3.0)
        .unwrap();
    world
        .try_set_spherical_joint_spring_damping_ratio(spherical, 0.65)
        .unwrap();
    world
        .try_set_spherical_joint_target_rotation(spherical, target_rotation)
        .unwrap();
    world
        .try_enable_spherical_joint_motor(spherical, true)
        .unwrap();
    world
        .try_set_spherical_joint_motor_velocity(spherical, [0.1, 0.2, 0.3])
        .unwrap();
    world
        .try_set_spherical_joint_max_motor_torque(spherical, 17.0)
        .unwrap();
    assert!(
        world
            .try_spherical_joint_cone_limit_enabled(spherical)
            .unwrap()
    );
    assert_close(
        world.try_spherical_joint_cone_limit(spherical).unwrap(),
        0.6,
    );
    assert!(
        world
            .try_spherical_joint_cone_angle(spherical)
            .unwrap()
            .is_finite()
    );
    assert!(
        world
            .try_spherical_joint_twist_limit_enabled(spherical)
            .unwrap()
    );
    assert_close(
        world
            .try_spherical_joint_lower_twist_limit(spherical)
            .unwrap(),
        -0.4,
    );
    assert_close(
        world
            .try_spherical_joint_upper_twist_limit(spherical)
            .unwrap(),
        0.4,
    );
    assert!(
        world
            .try_spherical_joint_twist_angle(spherical)
            .unwrap()
            .is_finite()
    );
    assert!(world.try_spherical_joint_spring_enabled(spherical).unwrap());
    assert_close(
        world.try_spherical_joint_spring_hertz(spherical).unwrap(),
        3.0,
    );
    assert_close(
        world
            .try_spherical_joint_spring_damping_ratio(spherical)
            .unwrap(),
        0.65,
    );
    assert_quat_close(
        world
            .try_spherical_joint_target_rotation(spherical)
            .unwrap(),
        target_rotation,
    );
    assert!(world.try_spherical_joint_motor_enabled(spherical).unwrap());
    assert_vec3_close(
        world.try_spherical_joint_motor_velocity(spherical).unwrap(),
        Vec3::new(0.1, 0.2, 0.3),
    );
    assert!(
        world
            .try_spherical_joint_motor_torque(spherical)
            .unwrap()
            .is_valid()
    );
    assert_close(
        world
            .try_spherical_joint_max_motor_torque(spherical)
            .unwrap(),
        17.0,
    );

    let weld = world.create_weld_joint(WeldJointDef::new(a, b));
    world.try_set_weld_joint_linear_hertz(weld, 2.0).unwrap();
    world
        .try_set_weld_joint_linear_damping_ratio(weld, 0.4)
        .unwrap();
    world.try_set_weld_joint_angular_hertz(weld, 2.5).unwrap();
    world
        .try_set_weld_joint_angular_damping_ratio(weld, 0.55)
        .unwrap();
    assert_close(world.try_weld_joint_linear_hertz(weld).unwrap(), 2.0);
    assert_close(
        world.try_weld_joint_linear_damping_ratio(weld).unwrap(),
        0.4,
    );
    assert_close(world.try_weld_joint_angular_hertz(weld).unwrap(), 2.5);
    assert_close(
        world.try_weld_joint_angular_damping_ratio(weld).unwrap(),
        0.55,
    );

    let wheel = world.create_wheel_joint(WheelJointDef::new(a, b));
    world
        .try_enable_wheel_joint_suspension(wheel, true)
        .unwrap();
    world
        .try_set_wheel_joint_suspension_hertz(wheel, 8.0)
        .unwrap();
    world
        .try_set_wheel_joint_suspension_damping_ratio(wheel, 0.6)
        .unwrap();
    world
        .try_enable_wheel_joint_suspension_limit(wheel, true)
        .unwrap();
    world
        .try_set_wheel_joint_suspension_limits(wheel, -0.2, 0.4)
        .unwrap();
    world
        .try_enable_wheel_joint_spin_motor(wheel, true)
        .unwrap();
    world
        .try_set_wheel_joint_spin_motor_speed(wheel, 5.0)
        .unwrap();
    world
        .try_set_wheel_joint_max_spin_torque(wheel, 18.0)
        .unwrap();
    world.try_enable_wheel_joint_steering(wheel, true).unwrap();
    world
        .try_set_wheel_joint_steering_hertz(wheel, 9.0)
        .unwrap();
    world
        .try_set_wheel_joint_steering_damping_ratio(wheel, 0.7)
        .unwrap();
    world
        .try_set_wheel_joint_max_steering_torque(wheel, 19.0)
        .unwrap();
    world
        .try_enable_wheel_joint_steering_limit(wheel, true)
        .unwrap();
    world
        .try_set_wheel_joint_steering_limits(wheel, -0.3, 0.3)
        .unwrap();
    world
        .try_set_wheel_joint_target_steering_angle(wheel, 0.2)
        .unwrap();

    assert!(world.try_wheel_joint_suspension_enabled(wheel).unwrap());
    assert_close(world.try_wheel_joint_suspension_hertz(wheel).unwrap(), 8.0);
    assert_close(
        world
            .try_wheel_joint_suspension_damping_ratio(wheel)
            .unwrap(),
        0.6,
    );
    assert!(
        world
            .try_wheel_joint_suspension_limit_enabled(wheel)
            .unwrap()
    );
    assert_close(
        world.try_wheel_joint_lower_suspension_limit(wheel).unwrap(),
        -0.2,
    );
    assert_close(
        world.try_wheel_joint_upper_suspension_limit(wheel).unwrap(),
        0.4,
    );
    assert!(world.try_wheel_joint_spin_motor_enabled(wheel).unwrap());
    assert_close(world.try_wheel_joint_spin_motor_speed(wheel).unwrap(), 5.0);
    assert_close(world.try_wheel_joint_max_spin_torque(wheel).unwrap(), 18.0);
    assert!(world.try_wheel_joint_spin_speed(wheel).unwrap().is_finite());
    assert!(
        world
            .try_wheel_joint_spin_torque(wheel)
            .unwrap()
            .is_finite()
    );
    assert!(world.try_wheel_joint_steering_enabled(wheel).unwrap());
    assert_close(world.try_wheel_joint_steering_hertz(wheel).unwrap(), 9.0);
    assert_close(
        world.try_wheel_joint_steering_damping_ratio(wheel).unwrap(),
        0.7,
    );
    assert_close(
        world.try_wheel_joint_max_steering_torque(wheel).unwrap(),
        19.0,
    );
    assert!(world.try_wheel_joint_steering_limit_enabled(wheel).unwrap());
    assert_close(
        world.try_wheel_joint_lower_steering_limit(wheel).unwrap(),
        -0.3,
    );
    assert_close(
        world.try_wheel_joint_upper_steering_limit(wheel).unwrap(),
        0.3,
    );
    assert_close(
        world.try_wheel_joint_target_steering_angle(wheel).unwrap(),
        0.2,
    );
    assert!(
        world
            .try_wheel_joint_steering_angle(wheel)
            .unwrap()
            .is_finite()
    );
    assert!(
        world
            .try_wheel_joint_steering_torque(wheel)
            .unwrap()
            .is_finite()
    );
}
