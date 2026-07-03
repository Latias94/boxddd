#![cfg_attr(all(target_arch = "wasm32", boxddd_wasm_provider), allow(dead_code))]

use crate::collision::{BoxCastInput, RayCastInput};
use crate::core::{box3d_lock, callback_state};
use crate::error::{Error, Result};
use crate::query::TreeStats;
use crate::types::{Aabb, Vec3};
use boxddd_sys::ffi;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DynamicTreeProxyId {
    index: i32,
    generation: u64,
}

impl DynamicTreeProxyId {
    #[inline]
    pub const fn index(self) -> i32 {
        self.index
    }

    #[inline]
    pub const fn generation(self) -> u64 {
        self.generation
    }

    #[inline]
    pub(crate) const fn from_raw_parts(index: i32, generation: u64) -> Self {
        Self { index, generation }
    }

    #[inline]
    const fn into_raw(self) -> i32 {
        self.index
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DynamicTreeProxy {
    pub aabb: Aabb,
    pub category_bits: u64,
    pub user_data: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DynamicTreeFilter {
    pub mask_bits: u64,
    pub require_all_bits: bool,
}

impl DynamicTreeFilter {
    #[inline]
    pub const fn new(mask_bits: u64) -> Self {
        Self {
            mask_bits,
            require_all_bits: false,
        }
    }

    #[inline]
    pub const fn require_all_bits(mut self, require_all_bits: bool) -> Self {
        self.require_all_bits = require_all_bits;
        self
    }
}

impl Default for DynamicTreeFilter {
    fn default() -> Self {
        Self {
            mask_bits: u64::MAX,
            require_all_bits: false,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DynamicTreeHit {
    pub proxy_id: DynamicTreeProxyId,
    pub user_data: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DynamicTreeClosestHit {
    pub min_distance_squared: f32,
    pub proxy_id: DynamicTreeProxyId,
    pub user_data: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DynamicTreeClosestResult {
    pub stats: TreeStats,
    pub min_distance_squared: f32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DynamicTreeRayCastHit {
    pub input: RayCastInput,
    pub proxy_id: DynamicTreeProxyId,
    pub user_data: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DynamicTreeBoxCastHit {
    pub input: BoxCastInput,
    pub proxy_id: DynamicTreeProxyId,
    pub user_data: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DynamicTreeCastControl {
    Continue,
    Clip(f32),
    Skip,
    Terminate,
}

impl DynamicTreeCastControl {
    fn into_raw(self, max_fraction: f32) -> Option<f32> {
        match self {
            Self::Continue => Some(max_fraction),
            Self::Clip(fraction)
                if fraction.is_finite() && fraction >= 0.0 && fraction <= max_fraction =>
            {
                Some(fraction)
            }
            Self::Clip(_) => None,
            Self::Skip => Some(-1.0),
            Self::Terminate => Some(0.0),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct ProxyEntry {
    generation: u64,
    proxy: DynamicTreeProxy,
}

pub struct DynamicTree {
    raw: ffi::b3DynamicTree,
    proxies: BTreeMap<i32, ProxyEntry>,
    generations: BTreeMap<i32, u64>,
    has_enlarged_nodes: bool,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl DynamicTree {
    pub fn new() -> Result<Self> {
        Self::with_capacity(0)
    }

    pub fn with_capacity(proxy_capacity: usize) -> Result<Self> {
        if proxy_capacity > i32::MAX as usize / 2 {
            return Err(Error::InvalidArgument);
        }
        let proxy_capacity = i32::try_from(proxy_capacity).map_err(|_| Error::InvalidArgument)?;
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        let raw = unsafe { ffi::b3DynamicTree_Create(proxy_capacity) };
        if raw.nodes.is_null() {
            return Err(Error::CreateDynamicTreeFailed);
        }
        Ok(Self {
            raw,
            proxies: BTreeMap::new(),
            generations: BTreeMap::new(),
            has_enlarged_nodes: false,
            _not_send_sync: PhantomData,
        })
    }

    pub fn create_proxy(&mut self, aabb: Aabb, user_data: u64) -> Result<DynamicTreeProxyId> {
        self.create_proxy_with_category_bits(aabb, u64::MAX, user_data)
    }

    pub fn create_proxy_with_category_bits(
        &mut self,
        aabb: Aabb,
        category_bits: u64,
        user_data: u64,
    ) -> Result<DynamicTreeProxyId> {
        callback_state::check_not_in_callback()?;
        let aabb = aabb.validate()?;
        let _guard = box3d_lock::lock();
        let proxy_id = unsafe {
            ffi::b3DynamicTree_CreateProxy(&mut self.raw, aabb.into_raw(), category_bits, user_data)
        };
        if proxy_id < 0 {
            return Err(Error::InvalidArgument);
        }
        let generation = self.next_generation(proxy_id);
        self.proxies.insert(
            proxy_id,
            ProxyEntry {
                generation,
                proxy: DynamicTreeProxy {
                    aabb,
                    category_bits,
                    user_data,
                },
            },
        );
        Ok(DynamicTreeProxyId::from_raw_parts(proxy_id, generation))
    }

    pub fn destroy_proxy(&mut self, proxy_id: DynamicTreeProxyId) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let proxy_index = self.proxy_index(proxy_id)?;
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3DynamicTree_DestroyProxy(&mut self.raw, proxy_index) };
        self.proxies.remove(&proxy_index);
        if self.proxies.is_empty() {
            self.has_enlarged_nodes = false;
        }
        Ok(())
    }

    pub fn move_proxy(&mut self, proxy_id: DynamicTreeProxyId, aabb: Aabb) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let proxy_index = self.proxy_index(proxy_id)?;
        let aabb = aabb.validate()?;
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3DynamicTree_MoveProxy(&mut self.raw, proxy_index, aabb.into_raw()) };
        self.proxies
            .get_mut(&proxy_index)
            .expect("proxy index validated")
            .proxy
            .aabb = aabb;
        Ok(())
    }

    pub fn enlarge_proxy(&mut self, proxy_id: DynamicTreeProxyId, aabb: Aabb) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let proxy_index = self.proxy_index(proxy_id)?;
        let aabb = aabb.validate()?;
        let current = self
            .proxies
            .get(&proxy_index)
            .expect("proxy index validated")
            .proxy
            .aabb;
        if !aabb_contains(aabb, current) || aabb_contains(current, aabb) {
            return Err(Error::InvalidArgument);
        }
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3DynamicTree_EnlargeProxy(&mut self.raw, proxy_index, aabb.into_raw()) };
        self.proxies
            .get_mut(&proxy_index)
            .expect("proxy index validated")
            .proxy
            .aabb = aabb;
        self.has_enlarged_nodes = true;
        Ok(())
    }

    pub fn set_category_bits(
        &mut self,
        proxy_id: DynamicTreeProxyId,
        category_bits: u64,
    ) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let proxy_index = self.proxy_index(proxy_id)?;
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3DynamicTree_SetCategoryBits(&mut self.raw, proxy_index, category_bits) };
        self.proxies
            .get_mut(&proxy_index)
            .expect("proxy index validated")
            .proxy
            .category_bits = category_bits;
        Ok(())
    }

    pub fn category_bits(&mut self, proxy_id: DynamicTreeProxyId) -> Result<u64> {
        callback_state::check_not_in_callback()?;
        let proxy_index = self.proxy_index(proxy_id)?;
        let _guard = box3d_lock::lock();
        Ok(unsafe { ffi::b3DynamicTree_GetCategoryBits(&mut self.raw, proxy_index) })
    }

    pub fn proxy(&self, proxy_id: DynamicTreeProxyId) -> Result<DynamicTreeProxy> {
        callback_state::check_not_in_callback()?;
        Ok(self.proxy_entry(proxy_id)?.proxy)
    }

    pub fn contains_proxy(&self, proxy_id: DynamicTreeProxyId) -> bool {
        self.proxy_entry(proxy_id).is_ok()
    }

    pub fn proxy_count(&self) -> Result<usize> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        let count = unsafe { ffi::b3DynamicTree_GetProxyCount(&self.raw) };
        usize::try_from(count).map_err(|_| Error::InvalidArgument)
    }

    pub fn byte_count(&self) -> Result<usize> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        let count = unsafe { ffi::b3DynamicTree_GetByteCount(&self.raw) };
        usize::try_from(count).map_err(|_| Error::InvalidArgument)
    }

    pub fn height(&self) -> Result<i32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        Ok(unsafe { ffi::b3DynamicTree_GetHeight(&self.raw) })
    }

    pub fn area_ratio(&self) -> Result<f32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        let ratio = unsafe { ffi::b3DynamicTree_GetAreaRatio(&self.raw) };
        if ratio.is_finite() {
            Ok(ratio)
        } else {
            Err(Error::InvalidArgument)
        }
    }

    pub fn root_bounds(&self) -> Result<Option<Aabb>> {
        callback_state::check_not_in_callback()?;
        if self.proxies.is_empty() {
            return Ok(None);
        }
        let _guard = box3d_lock::lock();
        let aabb = Aabb::from_raw(unsafe { ffi::b3DynamicTree_GetRootBounds(&self.raw) });
        Ok(Some(aabb.validate()?))
    }

    pub fn rebuild(&mut self, full_build: bool) -> Result<usize> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        let count = unsafe { ffi::b3DynamicTree_Rebuild(&mut self.raw, full_build) };
        self.has_enlarged_nodes = false;
        usize::try_from(count).map_err(|_| Error::InvalidArgument)
    }

    pub fn validate(&self) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3DynamicTree_Validate(&self.raw) };
        Ok(())
    }

