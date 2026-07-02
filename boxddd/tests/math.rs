use boxddd::{
    Aabb, Capacity, Error, Filter, MassData, Matrix3, Plane, Pos, Quat, Transform, Vec2, Vec3,
    WorldTransform,
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
}
