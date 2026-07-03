use crate::core::{box3d_lock, callback_state};
use crate::error::Result;
use crate::types::{BodyId, ContactId, JointId, Pos, ShapeId, Vec3, WorldTransform};
use crate::world::World;
use boxddd_sys::ffi;
use std::ffi::c_void;

#[derive(Copy, Clone)]
pub struct BodyMove<'a>(&'a ffi::b3BodyMoveEvent);

impl BodyMove<'_> {
    pub fn body_id(&self) -> BodyId {
        BodyId::from_raw(self.0.bodyId)
    }

    pub fn transform(&self) -> WorldTransform {
        WorldTransform::from_raw(self.0.transform)
    }

    pub fn fell_asleep(&self) -> bool {
        self.0.fellAsleep
    }

    pub fn raw_user_data(&self) -> *mut c_void {
        self.0.userData
    }
}

pub struct BodyMoveIter<'a>(std::slice::Iter<'a, ffi::b3BodyMoveEvent>);

impl<'a> Iterator for BodyMoveIter<'a> {
    type Item = BodyMove<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(BodyMove)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BodyMoveEvent {
    pub body_id: BodyId,
    pub transform: WorldTransform,
    pub fell_asleep: bool,
    pub raw_user_data: *mut c_void,
}

#[derive(Copy, Clone)]
pub struct SensorBeginTouch<'a>(&'a ffi::b3SensorBeginTouchEvent);

impl SensorBeginTouch<'_> {
    pub fn sensor_shape(&self) -> ShapeId {
        ShapeId::from_raw(self.0.sensorShapeId)
    }

    pub fn visitor_shape(&self) -> ShapeId {
        ShapeId::from_raw(self.0.visitorShapeId)
    }
}

#[derive(Copy, Clone)]
pub struct SensorEndTouch<'a>(&'a ffi::b3SensorEndTouchEvent);

impl SensorEndTouch<'_> {
    pub fn sensor_shape(&self) -> ShapeId {
        ShapeId::from_raw(self.0.sensorShapeId)
    }

    pub fn visitor_shape(&self) -> ShapeId {
        ShapeId::from_raw(self.0.visitorShapeId)
    }
}

pub struct SensorBeginIter<'a>(std::slice::Iter<'a, ffi::b3SensorBeginTouchEvent>);

impl<'a> Iterator for SensorBeginIter<'a> {
    type Item = SensorBeginTouch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(SensorBeginTouch)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct SensorEndIter<'a>(std::slice::Iter<'a, ffi::b3SensorEndTouchEvent>);

impl<'a> Iterator for SensorEndIter<'a> {
    type Item = SensorEndTouch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(SensorEndTouch)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SensorBeginTouchEvent {
    pub sensor_shape: ShapeId,
    pub visitor_shape: ShapeId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SensorEndTouchEvent {
    pub sensor_shape: ShapeId,
    pub visitor_shape: ShapeId,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SensorEvents {
    pub begin: Vec<SensorBeginTouchEvent>,
    pub end: Vec<SensorEndTouchEvent>,
}

#[derive(Copy, Clone)]
pub struct ContactBeginTouch<'a>(&'a ffi::b3ContactBeginTouchEvent);

impl ContactBeginTouch<'_> {
    pub fn shape_a(&self) -> ShapeId {
        ShapeId::from_raw(self.0.shapeIdA)
    }

    pub fn shape_b(&self) -> ShapeId {
        ShapeId::from_raw(self.0.shapeIdB)
    }

    pub fn contact_id(&self) -> ContactId {
        ContactId::from_raw(self.0.contactId)
    }
}

#[derive(Copy, Clone)]
pub struct ContactEndTouch<'a>(&'a ffi::b3ContactEndTouchEvent);

impl ContactEndTouch<'_> {
    pub fn shape_a(&self) -> ShapeId {
        ShapeId::from_raw(self.0.shapeIdA)
    }

    pub fn shape_b(&self) -> ShapeId {
        ShapeId::from_raw(self.0.shapeIdB)
    }

    pub fn contact_id(&self) -> ContactId {
        ContactId::from_raw(self.0.contactId)
    }
}

