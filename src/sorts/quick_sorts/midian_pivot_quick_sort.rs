const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick surrender";

use crate::traits;
use rand::{rngs::ThreadRng, Rng};
pub struct QuickSort {}

impl traits::sort_traits::SortAlgo for QuickSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &'static str {
        BIG_O
    }
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
        &self,
        arr: &mut [T],
        logger: &mut U,
    ) {
        sort::<T, U>(arr, logger);
    }
    fn name(&self) -> &'static str {
        NAME
    }
}
use std::fmt::Debug;
#[allow(clippy::derivable_impls)]
impl Default for QuickSort {
    fn default() -> Self {
        QuickSort {}
    }
}
impl Debug for QuickSort {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Result::Ok(())
    }
}

fn quick_select<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    target: usize,
    rng: &mut rand::rngs::ThreadRng,
    logger: &mut U,
) {
    let piv = partition(arr, rng, logger);
    if piv == target {
        return;
    }
    if piv < target {
        quick_select(&mut arr[piv + 1..], target, rng, logger);
    } else {
        quick_select(&mut arr[0..piv], target, rng, logger)
    };
}

fn sort_helper<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    rng: &mut ThreadRng,
    logger: &mut U,
) {
    if arr.is_empty() {
        return;
    }
    let mid = arr.len() / 2;
    quick_select(arr, mid, rng, logger);
    sort_helper(&mut arr[0..mid], rng, logger);
    sort_helper(&mut arr[mid..], rng, logger);
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let mut rng = rand::thread_rng();
    sort_helper(arr, &mut rng, logger);
}

fn partition<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    rng: &mut rand::rngs::ThreadRng,
    logger: &mut U,
) -> usize {
    // Choose a random index between start and end - 1 as the pivot
    let pivot_index: usize = rng.gen_range(0..arr.len());
    // Swap the pivot with the last element
    logger.swap(arr, pivot_index, arr.len() - 1);

    let pivot = arr[arr.len() - 1];
    let mut small = 0;
    for i in 0..arr.len() - 1 {
        if logger.cmp_lt_data(arr, i, pivot) {
            logger.swap(arr, i, small);
            small += 1;
        }
    }
    logger.swap(arr, small, arr.len() - 1);
    small
}
