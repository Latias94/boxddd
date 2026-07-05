use bevy_boxddd::math::{
    apply_boxddd_transform, to_bevy_local_transform, to_bevy_pos, to_bevy_quat, to_bevy_transform,
    to_bevy_vec3, to_boxddd_pos, to_boxddd_quat, to_boxddd_vec3, try_to_boxddd_quat,
};
use bevy_boxddd::prelude::{
    BevyQuatBoxdddExt, BevyTransformBoxdddExt, BevyVec3BoxdddExt, BoxdddQuatBevyExt,
    BoxdddTransformBevyExt, BoxdddVec3BevyExt,
};
use bevy_math::{Quat, Vec3};
use bevy_transform::components::Transform;

#[test]
fn bevy_math_functions_convert_boxddd_values() {
    let bevy_vec = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(to_boxddd_vec3(bevy_vec), boxddd::Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(to_boxddd_pos(bevy_vec), boxddd::Pos::new(1.0, 2.0, 3.0));

    let bevy_quat = Quat::from_rotation_y(0.5);
    let boxddd_quat = to_boxddd_quat(bevy_quat);
    assert_eq!(to_bevy_quat(boxddd_quat), bevy_quat);
    assert_eq!(try_to_boxddd_quat(bevy_quat).unwrap(), boxddd_quat);

    let boxddd_vec = boxddd::Vec3::new(4.0, 5.0, 6.0);
    let boxddd_pos = boxddd::Pos::new(7.0, 8.0, 9.0);
    assert_eq!(to_bevy_vec3(boxddd_vec), Vec3::new(4.0, 5.0, 6.0));
    assert_eq!(to_bevy_pos(boxddd_pos), Vec3::new(7.0, 8.0, 9.0));

    let world_transform = boxddd::WorldTransform::new(boxddd_pos, boxddd_quat);
    let bevy_transform = to_bevy_transform(world_transform);
    assert_eq!(bevy_transform.translation, Vec3::new(7.0, 8.0, 9.0));
    assert_eq!(bevy_transform.rotation, bevy_quat);

    let local_transform = boxddd::Transform::new(boxddd_vec, boxddd_quat);
    let bevy_local_transform = to_bevy_local_transform(local_transform);
    assert_eq!(bevy_local_transform.translation, Vec3::new(4.0, 5.0, 6.0));
    assert_eq!(bevy_local_transform.rotation, bevy_quat);
}

#[test]
fn bevy_math_extension_traits_convert_boxddd_values() {
    let bevy_vec = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(bevy_vec.to_boxddd_vec3(), boxddd::Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(bevy_vec.to_boxddd_pos(), boxddd::Pos::new(1.0, 2.0, 3.0));

    let bevy_quat = Quat::IDENTITY;
    assert_eq!(bevy_quat.to_boxddd_quat(), boxddd::Quat::IDENTITY);
    assert_eq!(
        bevy_quat.try_to_boxddd_quat().unwrap(),
        boxddd::Quat::IDENTITY
    );
    assert_eq!(boxddd::Quat::IDENTITY.to_bevy_quat(), Quat::IDENTITY);

    assert_eq!(
        boxddd::Vec3::new(1.0, 2.0, 3.0).to_bevy_vec3(),
        Vec3::new(1.0, 2.0, 3.0)
    );
    assert_eq!(
        boxddd::Pos::new(4.0, 5.0, 6.0).to_bevy_vec3(),
        Vec3::new(4.0, 5.0, 6.0)
    );

    let bevy_transform = Transform::from_xyz(1.0, 2.0, 3.0);
    assert_eq!(
        bevy_transform.to_boxddd_world_transform().p,
        boxddd::Pos::new(1.0, 2.0, 3.0)
    );
    assert_eq!(
        bevy_transform.to_boxddd_local_transform().p,
        boxddd::Vec3::new(1.0, 2.0, 3.0)
    );

    let world_transform =
        boxddd::WorldTransform::new(boxddd::Pos::new(2.0, 3.0, 4.0), boxddd::Quat::IDENTITY);
    assert_eq!(
        world_transform.to_bevy_transform().translation,
        Vec3::new(2.0, 3.0, 4.0)
    );

    let local_transform =
        boxddd::Transform::new(boxddd::Vec3::new(5.0, 6.0, 7.0), boxddd::Quat::IDENTITY);
    assert_eq!(
        local_transform.to_bevy_transform().translation,
        Vec3::new(5.0, 6.0, 7.0)
    );
}

#[test]
fn bevy_math_adapters_validate_untrusted_quaternions() {
    let invalid = Quat::from_xyzw(0.0, 0.0, 0.0, 2.0);
    assert_eq!(
        try_to_boxddd_quat(invalid).unwrap_err(),
        boxddd::Error::InvalidArgument
    );
    assert_eq!(
        invalid.try_to_boxddd_quat().unwrap_err(),
        boxddd::Error::InvalidArgument
    );
}

#[test]
fn applying_boxddd_world_transform_preserves_bevy_scale() {
    let mut transform = Transform::from_scale(Vec3::splat(2.0));
    let world_transform = boxddd::WorldTransform::new(
        boxddd::Pos::new(3.0, 4.0, 5.0),
        Quat::from_rotation_x(0.5).to_boxddd_quat(),
    );

    apply_boxddd_transform(&mut transform, world_transform);
    assert_eq!(transform.translation, Vec3::new(3.0, 4.0, 5.0));
    assert_eq!(transform.rotation, world_transform.q.to_bevy_quat());
    assert_eq!(transform.scale, Vec3::splat(2.0));

    let mut transform_from_trait = Transform::from_scale(Vec3::splat(3.0));
    world_transform.apply_to_bevy_transform(&mut transform_from_trait);
    assert_eq!(transform_from_trait.translation, Vec3::new(3.0, 4.0, 5.0));
    assert_eq!(transform_from_trait.scale, Vec3::splat(3.0));
}
