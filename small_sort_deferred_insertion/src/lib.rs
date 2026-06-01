//! Deferred-insertion small sort: quicksort stops at sub-arrays of
//! length ≤ N and lets a single final insertion-sort pass (using
//! strategy `S`) clean up.

use std::marker::PhantomData;

use array_vis_bench_traits::{windowed_insertion_sort_with, DeferredSmallSort, InsertionStrategy};
use sort_logger::SortLogger;

// Strategy types are needed for the registered specialisations to land
// here (orphan rule).
#[allow(unused_imports)]
use small_sort_insertion_strategy::{BinaryInsertion, LinearInsertion};

pub struct DeferredInsertion<S: InsertionStrategy, const N: usize>(PhantomData<S>);

impl<S: InsertionStrategy, const N: usize> DeferredSmallSort for DeferredInsertion<S, N> {
    const THRESHOLD: usize = N;
    #[inline(always)]
    fn final_pass<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        // Quicksort guaranteed each element is at most N positions away
        // from its sorted home, so the scan-back is capped at N. Cost is
        // O(n · N), and visually each insertion only swaps inside its
        // K-window instead of sweeping the whole array.
        let _ = windowed_insertion_sort_with::<S, _, _>(arr, N, logger);
    }
}
