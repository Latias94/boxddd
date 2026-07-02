use boxddd::{
    BodyDef, BodyId, BodyType, DistanceJointDef, Error, FilterJointDef, JointId, JointType,
    MotorJointDef, ParallelJointDef, PrismaticJointDef, RevoluteJointDef, ShapeDef, Sphere,
    SphericalJointDef, WeldJointDef, WheelJointDef, World, WorldDef,
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

fn contains_joint(joints: &[JointId], joint: JointId) -> bool {
    joints.iter().any(|candidate| *candidate == joint)
}

#[test]
fn creates_every_joint_family_and_enumerates_body_joints() {
    let (mut world, a, b) = body_pair();

    let joints = [
        (
            world.create_parallel_joint(ParallelJointDef::new(a, b)),
            JointType::Parallel,
        ),
        (
            world.create_distance_joint(DistanceJointDef::new(a, b).length(1.0)),
            JointType::Distance,
        ),
        (
            world.create_filter_joint(FilterJointDef::new(a, b)),
            JointType::Filter,
        ),
        (
            world.create_motor_joint(MotorJointDef::new(a, b)),
            JointType::Motor,
        ),
        (
            world.create_prismatic_joint(PrismaticJointDef::new(a, b)),
            JointType::Prismatic,
        ),
        (
            world.create_revolute_joint(RevoluteJointDef::new(a, b)),
            JointType::Revolute,
        ),
        (
            world.create_spherical_joint(SphericalJointDef::new(a, b)),
            JointType::Spherical,
        ),
        (
            world.create_weld_joint(WeldJointDef::new(a, b)),
            JointType::Weld,
        ),
        (
            world.create_wheel_joint(WheelJointDef::new(a, b)),
            JointType::Wheel,
        ),
    ];

    for (joint, expected_type) in joints {
        assert!(joint.is_valid());
        assert_eq!(world.try_joint_type(joint).unwrap(), expected_type);
        assert_eq!(world.try_joint_body_a(joint).unwrap(), a);
        assert_eq!(world.try_joint_body_b(joint).unwrap(), b);
    }

    let body_a_joints = world.try_body_joints(a).unwrap();
    let body_b_joints = world.try_body_joints(b).unwrap();
    for (joint, _) in joints {
        assert!(contains_joint(&body_a_joints, joint));
        assert!(contains_joint(&body_b_joints, joint));
    }

    let removed = joints[0].0;
    world.try_destroy_joint(removed, true).unwrap();
    assert_eq!(
        world.try_joint_type(removed).unwrap_err(),
        Error::InvalidJointId
    );
    assert!(!contains_joint(&world.try_body_joints(a).unwrap(), removed));
}

#[test]
fn create_joint_rejects_invalid_stale_and_wrong_world_bodies() {
    let (mut world, a, b) = body_pair();
    let stale = world.create_body(BodyDef::builder().body_type(BodyType::Dynamic).build());
    world.destroy_body(stale);

    assert_eq!(
        world
            .try_create_distance_joint(DistanceJointDef::new(stale, b).length(1.0))
            .unwrap_err(),
        Error::InvalidBodyId
    );
    assert_eq!(
        world
            .try_create_distance_joint(DistanceJointDef::new(a, a).length(1.0))
            .unwrap_err(),
        Error::InvalidArgument
    );

    let mut other_world = World::new(WorldDef::default()).unwrap();
    let other_body =
        other_world.create_body(BodyDef::builder().body_type(BodyType::Dynamic).build());
    assert_eq!(
        world
            .try_create_distance_joint(DistanceJointDef::new(a, other_body).length(1.0))
            .unwrap_err(),
        Error::InvalidArgument
    );
}

#[test]
fn distance_joint_keeps_a_simple_scene_within_a_coarse_bound() {
    let mut world = World::new(WorldDef::builder().gravity([0.0, 0.0, 0.0]).build()).unwrap();
    let anchor = world.create_body(BodyDef::builder().position([0.0, 0.0, 0.0]).build());
    let body = world.create_body(
        BodyDef::builder()
            .body_type(BodyType::Dynamic)
            .position([1.0, 0.0, 0.0])
            .build(),
    );
    world.create_sphere_shape(
        body,
        &ShapeDef::builder().density(1.0).build(),
        &Sphere::new([0.0, 0.0, 0.0], 0.25),
    );
    let joint = world.create_distance_joint(DistanceJointDef::new(anchor, body).length(1.0));

    world
        .try_apply_force_to_center(body, [250.0, 0.0, 0.0], true)
        .unwrap();
    for _ in 0..120 {
        world.step(1.0 / 60.0, 4);
    }

    let length = world.try_distance_joint_current_length(joint).unwrap();
    assert!(length < 1.5, "distance joint stretched too far: {length}");
}
