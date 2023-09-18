const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick surrender";

use crate::traits;
use rand::Rng;
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

fn quick_select<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    target: usize,
    rng: &mut rand::rngs::ThreadRng,
    logger: &mut U,
) {
    if end - start < 2 {
        return;
    }
    let piv = partition(arr, start, end, rng, logger);
    if piv == target {
        return;
    }
    if piv < target {
        quick_select(arr, piv + 1, end, target, rng, logger);
    } else {
        quick_select(arr, start, piv, target, rng, logger)
    };
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    use crate::traits::sort_traits::SortAlgo;
    let mut rng = rand::thread_rng();
    for i in (start..end).step_by(16) {
        quick_select(arr, i, end, std::cmp::min(i + 16, end), &mut rng, logger);
        crate::sorts::insertion_sorts::insertion_sort::InsertionSort::sort(
            arr,
            i,
            std::cmp::min(i + 16, end),
            logger,
        );
    }
    crate::sorts::insertion_sorts::insertion_sort::InsertionSort::sort(
        arr,
        16 * (end - start) / 16,
        end,
        logger,
    );
    //sort(arr, start, end, &mut rng, logger)
}

fn partition<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    rng: &mut rand::rngs::ThreadRng,
    logger: &mut U,
) -> usize {
    // Choose a random index between start and end - 1 as the pivot
    let pivot_index: usize = rng.gen_range(start..end);
    // Swap the pivot with the last element
    arr.swap(pivot_index, end - 1);
    logger.log(SortLog::Swap {
        name: &arr as *const _ as usize,
        ind_a: pivot_index,
        ind_b: end - 1,
    });

    let pivot = arr[end - 1];
    let mut small = start;
    for i in start..end - 1 {
        logger.log(SortLog::CmpSingle {
            name: &arr as *const _ as usize,
            ind_a: i,
            result: arr[i] < pivot,
        });
        logger.log(SortLog::CmpOuterData {
            name: 0,
            ind: i,
            data: pivot,
            result: arr[i] < pivot,
        });
        if arr[i] < pivot {
            logger.log(SortLog::Swap {
                name: &arr as *const _ as usize,
                ind_a: i,
                ind_b: small,
            });
            arr.swap(i, small);
            small += 1;
        }
    }
    logger.log(SortLog::Swap {
        name: &arr as *const _ as usize,
        ind_a: small,
        ind_b: end - 1,
    });
    arr.swap(small, end - 1);
    small
}
