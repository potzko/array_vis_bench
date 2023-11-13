const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "merge sort";
use super::utils::rotate_merge;
use crate::traits;
pub struct MergeSort {}

impl traits::sort_traits::SortAlgo for MergeSort {
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
impl Default for MergeSort {
    fn default() -> Self {
        MergeSort {}
    }
}
impl Debug for MergeSort {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Result::Ok(())
    }
}

const SMALL_SORT_SIZE: usize = 20;
use traits::sort_traits::SortAlgo;
fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for i in (0..arr.len()).step_by(SMALL_SORT_SIZE) {
        let end = std::cmp::min(i + SMALL_SORT_SIZE, arr.len());
        let small_sort = crate::sorts::insertion_sorts::insertion_sort::InsertionSort {};
        small_sort.sort(arr, logger);
    }
    let mut gap = SMALL_SORT_SIZE;
    let mut i;
    while gap < arr.len() {
        i = 0;
        while i < arr.len() {
            let end = std::cmp::min(i + 2 * gap, arr.len());
            if i + gap < arr.len() {
                rotate_merge(&mut arr[i..end], gap, logger);
            }
            i += gap * 2;
        }
        gap *= 2
    }
}
