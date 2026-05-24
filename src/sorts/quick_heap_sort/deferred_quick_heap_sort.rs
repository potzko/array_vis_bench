//! Deferred quick heap sort.
//!
//! Same shape as [`super::quick_heap_sort::QuickHeapSort`] — partition by
//! head-swapping a max-forward heap on the left and a min-reverse heap on
//! the right — but instead of finishing every small subarray with a small
//! sort, recursion stops once `arr.len() <= DSS::THRESHOLD`. After all
//! recursion unwinds, a single insertion-sort pass cleans up every
//! partially-sorted region in O(n + k·threshold) time.
//!
//! Carries the same `left_built` / `right_built` recursion-rebuild
//! optimization as the classic variant.

use std::marker::PhantomData;

use crate::sorts::heap_sort::arity::Arity;
use crate::sorts::heap_sort::arity_heap::ArityHeap;
use crate::sorts::heap_sort::deep_heapify::Iterative;
use crate::sorts::heap_sort::direction::{MaxForward, MinReverse};
use crate::sorts::heap_sort::heap_algorithm::HeapAlgorithm;
use crate::sorts::heap_sort::heap_sort::HeapSort;
use crate::traits::log_traits::SortLogger;
use crate::utils::small_sort::DeferredSmallSort;

type LeftHeap<A> = HeapSort<ArityHeap<A, MaxForward>, Iterative>;
type RightHeap<A> = HeapSort<ArityHeap<A, MinReverse>, Iterative>;

pub struct DeferredQuickHeapSort<A: Arity, DSS: DeferredSmallSort> {
    _phantom: PhantomData<(A, DSS)>,
}

impl<A: Arity, DSS: DeferredSmallSort> DeferredQuickHeapSort<A, DSS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        deferred_recurse::<T, U, A, DSS>(arr, false, false, logger);
        DSS::final_pass(arr, logger);
    }
}

fn deferred_recurse<
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    A: Arity,
    DSS: DeferredSmallSort,
>(
    arr: &mut [T],
    left_built: bool,
    right_built: bool,
    logger: &mut U,
) {
    if arr.len() < 2 {
        return;
    }
    if arr.len() <= DSS::THRESHOLD {
        return;
    }
    let mid = arr.len() / 2;
    let left_len = mid;
    let right_len = arr.len() - mid;

    let mut left_state = <LeftHeap<A> as HeapAlgorithm>::new_state::<T, U>(left_len, logger);
    let mut right_state = <RightHeap<A> as HeapAlgorithm>::new_state::<T, U>(right_len, logger);

    if !left_built {
        <LeftHeap<A> as HeapAlgorithm>::build(&mut arr[..mid], &mut left_state, logger);
    }
    if !right_built {
        <RightHeap<A> as HeapAlgorithm>::build(&mut arr[mid..], &mut right_state, logger);
    }

    let left_root = <LeftHeap<A> as HeapAlgorithm>::root_phys(left_len);
    let right_root = mid + <RightHeap<A> as HeapAlgorithm>::root_phys(right_len);

    while logger.cond_swap_gt(arr, left_root, right_root) {
        <LeftHeap<A> as HeapAlgorithm>::push_down(
            &mut arr[..mid],
            &mut left_state,
            left_len,
            logger,
        );
        <RightHeap<A> as HeapAlgorithm>::push_down(
            &mut arr[mid..],
            &mut right_state,
            right_len,
            logger,
        );
    }

    deferred_recurse::<T, U, A, DSS>(&mut arr[..mid], true, false, logger);
    deferred_recurse::<T, U, A, DSS>(&mut arr[mid..], false, true, logger);
}

combo_codegen::family!(
    type = DeferredQuickHeapSort<{A}, {DSS}>,
    uses = [
        "crate::sorts::heap_sort::arity::{Binary, Ternary, Base16, Base256}",
        "crate::utils::small_sort::DeferredInsertion",
        "crate::utils::small_sort::{LinearInsertion, BinaryInsertion}",
        "crate::sorts::quick_heap_sort::deferred_quick_heap_sort::DeferredQuickHeapSort",
    ],
    A: Arity,
    DSS: DeferredSmallSort,
    name = "quick heap sort deferred",
    big_o = inherited,
    stable = false,
    direct_sort = true,
    path = ["quick heap sorts", "deferred", "{A}", "{DSS}"],
);

// DeferredQuickHeapSort delegates to HeapSort<ArityHeap<_, _>, _> for
// the heap-extraction phase; total complexity is still O(N log N).
// Small-sort fan-out runs on bounded leaves, so its contribution is
// constant.
impl<A, DSS> crate::traits::composable::HasTimeBounds for DeferredQuickHeapSort<A, DSS>
where
    A: crate::sorts::heap_sort::arity::Arity,
    DSS: crate::utils::small_sort::DeferredSmallSort,
{
    const WORST: crate::traits::complexity::Complexity = crate::traits::complexity::Complexity::N_LOG_N;
    const BEST: crate::traits::complexity::Complexity = crate::traits::complexity::Complexity::N_LOG_N;
    const AVERAGE: crate::traits::complexity::Complexity = crate::traits::complexity::Complexity::N_LOG_N;
}
impl<A, DSS> crate::traits::composable::HasSpace for DeferredQuickHeapSort<A, DSS>
where
    A: crate::sorts::heap_sort::arity::Arity,
    DSS: crate::utils::small_sort::DeferredSmallSort,
{
    const SPACE: crate::traits::complexity::Complexity = crate::traits::complexity::Complexity::LOG_N;
}
impl<A, DSS> crate::traits::composable::HasStability for DeferredQuickHeapSort<A, DSS>
where
    A: crate::sorts::heap_sort::arity::Arity,
    DSS: crate::utils::small_sort::DeferredSmallSort,
{
    const STABLE: bool = false;
}
