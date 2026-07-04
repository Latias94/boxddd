use crate::core::callback_state::CallbackGuard;
use crate::error::Error;
use boxddd_sys::ffi;
#[cfg(not(target_arch = "wasm32"))]
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
#[cfg(not(target_arch = "wasm32"))]
use std::thread::{self, JoinHandle};

/// Safe task-system adapter installed on a Box3D world definition.
///
/// Box3D calls `enqueueTask` during `b3World_Step` and later calls `finishTask`
/// for every non-null task handle returned by enqueue. `finishTask` must block
/// until that task has completed; schedulers that cannot provide this blocking
/// guarantee must not be installed through this adapter.
#[derive(Clone, Debug)]
pub struct TaskSystem {
    inner: Arc<TaskSystemInner>,
}

impl TaskSystem {
    /// Runs each Box3D task on a dedicated blocking operating-system thread.
    ///
    /// This scheduler is intentionally conservative: it contains panics,
    /// returns them as [`Error::CallbackPanicked`] from `World::try_step`, and
    /// joins every task in the corresponding Box3D `finishTask` callback.
    #[cfg(not(target_arch = "wasm32"))]
    #[inline]
    pub fn blocking_threads() -> Self {
        Self::new(FaultMode::None)
    }

    /// Tries to create the blocking-thread scheduler.
    ///
    /// Browser and WASI targets do not expose this scheduler because Box3D requires
    /// `finishTask` to block until child tasks complete.
    #[inline]
    pub fn try_blocking_threads() -> crate::Result<Self> {
        #[cfg(target_arch = "wasm32")]
        {
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(Self::blocking_threads())
        }
    }

    /// Returns counters for diagnostics and tests.
    #[inline]
    pub fn stats(&self) -> TaskSystemStats {
        self.inner.stats()
    }

    #[doc(hidden)]
    #[inline]
    pub fn __guard_rejections_for_test(&self) -> usize {
        self.inner.guard_rejections.load(Ordering::Relaxed)
    }

    #[inline]
    pub(crate) fn raw_context(&self) -> *mut c_void {
        Arc::as_ptr(&self.inner) as *mut c_void
    }

    #[inline]
    pub(crate) fn reset_panics(&self) {
        self.inner.panicked.store(false, Ordering::Release);
    }

    #[inline]
    pub(crate) fn panicked(&self) -> bool {
        self.inner.panicked.load(Ordering::Acquire)
    }

    #[inline]
    fn new(fault_mode: FaultMode) -> Self {
        Self {
            inner: Arc::new(TaskSystemInner::new(fault_mode)),
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn __panic_on_enqueue_for_test() -> Self {
        Self::new(FaultMode::PanicOnEnqueue)
    }

    #[doc(hidden)]
    #[inline]
    pub fn __panic_on_task_for_test() -> Self {
        Self::new(FaultMode::PanicOnTask)
    }

    #[doc(hidden)]
    #[inline]
    pub fn __panic_on_finish_for_test() -> Self {
        Self::new(FaultMode::PanicOnFinish)
    }

    #[doc(hidden)]
    #[inline]
    pub fn __check_callback_guard_for_test() -> Self {
        Self::new(FaultMode::CheckCallbackGuard)
    }
}

/// Snapshot of a [`TaskSystem`]'s task counters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TaskSystemStats {
    /// Number of tasks enqueued.
    pub enqueued: usize,
    /// Number of tasks started.
    pub started: usize,
    /// Number of tasks completed.
    pub completed: usize,
    /// Number of `finishTask` callbacks observed.
    pub finished: usize,
    /// Number of task callbacks that panicked.
    pub panicked: bool,
}

#[derive(Debug)]
struct TaskSystemInner {
    fault_mode: FaultMode,
    panicked: AtomicBool,
    enqueued: AtomicUsize,
    started: AtomicUsize,
    completed: AtomicUsize,
    finished: AtomicUsize,
    guard_rejections: AtomicUsize,
}

impl TaskSystemInner {
    fn new(fault_mode: FaultMode) -> Self {
        Self {
            fault_mode,
            panicked: AtomicBool::new(false),
            enqueued: AtomicUsize::new(0),
            started: AtomicUsize::new(0),
            completed: AtomicUsize::new(0),
            finished: AtomicUsize::new(0),
            guard_rejections: AtomicUsize::new(0),
        }
    }

    fn stats(&self) -> TaskSystemStats {
        TaskSystemStats {
            enqueued: self.enqueued.load(Ordering::Relaxed),
            started: self.started.load(Ordering::Relaxed),
            completed: self.completed.load(Ordering::Relaxed),
            finished: self.finished.load(Ordering::Relaxed),
            panicked: self.panicked.load(Ordering::Acquire),
        }
    }

    fn mark_panicked(&self) {
        self.panicked.store(true, Ordering::Release);
    }

