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
//! marker, so `family!` registers them — multiplied by the
//! [`ReverseStorage`] axis (byte-per-bit vs bit-packed).

use std::marker::PhantomData;

use super::reverse_storage::ReverseStorage;
use crate::sorts::heap_sort::compare::Compare;
use crate::sorts::heap_sort::direction::Direction;
use crate::sorts::heap_sort::heap::HeapLayout;
use crate::sorts::heap_sort::heap_algorithm::HeapAlgorithm;
use crate::sorts::heap_sort::layout::Layout;
use crate::traits::log_traits::SortLogger;

pub struct WeakHeapSort<D: Direction, R: ReverseStorage> {
    _phantom: PhantomData<(D, R)>,
}

// Layout-only `Heap` membership: lets partition-style code that only
// needs `(Compare, phys)` accept a weak heap as "a heap" even though its
// build / sift-down operations are stateful (`Vec<u8>` reverse bits) and
// therefore live on [`HeapAlgorithm`] instead of [`super::super::heap_sort::heap::Heap`].
impl<D: Direction, R: ReverseStorage> HeapLayout for WeakHeapSort<D, R> {
    type Compare = D::Compare;

    #[inline(always)]
    fn phys(i: usize, n: usize) -> usize {
        <D::Layout as Layout>::phys(i, n)
    }
}

impl<D: Direction, R: ReverseStorage> HeapAlgorithm for WeakHeapSort<D, R> {
    /// Per-node reverse bits. Storage layout (byte-per-bit or bit-packed)
    /// is delegated to `R`; the underlying `Vec<u8>` is registered with
    /// the visualiser via the `_u8` aux family.
    type State = Vec<u8>;

    #[inline]
    fn new_state<T: Ord + Copy, U: ?Sized + SortLogger<T>>(n: usize, logger: &mut U) -> Vec<u8> {
        R::new::<T, U>(n, logger)
    }

    #[inline]
    fn drop_state<T: Ord + Copy, U: ?Sized + SortLogger<T>>(state: Vec<u8>, logger: &mut U) {
        R::drop::<T, U>(state, logger);
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
            let g = gparent::<R>(i, reverse);
            merge::<T, U, D, R>(arr, reverse, g, i, n, logger);
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
        while 2 * j + (R::get(reverse, j) as usize) < heap_size {
            j = 2 * j + (R::get(reverse, j) as usize);
        }
        // Walk back up to the root, merging at each step. Each merge
        // ensures the surviving root dominates the path's subtree.
        while j > 0 {
            merge::<T, U, D, R>(arr, reverse, 0, j, n, logger);
            j /= 2;
        }
    }
}

impl<D: Direction, R: ReverseStorage> WeakHeapSort<D, R> {
    /// Inherent thin delegate so `<WeakHeapSort<...>>::sort(arr, logger)`
    /// keeps working from `family!`-generated code without needing
    /// the `HeapAlgorithm` trait in scope at the call site.
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        <Self as HeapAlgorithm>::sort(arr, logger)
    }
}

/// Find the distinguished ancestor of `i`: walk up while `i` is a left
/// child (in the reverse-bit-flipped scheme), then take one more step.
#[inline]
fn gparent<R: ReverseStorage>(mut i: usize, reverse: &[u8]) -> usize {
    while i > 1 {
        let p = i / 2;
        if (i & 1) as u8 == R::get(reverse, p) {
            i = p;
        } else {
            break;
        }
    }
    i / 2
}

/// If `arr[j]` outranks `arr[i]` under the chosen direction, swap them
/// and flip `reverse[j]`. Post-merge, `arr[i]` dominates the right
/// subtree rooted at `j`. The flip is routed through `R::flip` so the
/// visualiser observes the bit change in whatever storage layout `R` uses.
#[inline]
fn merge<T, U, D, R>(
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
    R: ReverseStorage,
{
    let i_phys = <D::Layout as Layout>::phys(i, n);
    let j_phys = <D::Layout as Layout>::phys(j, n);
    if <D::Compare as Compare>::comes_first(logger, arr, j_phys, i_phys) {
        logger.swap(arr, i_phys, j_phys);
        R::flip::<T, U>(reverse, j, logger);
    }
}

combo_codegen::family!(
    type = WeakHeapSort<{D}, {R}>,
    uses = [
        "crate::sorts::heap_sort::direction::{MinReverse, MaxForward}",
        "crate::sorts::weak_heap_sort::reverse_storage::{ByteStorage, BitStorage}",
        "crate::sorts::weak_heap_sort::weak_heap_sort::WeakHeapSort",
    ],
    D: HeapDirection,
    R: ReverseStorage,
    name = "weak heap sort",
    big_o = inherited,
    stable = false,
    direct_sort = true,
    path = ["weak heap sorts", "{D}", "{R}"],
);

// ── Composable annotations ──────────────────────────────────────────
//
// Weak heap sort is `O(N log N)` worst, best, and average regardless
// of the `ReverseStorage` flavour (byte vs bit-packed). The storage
// choice only affects constants and visualisation, not asymptotic
// behaviour.
//
// Space: O(N) for the parity / reverse bit array (one bit or byte per
// node).

impl<D: crate::sorts::heap_sort::direction::Direction, R: super::reverse_storage::ReverseStorage>
    crate::traits::composable::HasTimeBounds for WeakHeapSort<D, R>
{
    const WORST: crate::traits::complexity::Complexity = crate::traits::complexity::Complexity::N_LOG_N;
    const BEST: crate::traits::complexity::Complexity = crate::traits::complexity::Complexity::N_LOG_N;
    const AVERAGE: crate::traits::complexity::Complexity = crate::traits::complexity::Complexity::N_LOG_N;
}
impl<D: crate::sorts::heap_sort::direction::Direction, R: super::reverse_storage::ReverseStorage>
    crate::traits::composable::HasSpace for WeakHeapSort<D, R>
{
    /// One bit/byte per node for the reverse/parity array.
    const SPACE: crate::traits::complexity::Complexity = crate::traits::complexity::Complexity::N1;
}
impl<D: crate::sorts::heap_sort::direction::Direction, R: super::reverse_storage::ReverseStorage>
    crate::traits::composable::HasStability for WeakHeapSort<D, R>
{
    const STABLE: bool = false;
}
