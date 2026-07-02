use crate::callbacks::{MaterialMixContext, MaterialMixInput};
use boxddd_sys::ffi;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

pub(crate) const MATERIAL_MIX_SLOT_COUNT: usize = 16;

struct MaterialMixSlot {
    in_use: AtomicBool,
    friction: AtomicPtr<MaterialMixContext>,
    restitution: AtomicPtr<MaterialMixContext>,
}

impl MaterialMixSlot {
    const fn new() -> Self {
        Self {
            in_use: AtomicBool::new(false),
            friction: AtomicPtr::new(core::ptr::null_mut()),
            restitution: AtomicPtr::new(core::ptr::null_mut()),
        }
    }
}

static MATERIAL_MIX_SLOTS: [MaterialMixSlot; MATERIAL_MIX_SLOT_COUNT] =
    [const { MaterialMixSlot::new() }; MATERIAL_MIX_SLOT_COUNT];

#[inline]
fn slot_ref(slot: usize) -> &'static MaterialMixSlot {
    &MATERIAL_MIX_SLOTS[slot]
}

pub(crate) fn acquire_slot() -> Option<usize> {
    MATERIAL_MIX_SLOTS
        .iter()
        .enumerate()
        .find_map(|(idx, slot)| {
            slot.in_use
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .ok()
                .map(|_| idx)
        })
}

pub(crate) fn release_slot(slot: usize) {
    debug_assert!(slot_ref(slot).friction.load(Ordering::Acquire).is_null());
    debug_assert!(slot_ref(slot).restitution.load(Ordering::Acquire).is_null());
    slot_ref(slot).in_use.store(false, Ordering::Release);
}

#[inline]
pub(crate) fn set_friction_ptr(slot: usize, ptr: *mut MaterialMixContext) {
    slot_ref(slot).friction.store(ptr, Ordering::Release);
}

#[inline]
pub(crate) fn set_restitution_ptr(slot: usize, ptr: *mut MaterialMixContext) {
    slot_ref(slot).restitution.store(ptr, Ordering::Release);
}

#[inline]
pub(crate) fn has_any_callback(slot: usize) -> bool {
    let slot = slot_ref(slot);
    !slot.friction.load(Ordering::Acquire).is_null()
        || !slot.restitution.load(Ordering::Acquire).is_null()
}

#[inline]
fn default_friction_mix(friction_a: f32, friction_b: f32) -> f32 {
    (friction_a * friction_b).sqrt()
}

#[inline]
fn default_restitution_mix(restitution_a: f32, restitution_b: f32) -> f32 {
    restitution_a.max(restitution_b)
}

unsafe fn invoke_mix_callback(
    ctx_ptr: *mut MaterialMixContext,
    value_a: f32,
    user_material_id_a: u64,
    value_b: f32,
    user_material_id_b: u64,
    default_mix: fn(f32, f32) -> f32,
) -> f32 {
    if ctx_ptr.is_null() {
        return default_mix(value_a, value_b);
    }

    let ctx = unsafe { &*ctx_ptr };
    if ctx.panicked.load(Ordering::Relaxed) {
        return default_mix(value_a, value_b);
    }

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = crate::core::callback_state::CallbackGuard::enter();
        (ctx.callback)(
            MaterialMixInput::new(value_a, user_material_id_a),
            MaterialMixInput::new(value_b, user_material_id_b),
        )
    })) {
        Ok(value) if value.is_finite() => value,
        Ok(_) => default_mix(value_a, value_b),
        Err(_) => {
            ctx.panicked.store(true, Ordering::SeqCst);
            default_mix(value_a, value_b)
        }
    }
}

unsafe fn invoke_friction_callback(
    slot: usize,
    friction_a: f32,
    user_material_id_a: u64,
    friction_b: f32,
    user_material_id_b: u64,
) -> f32 {
    let ctx_ptr = slot_ref(slot).friction.load(Ordering::Acquire);
    unsafe {
        invoke_mix_callback(
            ctx_ptr,
            friction_a,
            user_material_id_a,
            friction_b,
            user_material_id_b,
            default_friction_mix,
        )
    }
}

unsafe fn invoke_restitution_callback(
    slot: usize,
    restitution_a: f32,
    user_material_id_a: u64,
    restitution_b: f32,
    user_material_id_b: u64,
) -> f32 {
    let ctx_ptr = slot_ref(slot).restitution.load(Ordering::Acquire);
    unsafe {
        invoke_mix_callback(
            ctx_ptr,
            restitution_a,
            user_material_id_a,
            restitution_b,
            user_material_id_b,
            default_restitution_mix,
        )
    }
}

type MaterialMixTrampoline = unsafe extern "C" fn(f32, u64, f32, u64) -> f32;

unsafe extern "C" fn friction_trampoline<const SLOT: usize>(
    friction_a: f32,
    user_material_id_a: u64,
    friction_b: f32,
    user_material_id_b: u64,
) -> f32 {
    unsafe {
        invoke_friction_callback(
            SLOT,
            friction_a,
            user_material_id_a,
            friction_b,
            user_material_id_b,
        )
    }
}

unsafe extern "C" fn restitution_trampoline<const SLOT: usize>(
    restitution_a: f32,
    user_material_id_a: u64,
    restitution_b: f32,
    user_material_id_b: u64,
) -> f32 {
    unsafe {
        invoke_restitution_callback(
            SLOT,
            restitution_a,
            user_material_id_a,
            restitution_b,
            user_material_id_b,
        )
    }
}

static FRICTION_TRAMPOLINES: [MaterialMixTrampoline; MATERIAL_MIX_SLOT_COUNT] = [
    friction_trampoline::<0>,
    friction_trampoline::<1>,
    friction_trampoline::<2>,
    friction_trampoline::<3>,
    friction_trampoline::<4>,
    friction_trampoline::<5>,
    friction_trampoline::<6>,
    friction_trampoline::<7>,
    friction_trampoline::<8>,
    friction_trampoline::<9>,
    friction_trampoline::<10>,
    friction_trampoline::<11>,
    friction_trampoline::<12>,
    friction_trampoline::<13>,
    friction_trampoline::<14>,
    friction_trampoline::<15>,
];

static RESTITUTION_TRAMPOLINES: [MaterialMixTrampoline; MATERIAL_MIX_SLOT_COUNT] = [
    restitution_trampoline::<0>,
    restitution_trampoline::<1>,
    restitution_trampoline::<2>,
    restitution_trampoline::<3>,
    restitution_trampoline::<4>,
    restitution_trampoline::<5>,
    restitution_trampoline::<6>,
    restitution_trampoline::<7>,
    restitution_trampoline::<8>,
    restitution_trampoline::<9>,
    restitution_trampoline::<10>,
    restitution_trampoline::<11>,
    restitution_trampoline::<12>,
    restitution_trampoline::<13>,
    restitution_trampoline::<14>,
    restitution_trampoline::<15>,
];

#[inline]
pub(crate) fn friction_callback(slot: usize) -> ffi::b3FrictionCallback {
    Some(FRICTION_TRAMPOLINES[slot])
}

#[inline]
pub(crate) fn restitution_callback(slot: usize) -> ffi::b3RestitutionCallback {
    Some(RESTITUTION_TRAMPOLINES[slot])
}