#[derive(Copy, Clone)]
pub struct ContactHit<'a>(&'a ffi::b3ContactHitEvent);

impl ContactHit<'_> {
    pub fn shape_a(&self) -> ShapeId {
        ShapeId::from_raw(self.0.shapeIdA)
    }

    pub fn shape_b(&self) -> ShapeId {
        ShapeId::from_raw(self.0.shapeIdB)
    }

    pub fn contact_id(&self) -> ContactId {
        ContactId::from_raw(self.0.contactId)
    }

    pub fn point(&self) -> Pos {
        Pos::from_raw(self.0.point)
    }

    pub fn normal(&self) -> Vec3 {
        Vec3::from_raw(self.0.normal)
    }

    pub fn approach_speed(&self) -> f32 {
        self.0.approachSpeed
    }

    pub fn user_material_id_a(&self) -> u64 {
        self.0.userMaterialIdA
    }

    pub fn user_material_id_b(&self) -> u64 {
        self.0.userMaterialIdB
    }
}

pub struct ContactBeginIter<'a>(std::slice::Iter<'a, ffi::b3ContactBeginTouchEvent>);

impl<'a> Iterator for ContactBeginIter<'a> {
    type Item = ContactBeginTouch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(ContactBeginTouch)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct ContactEndIter<'a>(std::slice::Iter<'a, ffi::b3ContactEndTouchEvent>);

impl<'a> Iterator for ContactEndIter<'a> {
    type Item = ContactEndTouch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(ContactEndTouch)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct ContactHitIter<'a>(std::slice::Iter<'a, ffi::b3ContactHitEvent>);

impl<'a> Iterator for ContactHitIter<'a> {
    type Item = ContactHit<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(ContactHit)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContactBeginTouchEvent {
    pub shape_a: ShapeId,
    pub shape_b: ShapeId,
    pub contact_id: ContactId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContactEndTouchEvent {
    pub shape_a: ShapeId,
    pub shape_b: ShapeId,
    pub contact_id: ContactId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContactHitEvent {
    pub shape_a: ShapeId,
    pub shape_b: ShapeId,
    pub contact_id: ContactId,
    pub point: Pos,
    pub normal: Vec3,
    pub approach_speed: f32,
    pub user_material_id_a: u64,
    pub user_material_id_b: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContactEvents {
    pub begin: Vec<ContactBeginTouchEvent>,
    pub end: Vec<ContactEndTouchEvent>,
    pub hit: Vec<ContactHitEvent>,
}

#[derive(Copy, Clone)]
pub struct JointEventView<'a>(&'a ffi::b3JointEvent);

impl JointEventView<'_> {
    pub fn joint_id(&self) -> JointId {
        JointId::from_raw(self.0.jointId)
    }

    pub fn raw_user_data(&self) -> *mut c_void {
        self.0.userData
    }
}

pub struct JointEventIter<'a>(std::slice::Iter<'a, ffi::b3JointEvent>);

impl<'a> Iterator for JointEventIter<'a> {
    type Item = JointEventView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(JointEventView)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct JointEvent {
    pub joint_id: JointId,
    pub raw_user_data: *mut c_void,
}

fn map_snapshot_into<T, U>(out: &mut Vec<U>, input: &[T], mut map: impl FnMut(&T) -> U) {
    out.clear();
    out.reserve(input.len());
    out.extend(input.iter().map(|value| map(value)));
}

fn raw_event_slice<'a, T>(ptr: *const T, count: i32) -> &'a [T] {
    if count > 0 && !ptr.is_null() {
        unsafe { std::slice::from_raw_parts(ptr, count as usize) }
    } else {
        &[]
    }
}

fn body_event_slice<'a>(raw: ffi::b3BodyEvents) -> &'a [ffi::b3BodyMoveEvent] {
    raw_event_slice(raw.moveEvents, raw.moveCount)
}

fn sensor_begin_slice<'a>(raw: ffi::b3SensorEvents) -> &'a [ffi::b3SensorBeginTouchEvent] {
    raw_event_slice(raw.beginEvents, raw.beginCount)
}

fn sensor_end_slice<'a>(raw: ffi::b3SensorEvents) -> &'a [ffi::b3SensorEndTouchEvent] {
    raw_event_slice(raw.endEvents, raw.endCount)
}

fn contact_begin_slice<'a>(raw: ffi::b3ContactEvents) -> &'a [ffi::b3ContactBeginTouchEvent] {
    raw_event_slice(raw.beginEvents, raw.beginCount)
}

fn contact_end_slice<'a>(raw: ffi::b3ContactEvents) -> &'a [ffi::b3ContactEndTouchEvent] {
    raw_event_slice(raw.endEvents, raw.endCount)
}

fn contact_hit_slice<'a>(raw: ffi::b3ContactEvents) -> &'a [ffi::b3ContactHitEvent] {
    raw_event_slice(raw.hitEvents, raw.hitCount)
}

fn joint_event_slice<'a>(raw: ffi::b3JointEvents) -> &'a [ffi::b3JointEvent] {
    raw_event_slice(raw.jointEvents, raw.count)
}

impl World {
    pub fn body_events(&self) -> Vec<BodyMoveEvent> {
        self.try_body_events().expect("invalid Box3D world")
    }

