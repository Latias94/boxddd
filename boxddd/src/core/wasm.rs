#[inline]
pub(crate) const fn is_provider_mode() -> bool {
    cfg!(all(target_arch = "wasm32", boxddd_wasm_provider))
}
