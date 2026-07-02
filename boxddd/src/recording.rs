use crate::core::{box3d_lock, callback_state};
use crate::debug_draw::{
    DebugDraw, DebugDrawCommand, DebugDrawOptions, DebugShape, HexColor, with_debug_draw,
};
use crate::error::{Error, Result};
use crate::query::QueryFilter;
use crate::types::{Aabb, BodyId, Pos, ShapeId, Vec3, WorldTransform};
use crate::world::World;
use boxddd_sys::ffi;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::path::Path;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::{Mutex, OnceLock};

#[derive(Copy, Clone, Debug)]
struct ActiveRecording {
    world: ffi::b3WorldId,
    recording: usize,
}

static ACTIVE_RECORDINGS: OnceLock<Mutex<Vec<ActiveRecording>>> = OnceLock::new();

fn active_recordings() -> &'static Mutex<Vec<ActiveRecording>> {
    ACTIVE_RECORDINGS.get_or_init(|| Mutex::new(Vec::new()))
}

fn prune_inactive_recordings_locked(records: &mut Vec<ActiveRecording>) {
    records.retain(|record| unsafe { ffi::b3World_IsValid(record.world) });
}

fn recording_key(recording: NonNull<ffi::b3Recording>) -> usize {
    recording.as_ptr() as usize
}

fn active_recording_for_world_locked(world: ffi::b3WorldId) -> Option<usize> {
    let mut records = active_recordings()
        .lock()
        .expect("Box3D recording registry lock poisoned");
    prune_inactive_recordings_locked(&mut records);
    records
        .iter()
        .find(|record| same_world_id(record.world, world))
        .map(|record| record.recording)
}

fn insert_active_recording_locked(world: ffi::b3WorldId, recording: NonNull<ffi::b3Recording>) {
    let mut records = active_recordings()
        .lock()
        .expect("Box3D recording registry lock poisoned");
    prune_inactive_recordings_locked(&mut records);
    records.push(ActiveRecording {
        world,
        recording: recording_key(recording),
    });
}

fn remove_active_recording_locked(recording: NonNull<ffi::b3Recording>) {
    let key = recording_key(recording);
    let mut records = active_recordings()
        .lock()
        .expect("Box3D recording registry lock poisoned");
    records.retain(|record| record.recording != key);
}

fn active_recording_matches_locked(
    world: ffi::b3WorldId,
    recording: NonNull<ffi::b3Recording>,
) -> bool {
    active_recording_for_world_locked(world) == Some(recording_key(recording))
}

pub(crate) fn detach_world_recording_locked(world: ffi::b3WorldId) {
    let mut records = active_recordings()
        .lock()
        .expect("Box3D recording registry lock poisoned");
    records.retain(|record| !same_world_id(record.world, world));
}

#[derive(Debug)]
pub struct Recording {
    raw: NonNull<ffi::b3Recording>,
    active_world: Option<ffi::b3WorldId>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl Recording {
    pub fn new() -> Result<Self> {
        Self::with_capacity(0)
    }

    pub fn with_capacity(byte_capacity: usize) -> Result<Self> {
        let byte_capacity = i32::try_from(byte_capacity).map_err(|_| Error::InvalidArgument)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        let raw = unsafe { ffi::b3CreateRecording(byte_capacity) };
        Ok(Self {
            raw: NonNull::new(raw).ok_or(Error::CreateRecordingFailed)?,
            active_world: None,
            _not_send_sync: PhantomData,
        })
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path_to_cstring(path)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        let raw = unsafe { ffi::b3LoadRecordingFromFile(path.as_ptr()) };
        Ok(Self {
            raw: NonNull::new(raw).ok_or(Error::RecordingIoFailed)?,
            active_world: None,
            _not_send_sync: PhantomData,
        })
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path_to_cstring(path)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        if self.active_world_is_valid_locked() {
            return Err(Error::ResourceLifetimeViolation);
        }
        if unsafe { ffi::b3SaveRecordingToFile(self.raw.as_ptr(), path.as_ptr()) } {
            Ok(())
        } else {
            Err(Error::RecordingIoFailed)
        }
    }

    pub fn len(&self) -> usize {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3Recording_GetSize(self.raw.as_ptr()) }.max(0) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn bytes(&self) -> Result<&[u8]> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        if self.active_world_is_valid_locked() {
            return Err(Error::ResourceLifetimeViolation);
        }
        Ok(unsafe { self.bytes_locked() })
    }

