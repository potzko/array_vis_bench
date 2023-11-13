const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N*log(N))";
const NAME: &str = "weak heap sort";

use crate::traits;

pub struct HeapSort {}

impl traits::sort_traits::SortAlgo for HeapSort {
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
impl Default for HeapSort {
    fn default() -> Self {
        HeapSort {}
    }
}
impl Debug for HeapSort {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Result::Ok(())
    }
}

pub fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let n = arr.len();

    if n < 2 {
        return;
    }

    let mut bottom_skips = vec![0; (n + 7) / 8];
    for i in (1..n).rev() {
        let mut ii = i;
        while ii & 1 == get_bitwise_flag(&bottom_skips, ii >> 1) {
            ii >>= 1;
        }
        let big_parent = ii >> 1;
        weak_heap_merge(arr, &mut bottom_skips, big_parent, i, logger);
    }

    for i in (2..n).rev() {
        logger.swap(arr, 0, i);
        let mut current = 1;
        let mut next = 2 * current + get_bitwise_flag(&bottom_skips, current);
        while next < i {
            current = next;
            next = 2 * current + get_bitwise_flag(&bottom_skips, current);
        }
        while current > 0 {
            weak_heap_merge(arr, &mut bottom_skips, 0, current, logger);
            current >>= 1;
        }
    }
    arr.swap(0, 1);
}

fn weak_heap_merge<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    bottom_skips: &mut [u8],
    ind_a: usize,
    ind_b: usize,
    logger: &mut U,
) {
    if logger.cond_swap_lt(arr, ind_a, ind_b) {
        toggle_bitwise_flag(bottom_skips, ind_b);
    }
}

fn get_bitwise_flag(bottom_skips: &[u8], ind: usize) -> usize {
    ((bottom_skips[ind >> 3] >> (ind & 7)) & 1).into()
}

fn toggle_bitwise_flag(bottom_skips: &mut [u8], ind: usize) {
    bottom_skips[ind >> 3] ^= 1 << (ind & 7)
}
