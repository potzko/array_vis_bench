//! Weak heap sort.
//!
//! A weak heap is a binary tree relaxation where each right subtree is
//! bounded by its root (max-weak-heap: subtree ≤ root) but left subtrees
//! are unconstrained. A per-node "reverse" bit virtually swaps a node's
//! children, letting the build flip subtrees without physical moves —
//! that's how it gets to ~n − 1 comparisons instead of the binary heap's
//! ~1.88 n.
//!
//! The structure abstractions reused from [`super::super::heap_sort`]:
//! [`Compare`] (min vs max ordering) and [`Layout`] (forward vs reverse
//! placement in the array) are paired by [`Direction`]. Only the two
//! ascending-output combinations carry the `HeapDirection` `component!`
//! marker, so `sort_family!` registers two variants. The build/swap/push-
//! down primitives feed the [`HeapAlgorithm`] default `sort`, so weak heap
//! plugs into anything that consumes a `HeapAlgorithm` (e.g. introsort).

use std::marker::PhantomData;

use crate::sorts::heap_sort::compare::Compare;
use crate::sorts::heap_sort::direction::Direction;
use crate::sorts::heap_sort::heap::HeapLayout;
use crate::sorts::heap_sort::heap_algorithm::HeapAlgorithm;
use crate::sorts::heap_sort::layout::Layout;
use crate::traits::log_traits::SortLogger;

pub struct WeakHeapSort<D: Direction> {
    _phantom: PhantomData<D>,
}

// Layout-only `Heap` membership: lets partition-style code that only
// needs `(Compare, phys)` accept a weak heap as "a heap" even though its
// build / sift-down operations are stateful (`Vec<u8>` reverse bits) and
// therefore live on [`HeapAlgorithm`] instead of [`super::super::heap_sort::heap::Heap`].
impl<D: Direction> HeapLayout for WeakHeapSort<D> {
    type Compare = D::Compare;

    #[inline(always)]
    fn phys(i: usize, n: usize) -> usize {
        <D::Layout as Layout>::phys(i, n)
    }
}

impl<D: Direction> HeapAlgorithm for WeakHeapSort<D> {
    /// Per-node reverse bits — the heart of the weak heap representation.
    /// Stored as `Vec<u8>` (one byte per node, value 0 or 1) so the buffer
    /// can be registered with the visualiser via the logger's `_u8` aux
    /// family — flips are observable as `WriteDataU` events.
    type State = Vec<u8>;

    #[inline]
    fn new_state<T: Ord + Copy, U: ?Sized + SortLogger<T>>(n: usize, logger: &mut U) -> Vec<u8> {
        let reverse = vec![0u8; n];
        logger.log_aux_arr_u8(&reverse);
        reverse
    }

    #[inline]
    fn drop_state<T: Ord + Copy, U: ?Sized + SortLogger<T>>(state: Vec<u8>, logger: &mut U) {
        logger.free_aux_arr_u8(&state);
    }

    #[inline(always)]
    fn root_phys(n: usize) -> usize {
        <D::Layout as Layout>::phys(0, n)
    }

    fn build<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        reverse: &mut Vec<u8>,
        logger: &mut U,
    ) {
        let n = arr.len();
        // Walk i from n − 1 down to 1, merging each node with its
        // distinguished ancestor. After this pass the logical root holds
        // the heap's "champion" value under the chosen direction.
        for i in (1..n).rev() {
            let g = gparent(i, reverse);
            merge::<T, U, D>(arr, reverse, g, i, n, logger);
        }
    }

    #[inline]
    fn swap_root_to_end<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        _state: &mut Vec<u8>,
        heap_size: usize,
        logger: &mut U,
    ) {
        let n = arr.len();
        let root_phys = <D::Layout as Layout>::phys(0, n);
        let end_phys = <D::Layout as Layout>::phys(heap_size - 1, n);
        logger.swap(arr, root_phys, end_phys);
    }

    fn push_down<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        reverse: &mut Vec<u8>,
        heap_size: usize,
        logger: &mut U,
    ) {
        // Heap of size ≤ 1 has nothing to restore.
        if heap_size <= 1 {
            return;
        }
        let n = arr.len();
        // Descend through left children (under reverse bits) to the
        // bottom of the heap that still lies inside the unsorted region.
        let mut j: usize = 1;
        while 2 * j + (reverse[j] as usize) < heap_size {
            j = 2 * j + (reverse[j] as usize);
        }
        // Walk back up to the root, merging at each step. Each merge
        // ensures the surviving root dominates the path's subtree.
        while j > 0 {
            merge::<T, U, D>(arr, reverse, 0, j, n, logger);
            j /= 2;
        }
    }
}

impl<D: Direction> WeakHeapSort<D> {
    /// Inherent thin delegate so `<WeakHeapSort<...>>::sort(arr, logger)`
    /// keeps working from `sort_family!`-generated code without needing
    /// the `HeapAlgorithm` trait in scope at the call site.
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        <Self as HeapAlgorithm>::sort(arr, logger)
    }
}

/// Find the distinguished ancestor of `i`: walk up while `i` is a left
/// child (in the reverse-bit-flipped scheme), then take one more step.
#[inline]
fn gparent(mut i: usize, reverse: &[u8]) -> usize {
    while i > 1 {
        let p = i / 2;
        if (i & 1) as u8 == reverse[p] {
            i = p;
        } else {
            break;
        }
    }
    i / 2
}

/// If `arr[j]` outranks `arr[i]` under the chosen direction, swap them
/// and flip `reverse[j]`. Post-merge, `arr[i]` dominates the right
/// subtree rooted at `j`. The flip is routed through `write_data_u8` so
/// the visualiser observes the bit change.
#[inline]
fn merge<T, U, D>(
    arr: &mut [T],
    reverse: &mut [u8],
    i: usize,
    j: usize,
    n: usize,
    logger: &mut U,
) where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    D: Direction,
{
    let i_phys = <D::Layout as Layout>::phys(i, n);
    let j_phys = <D::Layout as Layout>::phys(j, n);
    if <D::Compare as Compare>::comes_first(logger, arr, j_phys, i_phys) {
        logger.swap(arr, i_phys, j_phys);
        logger.write_data_u8(reverse, j, reverse[j] ^ 1);
    }
}

combo_codegen::sort_family!(
    type = WeakHeapSort<{D}>,
    uses = [
        "crate::sorts::heap_sort::direction::{MinReverse, MaxForward}",
        "crate::sorts::weak_heap_sort::weak_heap_sort::WeakHeapSort",
    ],
    D: HeapDirection,
    name = "weak heap sort",
    big_o = "O(N log N)",
    stable = false,
    direct_sort = true,
    path = ["weak heap sorts", "{D}"],
);
