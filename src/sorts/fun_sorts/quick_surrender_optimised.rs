const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick surrender";

use crate::traits;
use rand::Rng;
pub struct FunSort {}

impl traits::sort_traits::SortAlgo for FunSort {
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
impl Default for FunSort {
    fn default() -> Self {
        FunSort {}
    }
}
impl Debug for FunSort {
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
    if arr.len() < 2 {
        return;
    }
    let piv = partition(arr, rng, logger);
    if piv == target {
        return;
    }
    if piv < target {
        quick_select(&mut arr[piv + 1..], target, rng, logger);
    } else {
        quick_select(&mut arr[..piv], target, rng, logger);
    };
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    use crate::traits::sort_traits::SortAlgo;
    let mut rng = rand::thread_rng();
    let len = arr.len();
    let small_sort = crate::sorts::insertion_sorts::insertion_sort::InsertionSort {};

    for i in (0..arr.len()).step_by(16) {
        quick_select(&mut arr[i..], std::cmp::min(i + 16, len), &mut rng, logger);
        small_sort.sort(&mut arr[i..std::cmp::min(i + 16, len)], logger);
    }
    small_sort.sort(&mut arr[16 * len / 16..len], logger);
    //sort(arr, start, end, &mut rng, logger)
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
