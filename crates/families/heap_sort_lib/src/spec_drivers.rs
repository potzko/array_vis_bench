//! Spec-system driver wrappers with distinct Rust type-heads.
//!
//! The legacy heap variants are all registered as `HeapSort<…>`, which means
//! they share the single Rust head `HeapSort`. AVBS emit resolves by Rust
//! type-head first-wins, so only one of them would ever survive. To make the
//! distinct heap-build strategies independently emittable, this module
//! introduces three thin newtype drivers — each with its own unique head —
//! that all delegate to the canonical
//! `HeapSort::<NaryHeapSort<ArityHeap<A, D>, DH>>::sort` chain.
//!
//! Each driver is generic over `<A: Arity, D: Direction, DH: DeepHeapify>` and
//! carries the same composable annotations the inner
//! `NaryHeapSort<ArityHeap<A, D>, DH>` composes to:
//! `WORST = BEST = AVERAGE = N_LOG_N` (N extractions × `O(log N)` heapify),
//! `SPACE = LOG_N`, `STABLE = false`.

use std::marker::PhantomData;

use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use sort_logger::SortLogger;

use super::arity::Arity;
use super::arity_heap::ArityHeap;
use super::deep_heapify::DeepHeapify;
use super::direction::Direction;
use super::heap_sort::{HeapSort, NaryHeapSort};

/// Distinct-head driver for the classic heap-build heap sort.
pub struct HeapSortClassicOf<A: Arity, D: Direction, DH: DeepHeapify> {
    _phantom: PhantomData<(A, D, DH)>,
}

impl<A: Arity, D: Direction, DH: DeepHeapify> HeapSortClassicOf<A, D, DH> {
    #[inline(always)]
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        HeapSort::<NaryHeapSort<ArityHeap<A, D>, DH>>::sort(arr, logger)
    }
}

impl<A: Arity, D: Direction, DH: DeepHeapify> HasTimeBounds for HeapSortClassicOf<A, D, DH> {
    const WORST: Complexity = Complexity::N_LOG_N;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}

impl<A: Arity, D: Direction, DH: DeepHeapify> HasSpace for HeapSortClassicOf<A, D, DH> {
    const SPACE: Complexity = Complexity::LOG_N;
}

impl<A: Arity, D: Direction, DH: DeepHeapify> HasStability for HeapSortClassicOf<A, D, DH> {
    const STABLE: bool = false;
}

/// Distinct-head driver for the quickselect-build heap sort.
pub struct HeapSortQuickBuildOf<A: Arity, D: Direction, DH: DeepHeapify> {
    _phantom: PhantomData<(A, D, DH)>,
}

impl<A: Arity, D: Direction, DH: DeepHeapify> HeapSortQuickBuildOf<A, D, DH> {
    #[inline(always)]
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        HeapSort::<NaryHeapSort<ArityHeap<A, D>, DH>>::sort(arr, logger)
    }
}

impl<A: Arity, D: Direction, DH: DeepHeapify> HasTimeBounds for HeapSortQuickBuildOf<A, D, DH> {
    const WORST: Complexity = Complexity::N_LOG_N;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}

impl<A: Arity, D: Direction, DH: DeepHeapify> HasSpace for HeapSortQuickBuildOf<A, D, DH> {
    const SPACE: Complexity = Complexity::LOG_N;
}

impl<A: Arity, D: Direction, DH: DeepHeapify> HasStability for HeapSortQuickBuildOf<A, D, DH> {
    const STABLE: bool = false;
}

/// Distinct-head driver for the dual-pivot-build heap sort.
pub struct HeapSortDualBuildOf<A: Arity, D: Direction, DH: DeepHeapify> {
    _phantom: PhantomData<(A, D, DH)>,
}

impl<A: Arity, D: Direction, DH: DeepHeapify> HeapSortDualBuildOf<A, D, DH> {
    #[inline(always)]
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        HeapSort::<NaryHeapSort<ArityHeap<A, D>, DH>>::sort(arr, logger)
    }
}

impl<A: Arity, D: Direction, DH: DeepHeapify> HasTimeBounds for HeapSortDualBuildOf<A, D, DH> {
    const WORST: Complexity = Complexity::N_LOG_N;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}

impl<A: Arity, D: Direction, DH: DeepHeapify> HasSpace for HeapSortDualBuildOf<A, D, DH> {
    const SPACE: Complexity = Complexity::LOG_N;
}

impl<A: Arity, D: Direction, DH: DeepHeapify> HasStability for HeapSortDualBuildOf<A, D, DH> {
    const STABLE: bool = false;
}
