use crate::core::{box3d_lock, callback_state};
use crate::debug_draw::{
    CollectDebugDraw, DebugDraw, DebugDrawCommand, DebugDrawOptions, with_debug_draw,
};
use crate::error::{Error, Result};
use crate::query::QueryFilter;
use crate::types::{Aabb, BodyId, Pos, ShapeId, Vec3};
use crate::world::World;
use boxddd_sys::ffi;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Write;
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

/// Owning handle for a Box3D recording stream.
#[derive(Debug)]
pub struct Recording {
    raw: NonNull<ffi::b3Recording>,
    active_world: Option<ffi::b3WorldId>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl Recording {
    /// Creates a recording buffer with Box3D's default initial capacity.
    pub fn new() -> Result<Self> {
        Self::with_capacity(0)
    }

    /// Creates a recording buffer with an optional initial byte capacity.
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

    /// Loads a recording buffer from a file.
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

    /// Saves the current recording bytes to a file.
    ///
    /// This fails while the recording is attached to a live world because Box3D
    /// may still be mutating the backing buffer.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        callback_state::check_not_in_callback()?;
        let bytes = {
            let _guard = box3d_lock::lock();
            if self.active_world_is_valid_locked() {
                return Err(Error::ResourceLifetimeViolation);
            }
            unsafe { self.bytes_locked() }.to_vec()
        };

        let mut file = File::create(path).map_err(|_| Error::RecordingIoFailed)?;
        file.write_all(&bytes)
            .and_then(|_| file.flush())
            .map_err(|_| Error::RecordingIoFailed)
    }

    /// Returns the number of bytes currently stored in the recording buffer.
    pub fn len(&self) -> usize {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3Recording_GetSize(self.raw.as_ptr()) }.max(0) as usize
    }

    /// Returns whether the recording buffer currently has no bytes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Borrows the raw recording bytes.
    ///
    /// The returned slice is tied to `self` and is unavailable while recording is
    /// active on a live world.
    pub fn bytes(&self) -> Result<&[u8]> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        if self.active_world_is_valid_locked() {
            return Err(Error::ResourceLifetimeViolation);
        }
        Ok(unsafe { self.bytes_locked() })
    }

    /// Copies the raw recording bytes into an owned vector.
    pub fn to_vec(&self) -> Result<Vec<u8>> {
        Ok(self.bytes()?.to_vec())
    }

    /// Validates that this recording can replay with the requested worker count.
    pub fn validate_replay(&self, worker_count: i32) -> Result<bool> {
        validate_replay_bytes(self.bytes()?, worker_count)
    }

    /// Creates a replay player from this recording.
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

/// Kind of query captured in a recording.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RecQueryType {
    /// Axis-aligned bounding-box overlap query.
    OverlapAabb,
    /// Shape overlap query.
    OverlapShape,
    /// Ray-cast query.
    CastRay,
    /// Shape-cast query.
    CastShape,
    /// Closest-hit ray-cast query.
    CastRayClosest,
    /// Capsule mover cast query.
    CastMover,
    /// Capsule mover collision-plane query.
    CollideMover,
}

impl RecQueryType {
    /// Converts raw Box3D data into the safe value type.
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

/// Metadata stored by a recording replay player.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct RecPlayerInfo {
    /// Number of recorded frames.
    pub frame_count: i32,
    /// Worker count used by the recording.
    pub worker_count: i32,
    /// Simulation time step for the frame.
    pub time_step: f32,
    /// Sub-step count used by the recording.
    pub sub_step_count: i32,
    /// Length scale used by the recording.
    pub length_scale: f32,
    /// Bounds stored in the recording metadata.
    pub bounds: Aabb,
}

