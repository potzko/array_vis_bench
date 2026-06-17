//! `InsertionSort<S>` — full insertion sort routed through the chosen
//! [`InsertionStrategy`] (linear or binary). Cross-product registration
//! lives in this crate's `Cargo.toml`
//! (`[[package.metadata.array_vis_bench.families]]`); the wiring crate's
//! build script picks it up via the dep-graph walker.

use std::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::{insertion_sort_with, Complexity, InsertionStrategy};
use sort_logger::SortLogger;

pub struct InsertionSort<S: InsertionStrategy>(PhantomData<S>);

impl<S: InsertionStrategy> InsertionSort<S> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let _ = insertion_sort_with::<S, _, _>(arr, logger);
    }
}

// Composable annotations — the spec compiler emits each entry's complexity by
// inheriting these from the concrete type. Insertion sort's bounds are the same
// for both strategies (linear and binary differ only in compares, not the
// asymptotic class): adaptive `O(N)` best, `O(N²)` average/worst, in-place,
// stable (both strategies insert after equal keys).
impl<S: InsertionStrategy> HasTimeBounds for InsertionSort<S> {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N_SQUARED;
}

impl<S: InsertionStrategy> HasSpace for InsertionSort<S> {
    const SPACE: Complexity = Complexity::CONST;
}

impl<S: InsertionStrategy> HasStability for InsertionSort<S> {
    const STABLE: bool = true;
}
