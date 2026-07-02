use std::cell::Cell;

thread_local! {
    static DEPTH: Cell<usize> = const { Cell::new(0) };
}

pub struct CallbackGuard;

impl CallbackGuard {
    pub fn enter() -> Self {
        DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
        Self
    }
}

impl Drop for CallbackGuard {
    fn drop(&mut self) {
        DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

#[inline]
pub(crate) fn in_callback() -> bool {
    DEPTH.with(|depth| depth.get() > 0)
}

#[inline]
#[track_caller]
#[allow(dead_code)]
pub(crate) fn assert_not_in_callback() {
    assert!(
        !in_callback(),
        "boxddd API called from a Box3D callback; call is not allowed because Box3D world is locked"
    );
}

#[inline]
pub(crate) fn check_not_in_callback() -> crate::error::Result<()> {
    if in_callback() {
        Err(crate::error::Error::InCallback)
    } else {
        Ok(())
    }
}