    pub fn to_vec(&self) -> Result<Vec<u8>> {
        Ok(self.bytes()?.to_vec())
    }

    pub fn validate_replay(&self, worker_count: i32) -> Result<bool> {
        validate_replay_bytes(self.bytes()?, worker_count)
    }

    pub fn create_player(&self, worker_count: i32) -> Result<RecPlayer> {
        RecPlayer::from_bytes(self.bytes()?, worker_count)
    }

    #[inline]
    fn active_world_is_valid_locked(&self) -> bool {
        self.active_world
            .is_some_and(|world| unsafe { ffi::b3World_IsValid(world) })
    }

    #[inline]
    fn clear_inactive_world_locked(&mut self) {
        if self
            .active_world
            .is_some_and(|world| !unsafe { ffi::b3World_IsValid(world) })
        {
            self.active_world = None;
        }
    }

    #[inline]
    unsafe fn bytes_locked(&self) -> &[u8] {
        let size = unsafe { ffi::b3Recording_GetSize(self.raw.as_ptr()) }.max(0) as usize;
        let data = unsafe { ffi::b3Recording_GetData(self.raw.as_ptr()) };
        if data.is_null() || size == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(data, size) }
        }
    }
}

impl Drop for Recording {
    fn drop(&mut self) {
        let _guard = box3d_lock::lock();
        if let Some(world) = self.active_world.take()
            && unsafe { ffi::b3World_IsValid(world) }
            && active_recording_matches_locked(world, self.raw)
        {
            unsafe { ffi::b3World_StopRecording(world) };
        }
        remove_active_recording_locked(self.raw);
        unsafe { ffi::b3DestroyRecording(self.raw.as_ptr()) };
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RecQueryType {
    OverlapAabb,
    OverlapShape,
    CastRay,
    CastShape,
    CastRayClosest,
    CastMover,
    CollideMover,
}

impl RecQueryType {
    pub const fn from_raw(raw: ffi::b3RecQueryType) -> Option<Self> {
        match raw {
            ffi::b3RecQueryType_b3_recQueryOverlapAABB => Some(Self::OverlapAabb),
            ffi::b3RecQueryType_b3_recQueryOverlapShape => Some(Self::OverlapShape),
            ffi::b3RecQueryType_b3_recQueryCastRay => Some(Self::CastRay),
            ffi::b3RecQueryType_b3_recQueryCastShape => Some(Self::CastShape),
            ffi::b3RecQueryType_b3_recQueryCastRayClosest => Some(Self::CastRayClosest),
            ffi::b3RecQueryType_b3_recQueryCastMover => Some(Self::CastMover),
            ffi::b3RecQueryType_b3_recQueryCollideMover => Some(Self::CollideMover),
            _ => None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct RecPlayerInfo {
    pub frame_count: i32,
    pub worker_count: i32,
    pub time_step: f32,
    pub sub_step_count: i32,
    pub length_scale: f32,
    pub bounds: Aabb,
}

impl RecPlayerInfo {
    #[inline]
    pub fn from_raw(raw: ffi::b3RecPlayerInfo) -> Self {
        Self {
            frame_count: raw.frameCount,
            worker_count: raw.workerCount,
            time_step: raw.timeStep,
            sub_step_count: raw.subStepCount,
            length_scale: raw.lengthScale,
            bounds: Aabb::from_raw(raw.bounds),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct RecQueryInfo {
    pub query_type: RecQueryType,
    pub filter: QueryFilter,
    pub aabb: Aabb,
    pub origin: Pos,
    pub translation: Vec3,
    pub hit_count: i32,
    pub key: u64,
    pub id: u64,
    pub name: Option<String>,
}

impl RecQueryInfo {
    fn from_raw(raw: ffi::b3RecQueryInfo) -> Result<Self> {
        Ok(Self {
            query_type: RecQueryType::from_raw(raw.type_).ok_or(Error::InvalidArgument)?,
            filter: QueryFilter {
                category_bits: raw.filter.categoryBits,
                mask_bits: raw.filter.maskBits,
                id: raw.filter.id,
            },
            aabb: Aabb::from_raw(raw.aabb),
            origin: Pos::from_raw(raw.origin),
            translation: Vec3::from_raw(raw.translation),
            hit_count: raw.hitCount,
            key: raw.key,
            id: raw.id,
            name: if raw.name.is_null() {
                None
            } else {
                Some(
                    unsafe { CStr::from_ptr(raw.name) }
                        .to_string_lossy()
                        .into_owned(),
                )
            },
        })
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RecQueryHit {
    pub shape_id: ShapeId,
    pub point: Pos,
    pub normal: Vec3,
    pub fraction: f32,
}

impl RecQueryHit {
    #[inline]
    fn from_raw(raw: ffi::b3RecQueryHit) -> Self {
        Self {
            shape_id: ShapeId::from_raw(raw.shape),
            point: Pos::from_raw(raw.point),
            normal: Vec3::from_raw(raw.normal),
            fraction: raw.fraction,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReplayWorldId {
    pub index1: u16,
    pub generation: u16,
}

impl ReplayWorldId {
    #[inline]
    pub const fn from_raw(raw: ffi::b3WorldId) -> Self {
        Self {
            index1: raw.index1,
            generation: raw.generation,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> ffi::b3WorldId {
        ffi::b3WorldId {
            index1: self.index1,
            generation: self.generation,
        }
    }

    pub fn is_valid(self) -> bool {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3World_IsValid(self.into_raw()) }
    }
}

#[derive(Debug)]
pub struct RecPlayer {
    raw: NonNull<ffi::b3RecPlayer>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl RecPlayer {
    pub fn from_bytes(bytes: &[u8], worker_count: i32) -> Result<Self> {
        validate_replay_input(bytes, worker_count)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        let raw = unsafe {
            ffi::b3RecPlayer_Create(bytes.as_ptr().cast(), bytes.len() as i32, worker_count)
        };
        Ok(Self {
            raw: NonNull::new(raw).ok_or(Error::CreateRecPlayerFailed)?,
            _not_send_sync: PhantomData,
        })
    }

    pub fn world_id(&self) -> ReplayWorldId {
        let _guard = box3d_lock::lock();
        ReplayWorldId::from_raw(unsafe { ffi::b3RecPlayer_GetWorldId(self.raw.as_ptr()) })
    }

    pub fn step_frame(&mut self) -> Result<bool> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        Ok(unsafe { ffi::b3RecPlayer_StepFrame(self.raw.as_ptr()) })
    }

    pub fn restart(&mut self) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_Restart(self.raw.as_ptr()) };
        Ok(())
    }

    pub fn seek_frame(&mut self, target_frame: i32) -> Result<()> {
        if target_frame < 0 {
            return Err(Error::InvalidArgument);
        }
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_SeekFrame(self.raw.as_ptr(), target_frame) };
        Ok(())
    }

    pub fn frame(&self) -> i32 {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetFrame(self.raw.as_ptr()) }
    }

    pub fn frame_count(&self) -> i32 {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetFrameCount(self.raw.as_ptr()) }
    }

    pub fn is_at_end(&self) -> bool {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_IsAtEnd(self.raw.as_ptr()) }
    }

    pub fn has_diverged(&self) -> bool {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_HasDiverged(self.raw.as_ptr()) }
    }

    pub fn info(&self) -> RecPlayerInfo {
        let _guard = box3d_lock::lock();
        RecPlayerInfo::from_raw(unsafe { ffi::b3RecPlayer_GetInfo(self.raw.as_ptr()) })
    }

    pub fn diverge_frame(&self) -> Option<i32> {
        let _guard = box3d_lock::lock();
        let frame = unsafe { ffi::b3RecPlayer_GetDivergeFrame(self.raw.as_ptr()) };
        (frame >= 0).then_some(frame)
    }

    pub fn set_worker_count(&mut self, count: i32) -> Result<()> {
        if count < 1 {
            return Err(Error::InvalidArgument);
        }
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_SetWorkerCount(self.raw.as_ptr(), count) };
        Ok(())
    }

    pub fn set_keyframe_policy(
        &mut self,
        budget_bytes: usize,
        min_interval_frames: i32,
    ) -> Result<()> {
        if min_interval_frames < 0 {
            return Err(Error::InvalidArgument);
        }
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        unsafe {
            ffi::b3RecPlayer_SetKeyframePolicy(self.raw.as_ptr(), budget_bytes, min_interval_frames)
        };
        Ok(())
    }

    pub fn keyframe_budget(&self) -> usize {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetKeyframeBudget(self.raw.as_ptr()) }
    }

    pub fn keyframe_min_interval(&self) -> i32 {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetKeyframeMinInterval(self.raw.as_ptr()) }
    }

    pub fn keyframe_interval(&self) -> i32 {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetKeyframeInterval(self.raw.as_ptr()) }
    }

    pub fn keyframe_bytes(&self) -> usize {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetKeyframeBytes(self.raw.as_ptr()) }
    }

    pub fn body_count(&self) -> i32 {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetBodyCount(self.raw.as_ptr()) }
    }

    pub fn body_id(&self, index: i32) -> Result<Option<BodyId>> {
        if index < 0 {
            return Err(Error::IndexOutOfRange);
        }
        let _guard = box3d_lock::lock();
        let raw = unsafe { ffi::b3RecPlayer_GetBodyId(self.raw.as_ptr(), index) };
        if raw.index1 == 0 {
            Ok(None)
        } else {
            Ok(Some(BodyId::from_raw(raw)))
        }
    }

    pub fn frame_query_count(&self) -> i32 {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetFrameQueryCount(self.raw.as_ptr()) }
    }

