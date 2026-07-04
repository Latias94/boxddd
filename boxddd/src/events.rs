use crate::core::{box3d_lock, callback_state};
use crate::error::Result;
use crate::types::{BodyId, ContactData, ContactId, JointId, Pos, ShapeId, Vec3, WorldTransform};
use crate::world::World;
use boxddd_sys::ffi;
use std::ffi::c_void;

#[derive(Copy, Clone)]
/// Borrowed view of a body movement event from the latest world step.
///
/// The view is only valid for the callback passed to `with_body_events_view`.
pub struct BodyMove<'a>(&'a ffi::b3BodyMoveEvent);

impl BodyMove<'_> {
    /// Returns the body that moved.
    pub fn body_id(&self) -> BodyId {
        BodyId::from_raw(self.0.bodyId)
    }

    /// Returns the body's world transform after the movement.
    pub fn transform(&self) -> WorldTransform {
        WorldTransform::from_raw(self.0.transform)
    }

    /// Returns true when the body went to sleep during the step.
    pub fn fell_asleep(&self) -> bool {
        self.0.fellAsleep
    }

    /// Returns the raw Box3D `userData` pointer value observed in this event.
    ///
    /// The pointer is untyped and not owned by `boxddd`; interpreting or dereferencing it is the
    /// caller's unsafe interop responsibility.
    pub fn raw_user_data(&self) -> *mut c_void {
        self.0.userData
    }
}

/// Iterator over borrowed body movement events.
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
/// Owned body movement event snapshot.
pub struct BodyMoveEvent {
    /// Body that moved.
    pub body_id: BodyId,
    /// Body transform after the movement.
    pub transform: WorldTransform,
    /// Whether the body went to sleep during the step.
    pub fell_asleep: bool,
    /// Raw Box3D `userData` pointer value observed in the event.
    pub raw_user_data: *mut c_void,
}

#[derive(Copy, Clone)]
/// Borrowed view of a sensor begin-touch event.
pub struct SensorBeginTouch<'a>(&'a ffi::b3SensorBeginTouchEvent);

impl SensorBeginTouch<'_> {
    /// Returns the sensor shape receiving the touch.
    pub fn sensor_shape(&self) -> ShapeId {
        ShapeId::from_raw(self.0.sensorShapeId)
    }

    /// Returns the non-sensor shape entering the sensor.
    pub fn visitor_shape(&self) -> ShapeId {
        ShapeId::from_raw(self.0.visitorShapeId)
    }
}

#[derive(Copy, Clone)]
/// Borrowed view of a sensor end-touch event.
pub struct SensorEndTouch<'a>(&'a ffi::b3SensorEndTouchEvent);

impl SensorEndTouch<'_> {
    /// Returns the sensor shape ending the touch.
    pub fn sensor_shape(&self) -> ShapeId {
        ShapeId::from_raw(self.0.sensorShapeId)
    }

    /// Returns the non-sensor shape leaving the sensor.
    pub fn visitor_shape(&self) -> ShapeId {
        ShapeId::from_raw(self.0.visitorShapeId)
    }
}

/// Iterator over borrowed sensor begin-touch events.
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

/// Iterator over borrowed sensor end-touch events.
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
/// Owned sensor begin-touch event snapshot.
pub struct SensorBeginTouchEvent {
    /// Sensor shape receiving the touch.
    pub sensor_shape: ShapeId,
    /// Shape entering the sensor.
    pub visitor_shape: ShapeId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Owned sensor end-touch event snapshot.
pub struct SensorEndTouchEvent {
    /// Sensor shape ending the touch.
    pub sensor_shape: ShapeId,
    /// Shape leaving the sensor.
    pub visitor_shape: ShapeId,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// Owned sensor event snapshot for the latest world step.
pub struct SensorEvents {
    /// Sensor touches that began during the step.
    pub begin: Vec<SensorBeginTouchEvent>,
    /// Sensor touches that ended during the step.
    pub end: Vec<SensorEndTouchEvent>,
}

#[derive(Copy, Clone)]
/// Borrowed view of a contact begin-touch event.
pub struct ContactBeginTouch<'a>(&'a ffi::b3ContactBeginTouchEvent);

impl ContactBeginTouch<'_> {
    /// Returns the first shape in the contact pair.
    pub fn shape_a(&self) -> ShapeId {
        ShapeId::from_raw(self.0.shapeIdA)
    }

