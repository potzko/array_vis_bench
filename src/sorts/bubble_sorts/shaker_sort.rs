const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "shaker sort";

use crate::traits;
pub struct BubbleSort {}

impl traits::sort_traits::SortAlgo for BubbleSort {
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
    use traits::log_traits::SortLog;
    let len = end - start;
    for ii in start..len / 2 + start {
        for i in start + 1..end - (ii - start) {
            logger.log(SortLog::Cmp {
                name: &arr as *const _ as usize,
                ind_a: i,
                ind_b: i - 1,
                result: arr[i] < arr[i - 1],
            });
            if arr[i] < arr[i - 1] {
                logger.log(SortLog::Swap {
                    name: &arr as *const _ as usize,
                    ind_a: i,
                    ind_b: i - 1,
                });
                arr.swap(i - 1, i);
            }
        }
        for i in (start + 1..end - (ii - start)).rev() {
            logger.log(SortLog::Cmp {
                name: &arr as *const _ as usize,
                ind_a: i,
                ind_b: i - 1,
                result: arr[i] < arr[i - 1],
            });
            if arr[i] < arr[i - 1] {
                logger.log(SortLog::Swap {
                    name: &arr as *const _ as usize,
                    ind_a: i,
                    ind_b: i - 1,
                });
                arr.swap(i - 1, i);
            }
        }
    }
}