impl RecPlayerInfo {
    /// Converts raw Box3D data into the safe value type.
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

/// Metadata for a query captured in the current replay frame.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct RecQueryInfo {
    /// Recorded query kind.
    pub query_type: RecQueryType,
    /// Collision filter used by the query.
    pub filter: QueryFilter,
    /// Axis-aligned bounding box.
    pub aabb: Aabb,
    /// Query origin.
    pub origin: Pos,
    /// Translation vector used by the query.
    pub translation: Vec3,
    /// Number of hits recorded for the query.
    pub hit_count: i32,
    /// Recording key.
    pub key: u64,
    /// User-defined query identifier.
    pub id: u64,
    /// Recorded name.
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

/// Hit recorded for a replay-frame query.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RecQueryHit {
    /// Shape associated with the result.
    pub shape_id: ShapeId,
    /// World-space point for the result.
    pub point: Pos,
    /// World-space normal for the result.
    pub normal: Vec3,
    /// Fraction along the query translation.
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

/// World id created by a replay player.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReplayWorldId {
    /// Native Box3D handle index.
    pub index1: u16,
    /// Handle generation value.
    pub generation: u16,
}

impl ReplayWorldId {
    /// Converts raw Box3D data into the safe value type.
    #[inline]
    pub const fn from_raw(raw: ffi::b3WorldId) -> Self {
        Self {
            index1: raw.index1,
            generation: raw.generation,
        }
    }

    /// Converts this value into the raw Box3D representation.
    #[inline]
    pub const fn into_raw(self) -> ffi::b3WorldId {
        ffi::b3WorldId {
            index1: self.index1,
            generation: self.generation,
        }
    }

    /// Returns whether the Box3D handle is still valid.
    pub fn is_valid(self) -> bool {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3World_IsValid(self.into_raw()) }
    }
}

/// Player used to replay and inspect a Box3D recording.
#[derive(Debug)]
pub struct RecPlayer {
    raw: NonNull<ffi::b3RecPlayer>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl RecPlayer {
    /// Creates a replay player from raw recording bytes.
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

    /// Returns the world id owned by the replay player.
    pub fn world_id(&self) -> ReplayWorldId {
        let _guard = box3d_lock::lock();
        ReplayWorldId::from_raw(unsafe { ffi::b3RecPlayer_GetWorldId(self.raw.as_ptr()) })
    }

    /// Steps the replay by one recorded frame.
    pub fn step_frame(&mut self) -> Result<bool> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        Ok(unsafe { ffi::b3RecPlayer_StepFrame(self.raw.as_ptr()) })
    }

