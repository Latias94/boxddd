#![cfg(feature = "serde")]

use boxddd::{
    Aabb, DebugDrawCommand, DebugDrawFrame, DebugDrawOptions, DebugShapeAsset, DebugShapeEvent,
    DebugShapeGeometry, DebugShapeHandle, HexColor, QueryFilter, RecPlayerInfo, ShapeId, ShapeType,
    SurfaceMaterial, Vec3,
};

#[test]
fn value_types_round_trip_through_json() {
    let aabb = Aabb {
        lower_bound: Vec3::new(-1.0, -2.0, -3.0),
        upper_bound: Vec3::new(1.0, 2.0, 3.0),
    };
    let json = serde_json::to_string(&aabb).unwrap();
    assert_eq!(serde_json::from_str::<Aabb>(&json).unwrap(), aabb);

    let material = SurfaceMaterial {
        friction: 0.5,
        restitution: 0.25,
        rolling_resistance: 0.1,
        tangent_velocity: Vec3::X,
        user_material_id: 7,
        custom_color: 0xff00ff,
    };
    let json = serde_json::to_string(&material).unwrap();
    assert_eq!(
        serde_json::from_str::<SurfaceMaterial>(&json).unwrap(),
        material
    );
}

#[test]
fn serialized_public_values_do_not_expose_raw_pointer_fields() {
    let filter = QueryFilter::new()
        .category_bits(0b10)
        .mask_bits(0b11)
        .id(42);
    let value = serde_json::to_value(filter).unwrap();
    assert_eq!(value["id"], 42);
    assert!(value.get("name").is_none());
    assert!(value.get("raw").is_none());

    let options = serde_json::to_value(DebugDrawOptions::default()).unwrap();
    assert!(options.get("drawing_bounds").is_some());
    assert!(options.get("context").is_none());
    assert!(options.get("raw").is_none());
}

#[test]
fn debug_draw_and_replay_metadata_are_serializable() {
    let handle = DebugShapeHandle::new(1, 2).unwrap();
    let asset = DebugShapeAsset {
        handle,
        shape_id: ShapeId::default(),
        shape_type: ShapeType::Sphere,
        geometry: DebugShapeGeometry::Sphere {
            center: Vec3::ZERO,
            radius: 1.0,
        },
    };
    let frame = DebugDrawFrame {
        events: vec![DebugShapeEvent::Created(asset)],
        commands: vec![DebugDrawCommand::Shape {
            handle: Some(handle),
            transform: Default::default(),
            color: HexColor::from_raw(0x02_ff_00_ff),
        }],
        diagnostics: Vec::new(),
    };
    let json = serde_json::to_string(&frame).unwrap();
    let decoded: DebugDrawFrame = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, frame);

    let command = DebugDrawCommand::Point {
        position: [1.0, 2.0, 3.0].into(),
        size: 4.0,
        color: HexColor::RED,
    };
    let json = serde_json::to_string(&command).unwrap();
    let decoded: DebugDrawCommand = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, command);

    let info = RecPlayerInfo {
        frame_count: 3,
        worker_count: 1,
        time_step: 1.0 / 60.0,
        sub_step_count: 4,
        length_scale: 1.0,
        bounds: Aabb {
            lower_bound: Vec3::new(-1.0, -1.0, -1.0),
            upper_bound: Vec3::new(1.0, 1.0, 1.0),
        },
    };
    let json = serde_json::to_string(&info).unwrap();
    assert_eq!(serde_json::from_str::<RecPlayerInfo>(&json).unwrap(), info);
}