    /// Returns the second shape in the contact pair.
    pub fn shape_b(&self) -> ShapeId {
        ShapeId::from_raw(self.0.shapeIdB)
    }

    /// Returns the contact identifier for the pair.
    pub fn contact_id(&self) -> ContactId {
        ContactId::from_raw(self.0.contactId)
    }
}

#[derive(Copy, Clone)]
/// Borrowed view of a contact end-touch event.
pub struct ContactEndTouch<'a>(&'a ffi::b3ContactEndTouchEvent);

impl ContactEndTouch<'_> {
    /// Returns the first shape in the contact pair.
    pub fn shape_a(&self) -> ShapeId {
        ShapeId::from_raw(self.0.shapeIdA)
    }

    /// Returns the second shape in the contact pair.
    pub fn shape_b(&self) -> ShapeId {
        ShapeId::from_raw(self.0.shapeIdB)
    }

    /// Returns the contact identifier for the pair.
    pub fn contact_id(&self) -> ContactId {
        ContactId::from_raw(self.0.contactId)
    }
}

#[derive(Copy, Clone)]
/// Borrowed view of a contact hit event.
pub struct ContactHit<'a>(&'a ffi::b3ContactHitEvent);

impl ContactHit<'_> {
    /// Returns the first shape in the contact pair.
    pub fn shape_a(&self) -> ShapeId {
        ShapeId::from_raw(self.0.shapeIdA)
    }

    /// Returns the second shape in the contact pair.
    pub fn shape_b(&self) -> ShapeId {
        ShapeId::from_raw(self.0.shapeIdB)
    }

    /// Returns the contact identifier for the pair.
    pub fn contact_id(&self) -> ContactId {
        ContactId::from_raw(self.0.contactId)
    }

    /// Returns the hit point in world coordinates.
    pub fn point(&self) -> Pos {
        Pos::from_raw(self.0.point)
    }

    /// Returns the hit normal.
    pub fn normal(&self) -> Vec3 {
        Vec3::from_raw(self.0.normal)
    }

    /// Returns the relative approach speed at impact.
    pub fn approach_speed(&self) -> f32 {
        self.0.approachSpeed
    }

    /// Returns the user material identifier for shape A.
    pub fn user_material_id_a(&self) -> u64 {
        self.0.userMaterialIdA
    }

    /// Returns the user material identifier for shape B.
    pub fn user_material_id_b(&self) -> u64 {
        self.0.userMaterialIdB
    }
}

/// Iterator over borrowed contact begin-touch events.
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

/// Iterator over borrowed contact end-touch events.
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

/// Iterator over borrowed contact hit events.
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
/// Owned contact begin-touch event snapshot.
pub struct ContactBeginTouchEvent {
    /// First shape in the contact pair.
    pub shape_a: ShapeId,
    /// Second shape in the contact pair.
    pub shape_b: ShapeId,
    /// Contact identifier for the pair.
    pub contact_id: ContactId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Owned contact end-touch event snapshot.
pub struct ContactEndTouchEvent {
    /// First shape in the contact pair.
    pub shape_a: ShapeId,
    /// Second shape in the contact pair.
    pub shape_b: ShapeId,
    /// Contact identifier for the pair.
    pub contact_id: ContactId,
}

#[derive(Clone, Debug, PartialEq)]
/// Owned contact hit event snapshot.
pub struct ContactHitEvent {
    /// First shape in the contact pair.
    pub shape_a: ShapeId,
    /// Second shape in the contact pair.
    pub shape_b: ShapeId,
    /// Contact identifier for the pair.
    pub contact_id: ContactId,
    /// Hit point in world coordinates.
    pub point: Pos,
    /// Hit normal.
    pub normal: Vec3,
    /// Relative approach speed at impact.
    pub approach_speed: f32,
    /// User material identifier for shape A.
    pub user_material_id_a: u64,
    /// User material identifier for shape B.
    pub user_material_id_b: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Owned contact event snapshot for the latest world step.
pub struct ContactEvents {
    /// Contacts that began touching during the step.
    pub begin: Vec<ContactBeginTouchEvent>,
    /// Contacts that stopped touching during the step.
    pub end: Vec<ContactEndTouchEvent>,
    /// Contact hit events reported during the step.
    pub hit: Vec<ContactHitEvent>,
}

#[derive(Copy, Clone)]
/// Borrowed view of a joint event.
pub struct JointEventView<'a>(&'a ffi::b3JointEvent);

impl JointEventView<'_> {
    /// Returns the joint that generated the event.
    pub fn joint_id(&self) -> JointId {
        JointId::from_raw(self.0.jointId)
    }