    pub fn validate_no_enlarged(&self) -> Result<()> {
        callback_state::check_not_in_callback()?;
        if self.has_enlarged_nodes {
            return Err(Error::InvalidArgument);
        }
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3DynamicTree_ValidateNoEnlarged(&self.raw) };
        Ok(())
    }

    pub fn query(&self, aabb: Aabb, filter: DynamicTreeFilter) -> Result<Vec<DynamicTreeHit>> {
        let mut out = Vec::new();
        self.query_into(aabb, filter, &mut out)?;
        Ok(out)
    }

    pub fn query_into(
        &self,
        aabb: Aabb,
        filter: DynamicTreeFilter,
        out: &mut Vec<DynamicTreeHit>,
    ) -> Result<TreeStats> {
        out.clear();
        self.visit_query(aabb, filter, |hit| {
            out.push(hit);
            true
        })
    }

    pub fn visit_query<F>(
        &self,
        aabb: Aabb,
        filter: DynamicTreeFilter,
        visitor: F,
    ) -> Result<TreeStats>
    where
        F: FnMut(DynamicTreeHit) -> bool,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = (aabb, filter, visitor);
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let aabb = aabb.validate()?;
            let mut ctx = QueryContext {
                visitor,
                proxies: &self.proxies as *const BTreeMap<i32, ProxyEntry>,
                panicked: false,
            };
            let _guard = box3d_lock::lock();
            let stats = unsafe {
                ffi::b3DynamicTree_Query(
                    &self.raw,
                    aabb.into_raw(),
                    filter.mask_bits,
                    filter.require_all_bits,
                    Some(query_trampoline::<F>),
                    (&mut ctx as *mut QueryContext<_>).cast(),
                )
            };
            if ctx.panicked {
                Err(Error::CallbackPanicked)
            } else {
                Ok(TreeStats::from_raw(stats))
            }
        }
    }

    pub fn visit_query_closest<F>(
        &self,
        point: impl Into<Vec3>,
        filter: DynamicTreeFilter,
        min_distance_squared: f32,
        visitor: F,
    ) -> Result<DynamicTreeClosestResult>
    where
        F: FnMut(DynamicTreeClosestHit) -> f32,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = (point, filter, min_distance_squared, visitor);
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let point = point.into().validate()?;
            if !min_distance_squared.is_finite() || min_distance_squared < 0.0 {
                return Err(Error::InvalidArgument);
            }
            let mut ctx = ClosestContext {
                visitor,
                proxies: &self.proxies as *const BTreeMap<i32, ProxyEntry>,
                panicked: false,
            };
            let mut min_distance_squared = min_distance_squared;
            let _guard = box3d_lock::lock();
            let stats = unsafe {
                ffi::b3DynamicTree_QueryClosest(
                    &self.raw,
                    point.into_raw(),
                    filter.mask_bits,
                    filter.require_all_bits,
                    Some(closest_trampoline::<F>),
                    (&mut ctx as *mut ClosestContext<_>).cast(),
                    &mut min_distance_squared,
                )
            };
            if ctx.panicked {
                Err(Error::CallbackPanicked)
            } else {
                Ok(DynamicTreeClosestResult {
                    stats: TreeStats::from_raw(stats),
                    min_distance_squared,
                })
            }
        }
    }

    pub fn query_closest<F>(
        &self,
        point: impl Into<Vec3>,
        filter: DynamicTreeFilter,
        min_distance_squared: f32,
        visitor: F,
    ) -> Result<DynamicTreeClosestResult>
    where
        F: FnMut(DynamicTreeClosestHit) -> f32,
    {
        self.visit_query_closest(point, filter, min_distance_squared, visitor)
    }

    pub fn visit_ray_cast<F>(
        &self,
        input: RayCastInput,
        filter: DynamicTreeFilter,
        visitor: F,
    ) -> Result<TreeStats>
    where
        F: FnMut(DynamicTreeRayCastHit) -> DynamicTreeCastControl,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = (input, filter, visitor);
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let input = input.validate()?;
            let raw_input = input.raw();
            if !unsafe { ffi::b3IsValidRay(&raw_input) } {
                return Err(Error::InvalidArgument);
            }
            let mut ctx = RayCastContext {
                visitor,
                proxies: &self.proxies as *const BTreeMap<i32, ProxyEntry>,
                panicked: false,
                invalid_input: false,
            };
            let _guard = box3d_lock::lock();
            let stats = unsafe {
                ffi::b3DynamicTree_RayCast(
                    &self.raw,
                    &raw_input,
                    filter.mask_bits,
                    filter.require_all_bits,
                    Some(ray_cast_trampoline::<F>),
                    (&mut ctx as *mut RayCastContext<_>).cast(),
                )
            };
            if ctx.panicked {
                Err(Error::CallbackPanicked)
            } else if ctx.invalid_input {
                Err(Error::InvalidArgument)
            } else {
                Ok(TreeStats::from_raw(stats))
            }
        }
    }

    pub fn ray_cast<F>(
        &self,
        input: RayCastInput,
        filter: DynamicTreeFilter,
        visitor: F,
    ) -> Result<TreeStats>
    where
        F: FnMut(DynamicTreeRayCastHit) -> DynamicTreeCastControl,
    {
        self.visit_ray_cast(input, filter, visitor)
    }

    pub fn visit_box_cast<F>(
        &self,
        input: BoxCastInput,
        filter: DynamicTreeFilter,
        visitor: F,
    ) -> Result<TreeStats>
    where
        F: FnMut(DynamicTreeBoxCastHit) -> DynamicTreeCastControl,
    {
        callback_state::check_not_in_callback()?;
        #[cfg(all(target_arch = "wasm32", boxddd_wasm_provider))]
        {
            let _ = (input, filter, visitor);
            Err(Error::UnsupportedOnWasm)
        }
        #[cfg(not(all(target_arch = "wasm32", boxddd_wasm_provider)))]
        {
            let raw_input = input.validate()?.raw();
            let mut ctx = BoxCastContext {
                visitor,
                proxies: &self.proxies as *const BTreeMap<i32, ProxyEntry>,
                panicked: false,
                invalid_input: false,
            };
            let _guard = box3d_lock::lock();
            let stats = unsafe {
                ffi::b3DynamicTree_BoxCast(
                    &self.raw,
                    &raw_input,
                    filter.mask_bits,
                    filter.require_all_bits,
                    Some(box_cast_trampoline::<F>),
                    (&mut ctx as *mut BoxCastContext<_>).cast(),
                )
            };
            if ctx.panicked {
                Err(Error::CallbackPanicked)
            } else if ctx.invalid_input {
                Err(Error::InvalidArgument)
            } else {
                Ok(TreeStats::from_raw(stats))
            }
        }
    }

    pub fn box_cast<F>(
        &self,
        input: BoxCastInput,
        filter: DynamicTreeFilter,
        visitor: F,
    ) -> Result<TreeStats>
    where
        F: FnMut(DynamicTreeBoxCastHit) -> DynamicTreeCastControl,
    {
        self.visit_box_cast(input, filter, visitor)
    }

    fn proxy_index(&self, proxy_id: DynamicTreeProxyId) -> Result<i32> {
        let index = proxy_id.into_raw();
        self.proxy_entry(proxy_id)?;
        Ok(index)
    }

    fn proxy_entry(&self, proxy_id: DynamicTreeProxyId) -> Result<&ProxyEntry> {
        let index = proxy_id.into_raw();
        let Some(entry) = self.proxies.get(&index) else {
            return Err(Error::InvalidArgument);
        };
        if index >= 0 && proxy_id.generation() != 0 && proxy_id.generation() == entry.generation {
            Ok(entry)
        } else {
            Err(Error::InvalidArgument)
        }
    }

    fn next_generation(&mut self, index: i32) -> u64 {
        let generation = self
            .generations
            .get(&index)
            .copied()
            .unwrap_or(0)
            .wrapping_add(1);
        let generation = if generation == 0 { 1 } else { generation };
        self.generations.insert(index, generation);
        generation
    }
}

