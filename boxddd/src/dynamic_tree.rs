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
/// Stable handle for a proxy stored in a [`DynamicTree`].
///
/// The generation component prevents accidentally reusing a stale handle after a proxy has been
/// destroyed and another proxy has reused the same Box3D proxy index.
pub struct DynamicTreeProxyId {
    index: i32,
    generation: u64,
}

impl DynamicTreeProxyId {
    /// Returns the Box3D proxy index portion of this handle.
    #[inline]
    pub const fn index(self) -> i32 {
        self.index
    }

    /// Returns the generation paired with the native proxy index.
    ///
    /// A mismatched generation means the handle refers to a proxy that has already been removed
    /// or whose index has been recycled by Box3D.
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
/// Snapshot of a proxy tracked by a [`DynamicTree`].
pub struct DynamicTreeProxy {
    /// Current axis-aligned bounds for the proxy.
    pub aabb: Aabb,
    /// Category bits used by query and cast filters.
    pub category_bits: u64,
    /// Caller-owned payload returned by query and cast callbacks.
    pub user_data: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Category filter used by dynamic tree queries and casts.
pub struct DynamicTreeFilter {
    /// Category bit mask to test against candidate proxies.
    pub mask_bits: u64,
    /// When true, every bit in `mask_bits` must be present on the proxy category.
    pub require_all_bits: bool,
}

impl DynamicTreeFilter {
    /// Creates a filter that accepts proxies sharing any bit in `mask_bits`.
    #[inline]
    pub const fn new(mask_bits: u64) -> Self {
        Self {
            mask_bits,
            require_all_bits: false,
        }
    }

    /// Sets whether matching requires all mask bits instead of any shared bit.
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
/// Result item produced by an AABB query.
pub struct DynamicTreeHit {
    /// Proxy that matched the query.
    pub proxy_id: DynamicTreeProxyId,
    /// Caller-owned payload stored on the proxy.
    pub user_data: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
/// Candidate passed to a closest-point query callback.
pub struct DynamicTreeClosestHit {
    /// Current squared distance bound reported by Box3D for this candidate.
    pub min_distance_squared: f32,
    /// Proxy being visited.
    pub proxy_id: DynamicTreeProxyId,
    /// Caller-owned payload stored on the proxy.
    pub user_data: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
/// Summary returned by a closest-point query.
pub struct DynamicTreeClosestResult {
    /// Traversal statistics reported by Box3D.
    pub stats: TreeStats,
    /// Final squared distance bound after all callback updates.
    pub min_distance_squared: f32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
/// Candidate passed to a dynamic-tree ray-cast callback.
pub struct DynamicTreeRayCastHit {
    /// Ray input clipped to the current candidate interval.
    pub input: RayCastInput,
    /// Proxy being visited.
    pub proxy_id: DynamicTreeProxyId,
    /// Caller-owned payload stored on the proxy.
    pub user_data: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
/// Candidate passed to a dynamic-tree box-cast callback.
pub struct DynamicTreeBoxCastHit {
    /// Box-cast input clipped to the current candidate interval.
    pub input: BoxCastInput,
    /// Proxy being visited.
    pub proxy_id: DynamicTreeProxyId,
    /// Caller-owned payload stored on the proxy.
    pub user_data: u64,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
/// Callback control value for dynamic-tree ray and box casts.
pub enum DynamicTreeCastControl {
    /// Continue traversal without changing the current maximum fraction.
    Continue,
    /// Clip future traversal to the supplied fraction.
    Clip(f32),
    /// Skip the current hit while continuing traversal.
    Skip,
    /// Stop traversal immediately.
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

/// Standalone Box3D dynamic AABB tree.
///
/// `DynamicTree` owns the native tree and releases it on drop. It is intentionally neither `Send`
/// nor `Sync` because callbacks enter Rust through raw context pointers and Box3D access is
/// serialized by the crate-wide Box3D lock.
pub struct DynamicTree {
    raw: ffi::b3DynamicTree,
    proxies: BTreeMap<i32, ProxyEntry>,
    generations: BTreeMap<i32, u64>,
    has_enlarged_nodes: bool,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl DynamicTree {
    /// Creates an empty dynamic tree.
    pub fn new() -> Result<Self> {
        Self::with_capacity(0)
    }

    /// Creates an empty dynamic tree with an initial proxy capacity hint.
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

    /// Inserts a proxy with default category bits and caller-owned `user_data`.
    pub fn create_proxy(&mut self, aabb: Aabb, user_data: u64) -> Result<DynamicTreeProxyId> {
        self.create_proxy_with_category_bits(aabb, u64::MAX, user_data)
    }

    /// Inserts a proxy with explicit category bits and caller-owned `user_data`.
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

    /// Removes a proxy from the tree.
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

    /// Moves an existing proxy to a new AABB.
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

    /// Enlarges an existing proxy AABB without rebuilding the tree.
    ///
    /// The new AABB must strictly contain the current one; call [`Self::rebuild`] before
    /// [`Self::validate_no_enlarged`] if enlarged nodes should be eliminated.
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

    /// Replaces the category bits for a proxy.
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

    /// Returns the category bits currently stored by Box3D for a proxy.
    pub fn category_bits(&mut self, proxy_id: DynamicTreeProxyId) -> Result<u64> {
        callback_state::check_not_in_callback()?;
        let proxy_index = self.proxy_index(proxy_id)?;
        let _guard = box3d_lock::lock();
        Ok(unsafe { ffi::b3DynamicTree_GetCategoryBits(&mut self.raw, proxy_index) })
    }

    /// Returns the Rust-side snapshot for a live proxy.
    pub fn proxy(&self, proxy_id: DynamicTreeProxyId) -> Result<DynamicTreeProxy> {
        callback_state::check_not_in_callback()?;
        Ok(self.proxy_entry(proxy_id)?.proxy)
    }

    /// Returns true when `proxy_id` still refers to a live proxy in this tree.
    pub fn contains_proxy(&self, proxy_id: DynamicTreeProxyId) -> bool {
        self.proxy_entry(proxy_id).is_ok()
    }

    /// Returns the number of live proxies stored in the native tree.
    pub fn proxy_count(&self) -> Result<usize> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        let count = unsafe { ffi::b3DynamicTree_GetProxyCount(&self.raw) };
        usize::try_from(count).map_err(|_| Error::InvalidArgument)
    }