    fn run_task(&self, invocation: TaskInvocation) {
        self.started.fetch_add(1, Ordering::Relaxed);
        let result = catch_unwind(AssertUnwindSafe(|| {
            let _guard = CallbackGuard::enter();
            if self.fault_mode == FaultMode::CheckCallbackGuard
                && matches!(
                    crate::core::callback_state::check_not_in_callback(),
                    Err(Error::InCallback)
                )
            {
                self.guard_rejections.fetch_add(1, Ordering::Relaxed);
            }
            if let Some(task) = invocation.task {
                unsafe { task(invocation.task_context) };
            }
            if self.fault_mode == FaultMode::PanicOnTask {
                panic!("injected Box3D task panic");
            }
        }));
        if result.is_err() {
            self.mark_panicked();
        }
        self.completed.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FaultMode {
    #[cfg(not(target_arch = "wasm32"))]
    None,
    PanicOnEnqueue,
    PanicOnTask,
    PanicOnFinish,
    CheckCallbackGuard,
}

#[derive(Clone, Copy)]
struct TaskInvocation {
    task: ffi::b3TaskCallback,
    task_context: *mut c_void,
}

unsafe impl Send for TaskInvocation {}

#[cfg(not(target_arch = "wasm32"))]
struct TaskHandle {
    join: Option<JoinHandle<()>>,
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) unsafe extern "C" fn enqueue_task(
    task: ffi::b3TaskCallback,
    task_context: *mut c_void,
    user_context: *mut c_void,
    task_name: *const c_char,
) -> *mut c_void {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let scheduler = unsafe { scheduler_from_context(user_context) };
        scheduler.enqueued.fetch_add(1, Ordering::Relaxed);
        if task.is_none() {
            return std::ptr::null_mut();
        }

        let invocation = TaskInvocation { task, task_context };
        if scheduler.fault_mode == FaultMode::PanicOnEnqueue {
            scheduler.run_task(invocation);
            panic!("injected Box3D enqueue panic");
        }

        let scheduler_for_task = unsafe { clone_scheduler(scheduler as *const TaskSystemInner) };
        let thread_name = task_thread_name(task_name);
        match thread::Builder::new()
            .name(thread_name)
            .spawn(move || scheduler_for_task.run_task(invocation))
        {
            Ok(join) => Box::into_raw(Box::new(TaskHandle { join: Some(join) })).cast(),
            Err(_) => {
                scheduler.run_task(invocation);
                std::ptr::null_mut()
            }
        }
    }));

    match result {
        Ok(user_task) => user_task,
        Err(_) => {
            if let Some(scheduler) = unsafe { scheduler_from_context_checked(user_context) } {
                scheduler.mark_panicked();
            }
            std::ptr::null_mut()
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) unsafe extern "C" fn enqueue_task(
    task: ffi::b3TaskCallback,
    task_context: *mut c_void,
    user_context: *mut c_void,
    _task_name: *const c_char,
) -> *mut c_void {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let scheduler = unsafe { scheduler_from_context(user_context) };
        scheduler.enqueued.fetch_add(1, Ordering::Relaxed);
        if let Some(task) = task {
            scheduler.run_task(TaskInvocation {
                task: Some(task),
                task_context,
            });
        }
    }));

    if result.is_err() {
        if let Some(scheduler) = unsafe { scheduler_from_context_checked(user_context) } {
            scheduler.mark_panicked();
        }
    }
    std::ptr::null_mut()
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) unsafe extern "C" fn finish_task(user_task: *mut c_void, user_context: *mut c_void) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if user_task.is_null() {
            return;
        }
        let scheduler = unsafe { scheduler_from_context(user_context) };

        let mut handle = unsafe { Box::from_raw(user_task.cast::<TaskHandle>()) };
        if let Some(join) = handle.join.take() {
            if join.join().is_err() {
                scheduler.mark_panicked();
            }
        }
        scheduler.finished.fetch_add(1, Ordering::Relaxed);
        if scheduler.fault_mode == FaultMode::PanicOnFinish {
            panic!("injected Box3D finish panic");
        }
    }));

    if result.is_err() {
        if let Some(scheduler) = unsafe { scheduler_from_context_checked(user_context) } {
            scheduler.mark_panicked();
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) unsafe extern "C" fn finish_task(user_task: *mut c_void, user_context: *mut c_void) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let scheduler = unsafe { scheduler_from_context(user_context) };
        if !user_task.is_null() {
            scheduler.mark_panicked();
        }
        scheduler.finished.fetch_add(1, Ordering::Relaxed);
    }));

    if result.is_err() {
        if let Some(scheduler) = unsafe { scheduler_from_context_checked(user_context) } {
            scheduler.mark_panicked();
        }
    }
}

#[inline]
pub(crate) fn install_callbacks(raw_def: &mut ffi::b3WorldDef, task_system: &TaskSystem) {
    raw_def.enqueueTask = Some(enqueue_task);
    raw_def.finishTask = Some(finish_task);
    raw_def.userTaskContext = task_system.raw_context();
}

unsafe fn scheduler_from_context<'a>(context: *mut c_void) -> &'a TaskSystemInner {
    debug_assert!(!context.is_null());
    unsafe { &*context.cast::<TaskSystemInner>() }
}

unsafe fn scheduler_from_context_checked<'a>(context: *mut c_void) -> Option<&'a TaskSystemInner> {
    if context.is_null() {
        None
    } else {
        Some(unsafe { scheduler_from_context(context) })
    }
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn clone_scheduler(ptr: *const TaskSystemInner) -> Arc<TaskSystemInner> {
    unsafe {
        Arc::increment_strong_count(ptr);
        Arc::from_raw(ptr)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn task_thread_name(task_name: *const c_char) -> String {
    let suffix: String = if task_name.is_null() {
        "unnamed".into()
    } else {
        unsafe { CStr::from_ptr(task_name) }
            .to_string_lossy()
            .chars()
            .map(|ch| if ch == '\0' { '_' } else { ch })
            .take(48)
            .collect()
    };
    format!("boxddd-task-{suffix}")
}
