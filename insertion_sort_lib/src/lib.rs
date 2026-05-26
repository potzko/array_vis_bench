//! `InsertionSort<S>` — full insertion sort routed through the chosen
//! [`InsertionStrategy`] (linear or binary). Cross-product registration
//! lives in this crate's `Cargo.toml`
//! (`[[package.metadata.array_vis_bench.families]]`); the wiring crate's
//! build script picks it up via the dep-graph walker.

use std::marker::PhantomData;

use array_vis_bench_traits::{insertion_sort_with, InsertionStrategy};
use sort_logger::SortLogger;

pub struct InsertionSort<S: InsertionStrategy>(PhantomData<S>);

impl<S: InsertionStrategy> InsertionSort<S> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let _ = insertion_sort_with::<S, _, _>(arr, logger);
    }
}
