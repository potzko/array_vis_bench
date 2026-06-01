//! Shared "build two heaps + converge heads" routine.
//!
//! Both [`crate::quick_heap_sort::QuickHeapSort`] (which keeps the
//! recursion-rebuild optimization, reusing pre-heaped outer halves across
//! recursive levels) and [`crate::heap_extract::HeapExtract`] (the plain
//! [`PartitionScheme`] for the generic QuickSort driver) call this one
//! routine. Splitting it out keeps the dual-heap convergence logic in a
//! single place, so any change to the heap-partition algorithm flows
//! through both call sites.
//!
//! The routine is "pivotless" — split is at the midpoint. After return,
//! every value in `arr[..mid]` is ≤ every value in `arr[mid..]`.
//!
//! Generic over [`HeapAlgorithmPair`] so the caller picks the underlying
//! heap kind (d-ary via [`crate::AryPair`], bi-parental via
//! [`crate::BeapPair`], …). `QuickHeapSort` always hands in `AryPair<A>`;
//! `HeapExtract` exposes the choice through its `P` type parameter.

use heap_sort_lib::deep_heapify::DeepHeapify;
use heap_sort_lib::heap_algorithm::HeapAlgorithm;
use sort_logger::SortLogger;

use crate::heap_pair::HeapAlgorithmPair;

/// Build the `P::Left` heap on `arr[..mid]` and the `P::Right` heap on
/// `arr[mid..]` (skipping either build if the caller knows that side is
/// already in heap order), then converge the two heads by swap + push-down
/// until `arr[left_root] ≤ arr[right_root]`. After return, all of
/// `arr[..mid]` is ≤ all of `arr[mid..]`.
///
/// `left_built` / `right_built` carry the quick-heap-sort recursion-rebuild
/// optimization: when a recursive call inherits a half that the parent
/// level already heapified, the build is skipped. [`crate::heap_extract::HeapExtract`]
/// passes `false, false` (no inheritance — every call rebuilds).
///
/// `arr.len()` must be ≥ 2, and `1 ≤ mid < arr.len()`.
#[inline]
pub fn build_and_converge<T, U, P, DH>(
    arr: &mut [T],
    mid: usize,
    left_built: bool,
    right_built: bool,
    logger: &mut U,
) where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    P: HeapAlgorithmPair<DH>,
    DH: DeepHeapify,
{
    let left_len = mid;
    let right_len = arr.len() - mid;

    let mut left_state = <P::Left as HeapAlgorithm>::new_state::<T, U>(left_len, logger);
    let mut right_state = <P::Right as HeapAlgorithm>::new_state::<T, U>(right_len, logger);

    if !left_built {
        <P::Left as HeapAlgorithm>::build(&mut arr[..mid], &mut left_state, logger);
    }
    if !right_built {
        <P::Right as HeapAlgorithm>::build(&mut arr[mid..], &mut right_state, logger);
    }

    let left_root = <P::Left as HeapAlgorithm>::root_phys(left_len);
    let right_root = mid + <P::Right as HeapAlgorithm>::root_phys(right_len);

    while logger.cond_swap_gt(arr, left_root, right_root) {
        <P::Left as HeapAlgorithm>::push_down(
            &mut arr[..mid],
            &mut left_state,
            left_len,
            logger,
        );
        <P::Right as HeapAlgorithm>::push_down(
            &mut arr[mid..],
            &mut right_state,
            right_len,
            logger,
        );
    }
}