    pub fn try_body_events(&self) -> Result<Vec<BodyMoveEvent>> {
        let mut out = Vec::new();
        self.try_body_events_into(&mut out)?;
        Ok(out)
    }

    pub fn body_events_into(&self, out: &mut Vec<BodyMoveEvent>) {
        self.try_body_events_into(out).expect("invalid Box3D world");
    }

    pub fn try_body_events_into(&self, out: &mut Vec<BodyMoveEvent>) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let raw = unsafe { ffi::b3World_GetBodyEvents(self.raw()) };
        let events = body_event_slice(raw);
        map_snapshot_into(out, events, |event| BodyMoveEvent {
            body_id: BodyId::from_raw(event.bodyId),
            transform: WorldTransform::from_raw(event.transform),
            fell_asleep: event.fellAsleep,
            raw_user_data: event.userData,
        });
        Ok(())
    }

    pub fn try_with_body_events_view<T>(&self, f: impl FnOnce(BodyMoveIter<'_>) -> T) -> Result<T> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let raw = unsafe { ffi::b3World_GetBodyEvents(self.raw()) };
        let events = body_event_slice(raw);
        drop(_guard);
        Ok(f(BodyMoveIter(events.iter())))
    }

    pub fn with_body_events_view<T>(&self, f: impl FnOnce(BodyMoveIter<'_>) -> T) -> T {
        self.try_with_body_events_view(f)
            .expect("invalid Box3D world")
    }

    pub unsafe fn try_with_body_events_raw<T>(
        &self,
        f: impl FnOnce(&[ffi::b3BodyMoveEvent]) -> T,
    ) -> Result<T> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let raw = unsafe { ffi::b3World_GetBodyEvents(self.raw()) };
        let events = body_event_slice(raw);
        drop(_guard);
        Ok(f(events))
    }

    pub unsafe fn with_body_events_raw<T>(
        &self,
        f: impl FnOnce(&[ffi::b3BodyMoveEvent]) -> T,
    ) -> T {
        unsafe { self.try_with_body_events_raw(f) }.expect("invalid Box3D world")
    }

    pub fn sensor_events(&self) -> SensorEvents {
        self.try_sensor_events().expect("invalid Box3D world")
    }

    pub fn try_sensor_events(&self) -> Result<SensorEvents> {
        let mut out = SensorEvents::default();
        self.try_sensor_events_into(&mut out)?;
        Ok(out)
    }

    pub fn sensor_events_into(&self, out: &mut SensorEvents) {
        self.try_sensor_events_into(out)
            .expect("invalid Box3D world");
    }

    pub fn try_sensor_events_into(&self, out: &mut SensorEvents) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let raw = unsafe { ffi::b3World_GetSensorEvents(self.raw()) };
        map_snapshot_into(&mut out.begin, sensor_begin_slice(raw), |event| {
            SensorBeginTouchEvent {
                sensor_shape: ShapeId::from_raw(event.sensorShapeId),
                visitor_shape: ShapeId::from_raw(event.visitorShapeId),
            }
        });
        map_snapshot_into(&mut out.end, sensor_end_slice(raw), |event| {
            SensorEndTouchEvent {
                sensor_shape: ShapeId::from_raw(event.sensorShapeId),
                visitor_shape: ShapeId::from_raw(event.visitorShapeId),
            }
        });
        Ok(())
    }