    pub fn frame_query(&self, index: i32) -> Result<RecQueryInfo> {
        if index < 0 || index >= self.frame_query_count() {
            return Err(Error::IndexOutOfRange);
        }
        let _guard = box3d_lock::lock();
        RecQueryInfo::from_raw(unsafe { ffi::b3RecPlayer_GetFrameQuery(self.raw.as_ptr(), index) })
    }

    pub fn frame_query_hit(&self, query_index: i32, hit_index: i32) -> Result<RecQueryHit> {
        let query = self.frame_query(query_index)?;
        if hit_index < 0 || hit_index >= query.hit_count {
            return Err(Error::IndexOutOfRange);
        }
        let _guard = box3d_lock::lock();
        Ok(RecQueryHit::from_raw(unsafe {
            ffi::b3RecPlayer_GetFrameQueryHit(self.raw.as_ptr(), query_index, hit_index)
        }))
    }

    pub fn draw_frame_queries_collect(
        &mut self,
        options: DebugDrawOptions,
        query_index: Option<i32>,
        selected_index: Option<i32>,
    ) -> Result<Vec<DebugDrawCommand>> {
        let mut commands = Vec::new();
        self.draw_frame_queries_collect_into(&mut commands, options, query_index, selected_index)?;
        Ok(commands)
    }