impl Drop for DynamicTree {
    fn drop(&mut self) {
        if self.raw.nodes.is_null() {
            return;
        }
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3DynamicTree_Destroy(&mut self.raw) };
    }
}

struct QueryContext<F> {
    visitor: F,
    proxies: *const BTreeMap<i32, ProxyEntry>,
    panicked: bool,
}

unsafe extern "C" fn query_trampoline<F>(
    proxy_id: i32,
    user_data: u64,
    context: *mut std::ffi::c_void,
) -> bool
where
    F: FnMut(DynamicTreeHit) -> bool,
{
    let _guard = callback_state::CallbackGuard::enter();
    let ctx = unsafe { &mut *context.cast::<QueryContext<F>>() };
    let hit = DynamicTreeHit {
        proxy_id: proxy_id_from_context(proxy_id, ctx.proxies),
        user_data,
    };
    match catch_unwind(AssertUnwindSafe(|| (ctx.visitor)(hit))) {
        Ok(keep_going) => keep_going,
        Err(_) => {
            ctx.panicked = true;
            false
        }
    }
}

struct ClosestContext<F> {
    visitor: F,
    proxies: *const BTreeMap<i32, ProxyEntry>,
    panicked: bool,
}

unsafe extern "C" fn closest_trampoline<F>(
    min_distance_squared: f32,
    proxy_id: i32,
    user_data: u64,
    context: *mut std::ffi::c_void,
) -> f32
where
    F: FnMut(DynamicTreeClosestHit) -> f32,
{
    let _guard = callback_state::CallbackGuard::enter();
    let ctx = unsafe { &mut *context.cast::<ClosestContext<F>>() };
    let hit = DynamicTreeClosestHit {
        min_distance_squared,
        proxy_id: proxy_id_from_context(proxy_id, ctx.proxies),
        user_data,
    };
    match catch_unwind(AssertUnwindSafe(|| (ctx.visitor)(hit))) {
        Ok(next_min) if next_min.is_finite() && next_min >= 0.0 => next_min,
        Ok(_) => min_distance_squared,
        Err(_) => {
            ctx.panicked = true;
            min_distance_squared
        }
    }
}

