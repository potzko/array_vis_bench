use std::marker::PhantomData;

use crate::traits::log_traits::SortLogger;
use crate::utils::small_sort::{insertion_sort_with, InsertionStrategy};

combo_codegen::family!(
    type = InsertionSort<{S}>,
    uses = [
        "crate::utils::small_sort::{LinearInsertion, BinaryInsertion}",
        "super::insertion_sort::InsertionSort",
    ],
    S: InsertionStrategy,
    name = "insertion sort",
    big_o = "O(N^2)",
    stable = true,
    adaptive = true,
    direct_sort = true,
    path = ["insertion sorts", "{S}"],
);

pub struct InsertionSort<S: InsertionStrategy>(PhantomData<S>);

impl<S: InsertionStrategy> InsertionSort<S> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let _ = insertion_sort_with::<S, _, _>(arr, logger);
    }
}
