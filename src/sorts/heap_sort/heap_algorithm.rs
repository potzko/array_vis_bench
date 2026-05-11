//! High-level abstraction for heap-based sorting algorithms.
//!
//! `HeapAlgorithm` captures the build / swap-root / push-down shape shared
//! by the binary/n-ary heap and the weak heap. Direction (compare + layout)
//! is encapsulated inside each impl rather than exposed as an associated
//! type — both impls already carry direction in their type parameters.
//!
//! The trait provides a default [`HeapAlgorithm::sort`] that runs the
//! standard build-then-extract loop, so external sorts (introsort, etc.)
//! can plug in any heap variant without re-implementing the orchestration.

use crate::traits::log_traits::SortLogger;

pub trait HeapAlgorithm {
    /// Per-sort scratch state. `()` for sorts that need none; the weak
    /// heap stores its `Vec<u8>` of reverse bits here.
    type State;

    /// Allocate fresh scratch state for an array of length `n`. Receives the
    /// logger so impls can register the allocation as an aux array.
    fn new_state<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        n: usize,
        logger: &mut U,
    ) -> Self::State;

    /// Release the scratch state. Default just drops it; impls override to
    /// emit a `FreeAuxArr` event for any logger-registered allocation.
    #[inline(always)]
    fn drop_state<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        _state: Self::State,
        _logger: &mut U,
    ) {
    }

    /// Physical array index of the logical root of a heap of size `n`.
    /// Forward layouts return `0`; reverse layouts return `n - 1`.
    /// Lets callers (e.g. quick heap sort) reach the root directly without
    /// going through a heap method.
    fn root_phys(n: usize) -> usize;

    /// Turn the whole of `arr` into a heap.
    fn build<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        state: &mut Self::State,
        logger: &mut U,
    );

    /// Swap the logical root with the logical last element of a heap of
    /// size `heap_size`. Each impl handles its own layout (forward vs
    /// reverse) so the caller stays layout-agnostic.
    fn swap_root_to_end<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        state: &mut Self::State,
        heap_size: usize,
        logger: &mut U,
    );

    /// Restore the heap property at the root over the first `heap_size`
    /// logical positions.
    fn push_down<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        state: &mut Self::State,
        heap_size: usize,
        logger: &mut U,
    );

    /// Default sort: build, then for each `heap_size` from `n` down to 2,
    /// swap the root past the heap and push the new root down through the
    /// shrunken heap.
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        let mut state = Self::new_state::<T, U>(n, logger);
        Self::build(arr, &mut state, logger);
        for heap_size in (2..=n).rev() {
            Self::swap_root_to_end(arr, &mut state, heap_size, logger);
            Self::push_down(arr, &mut state, heap_size - 1, logger);
        }
        Self::drop_state::<T, U>(state, logger);
    }
}