    /// Returns the raw Box3D `userData` pointer value observed in this event.
    ///
    /// The pointer is untyped and not owned by `boxddd`; interpreting or dereferencing it is the
    /// caller's unsafe interop responsibility.
    pub fn raw_user_data(&self) -> *mut c_void {
        self.0.userData
    }
}

/// Iterator over borrowed joint events.
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
/// Owned joint event snapshot.
pub struct JointEvent {
    /// Joint that generated the event.
    pub joint_id: JointId,
    /// Raw Box3D `userData` pointer value observed in the event.
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
    /// Returns owned body movement events from the latest world step.
    pub fn body_events(&self) -> Vec<BodyMoveEvent> {
        self.try_body_events().expect("invalid Box3D world")
    }

    /// Tries to return owned body movement events from the latest world step.
    pub fn try_body_events(&self) -> Result<Vec<BodyMoveEvent>> {
        let mut out = Vec::new();
        self.try_body_events_into(&mut out)?;
        Ok(out)
    }

    /// Writes owned body movement events into `out`, clearing it first.
    pub fn body_events_into(&self, out: &mut Vec<BodyMoveEvent>) {
        self.try_body_events_into(out).expect("invalid Box3D world");
    }

    /// Tries to write owned body movement events into `out`, clearing it first.
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