    /// Restarts replay from the first recorded frame.
    pub fn restart(&mut self) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_Restart(self.raw.as_ptr()) };
        Ok(())
    }

    /// Seeks replay to a recorded frame.
    pub fn seek_frame(&mut self, target_frame: i32) -> Result<()> {
        if target_frame < 0 {
            return Err(Error::InvalidArgument);
        }
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_SeekFrame(self.raw.as_ptr(), target_frame) };
        Ok(())
    }

    /// Returns the current replay frame index.
    pub fn frame(&self) -> i32 {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetFrame(self.raw.as_ptr()) }
    }

    /// Returns the number of recorded frames.
    pub fn frame_count(&self) -> i32 {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetFrameCount(self.raw.as_ptr()) }
    }

    /// Returns whether replay has reached the end of the recording.
    pub fn is_at_end(&self) -> bool {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_IsAtEnd(self.raw.as_ptr()) }
    }

    /// Returns whether replay has diverged from the recorded state.
    pub fn has_diverged(&self) -> bool {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_HasDiverged(self.raw.as_ptr()) }
    }

    /// Returns metadata for the loaded recording.
    pub fn info(&self) -> RecPlayerInfo {
        let _guard = box3d_lock::lock();
        RecPlayerInfo::from_raw(unsafe { ffi::b3RecPlayer_GetInfo(self.raw.as_ptr()) })
    }

    /// Returns the first frame where replay diverged, if any.
    pub fn diverge_frame(&self) -> Option<i32> {
        let _guard = box3d_lock::lock();
        let frame = unsafe { ffi::b3RecPlayer_GetDivergeFrame(self.raw.as_ptr()) };
        (frame >= 0).then_some(frame)
    }

    /// Sets the worker count.
    pub fn set_worker_count(&mut self, count: i32) -> Result<()> {
        validate_replay_worker_count(count)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_SetWorkerCount(self.raw.as_ptr(), count) };
        Ok(())
    }

    /// Sets the keyframe policy.
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

    /// Returns the configured replay keyframe memory budget.
    pub fn keyframe_budget(&self) -> usize {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetKeyframeBudget(self.raw.as_ptr()) }
    }

    /// Returns the minimum frame interval between replay keyframes.
    pub fn keyframe_min_interval(&self) -> i32 {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetKeyframeMinInterval(self.raw.as_ptr()) }
    }

    /// Returns the current frame interval between replay keyframes.
    pub fn keyframe_interval(&self) -> i32 {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetKeyframeInterval(self.raw.as_ptr()) }
    }

    /// Returns the bytes currently used by replay keyframes.
    pub fn keyframe_bytes(&self) -> usize {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetKeyframeBytes(self.raw.as_ptr()) }
    }

    /// Returns the number of bodies in the replay world.
    pub fn body_count(&self) -> i32 {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetBodyCount(self.raw.as_ptr()) }
    }

    /// Returns a body id from the replay world by index.
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

    /// Returns the number of queries captured for the current replay frame.
    pub fn frame_query_count(&self) -> i32 {
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3RecPlayer_GetFrameQueryCount(self.raw.as_ptr()) }
    }

    /// Returns metadata for a captured query in the current replay frame.
    pub fn frame_query(&self, index: i32) -> Result<RecQueryInfo> {
        if index < 0 || index >= self.frame_query_count() {
            return Err(Error::IndexOutOfRange);
        }
        let _guard = box3d_lock::lock();
        RecQueryInfo::from_raw(unsafe { ffi::b3RecPlayer_GetFrameQuery(self.raw.as_ptr(), index) })
    }

    /// Returns a hit recorded for a captured frame query.
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

    /// Collects debug draw commands for recorded frame queries.
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

    /// Collects debug draw commands for recorded frame queries into `out`.
    pub fn draw_frame_queries_collect_into(
        &mut self,
        out: &mut Vec<DebugDrawCommand>,
        options: DebugDrawOptions,
        query_index: Option<i32>,
        selected_index: Option<i32>,
    ) -> Result<()> {
        out.clear();
        let mut collector = CollectDebugDraw::new(out);
        self.draw_frame_queries(&mut collector, options, query_index, selected_index)?;
        collector.finish();
        Ok(())
    }

    /// Draws recorded frame queries with a custom debug draw sink.
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
    /// Tries to begin recording world mutations into the provided buffer.
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

    /// Starts recording world mutations or panics if Box3D rejects the request.
    pub fn start_recording(&mut self, recording: &mut Recording) {
        self.try_start_recording(recording)
            .expect("Box3D failed to start recording");
    }

    /// Tries to stop recording world mutations into the provided buffer.
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

    /// Stops recording world mutations or panics if Box3D rejects the request.
    pub fn stop_recording(&mut self, recording: &mut Recording) {
        self.try_stop_recording(recording)
            .expect("Box3D failed to stop recording");
    }
}

/// Validates that raw recording bytes can replay with the requested worker count.
pub fn validate_replay_bytes(bytes: &[u8], worker_count: i32) -> Result<bool> {
    validate_replay_input(bytes, worker_count)?;
    callback_state::check_not_in_callback()?;
    let _guard = box3d_lock::lock();
    Ok(unsafe { ffi::b3ValidateReplay(bytes.as_ptr().cast(), bytes.len() as i32, worker_count) })
}

fn validate_replay_input(bytes: &[u8], worker_count: i32) -> Result<()> {
    validate_replay_worker_count(worker_count)?;
    if bytes.is_empty() || bytes.len() > i32::MAX as usize {
        Err(Error::InvalidArgument)
    } else {
        Ok(())
    }
}

fn validate_replay_worker_count(worker_count: i32) -> Result<()> {
    if worker_count < 1 {
        return Err(Error::InvalidArgument);
    }
    #[cfg(target_arch = "wasm32")]
    if worker_count > 1 {
        return Err(Error::UnsupportedOnWasm);
    }
    Ok(())
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
