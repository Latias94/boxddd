use boxddd::{
    Aabb, Capacity, Error, Filter, MassData, Matrix3, Plane, Pos, Quat, Transform, Vec2, Vec3,
    WorldTransform, closest_point_on_segment, compute_cos_sin, deterministic_atan2, line_distance,
    segment_distance, steiner_inertia,
};

#[test]
fn value_types_round_trip_through_raw() {
    let vec2 = Vec2::new(1.0, 2.0);
    assert_eq!(Vec2::from_raw(vec2.into_raw()), vec2);

    let vec3 = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(Vec3::from_raw(vec3.into_raw()), vec3);

    let quat = Quat::new(Vec3::new(0.0, 0.0, 0.0), 1.0);
    assert_eq!(Quat::from_raw(quat.into_raw()), quat);

    let transform = Transform::new(vec3, quat);
    assert_eq!(Transform::from_raw(transform.into_raw()), transform);

    let pos = Pos::from([4.0_f32, 5.0, 6.0]);
    assert_eq!(Pos::from_raw(pos.into_raw()), pos);

    let world_transform = WorldTransform::new(pos, quat);
    assert_eq!(
        WorldTransform::from_raw(world_transform.into_raw()),
        world_transform
    );

    let matrix = Matrix3 {
        cx: Vec3::X,
        cy: Vec3::Y,
        cz: Vec3::Z,
    };
    assert_eq!(Matrix3::from_raw(matrix.into_raw()), matrix);

    let aabb = Aabb {
        lower_bound: [-1.0, -2.0, -3.0].into(),
        upper_bound: [1.0, 2.0, 3.0].into(),
    };
    assert_eq!(Aabb::from_raw(aabb.into_raw()), aabb);

    let plane = Plane {
        normal: Vec3::Y,
        offset: 2.5,
    };
    assert_eq!(Plane::from_raw(plane.into_raw()), plane);

    let filter = Filter {
        category_bits: 0b10,
        mask_bits: 0b11,
        group_index: -2,
    };
    assert_eq!(Filter::from_raw(filter.into_raw()), filter);

    let mass = MassData {
        mass: 12.0,
        center: vec3,
        inertia: matrix,
    };
    assert_eq!(MassData::from_raw(mass.into_raw()), mass);

    let capacity = Capacity {
        static_shape_count: 1,
        dynamic_shape_count: 2,
        static_body_count: 3,
        dynamic_body_count: 4,
        contact_count: 5,
    };
    assert_eq!(Capacity::from_raw(capacity.into_raw()), capacity);
}

#[test]
fn invalid_numeric_values_are_rejected_before_ffi() {
    assert_eq!(
        Vec3::new(f32::NAN, 0.0, 0.0).validate(),
        Err(Error::InvalidArgument)
    );
    assert_eq!(
        Quat::new(Vec3::ZERO, f32::INFINITY).validate(),
        Err(Error::InvalidArgument)
    );
    assert_eq!(
        Transform::new(Vec3::new(0.0, f32::NEG_INFINITY, 0.0), Quat::IDENTITY).validate(),
        Err(Error::InvalidArgument)
    );
    assert_eq!(
        Pos::from([f32::NAN, 0.0, 0.0]).validate(),
        Err(Error::InvalidArgument)
    );
    assert_eq!(
        (Plane {
            normal: Vec3::new(2.0, 0.0, 0.0),
            offset: 0.0,
        })
        .validate(),
        Err(Error::InvalidArgument)
    );
}

#[test]
fn segment_distance_helpers_return_owned_values() {
    let closest = closest_point_on_segment(Vec3::ZERO, Vec3::X, Vec3::new(0.25, 1.0, 0.0)).unwrap();
    assert_vec3_close(closest, Vec3::new(0.25, 0.0, 0.0));

    let line = line_distance(Vec3::ZERO, Vec3::X, Vec3::new(0.0, 1.0, 1.0), Vec3::Y).unwrap();
    assert_vec3_close(line.point1, Vec3::ZERO);
    assert_vec3_close(line.point2, Vec3::new(0.0, 0.0, 1.0));
    assert_close(line.distance(), 1.0);

    let segment = segment_distance(
        Vec3::ZERO,
        Vec3::X,
        Vec3::new(0.5, -1.0, 1.0),
        Vec3::new(0.5, 1.0, 1.0),
    )
    .unwrap();
    assert_vec3_close(segment.point1, Vec3::new(0.5, 0.0, 0.0));
    assert_vec3_close(segment.point2, Vec3::new(0.5, 0.0, 1.0));
    assert_close(segment.fraction1, 0.5);
    assert_close(segment.fraction2, 0.5);
    assert_close(segment.distance_squared(), 1.0);
}