    pub fn try_with_sensor_events_view<T>(
        &self,
        f: impl FnOnce(SensorBeginIter<'_>, SensorEndIter<'_>) -> T,
    ) -> Result<T> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let raw = unsafe { ffi::b3World_GetSensorEvents(self.raw()) };
        let begin = sensor_begin_slice(raw);
        let end = sensor_end_slice(raw);
        drop(_guard);
        Ok(f(SensorBeginIter(begin.iter()), SensorEndIter(end.iter())))
    }

    pub fn with_sensor_events_view<T>(
        &self,
        f: impl FnOnce(SensorBeginIter<'_>, SensorEndIter<'_>) -> T,
    ) -> T {
        self.try_with_sensor_events_view(f)
            .expect("invalid Box3D world")
    }

    pub unsafe fn try_with_sensor_events_raw<T>(
        &self,
        f: impl FnOnce(&[ffi::b3SensorBeginTouchEvent], &[ffi::b3SensorEndTouchEvent]) -> T,
    ) -> Result<T> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let raw = unsafe { ffi::b3World_GetSensorEvents(self.raw()) };
        let begin = sensor_begin_slice(raw);
        let end = sensor_end_slice(raw);
        drop(_guard);
        Ok(f(begin, end))
    }

    pub unsafe fn with_sensor_events_raw<T>(
        &self,
        f: impl FnOnce(&[ffi::b3SensorBeginTouchEvent], &[ffi::b3SensorEndTouchEvent]) -> T,
    ) -> T {
        unsafe { self.try_with_sensor_events_raw(f) }.expect("invalid Box3D world")
    }

    pub fn contact_events(&self) -> ContactEvents {
        self.try_contact_events().expect("invalid Box3D world")
    }

    pub fn try_contact_events(&self) -> Result<ContactEvents> {
        let mut out = ContactEvents::default();
        self.try_contact_events_into(&mut out)?;
        Ok(out)
    }

    pub fn contact_events_into(&self, out: &mut ContactEvents) {
        self.try_contact_events_into(out)
            .expect("invalid Box3D world");
    }

    pub fn try_contact_events_into(&self, out: &mut ContactEvents) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let raw = unsafe { ffi::b3World_GetContactEvents(self.raw()) };
        map_snapshot_into(&mut out.begin, contact_begin_slice(raw), |event| {
            ContactBeginTouchEvent {
                shape_a: ShapeId::from_raw(event.shapeIdA),
                shape_b: ShapeId::from_raw(event.shapeIdB),
                contact_id: ContactId::from_raw(event.contactId),
            }
        });
        map_snapshot_into(&mut out.end, contact_end_slice(raw), |event| {
            ContactEndTouchEvent {
                shape_a: ShapeId::from_raw(event.shapeIdA),
                shape_b: ShapeId::from_raw(event.shapeIdB),
                contact_id: ContactId::from_raw(event.contactId),
            }
        });
        map_snapshot_into(&mut out.hit, contact_hit_slice(raw), |event| {
            ContactHitEvent {
                shape_a: ShapeId::from_raw(event.shapeIdA),
                shape_b: ShapeId::from_raw(event.shapeIdB),
                contact_id: ContactId::from_raw(event.contactId),
                point: Pos::from_raw(event.point),
                normal: Vec3::from_raw(event.normal),
                approach_speed: event.approachSpeed,
                user_material_id_a: event.userMaterialIdA,
                user_material_id_b: event.userMaterialIdB,
            }
        });
        Ok(())
    }

    pub fn try_with_contact_events_view<T>(
        &self,
        f: impl FnOnce(ContactBeginIter<'_>, ContactEndIter<'_>, ContactHitIter<'_>) -> T,
    ) -> Result<T> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let raw = unsafe { ffi::b3World_GetContactEvents(self.raw()) };
        let begin = contact_begin_slice(raw);
        let end = contact_end_slice(raw);
        let hit = contact_hit_slice(raw);
        drop(_guard);
        Ok(f(
            ContactBeginIter(begin.iter()),
            ContactEndIter(end.iter()),
            ContactHitIter(hit.iter()),
        ))
    }

    pub fn with_contact_events_view<T>(
        &self,
        f: impl FnOnce(ContactBeginIter<'_>, ContactEndIter<'_>, ContactHitIter<'_>) -> T,
    ) -> T {
        self.try_with_contact_events_view(f)
            .expect("invalid Box3D world")
    }

    pub unsafe fn try_with_contact_events_raw<T>(
        &self,
        f: impl FnOnce(
            &[ffi::b3ContactBeginTouchEvent],
            &[ffi::b3ContactEndTouchEvent],
            &[ffi::b3ContactHitEvent],
        ) -> T,
    ) -> Result<T> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let raw = unsafe { ffi::b3World_GetContactEvents(self.raw()) };
        let begin = contact_begin_slice(raw);
        let end = contact_end_slice(raw);
        let hit = contact_hit_slice(raw);
        drop(_guard);
        Ok(f(begin, end, hit))
    }

    pub unsafe fn with_contact_events_raw<T>(
        &self,
        f: impl FnOnce(
            &[ffi::b3ContactBeginTouchEvent],
            &[ffi::b3ContactEndTouchEvent],
            &[ffi::b3ContactHitEvent],
        ) -> T,
    ) -> T {
        unsafe { self.try_with_contact_events_raw(f) }.expect("invalid Box3D world")
    }

    pub fn joint_events(&self) -> Vec<JointEvent> {
        self.try_joint_events().expect("invalid Box3D world")
    }

    pub fn try_joint_events(&self) -> Result<Vec<JointEvent>> {
        let mut out = Vec::new();
        self.try_joint_events_into(&mut out)?;
        Ok(out)
    }

    pub fn joint_events_into(&self, out: &mut Vec<JointEvent>) {
        self.try_joint_events_into(out)
            .expect("invalid Box3D world");
    }

    pub fn try_joint_events_into(&self, out: &mut Vec<JointEvent>) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let raw = unsafe { ffi::b3World_GetJointEvents(self.raw()) };
        map_snapshot_into(out, joint_event_slice(raw), |event| JointEvent {
            joint_id: JointId::from_raw(event.jointId),
            raw_user_data: event.userData,
        });
        Ok(())
    }

    pub fn try_with_joint_events_view<T>(
        &self,
        f: impl FnOnce(JointEventIter<'_>) -> T,
    ) -> Result<T> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let raw = unsafe { ffi::b3World_GetJointEvents(self.raw()) };
        let events = joint_event_slice(raw);
        drop(_guard);
        Ok(f(JointEventIter(events.iter())))
    }

    pub fn with_joint_events_view<T>(&self, f: impl FnOnce(JointEventIter<'_>) -> T) -> T {
        self.try_with_joint_events_view(f)
            .expect("invalid Box3D world")
    }

    pub unsafe fn try_with_joint_events_raw<T>(
        &self,
        f: impl FnOnce(&[ffi::b3JointEvent]) -> T,
    ) -> Result<T> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let raw = unsafe { ffi::b3World_GetJointEvents(self.raw()) };
        let events = joint_event_slice(raw);
        drop(_guard);
        Ok(f(events))
    }

    pub unsafe fn with_joint_events_raw<T>(&self, f: impl FnOnce(&[ffi::b3JointEvent]) -> T) -> T {
        unsafe { self.try_with_joint_events_raw(f) }.expect("invalid Box3D world")
    }
}