struct RayCastContext<F> {
    visitor: F,
    proxies: *const BTreeMap<i32, ProxyEntry>,
    panicked: bool,
    invalid_input: bool,
}

unsafe extern "C" fn ray_cast_trampoline<F>(
    input: *const ffi::b3RayCastInput,
    proxy_id: i32,
    user_data: u64,
    context: *mut std::ffi::c_void,
) -> f32
where
    F: FnMut(DynamicTreeRayCastHit) -> DynamicTreeCastControl,
{
    let _guard = callback_state::CallbackGuard::enter();
    let ctx = unsafe { &mut *context.cast::<RayCastContext<F>>() };
    if input.is_null() {
        ctx.invalid_input = true;
        return 0.0;
    }
    let input = unsafe { *input };
    let Ok(input) = RayCastInput::with_max_fraction(
        Vec3::from_raw(input.origin),
        Vec3::from_raw(input.translation),
        input.maxFraction,
    ) else {
        ctx.invalid_input = true;
        return 0.0;
    };
    let hit = DynamicTreeRayCastHit {
        input,
        proxy_id: proxy_id_from_context(proxy_id, ctx.proxies),
        user_data,
    };
    match catch_unwind(AssertUnwindSafe(|| (ctx.visitor)(hit))) {
        Ok(control) => match control.into_raw(input.max_fraction) {
            Some(next_fraction) => next_fraction,
            None => {
                ctx.invalid_input = true;
                0.0
            }
        },
        Err(_) => {
            ctx.panicked = true;
            0.0
        }
    }
}