#[test]
fn segment_distance_helpers_validate_inputs() {
    assert_eq!(
        closest_point_on_segment(Vec3::ZERO, Vec3::X, Vec3::new(f32::NAN, 0.0, 0.0)).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        line_distance(Vec3::ZERO, Vec3::ZERO, Vec3::X, Vec3::Y).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        segment_distance(
            Vec3::ZERO,
            Vec3::X,
            Vec3::Y,
            Vec3::new(f32::INFINITY, 0.0, 0.0)
        )
        .unwrap_err(),
        Error::InvalidArgument
    );
}

#[test]
fn deterministic_math_helpers_validate_and_return_owned_values() {
    assert_close(deterministic_atan2(0.0, 0.0).unwrap(), 0.0);
    assert_close(
        deterministic_atan2(1.0, 0.0).unwrap(),
        std::f32::consts::FRAC_PI_2,
    );
    assert_eq!(
        deterministic_atan2(f32::NAN, 1.0).unwrap_err(),
        Error::InvalidArgument
    );

    let zero = compute_cos_sin(0.0).unwrap();
    assert_close(zero.cosine, 1.0);
    assert_close(zero.sine, 0.0);
    assert_eq!(
        compute_cos_sin(f32::INFINITY).unwrap_err(),
        Error::InvalidArgument
    );

    let identity_matrix = Matrix3 {
        cx: Vec3::X,
        cy: Vec3::Y,
        cz: Vec3::Z,
    };
    assert_eq!(identity_matrix.validate().unwrap(), identity_matrix);
    assert_eq!(Quat::from_matrix(identity_matrix).unwrap(), Quat::IDENTITY);
    let turn_z = Quat::between_unit_vectors(Vec3::X, Vec3::Y).unwrap();
    assert!(turn_z.is_valid());

    let steiner = steiner_inertia(2.0, Vec3::new(1.0, 2.0, 3.0)).unwrap();
    assert!(steiner.is_valid());
    assert_close(steiner.cx.x, 26.0);
    assert_close(steiner.cy.y, 20.0);
    assert_close(steiner.cz.z, 10.0);
}

#[test]
fn deterministic_math_helpers_reject_invalid_inputs() {
    let invalid_matrix = Matrix3 {
        cx: Vec3::new(f32::NAN, 0.0, 0.0),
        cy: Vec3::Y,
        cz: Vec3::Z,
    };
    assert!(!invalid_matrix.is_valid());
    assert_eq!(
        invalid_matrix.validate().unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        Quat::from_matrix(invalid_matrix).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        Quat::between_unit_vectors(Vec3::new(2.0, 0.0, 0.0), Vec3::Y).unwrap_err(),
        Error::InvalidArgument
    );
    assert_eq!(
        steiner_inertia(-1.0, Vec3::ZERO).unwrap_err(),
        Error::InvalidArgument
    );

    let valid_aabb = Aabb {
        lower_bound: [-1.0, -1.0, -1.0].into(),
        upper_bound: [1.0, 1.0, 1.0].into(),
    };
    assert!(valid_aabb.is_bounded());
    assert!(valid_aabb.is_sane());

    let unbounded_aabb = Aabb {
        lower_bound: [-1.0e31, 0.0, 0.0].into(),
        upper_bound: [1.0, 1.0, 1.0].into(),
    };
    assert!(unbounded_aabb.is_valid());
    assert!(!unbounded_aabb.is_bounded());
    assert!(!unbounded_aabb.is_sane());
}

fn assert_vec3_close(actual: Vec3, expected: Vec3) {
    assert_close(actual.x, expected.x);
    assert_close(actual.y, expected.y);
    assert_close(actual.z, expected.z);
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 1.0e-5,
        "expected {actual} to be close to {expected}"
    );
}
