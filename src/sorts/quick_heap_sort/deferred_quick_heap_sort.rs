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
use crate::utils::small_sort::{insertion_sort, DeferredSmallSort};

type LeftHeap<A> = HeapSort<ArityHeap<A, MaxForward>, Iterative>;
type RightHeap<A> = HeapSort<ArityHeap<A, MinReverse>, Iterative>;

pub struct DeferredQuickHeapSort<A: Arity, DSS: DeferredSmallSort> {
    _phantom: PhantomData<(A, DSS)>,
}

impl<A: Arity, DSS: DeferredSmallSort> DeferredQuickHeapSort<A, DSS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        deferred_recurse::<T, U, A, DSS>(arr, false, false, logger);
        insertion_sort(arr, logger);
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

    let mut left_state = <LeftHeap<A> as HeapAlgorithm>::new_state(left_len);
    let mut right_state = <RightHeap<A> as HeapAlgorithm>::new_state(right_len);

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

combo_codegen::sort_family!(
    type = DeferredQuickHeapSort<{A}, {DSS}>,
    uses = [
        "crate::sorts::heap_sort::arity::{Binary, Ternary, Base16, Base256}",
        "crate::utils::small_sort::DeferredInsertion",
        "crate::sorts::quick_heap_sort::deferred_quick_heap_sort::DeferredQuickHeapSort",
    ],
    A: Arity,
    DSS: DeferredSmallSort,
    name = "quick heap sort deferred",
    big_o = "O(N log N)",
    stable = false,
    direct_sort = true,
    path = ["quick heap sorts", "deferred", "{A}", "{DSS}"],
);
