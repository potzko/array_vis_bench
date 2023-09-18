const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "odd-even bubble sort";

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
    if end - start < 2 {
        return;
    }
    let mut sorted: bool = false;
    while !sorted {
        sorted = true;
        logger.log(SortLog::Mark("parity = 0".to_string()));
        for ii in (start + 2..end).step_by(2) {
            logger.log(SortLog::Cmp {
                name: &arr as *const _ as usize,
                ind_a: ii - 1,
                ind_b: ii,
                result: arr[ii] < arr[ii - 1],
            });
            if arr[ii] < arr[ii - 1] {
                sorted = false;
                logger.log(SortLog::Swap {
                    name: &arr as *const _ as usize,
                    ind_a: ii,
                    ind_b: ii - 1,
                });
                arr.swap(ii, ii - 1);
            }
        }
        logger.log(SortLog::Mark("parity = 1".to_string()));
        for ii in (start + 1..end).step_by(2) {
            logger.log(SortLog::Cmp {
                name: &arr as *const _ as usize,
                ind_a: ii,
                ind_b: ii - 1,
                result: arr[ii] < arr[ii - 1],
            });
            if arr[ii] < arr[ii - 1] {
                sorted = false;
                logger.log(SortLog::Swap {
                    name: &arr as *const _ as usize,
                    ind_a: ii,
                    ind_b: ii - 1,
                });
                arr.swap(ii, ii - 1);
            }
        }
    }
}
