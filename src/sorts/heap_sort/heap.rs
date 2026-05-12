//! Heap traits — split into a minimal *layout* surface and a richer
//! *operations* surface on top of it.
//!
//! - [`HeapLayout`]: the bare minimum that makes something "a heap" for
//!   layout-aware code (partitions, direction-aware compares). Holds the
//!   compare direction and the logical→physical mapping. Default-impls
//!   `swap` because that's just `swap(phys(i), phys(j))` over the logger.
//!   Weak heap, n-ary heap, beap heap and any future array-laid-out heap
//!   should impl this.
//!
//! - [`Heap: HeapLayout`]: adds the operations that *layered* heaps share —
//!   single-node sift-down (`heapify`), recursive subtree build
//!   (`deep_heapify`), and the layer-structure constants
//!   (`last_internal_node`, `layer_boundaries`) the build strategies in
//!   [`super::deep_heapify`] and [`super::quick_deep_heapify`] consume.
//!   Weak heap deliberately doesn't impl this — its operations need state
//!   (`Vec<u8>` reverse bits) and live on
//!   [`super::heap_algorithm::HeapAlgorithm`] instead.

use super::compare::Compare;
use crate::traits::log_traits::SortLogger;

pub trait HeapLayout {
    /// Direction (Min vs Max) used to decide rootward-ness. Exposed so
    /// direction-aware code (heap partitions, quickselect-based builds)
    /// can pick up the right compare without re-deriving it.
    type Compare: Compare;

    /// Physical array index for logical heap index `i` over an array of
    /// length `n`. Forward layout returns `i`; reverse returns `n - 1 - i`.
    fn phys(i: usize, n: usize) -> usize;

    /// Swap two logical heap positions. Default-impl routes both indices
    /// through `phys` so each layout-specific override is unnecessary.
    #[inline(always)]
    fn swap<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        i: usize,
        j: usize,
        logger: &mut U,
    ) {
        let n = arr.len();
        logger.swap(arr, Self::phys(i, n), Self::phys(j, n));
    }
}

pub trait Heap: HeapLayout {
    /// Single-node sift-down. Assume each child's subtree already
    /// satisfies the heap predicate; move logical node `i` toward the
    /// leaves until the invariant holds at `i`.
    fn heapify<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        heap_size: usize,
        i: usize,
        logger: &mut U,
    );

    /// Recursively make the entire subtree rooted at `i` a heap.
    fn deep_heapify<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        heap_size: usize,
        i: usize,
        logger: &mut U,
    );

    /// Largest logical index that has at least one child in a heap of
    /// length `n`. Iterative bottom-up build iterates this down to 0.
    fn last_internal_node(n: usize) -> usize;

    /// Layer-boundary logical indices in ascending order:
    /// `[1, B_1, B_2, …]`. Each entry is the index of the *first* node of
    /// some non-root layer (equivalently, one past the last node of the
    /// previous layer). Excludes 0 and any boundary ≥ n. Used by the
    /// quickselect-based build strategies in [`super::quick_deep_heapify`].
    fn layer_boundaries(n: usize) -> Vec<usize>;
}