    /// Borrows body movement events for the duration of `f`.
    ///
    /// Use this when you want to avoid allocation and do not need to keep events after the
    /// callback returns.
    pub fn try_with_body_events_view<T>(&self, f: impl FnOnce(BodyMoveIter<'_>) -> T) -> Result<T> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        let raw = unsafe { ffi::b3World_GetBodyEvents(self.raw()) };
        let events = body_event_slice(raw);
        drop(_guard);
        Ok(f(BodyMoveIter(events.iter())))
    }

    /// Borrows body movement events for the duration of `f`.
    pub fn with_body_events_view<T>(&self, f: impl FnOnce(BodyMoveIter<'_>) -> T) -> T {
        self.try_with_body_events_view(f)
            .expect("invalid Box3D world")
    }

    /// Borrows raw Box3D body movement events for the duration of `f`.
    ///
    /// # Safety
    ///
    /// The raw event records contain untyped `userData` pointers owned by the application. The
    /// callback must not store the slice or dereference those pointers unless it can uphold the
    /// original pointer validity and aliasing requirements.
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

    /// Borrows raw Box3D body movement events for the duration of `f`.
    ///
    /// # Safety
    ///
    /// See [`Self::try_with_body_events_raw`].
    pub unsafe fn with_body_events_raw<T>(
        &self,
        f: impl FnOnce(&[ffi::b3BodyMoveEvent]) -> T,
    ) -> T {
        unsafe { self.try_with_body_events_raw(f) }.expect("invalid Box3D world")
    }

    /// Returns owned sensor events from the latest world step.
    pub fn sensor_events(&self) -> SensorEvents {
        self.try_sensor_events().expect("invalid Box3D world")
    }

    /// Tries to return owned sensor events from the latest world step.
    pub fn try_sensor_events(&self) -> Result<SensorEvents> {
        let mut out = SensorEvents::default();
        self.try_sensor_events_into(&mut out)?;
        Ok(out)
    }

    /// Writes owned sensor events into `out`, clearing its vectors first.
    pub fn sensor_events_into(&self, out: &mut SensorEvents) {
        self.try_sensor_events_into(out)
            .expect("invalid Box3D world");
    }

    /// Tries to write owned sensor events into `out`, clearing its vectors first.
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

    /// Borrows sensor begin and end events for the duration of `f`.
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

    /// Borrows sensor begin and end events for the duration of `f`.
    pub fn with_sensor_events_view<T>(
        &self,
        f: impl FnOnce(SensorBeginIter<'_>, SensorEndIter<'_>) -> T,
    ) -> T {
        self.try_with_sensor_events_view(f)
            .expect("invalid Box3D world")
    }

    /// Borrows raw Box3D sensor events for the duration of `f`.
    ///
    /// # Safety
    ///
    /// The callback must not store the slices beyond its call. The raw records must be interpreted
    /// according to Box3D's event layout.
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

    /// Borrows raw Box3D sensor events for the duration of `f`.
    ///
    /// # Safety
    ///
    /// See [`Self::try_with_sensor_events_raw`].
    pub unsafe fn with_sensor_events_raw<T>(
        &self,
        f: impl FnOnce(&[ffi::b3SensorBeginTouchEvent], &[ffi::b3SensorEndTouchEvent]) -> T,
    ) -> T {
        unsafe { self.try_with_sensor_events_raw(f) }.expect("invalid Box3D world")
    }

    /// Returns owned contact events from the latest world step.
    pub fn contact_events(&self) -> ContactEvents {
        self.try_contact_events().expect("invalid Box3D world")
    }

    /// Tries to return owned contact events from the latest world step.
    pub fn try_contact_events(&self) -> Result<ContactEvents> {
        let mut out = ContactEvents::default();
        self.try_contact_events_into(&mut out)?;
        Ok(out)
    }

    /// Writes owned contact events into `out`, clearing its vectors first.
    pub fn contact_events_into(&self, out: &mut ContactEvents) {
        self.try_contact_events_into(out)
            .expect("invalid Box3D world");
    }

    /// Returns the current data for a live contact.
    pub fn contact_data(&self, contact_id: ContactId) -> ContactData {
        self.try_contact_data(contact_id)
            .expect("invalid ContactId or Box3D world")
    }

    /// Tries to return the current data for a live contact.
    pub fn try_contact_data(&self, contact_id: ContactId) -> Result<ContactData> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_contact_belongs_locked(contact_id)?;
        let raw = unsafe { ffi::b3Contact_GetData(contact_id.into_raw()) };
        Ok(unsafe { ContactData::from_raw(raw) })
    }

    /// Tries to write owned contact events into `out`, clearing its vectors first.
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

    /// Borrows contact begin, end, and hit events for the duration of `f`.
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

    /// Borrows contact begin, end, and hit events for the duration of `f`.
    pub fn with_contact_events_view<T>(
        &self,
        f: impl FnOnce(ContactBeginIter<'_>, ContactEndIter<'_>, ContactHitIter<'_>) -> T,
    ) -> T {
        self.try_with_contact_events_view(f)
            .expect("invalid Box3D world")
    }

    /// Borrows raw Box3D contact events for the duration of `f`.
    ///
    /// # Safety
    ///
    /// The callback must not store the slices beyond its call. The raw records must be interpreted
    /// according to Box3D's event layout.
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

    /// Borrows raw Box3D contact events for the duration of `f`.
    ///
    /// # Safety
    ///
    /// See [`Self::try_with_contact_events_raw`].
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

    /// Returns owned joint events from the latest world step.
    pub fn joint_events(&self) -> Vec<JointEvent> {
        self.try_joint_events().expect("invalid Box3D world")
    }

    /// Tries to return owned joint events from the latest world step.
    pub fn try_joint_events(&self) -> Result<Vec<JointEvent>> {
        let mut out = Vec::new();
        self.try_joint_events_into(&mut out)?;
        Ok(out)
    }

    /// Writes owned joint events into `out`, clearing it first.
    pub fn joint_events_into(&self, out: &mut Vec<JointEvent>) {
        self.try_joint_events_into(out)
            .expect("invalid Box3D world");
    }

    /// Tries to write owned joint events into `out`, clearing it first.
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

    /// Borrows joint events for the duration of `f`.
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

    /// Borrows joint events for the duration of `f`.
    pub fn with_joint_events_view<T>(&self, f: impl FnOnce(JointEventIter<'_>) -> T) -> T {
        self.try_with_joint_events_view(f)
            .expect("invalid Box3D world")
    }

    /// Borrows raw Box3D joint events for the duration of `f`.
    ///
    /// # Safety
    ///
    /// The callback must not store the slice beyond its call. Raw `userData` pointers are
    /// application-owned and must only be dereferenced when their validity is known.
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

    /// Borrows raw Box3D joint events for the duration of `f`.
    ///
    /// # Safety
    ///
    /// See [`Self::try_with_joint_events_raw`].
    pub unsafe fn with_joint_events_raw<T>(&self, f: impl FnOnce(&[ffi::b3JointEvent]) -> T) -> T {
        unsafe { self.try_with_joint_events_raw(f) }.expect("invalid Box3D world")
    }
}
