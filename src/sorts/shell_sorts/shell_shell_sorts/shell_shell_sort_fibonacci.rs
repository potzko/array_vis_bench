const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "shell sort 2.25 shrink factor";

use crate::traits;
pub struct ShellSort {}

impl traits::sort_traits::SortAlgo for ShellSort {
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
impl Default for ShellSort {
    fn default() -> Self {
        ShellSort {}
    }
}
impl Debug for ShellSort {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Result::Ok(())
    }
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let mut fib = Vec::with_capacity((arr.len() as f64).log2() as usize);
    fib.push(1);
    let mut a = 1;
    let mut b = 1;
    while b < arr.len() {
        fib.push(b);
        let tmp = b;
        b += a;
        a = tmp
    }
    sort_rec(arr, 1, &fib, logger);
}

fn sort_rec<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    jump: usize,
    fib: &[usize],
    logger: &mut U,
) {
    let len = arr.len() / jump;
    if len == 0 {
        return;
    }
    if len < 16 {
        insertion_sort_jump(arr, jump, logger);
        return;
    }
    let num: usize = match fib.binary_search(&len) {
        Ok(num) => num,
        Err(num) => num,
    };
    for i in 0..num {
        sort_rec(&mut arr[jump * i..], jump * num, &fib[0..num], logger);
    }
    let num = num - 1;
    if len >= num * 16 {
        for i in 0..num {
            insertion_sort_jump(&mut arr[jump * i..], jump * num, logger);
        }
    }
    insertion_sort_jump(arr, jump, logger);
}

fn insertion_sort_jump<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    jump: usize,
    logger: &mut U,
) {
    for i in (0..arr.len()).step_by(jump) {
        let mut ind = i;
        while ind != 0 {
            if logger.cond_swap_le(arr, ind, ind - jump) {
                ind -= jump;
            } else {
                break;
            }
        }
    }
}
