//! Generic n-ary heap sort + the family marker.
//!
//! Two types live here:
//!
//! - [`NaryHeapSort<H, DH>`] — the stateless-heap adapter. Layout / compare
//!   / arity-agnostic (all of that lives in `H: Heap`) and
//!   deep-heapify-strategy-agnostic via `DH`. It implements
//!   [`HeapAlgorithm`] by supplying the build / swap-root / push-down
//!   primitives; the build-then-extract loop is the trait default. Used by
//!   the d-ary family (`ArityHeap`) and the beap family (`BeapHeap`).
//! - [`HeapSort<HA>`] — the family marker. A zero-cost pass-through generic
//!   over *any* [`HeapAlgorithm`] (including the stateful `WeakHeapSort`),
//!   so every heap-family sort routes through one canonical type. It only
//!   exposes `sort`; monomorphization collapses the indirection away.
//!
//! The `family!` calls in the crate `Cargo.toml`s cross-product the
//! supporting axes to register every ascending-producing variant.
//! `HeapDirection` only has component! markers on `MinReverse` and
//! `MaxForward`, so the other two directions (which would sort to
//! descending) are excluded.

use std::marker::PhantomData;

use super::deep_heapify::DeepHeapify;
use super::heap::Heap;
use super::heap_algorithm::HeapAlgorithm;
use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use sort_logger::SortLogger;

pub struct NaryHeapSort<H: Heap, DH: DeepHeapify> {
    _phantom: PhantomData<(H, DH)>,
}

impl<H: Heap, DH: DeepHeapify> HeapAlgorithm for NaryHeapSort<H, DH> {
    type State = ();

    #[inline(always)]
    fn new_state<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_n: usize, _logger: &mut U) {}

    #[inline(always)]
    fn root_phys(n: usize) -> usize {
        H::phys(0, n)
    }

    #[inline(always)]
    fn build<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        _state: &mut (),
        logger: &mut U,
    ) {
        DH::deep_heapify::<H, T, U>(arr, logger);
    }

    #[inline(always)]
    fn swap_root_to_end<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        _state: &mut (),
        heap_size: usize,
        logger: &mut U,
    ) {
        H::swap(arr, 0, heap_size - 1, logger);
    }

    #[inline(always)]
    fn push_down<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        _state: &mut (),
        heap_size: usize,
        logger: &mut U,
    ) {
        H::heapify(arr, heap_size, 0, logger);
    }
}

impl<H: Heap, DH: DeepHeapify> NaryHeapSort<H, DH> {
    /// Inherent thin delegate so `<NaryHeapSort<...>>::sort(arr, logger)`
    /// keeps working from `family!`-generated code without needing the
    /// `HeapAlgorithm` trait in scope at the call site.
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        <Self as HeapAlgorithm>::sort(arr, logger)
    }
}

/// Family marker: a zero-cost pass-through over any [`HeapAlgorithm`].
///
/// Every heap-family sort (d-ary `NaryHeapSort<ArityHeap<…>, DH>`, beap
/// `NaryHeapSort<BeapHeap<…>, DH>`, and the stateful `WeakHeapSort<D, R>`)
/// is registered as `HeapSort<…>` so they share one canonical type. The
/// wrapper holds no data and forwards `sort` straight to the inner
/// algorithm; monomorphization erases it entirely.
pub struct HeapSort<HA: HeapAlgorithm> {
    _phantom: PhantomData<HA>,
}

impl<HA: HeapAlgorithm> HeapSort<HA> {
    #[inline(always)]
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        HA::sort(arr, logger)
    }
}

// Forward the composable annotations from the inner algorithm so the
// marker carries the same complexity / stability facts.
impl<HA: HeapAlgorithm + HasTimeBounds> HasTimeBounds for HeapSort<HA> {
    const WORST: Complexity = HA::WORST;
    const BEST: Complexity = HA::BEST;
    const AVERAGE: Complexity = HA::AVERAGE;
}

impl<HA: HeapAlgorithm + HasSpace> HasSpace for HeapSort<HA> {
    const SPACE: Complexity = HA::SPACE;
}

impl<HA: HeapAlgorithm + HasStability> HasStability for HeapSort<HA> {
    const STABLE: bool = HA::STABLE;
}

// Family declaration is now `[[package.metadata.array_vis_bench.families]]`
// in `heap_sort_lib/Cargo.toml` — picked up by the build script's
// dep-graph TOML scanner.

// ── Composable annotations ──────────────────────────────────────────
//
// HeapSort = N extractions × per-extraction heapify cost. Each Heap
// impl declares its own heapify complexity (binary heap → log N,
// beap → √N), and the outer composition multiplies by N. The
// DeepHeapify strategy contributes `O(N)` build cost, which is
// dominated by the extraction phase.
//
// Stability is uniformly false for all heap variants (sift-down
// reorders equal keys). Space is `O(log N)` recursion stack worst
// case (recursive deep-heapify); the iterative variant is `O(1)`.

impl<H: Heap + HasTimeBounds, DH: DeepHeapify> HasTimeBounds for NaryHeapSort<H, DH> {
    /// N extractions × per-operation heapify complexity (H::WORST).
    const WORST: Complexity = Complexity::product(Complexity::N1, H::WORST);
    const BEST: Complexity = Complexity::product(Complexity::N1, H::BEST);
    const AVERAGE: Complexity = Complexity::product(Complexity::N1, H::AVERAGE);
}

impl<H: Heap, DH: DeepHeapify> HasSpace for NaryHeapSort<H, DH> {
    /// Conservative bound — Recursive deep-heapify uses O(log N) stack;
    /// Iterative uses O(1).
    const SPACE: Complexity = Complexity::LOG_N;
}

impl<H: Heap, DH: DeepHeapify> HasStability for NaryHeapSort<H, DH> {
    const STABLE: bool = false;
}
