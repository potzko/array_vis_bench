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
//! restricted to `ArityHeap` — one variant per arity. Build is hardcoded
//! to `Iterative` since the optimization slots into the textbook
//! bottom-up build cleanly.

use std::marker::PhantomData;

use crate::sorts::heap_sort::arity::Arity;
use crate::sorts::heap_sort::arity_heap::ArityHeap;
use crate::sorts::heap_sort::deep_heapify::Iterative;
use crate::sorts::heap_sort::direction::{MaxForward, MinReverse};
use crate::sorts::heap_sort::heap_algorithm::HeapAlgorithm;
use crate::sorts::heap_sort::heap_sort::HeapSort;
use crate::traits::log_traits::SortLogger;
use crate::utils::small_sort::SmallSort;

type LeftHeap<A> = HeapSort<ArityHeap<A, MaxForward>, Iterative>;
type RightHeap<A> = HeapSort<ArityHeap<A, MinReverse>, Iterative>;

pub struct QuickHeapSort<A: Arity, SS: SmallSort> {
    _phantom: PhantomData<(A, SS)>,
}

impl<A: Arity, SS: SmallSort> QuickHeapSort<A, SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        recurse::<T, U, A, SS>(arr, false, false, logger);
    }
}

fn recurse<T: Ord + Copy, U: ?Sized + SortLogger<T>, A: Arity, SS: SmallSort>(
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

    // The outer-left of the left recursion is the upper half of left's
    // max-forward heap → already a max-forward heap. The outer-right of
    // the right recursion is the upper half of right's min-reverse heap
    // → already a min-reverse heap. Both inner halves need a fresh build.
    recurse::<T, U, A, SS>(&mut arr[..mid], true, false, logger);
    recurse::<T, U, A, SS>(&mut arr[mid..], false, true, logger);
}

combo_codegen::sort_family!(
    type = QuickHeapSort<{A}, {SS}>,
    uses = [
        "crate::sorts::heap_sort::arity::{Binary, Ternary, Base16, Base256}",
        "crate::utils::small_sort::{InsertionSmallSort, Network16SmallSort, NetworkSmallSort, NoSmallSort, Size1SmallSort, Size2SmallSort}",
        "crate::utils::small_sort::{LinearInsertion, BinaryInsertion}",
        "crate::sorts::quick_heap_sort::quick_heap_sort::QuickHeapSort",
    ],
    A: Arity,
    SS: SmallSort,
    name = "quick heap sort",
    big_o = "O(N log N)",
    stable = false,
    direct_sort = true,
    path = ["quick heap sorts", "classic", "{A}", "{SS}"],
);
