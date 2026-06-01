//! `HeapExtract` — heap-based partition as a [`PartitionScheme`].
//!
//! Builds a left heap on `arr[..mid]` (max-rooted at low end) and a right
//! heap on `arr[mid..]` (min-rooted at high end) via the chosen
//! [`HeapAlgorithmPair`], then converges the two heads by swap + push-down
//! until `arr[..mid]` ≤ `arr[mid..]`. Announces the two halves as
//! unsorted — the generic
//! [`QuickSort`](https://docs.rs/quick_sort_lib) driver recurses into each.
//!
//! Pivotless ([`N_PIVOTS = 0`](PartitionScheme::N_PIVOTS)) — splits at the
//! midpoint without consulting pivot values. Paired with `NoPivot` for
//! `QuickSort<HeapExtract<P, DH>, NoPivot, SS>`.
//!
//! `P` selects which heap kind backs the build: [`crate::AryPair<A>`] for
//! d-ary heaps (parameterised by [`heap_sort_lib::arity::Arity`]) or
//! [`crate::BeapPair`] for bi-parental heaps. Adding more heap families is
//! one more `impl HeapAlgorithmPair` away.
//!
//! Shares the build-and-converge routine with `QuickHeapSort` via
//! [`crate::heap_partition_core::build_and_converge`]. The difference
//! between the two is purely the recursion: `QuickHeapSort` reuses the
//! outer pre-heaped halves across recursive levels (the no-rebuild
//! optimization), while the generic QuickSort driver hands `HeapExtract`
//! a fresh slice each time, so it rebuilds both heaps every call.

use std::marker::PhantomData;

use array_vis_bench_traits::{Complexity, PartitionScheme, PartitionVisitor};
use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use heap_sort_lib::deep_heapify::DeepHeapify;
use sort_logger::SortLogger;

use crate::heap_pair::HeapAlgorithmPair;
use crate::heap_partition_core::build_and_converge;

/// Heap-based partition: build two heaps from opposite ends (via the
/// chosen [`HeapAlgorithmPair`]), converge their heads, emit the two
/// halves as unsorted regions.
pub struct HeapExtract<P, DH>
where
    P: HeapAlgorithmPair<DH>,
    DH: DeepHeapify,
{
    _phantom: PhantomData<(P, DH)>,
}

impl<P, DH> PartitionScheme for HeapExtract<P, DH>
where
    P: HeapAlgorithmPair<DH>,
    DH: DeepHeapify,
{
    const NAME: &'static str = "heap extract";
    const N_PIVOTS: usize = 0;

    fn partition<T, U, V>(
        arr: &mut [T],
        logger: &mut U,
        _pivots: &[usize],
        _scratch: &mut [usize],
        visitor: &mut V,
    )
    where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        V: PartitionVisitor,
    {
        let n = arr.len();
        if n < 2 {
            return;
        }
        let mid = n / 2;
        build_and_converge::<T, U, P, DH>(arr, mid, false, false, logger);
        visitor.unsorted(0..mid);
        visitor.unsorted(mid..n);
    }
}

// ── Composable annotations ──────────────────────────────────────────
//
// One call = O(n) build (across both halves) + O(n log n) convergence
// in the worst case (push-down per swap, bounded by the heap size).
// Space is O(1) — both heaps are in-place.

impl<P, DH> HasTimeBounds for HeapExtract<P, DH>
where
    P: HeapAlgorithmPair<DH>,
    DH: DeepHeapify,
{
    const WORST: Complexity = Complexity::N_LOG_N;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}
impl<P, DH> HasSpace for HeapExtract<P, DH>
where
    P: HeapAlgorithmPair<DH>,
    DH: DeepHeapify,
{
    const SPACE: Complexity = Complexity::CONST;
}
impl<P, DH> HasStability for HeapExtract<P, DH>
where
    P: HeapAlgorithmPair<DH>,
    DH: DeepHeapify,
{
    const STABLE: bool = false;
}