struct BoxCastContext<F> {
    visitor: F,
    proxies: *const BTreeMap<i32, ProxyEntry>,
    panicked: bool,
    invalid_input: bool,
}

unsafe extern "C" fn box_cast_trampoline<F>(
    input: *const ffi::b3BoxCastInput,
    proxy_id: i32,
    user_data: u64,
    context: *mut std::ffi::c_void,
) -> f32
where
    F: FnMut(DynamicTreeBoxCastHit) -> DynamicTreeCastControl,
{
    let _guard = callback_state::CallbackGuard::enter();
    let ctx = unsafe { &mut *context.cast::<BoxCastContext<F>>() };
    if input.is_null() {
        ctx.invalid_input = true;
        return 0.0;
    }
    let input = unsafe { *input };
    let Ok(input) = BoxCastInput::with_max_fraction(
        Aabb::from_raw(input.box_),
        Vec3::from_raw(input.translation),
        input.maxFraction,
    ) else {
        ctx.invalid_input = true;
        return 0.0;
    };
    let hit = DynamicTreeBoxCastHit {
        input,
        proxy_id: proxy_id_from_context(proxy_id, ctx.proxies),
        user_data,
    };
    match catch_unwind(AssertUnwindSafe(|| (ctx.visitor)(hit))) {
        Ok(control) => match control.into_raw(input.max_fraction) {
            Some(next_fraction) => next_fraction,
            None => {
                ctx.invalid_input = true;
                0.0
            }
        },
        Err(_) => {
            ctx.panicked = true;
            0.0
        }
    }
}

fn proxy_id_from_context(
    proxy_id: i32,
    proxies: *const BTreeMap<i32, ProxyEntry>,
) -> DynamicTreeProxyId {
    let generation = unsafe { proxies.as_ref() }
        .and_then(|proxies| proxies.get(&proxy_id))
        .map(|entry| entry.generation)
        .unwrap_or(0);
    DynamicTreeProxyId::from_raw_parts(proxy_id, generation)
}

fn aabb_contains(outer: Aabb, inner: Aabb) -> bool {
    outer.lower_bound.x <= inner.lower_bound.x
        && outer.lower_bound.y <= inner.lower_bound.y
        && outer.lower_bound.z <= inner.lower_bound.z
        && inner.upper_bound.x <= outer.upper_bound.x
        && inner.upper_bound.y <= outer.upper_bound.y
        && inner.upper_bound.z <= outer.upper_bound.z
}