    pub fn draw_frame_queries_collect_into(
        &mut self,
        out: &mut Vec<DebugDrawCommand>,
        options: DebugDrawOptions,
        query_index: Option<i32>,
        selected_index: Option<i32>,
    ) -> Result<()> {
        struct Collector<'a> {
            commands: &'a mut Vec<DebugDrawCommand>,
        }

        impl DebugDraw for Collector<'_> {
            fn draw_shape(
                &mut self,
                shape: Option<DebugShape>,
                transform: WorldTransform,
                color: HexColor,
            ) -> bool {
                self.commands.push(DebugDrawCommand::Shape {
                    shape,
                    transform,
                    color,
                });
                true
            }

            fn draw_segment(&mut self, p1: Pos, p2: Pos, color: HexColor) {
                self.commands
                    .push(DebugDrawCommand::Segment { p1, p2, color });
            }

            fn draw_transform(&mut self, transform: WorldTransform) {
                self.commands.push(DebugDrawCommand::Transform(transform));
            }

            fn draw_point(&mut self, position: Pos, size: f32, color: HexColor) {
                self.commands.push(DebugDrawCommand::Point {
                    position,
                    size,
                    color,
                });
            }

            fn draw_sphere(&mut self, center: Pos, radius: f32, color: HexColor, alpha: f32) {
                self.commands.push(DebugDrawCommand::Sphere {
                    center,
                    radius,
                    color,
                    alpha,
                });
            }

            fn draw_capsule(&mut self, p1: Pos, p2: Pos, radius: f32, color: HexColor, alpha: f32) {
                self.commands.push(DebugDrawCommand::Capsule {
                    p1,
                    p2,
                    radius,
                    color,
                    alpha,
                });
            }

            fn draw_bounds(&mut self, aabb: Aabb, color: HexColor) {
                self.commands.push(DebugDrawCommand::Bounds { aabb, color });
            }

