const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N)^2)";
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

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }

    {
        let (left, right) = arr.split_at_mut(arr.len() / 2);
        sort(left, logger);
        sort(right, logger);
    }
    rotate_merge(arr, arr.len() / 2, logger)
}
