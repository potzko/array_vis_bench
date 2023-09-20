const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N^1.5)";
const NAME: &str = "shell sort knuth jumps";

use crate::traits;
use traits::sort_traits::SortAlgo;
pub struct ShellSort {}

impl traits::sort_traits::SortAlgo for ShellSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &str {
        BIG_O
    }
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
        arr: &mut [T],
        start: usize,
        end: usize,
        logger: &mut U,
    ) {
        sort::<T, U>(arr, start, end, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    for jump in [85, 24] {
        logger.mark_mssg(&format!("jump = {}", jump));
        for i in start + jump..end {
            let temp = arr[i];
            let mut j = i;
            while j >= jump + start && logger.cmp_gt_data(arr, j - jump, temp) {
                logger.write(arr, j, j - jump);
                j -= jump;
            }
            logger.write_data(arr, j, temp);
        }
    }
    crate::sorts::insertion_sorts::insertion_sort::InsertionSort::sort(arr, start, end, logger);
}
