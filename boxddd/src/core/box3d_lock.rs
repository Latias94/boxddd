use std::sync::{Mutex, MutexGuard, OnceLock};

static BOX3D_GLOBAL_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub(crate) fn lock<'a>() -> MutexGuard<'a, ()> {
    BOX3D_GLOBAL_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("Box3D global lock poisoned")
}