            fn draw_box(&mut self, extents: Vec3, transform: WorldTransform, color: HexColor) {
                self.commands.push(DebugDrawCommand::Box {
                    extents,
                    transform,
                    color,
                });
            }

            fn draw_string(&mut self, position: Pos, text: &str, color: HexColor) {
                self.commands.push(DebugDrawCommand::String {
                    position,
                    text: text.to_owned(),
                    color,
                });
            }
        }

        out.clear();
        let mut collector = Collector { commands: out };
        self.draw_frame_queries(&mut collector, options, query_index, selected_index)
    }

    pub fn draw_frame_queries(
        &mut self,
        drawer: &mut impl DebugDraw,
        options: DebugDrawOptions,
        query_index: Option<i32>,
        selected_index: Option<i32>,
    ) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let query_index = checked_optional_index(query_index)?;
        let selected_index = checked_optional_index(selected_index)?;
        if query_index >= 0 && query_index >= self.frame_query_count() {
            return Err(Error::IndexOutOfRange);
        }
        with_debug_draw(drawer, options, |draw| {
            let _guard = box3d_lock::lock();
            unsafe {
                ffi::b3RecPlayer_DrawFrameQueries(
                    self.raw.as_ptr(),
                    draw,
                    query_index,
                    selected_index,
                )
            };
            Ok(())
        })
    }
}

impl Drop for RecPlayer {
    fn drop(&mut self) {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_Destroy(self.raw.as_ptr()) };
    }
}

impl World {
    pub fn try_start_recording(&mut self, recording: &mut Recording) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        recording.clear_inactive_world_locked();
        if recording.active_world.is_some() {
            return Err(Error::ResourceLifetimeViolation);
        }
        if active_recording_for_world_locked(self.raw()).is_some() {
            return Err(Error::ResourceLifetimeViolation);
        }
        unsafe { ffi::b3World_StartRecording(self.raw(), recording.raw.as_ptr()) };
        insert_active_recording_locked(self.raw(), recording.raw);
        recording.active_world = Some(self.raw());
        Ok(())
    }

    pub fn start_recording(&mut self, recording: &mut Recording) {
        self.try_start_recording(recording)
            .expect("Box3D failed to start recording");
    }

    pub fn try_stop_recording(&mut self, recording: &mut Recording) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        self.check_world_valid_locked()?;
        recording.clear_inactive_world_locked();
        let Some(active_world) = recording.active_world else {
            return Err(Error::ResourceLifetimeViolation);
        };
        if !same_world_id(active_world, self.raw())
            || !active_recording_matches_locked(self.raw(), recording.raw)
        {
            return Err(Error::ResourceLifetimeViolation);
        }
        unsafe { ffi::b3World_StopRecording(self.raw()) };
        remove_active_recording_locked(recording.raw);
        recording.active_world = None;
        Ok(())
    }

    pub fn stop_recording(&mut self, recording: &mut Recording) {
        self.try_stop_recording(recording)
            .expect("Box3D failed to stop recording");
    }
}

pub fn validate_replay_bytes(bytes: &[u8], worker_count: i32) -> Result<bool> {
    validate_replay_input(bytes, worker_count)?;
    callback_state::check_not_in_callback()?;
    let _guard = box3d_lock::lock();
    Ok(unsafe { ffi::b3ValidateReplay(bytes.as_ptr().cast(), bytes.len() as i32, worker_count) })
}

fn validate_replay_input(bytes: &[u8], worker_count: i32) -> Result<()> {
    if bytes.is_empty() || bytes.len() > i32::MAX as usize || worker_count < 1 {
        Err(Error::InvalidArgument)
    } else {
        Ok(())
    }
}

fn checked_optional_index(index: Option<i32>) -> Result<i32> {
    match index {
        Some(index) if index < 0 => Err(Error::IndexOutOfRange),
        Some(index) => Ok(index),
        None => Ok(-1),
    }
}

fn same_world_id(a: ffi::b3WorldId, b: ffi::b3WorldId) -> bool {
    a.index1 == b.index1 && a.generation == b.generation
}

fn path_to_cstring(path: impl AsRef<Path>) -> Result<CString> {
    CString::new(path.as_ref().as_os_str().to_string_lossy().as_bytes())
        .map_err(|_| Error::NulByteInString)
}
