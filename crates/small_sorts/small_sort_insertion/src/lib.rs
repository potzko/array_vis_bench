//! Insertion sort for subarrays of length ≤ N, dispatched via an
//! [`InsertionStrategy`] (linear or binary).

use std::marker::PhantomData;

use array_vis_bench_traits::{
    insertion_sort_with, Complexity, HasSpace, HasStability, HasTimeBounds, InsertionStrategy,
    NonTrivialSmallSort, SmallSort,
};
use sort_logger::SortLogger;

// Bring strategy types into scope so the orphan-rule-required impls of
// `SmallSort for InsertionSmallSort<LinearInsertion, 16>` etc. land
// here. (Each specialisation impl below picks one explicitly.)
#[allow(unused_imports)]
use small_sort_insertion_strategy::{BinaryInsertion, LinearInsertion};

pub struct InsertionSmallSort<S: InsertionStrategy, const N: usize>(PhantomData<S>);

impl<S: InsertionStrategy, const N: usize> SmallSort for InsertionSmallSort<S, N> {
    const THRESHOLD: usize = N;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) -> bool {
        insertion_sort_with::<S, _, _>(arr, logger)
    }
}
impl<S: InsertionStrategy, const N: usize> NonTrivialSmallSort for InsertionSmallSort<S, N> {}

// Insertion sort: O(N²) swaps in the worst case, O(N) compares on a
// pre-sorted input (best case). Stable regardless of insertion strategy.
impl<S: InsertionStrategy, const N: usize> HasTimeBounds for InsertionSmallSort<S, N> {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N_SQUARED;
}
impl<S: InsertionStrategy, const N: usize> HasSpace for InsertionSmallSort<S, N> {
    const SPACE: Complexity = Complexity::CONST;
}
impl<S: InsertionStrategy, const N: usize> HasStability for InsertionSmallSort<S, N> {
    const STABLE: bool = true;
}
