const MAX_SIZE: usize = 500;
const BIG_O: &str = "O(N^?)";
const NAME: &str = "bad_heap";

use crate::traits;
use traits::log_traits::SortLog;
pub struct FunSort {}

impl traits::sort_traits::SortAlgo for FunSort {
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
    use traits::log_traits::SortLog::*;

    if end - start < 2 {
        return;
    }

    let left = start + 1;
    let right = start * 2 + 1;

    if right < end && arr[right] < arr[left] {
        // Logging the comparison
        logger.log(Cmp {
            name: &arr as *const _ as usize,
            ind_a: right,
            ind_b: left,
            result: arr[right] < arr[left],
        });

        // Logging the swap
        logger.log(Swap {
            name: &arr as *const _ as usize,
            ind_a: left,
            ind_b: right,
        });
        arr.swap(left, right);

        sort(arr, right, end, logger);
    }

    sort(arr, left, end, logger);

    // Logging the comparison
    logger.log(Cmp {
        name: &arr as *const _ as usize,
        ind_a: left,
        ind_b: start,
        result: arr[left] < arr[start],
    });

    if arr[left] < arr[start] {
        // Logging the swap
        logger.log(Swap {
            name: &arr as *const _ as usize,
            ind_a: left,
            ind_b: start,
        });
        arr.swap(left, start);

        sort(arr, start, end, logger);
    }
}
