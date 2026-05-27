//! Quick heap sort.
//!
//! Hybrid quicksort/heapsort that needs no pivot selection. Split at the
//! midpoint, build a **max heap on the left** with its head at the array's
//! left edge (`MaxForward`) and a **min heap on the right** with its head
//! at the array's right edge (`MinReverse`). While `arr[0] > arr[n-1]`
//! (left's max greater than right's min) swap the two heads and push each
//! down. Convergence ⇒ `left_max ≤ right_min` ⇒ every left value ≤ every
//! right value. Recurse on each half.
//!
//! ## Recursion-rebuild optimization
//!
//! With heads at the *outer* edges, the upper half of each side's heap
//! sits at the side of the array where the next recursive level expects
//! the same heap kind. Concretely:
//!
//! - Recursing on the left (max-forward) half: its left sub-half is the
//!   upper half of the parent max-forward heap, so it's *itself* a
//!   max-forward heap — the recursive level can skip its left build.
//! - Recursing on the right (min-reverse) half: its right sub-half is
//!   the upper half of the parent min-reverse heap, so it's *itself* a
//!   min-reverse heap — skip the right build there.
//!
//! Each recursive call carries `left_built` / `right_built` flags
//! describing which sub-half came in pre-heaped.
//!
//! ## Limited to n-ary heaps
//!
//! Weak heap doesn't fit this optimization (the per-node `reverse` bit
//! state can't be sliced across recursive boundaries), so the family is
//! restricted to `ArityHeap` — one variant per arity. The DeepHeapify
//! axis is plumbed through so each recursive call's heap build can use
//! any DH strategy (textbook `Iterative` and the quickselect-based
//! `quick_deep_heapify` variants).

use std::marker::PhantomData;

use heap_sort_lib::arity::Arity;
use heap_sort_lib::arity_heap::ArityHeap;
use heap_sort_lib::deep_heapify::DeepHeapify;
use heap_sort_lib::direction::{MaxForward, MinReverse};
use heap_sort_lib::heap_algorithm::HeapAlgorithm;
use heap_sort_lib::heap_sort::NaryHeapSort;
use sort_logger::SortLogger;
use array_vis_bench_traits::SmallSort;

type LeftHeap<A, DH> = NaryHeapSort<ArityHeap<A, MaxForward>, DH>;
type RightHeap<A, DH> = NaryHeapSort<ArityHeap<A, MinReverse>, DH>;

pub struct QuickHeapSort<A: Arity, DH: DeepHeapify, SS: SmallSort> {
    _phantom: PhantomData<(A, DH, SS)>,
}

impl<A: Arity, DH: DeepHeapify, SS: SmallSort> QuickHeapSort<A, DH, SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        recurse::<T, U, A, DH, SS>(arr, false, false, logger);
    }
}

fn recurse<T: Ord + Copy, U: ?Sized + SortLogger<T>, A: Arity, DH: DeepHeapify, SS: SmallSort>(
    arr: &mut [T],
    left_built: bool,
    right_built: bool,
    logger: &mut U,
) {
    if SS::THRESHOLD > 0 && arr.len() <= SS::THRESHOLD {
        SS::sort(arr, logger);
        return;
    }
    if arr.len() < 2 {
        return;
    }
    let mid = arr.len() / 2;
    let left_len = mid;
    let right_len = arr.len() - mid;

    let mut left_state = <LeftHeap<A, DH> as HeapAlgorithm>::new_state::<T, U>(left_len, logger);
    let mut right_state = <RightHeap<A, DH> as HeapAlgorithm>::new_state::<T, U>(right_len, logger);

    if !left_built {
        <LeftHeap<A, DH> as HeapAlgorithm>::build(&mut arr[..mid], &mut left_state, logger);
    }
    if !right_built {
        <RightHeap<A, DH> as HeapAlgorithm>::build(&mut arr[mid..], &mut right_state, logger);
    }

    let left_root = <LeftHeap<A, DH> as HeapAlgorithm>::root_phys(left_len);
    let right_root = mid + <RightHeap<A, DH> as HeapAlgorithm>::root_phys(right_len);

    while logger.cond_swap_gt(arr, left_root, right_root) {
        <LeftHeap<A, DH> as HeapAlgorithm>::push_down(
            &mut arr[..mid],
            &mut left_state,
            left_len,
            logger,
        );
        <RightHeap<A, DH> as HeapAlgorithm>::push_down(
            &mut arr[mid..],
            &mut right_state,
            right_len,
            logger,
        );
    }

    // The outer-left of the left recursion is the upper half of left's
    // max-forward heap → already a max-forward heap. The outer-right of
    // the right recursion is the upper half of right's min-reverse heap
    // → already a min-reverse heap. Both inner halves need a fresh build.
    recurse::<T, U, A, DH, SS>(&mut arr[..mid], true, false, logger);
    recurse::<T, U, A, DH, SS>(&mut arr[mid..], false, true, logger);
}

// Classic family: build hardcoded to `Iterative` (textbook bottom-up
// sift-down). Preserves the original `quick heap sort` lineup.
// Quick-build family: DH is one of the quickselect-based build strategies
// from `quick_deep_heapify`, parametrised over `HeapPartition` and
// `PivotSelector`. SS and V are intentionally restricted here to keep the
// menu navigable — the cross-product is already 4 × 3 × 3 × 4 × 3 = 432.
// ── Composable annotations ──────────────────────────────────────────
//
// QuickHeapSort builds a heap via quickselect then extracts. Both the
// build (O(N)) and the extraction (O(N log N) for binary arities) put
// the total at O(N log N) — same as plain heap sort. The arity choice
// only changes constants. The deep-heapify and small-sort axes are
// likewise constant-factor; the small-sort runs on bounded-size leaves.

impl<A, DH, SS> array_vis_bench_traits::composable::HasTimeBounds for QuickHeapSort<A, DH, SS>
where
    A: heap_sort_lib::arity::Arity,
    DH: heap_sort_lib::deep_heapify::DeepHeapify,
    SS: array_vis_bench_traits::SmallSort,
{
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N_LOG_N;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N_LOG_N;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::N_LOG_N;
}
impl<A, DH, SS> array_vis_bench_traits::composable::HasSpace for QuickHeapSort<A, DH, SS>
where
    A: heap_sort_lib::arity::Arity,
    DH: heap_sort_lib::deep_heapify::DeepHeapify,
    SS: array_vis_bench_traits::SmallSort,
{
    const SPACE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::LOG_N;
}
impl<A, DH, SS> array_vis_bench_traits::composable::HasStability for QuickHeapSort<A, DH, SS>
where
    A: heap_sort_lib::arity::Arity,
    DH: heap_sort_lib::deep_heapify::DeepHeapify,
    SS: array_vis_bench_traits::SmallSort,
{
    const STABLE: bool = false;
}