    /// Returns the native heap memory currently owned by the tree, in bytes.
    pub fn byte_count(&self) -> Result<usize> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        let count = unsafe { ffi::b3DynamicTree_GetByteCount(&self.raw) };
        usize::try_from(count).map_err(|_| Error::InvalidArgument)
    }

    /// Returns the current height of the native AABB tree.
    pub fn height(&self) -> Result<i32> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        Ok(unsafe { ffi::b3DynamicTree_GetHeight(&self.raw) })
    }

    /// Returns the tree area ratio reported by Box3D.
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

    /// Returns the root AABB, or `None` when the tree has no proxies.
    pub fn root_bounds(&self) -> Result<Option<Aabb>> {
        callback_state::check_not_in_callback()?;
        if self.proxies.is_empty() {
            return Ok(None);
        }
        let _guard = box3d_lock::lock();
        let aabb = Aabb::from_raw(unsafe { ffi::b3DynamicTree_GetRootBounds(&self.raw) });
        Ok(Some(aabb.validate()?))
    }

    /// Rebuilds the native tree and returns the number of boxes sorted by Box3D.
    pub fn rebuild(&mut self, full_build: bool) -> Result<usize> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        let count = unsafe { ffi::b3DynamicTree_Rebuild(&mut self.raw, full_build) };
        self.has_enlarged_nodes = false;
        usize::try_from(count).map_err(|_| Error::InvalidArgument)
    }

    /// Runs Box3D's internal dynamic-tree validation checks.
    pub fn validate(&self) -> Result<()> {
        callback_state::check_not_in_callback()?;
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3DynamicTree_Validate(&self.raw) };
        Ok(())
    }

    /// Runs Box3D validation that asserts no enlarged nodes remain.
    ///
    /// This returns [`Error::InvalidArgument`] if [`Self::enlarge_proxy`] has been called and the
    /// tree has not subsequently been rebuilt.
    pub fn validate_no_enlarged(&self) -> Result<()> {
        callback_state::check_not_in_callback()?;
        if self.has_enlarged_nodes {
            return Err(Error::InvalidArgument);
        }
        let _guard = box3d_lock::lock();
        unsafe { ffi::b3DynamicTree_ValidateNoEnlarged(&self.raw) };
        Ok(())
    }

    /// Collects all proxies whose bounds overlap `aabb` and pass `filter`.
    pub fn query(&self, aabb: Aabb, filter: DynamicTreeFilter) -> Result<Vec<DynamicTreeHit>> {
        let mut out = Vec::new();
        self.query_into(aabb, filter, &mut out)?;
        Ok(out)
    }

    /// Writes all AABB query hits into `out`, clearing it first.
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

    /// Visits proxies whose bounds overlap `aabb` and pass `filter`.
    ///
    /// Returning `false` from `visitor` stops traversal early.
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

    /// Visits closest-query candidates near `point`.
    ///
    /// The callback returns the next squared distance bound. Non-finite or negative callback
    /// results are ignored and leave the existing bound unchanged.
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

    /// Alias for [`Self::visit_query_closest`].
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

    /// Visits proxies intersected by a ray cast through the tree.
    ///
    /// The callback controls traversal by returning [`DynamicTreeCastControl`].
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

    /// Alias for [`Self::visit_ray_cast`].
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

    /// Visits proxies intersected by a swept AABB cast through the tree.
    ///
    /// The callback controls traversal by returning [`DynamicTreeCastControl`].
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

    /// Alias for [`Self::visit_box_cast`].
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
